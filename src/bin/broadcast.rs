use anyhow::Ok;
use distributed_systems::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{BufRead, StdoutLock},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    Broadcast {
        message: Value,
    },
    BroadcastOk,
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
    Read,
    ReadOk {
        messages: Vec<Value>,
    },
}

struct BroadcastNode {
    node_id: String,
    next_msg_id: usize,
    messages: Vec<Value>,
    topology: HashMap<String, Vec<String>>,
}

impl Node<Payload> for BroadcastNode {
    fn step(&mut self, input: Message<Payload>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut response = input.into_response(Some(&mut self.next_msg_id), &self.node_id);
        match response.body.kind {
            Payload::Broadcast { message } => {
                self.messages.push(message);
                response.body.kind = Payload::BroadcastOk;
                response.send_message(output)?;
            }
            Payload::Read => {
                response.body.kind = Payload::ReadOk {
                    messages: self.messages.clone(),
                };
                response.send_message(output)?;
            }
            Payload::Topology { topology } => {
                self.topology = topology;
                response.body.kind = Payload::TopologyOk;
                response.send_message(output)?;
            }
            Payload::BroadcastOk | Payload::ReadOk { .. } | Payload::TopologyOk => {}
        }
        Ok(())
    }
}

impl BroadcastNode {
    fn new(node_id: String, next_msg_id: usize) -> Self {
        Self {
            node_id,
            next_msg_id,
            messages: Vec::new(),
            topology: HashMap::new(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let mut stdin = stdin.lines();
    let mut output = std::io::stdout().lock();

    let node_id = send_init_message(&mut stdin, &mut output)?;
    let mut node = BroadcastNode::new(node_id, 1);

    main_loop(&mut node, &mut stdin, &mut output)
}
