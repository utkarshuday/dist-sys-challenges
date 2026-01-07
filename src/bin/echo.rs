use anyhow::Ok;
use distributed_systems::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, StdoutLock};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    node_id: String,
    next_msg_id: usize,
}

impl Node<Payload> for EchoNode {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut response = input.into_response(Some(&mut self.next_msg_id), &self.node_id);
        match response.body.kind {
            Payload::Echo { echo } => {
                response.body.kind = Payload::EchoOk { echo };
                response.send_message(output)?;
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

impl EchoNode {
    fn new(init: Init, next_msg_id: usize) -> Self {
        Self {
            node_id: init.node_id,
            next_msg_id,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut output = std::io::stdout().lock();

    let init = send_init_message(&mut stdin, &mut output)?;
    let mut node = EchoNode::new(init, 1);

    main_loop(&mut node, &mut stdin, &mut output)
}
