use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fmt::Debug,
    io::{BufRead, StdoutLock, Write},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message<T> {
    pub src: String,
    pub dest: String,
    pub body: Body<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Body<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "msg_id")]
    pub id: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,

    #[serde(flatten)]
    pub kind: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitPayload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

impl<P: Serialize> Message<P> {
    pub fn send_message(&self, output: &mut StdoutLock) -> anyhow::Result<()> {
        serde_json::to_writer(&mut *output, self).context("serialize response to echo failed")?;
        output.write_all(b"\n").context("write trailing newline")?;
        output.flush().context("failed to flush buffer")?;
        Ok(())
    }
}

pub trait Node<P: Serialize> {
    fn new(src: String, id: usize) -> Self;
    fn step(&mut self, input: Message<P>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn run<N, P>() -> anyhow::Result<()>
where
    N: Node<P>,
    P: Serialize + DeserializeOwned + Clone + Debug,
{
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();

    let mut output = std::io::stdout().lock();

    let init_message = stdin
        .next()
        .context("no message received")?
        .context("failed to read message from STDIN")?;

    let init_message: Message<InitPayload> =
        serde_json::from_str(&init_message).context("could not deserialize message")?;

    let InitPayload::Init { node_id, .. } = init_message.body.kind else {
        panic!("first message should be init");
    };

    let mut state: N = Node::new(node_id, 0);

    let response = Message {
        src: init_message.dest,
        dest: init_message.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_message.body.id,
            kind: InitPayload::InitOk,
        },
    };

    response.send_message(&mut output)?;

    for input in stdin {
        let input = input.context("Maelstrom input from STDIN could not be read")?;
        let input = serde_json::from_str(&input)
            .context("Maelstrom input from STDIN could not be deserialized")?;
        state
            .step(input, &mut output)
            .context("Node step function failed")?;
    }

    Ok(())
}
