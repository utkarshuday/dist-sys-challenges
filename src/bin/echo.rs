use anyhow::Ok;
use distributed_systems::*;
use serde::{Deserialize, Serialize};
use std::io::StdoutLock;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Echo { echo: String },
    EchoOk { echo: String },
}

struct EchoNode {
    src: String,
    id: usize,
}

impl Node<Payload> for EchoNode {
    fn new(src: String, id: usize) -> Self {
        Self { src, id }
    }

    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.kind {
            Payload::Echo { echo } => {
                let response = Message {
                    src: self.src.clone(),
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        kind: Payload::EchoOk { echo },
                    },
                };
                response.send_message(output)?;
                self.id += 1;
            }
            Payload::EchoOk { .. } => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    run::<EchoNode, Payload>()
}
