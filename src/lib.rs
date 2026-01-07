use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fmt::Debug,
    io::{Lines, StdinLock, StdoutLock, Write},
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
    pub msg_id: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<usize>,

    #[serde(flatten)]
    pub kind: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InitPayload {
    Init(Init),
    InitOk,
}

impl<P: Serialize> Message<P> {
    pub fn into_response(self, next_msg_id: Option<&mut usize>, node_id: &str) -> Self {
        Self {
            src: node_id.to_string(),
            dest: self.src,
            body: Body {
                msg_id: next_msg_id.map(|next_msg_id| {
                    let temp = *next_msg_id;
                    *next_msg_id += 1;
                    temp
                }),
                in_reply_to: self.body.msg_id,
                kind: self.body.kind,
            },
        }
    }

    pub fn send_message(&self, output: &mut StdoutLock) -> anyhow::Result<()> {
        serde_json::to_writer(&mut *output, self).context("serialize response to echo failed")?;
        output.write_all(b"\n").context("write trailing newline")?;
        output.flush().context("failed to flush buffer")?;
        Ok(())
    }
}

pub trait Node<P> {
    fn step(&mut self, input: Message<P>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn send_init_message(
    stdin: &mut Lines<StdinLock>,
    output: &mut StdoutLock,
) -> anyhow::Result<Init> {
    let init_message = stdin
        .next()
        .context("no message received")?
        .context("failed to read message from STDIN")?;

    let init_message: Message<InitPayload> =
        serde_json::from_str(&init_message).context("could not deserialize message")?;

    let InitPayload::Init(init) = init_message.body.kind else {
        panic!("first message should be init");
    };

    let response = Message {
        src: init_message.dest,
        dest: init_message.src,
        body: Body {
            msg_id: Some(0),
            in_reply_to: init_message.body.msg_id,
            kind: InitPayload::InitOk,
        },
    };

    response.send_message(output)?;
    Ok(init)
}

pub fn main_loop<N, P>(
    node: &mut N,
    stdin: &mut Lines<StdinLock>,
    output: &mut StdoutLock,
) -> anyhow::Result<()>
where
    N: Node<P>,
    P: DeserializeOwned,
{
    for input in stdin {
        let input = input.context("Maelstrom input from STDIN could not be read")?;
        let input = serde_json::from_str(&input)
            .context("Maelstrom input from STDIN could not be deserialized")?;
        node.step(input, output)
            .context("Node step function failed")?;
    }
    Ok(())
}
