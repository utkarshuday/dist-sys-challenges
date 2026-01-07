use anyhow::Ok;
use distributed_systems::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, StdoutLock};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Generate,
    GenerateOk {
        #[serde(rename = "id")]
        guid: String,
    },
}

struct GenerateNode {
    src: String,
    id: usize,
}

impl Node<Payload> for GenerateNode {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.kind {
            Payload::Generate => {
                let guid = format!("{}-{}", self.src, self.id);
                let response = Message {
                    src: self.src.clone(),
                    dest: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        kind: Payload::GenerateOk { guid },
                    },
                };
                response.send_message(output)?;
                self.id += 1;
            }
            Payload::GenerateOk { .. } => {}
        }
        Ok(())
    }
}

impl GenerateNode {
    fn new(src: String, id: usize) -> Self {
        Self { src, id }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut output = std::io::stdout().lock();

    let node_id = send_init_message(&mut stdin, &mut output)?;
    let mut node = GenerateNode::new(node_id, 1);

    main_loop(&mut node, &mut stdin, &mut output)
}
