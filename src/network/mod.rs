use crate::blockchain::block::Block;
use crate::blockchain::blockchain::Blockchain;
use crate::network::message::Message;
use crate::network::node::{Neighbor, Node};
use crate::network::world_state::WorldState;
use futures::future::join_all;
use futures::FutureExt;
use log::info;
use rand::seq::IteratorRandom;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time;

mod graph;
mod message;
pub mod node;
pub mod validator;
mod world_state;

pub async fn start_network(node_num: u32, trans_num_per_second: u32) {
    //1. new blockchain
    let genesis_block = Block::gen_genesis_block();
    let bc = Blockchain::new(genesis_block.clone());
    info!("generate genesis block");

    //2. world state
    let (mut world, world_sender, world_receiver) = WorldState::new(genesis_block);
    info!("generate world state");

    //3. nodes
    let mut node_map: HashMap<String, Node> = (0..node_num)
        .map(|i| {
            let node = Node::new(i, 0, 0, bc.clone(), world_sender.clone());
            (node.get_address(), node)
        })
        .collect();
    let nodes_sender: HashMap<String, Sender<Message>> = node_map
        .iter()
        .map(|(address, node)| (address.clone(), node.sender.clone()))
        .collect();
    let nodes_index: HashMap<String, u32> = node_map
        .iter()
        .map(|(address, node)| (address.clone(), node.index))
        .collect();
    let nodes_address: Vec<String> = node_map.keys().cloned().collect();
    info!("generate nodes");

    //4. gen the network graph
    // let graph = graph::random_graph(nodes_address.clone(), 0.3);
    let graph = graph::random_graph_with_ba_netwotk(nodes_address.clone());
    info!("generate network graph");

    //deal the node neighborhoods
    for edge in graph.edge_indices() {
        let (source, target) = graph.edge_endpoints(edge).unwrap();
        let from = graph[source].clone();
        let to = graph[target].clone();
        {
            let node_from = node_map.get_mut(&from).unwrap();
            node_from.neighbors.push(Neighbor::new(
                *nodes_index.get(&to).unwrap(),
                to.clone(),
                nodes_sender.get(&to).unwrap().clone(),
            ));
        }
        {
            let node_to = node_map.get_mut(&to).unwrap();
            node_to.neighbors.push(Neighbor::new(
                *nodes_index.get(&from).unwrap(),
                from.clone(),
                nodes_sender.get(&from).unwrap().clone(),
            ));
        }
    }

    //world should communicate with all node
    world.nodes_sender = nodes_sender.clone();

    //start the world and all node
    let mut tasks = vec![];
    let t = tokio::spawn(async move {
        world.run(world_receiver).await;
        info!("World state running");
    });
    tasks.push(t);

    for (_, mut node) in node_map {
        let t = tokio::spawn(async move {
            node.run().await;
            info!("Node[{}] running", node.index);
        });
        tasks.push(t);
    }

    //become validator
    for (_, sender) in nodes_sender.clone() {
        sender
            .send(Message::new_become_validator_msg())
            .await
            .unwrap();
    }

    let mut tg = TransactionGenerator::new(
        nodes_sender.clone(),
        nodes_address.clone(),
        Duration::from_secs(1),
        trans_num_per_second,
    );

    let t = tokio::spawn(async move {
        tg.run().await;
    });
    tasks.push(t);

    let mut printer = Printer::new(nodes_sender.clone(), Duration::from_secs(5));
    let t = tokio::spawn(async move {
        printer.run().await;
    });
    tasks.push(t);

    let _ = join_all(tasks).await;
}

struct TransactionGenerator {
    nodes_sender: HashMap<String, Sender<Message>>,
    nodes_address: Vec<String>,
    time_interval: Duration,
    trans_num_per_interval: u32,
}

impl TransactionGenerator {
    fn new(
        nodes_sender: HashMap<String, Sender<Message>>,
        nodes_address: Vec<String>,
        time_interval: Duration,
        trans_num_per_interval: u32,
    ) -> TransactionGenerator {
        TransactionGenerator {
            nodes_sender,
            nodes_address,
            time_interval,
            trans_num_per_interval,
        }
    }

    async fn run(&mut self) {
        let mut interval = time::interval(self.time_interval);

        loop {
            interval.tick().await;
            info!(
                "Transaction Generator send {} transaction in {}s",
                self.trans_num_per_interval,
                self.time_interval.as_secs()
            );
            for _ in 0..self.trans_num_per_interval {
                let node = self.nodes_sender.iter().choose(&mut rand::thread_rng());

                if let Some(node) = node {
                    let to = self
                        .nodes_address
                        .iter()
                        .filter(|x| **x != node.0.clone())
                        .choose(&mut rand::thread_rng())
                        .unwrap();
                    node.1
                        .send(Message::new_generate_transaction_path_msg(to.clone()))
                        .await
                        .unwrap();
                }
            }
        }
    }
}

struct Printer {
    nodes_sender: HashMap<String, Sender<Message>>,
    interval: Duration,
}

impl Printer {
    fn new(nodes_sender: HashMap<String, Sender<Message>>, interval: Duration) -> Printer {
        Printer {
            nodes_sender,
            interval,
        }
    }

    async fn run(&mut self) {
        let mut interval = time::interval(self.interval);
        loop {
            interval.tick().await;

            let node = self.nodes_sender.iter().choose(&mut rand::thread_rng());
            node.unwrap()
                .1
                .send(Message::new_print_blockchain_msg())
                .await
                .unwrap();
        }
    }
}
