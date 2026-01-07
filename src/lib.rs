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

pub trait Node<P> {
    fn step(&mut self, input: Message<P>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn send_init_message(
    stdin: &mut Lines<StdinLock>,
    output: &mut StdoutLock,
) -> anyhow::Result<String> {
    let init_message = stdin
        .next()
        .context("no message received")?
        .context("failed to read message from STDIN")?;

    let init_message: Message<InitPayload> =
        serde_json::from_str(&init_message).context("could not deserialize message")?;

    let InitPayload::Init { node_id, .. } = init_message.body.kind else {
        panic!("first message should be init");
    };

    let response = Message {
        src: init_message.dest,
        dest: init_message.src,
        body: Body {
            id: Some(0),
            in_reply_to: init_message.body.id,
            kind: InitPayload::InitOk,
        },
    };

    response.send_message(output)?;
    Ok(node_id)
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
