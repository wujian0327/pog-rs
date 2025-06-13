use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::ConsensusType;
use crate::network::graph::TopologyType;
use crate::network::message::Message;
use crate::network::node::{Neighbor, Node, NodeType};
use crate::network::world_state::WorldState;
use futures::future::join_all;
use log::info;
use rand::prelude::*;
use rand::thread_rng;
use rand_distr::{Distribution, Poisson};
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time;

pub mod graph;
pub mod message;
pub mod node;
pub mod world_state;

pub async fn start_network(
    node_num: u32,
    malicious_node_num: u32,
    fake_node_num: u32,
    trans_num_per_second: u32,
    consensus: ConsensusType,
    topology: TopologyType,
) {
    info!("Consensus Type is {}", consensus);

    //1. new blockchain
    let genesis_block = Block::gen_genesis_block();
    let bc = Blockchain::new(genesis_block.clone());
    info!("Generate genesis block");

    //2. world state
    let (mut world, world_sender, world_receiver) =
        WorldState::new(genesis_block, consensus, bc.clone());
    info!("Generate world state");

    //3. nodes
    let mut node_map: HashMap<String, Node> = (0..node_num + malicious_node_num)
        .map(|i| {
            if i < node_num {
                let node = Node::new(i, 0, 0, bc.clone(), world_sender.clone());
                node.simple_print();
                (node.get_address(), node)
            } else {
                let node = Node::new_with_sybil_nodes(
                    i,
                    0,
                    0,
                    bc.clone(),
                    world_sender.clone(),
                    fake_node_num as i32,
                );
                node.simple_print();
                (node.get_address(), node)
            }
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
    info!("Generate {} nodes", node_num + malicious_node_num);

    //4. gen the network graph
    let graph = match topology {
        TopologyType::ER => graph::random_er_graph(nodes_address.clone(), 0.2),
        TopologyType::BA => graph::random_graph_with_ba_network(nodes_address.clone()),
    };
    info!("Generate network graph[{}]", topology);
    tokio::time::sleep(Duration::from_secs(3)).await;

    //deal the node neighborhoods
    for edge in graph.edge_indices() {
        let (source, target) = graph.edge_endpoints(edge).unwrap();
        let from = graph[source].clone();
        let to = graph[target].clone();
        {
            let node_from = node_map.get_mut(&from).unwrap();
            if node_from
                .neighbors
                .iter()
                .find(|&x| x.address.clone() == to)
                .is_none()
            {
                node_from.neighbors.push(Neighbor::new(
                    *nodes_index.get(&to).unwrap(),
                    to.clone(),
                    nodes_sender.get(&to).unwrap().clone(),
                ));
            }
        }
        {
            let node_to = node_map.get_mut(&to).unwrap();
            if node_to
                .neighbors
                .iter()
                .find(|&x| x.address.clone() == from)
                .is_none()
            {
                node_to.neighbors.push(Neighbor::new(
                    *nodes_index.get(&from).unwrap(),
                    from.clone(),
                    nodes_sender.get(&from).unwrap().clone(),
                ));
            }
        }
    }

    //world should communicate with all node
    world.nodes_sender = nodes_sender.clone();
    node_map
        .iter()
        .for_each(|(_address, node)| match node.node_type {
            NodeType::Malicious => {
                // sybil的消息,由主节点控制
                node.sybil_nodes.iter().for_each(|sybil| {
                    world
                        .nodes_sender
                        .insert(sybil.get_address(), node.sender.clone());
                });
            }
            _ => {}
        });

    //start the world and all node
    let mut tasks = vec![];
    let t = tokio::spawn(async move {
        world.run(world_receiver).await;
        info!("World state running");
    });
    tasks.push(t);

    for (_, mut node) in node_map {
        let t = tokio::spawn(async move {
            info!("Node[{}] running", node.index);
            node.run().await;
        });
        tasks.push(t);
    }

    //become validator
    for (k, sender) in nodes_sender.clone() {
        info!("Node[{}] become validator", nodes_index.get(&k).unwrap());
        sender
            .send(Message::new_become_validator_msg(node_num as usize))
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
        info!(
            "Transaction Generator running, {} tx/s",
            trans_num_per_second
        );
        tg.run().await;
    });
    tasks.push(t);

    let mut printer = Printer::new(nodes_sender.clone(), Duration::from_secs(10));
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
            // 泊松分布生成器
            let poisson = Poisson::new(self.trans_num_per_interval as f64).unwrap();

            // 获取每秒生成的消息数
            let num_messages: usize = poisson.sample(&mut thread_rng()) as usize;

            for _ in 0..num_messages {
                let node = self.nodes_sender.iter().choose(&mut thread_rng());

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
            info!(
                "[{}]Transactions generated (λ={})",
                num_messages, self.trans_num_per_interval
            );
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

#[cfg(test)]
mod tests {
    use log::info;
    use rand::prelude::Distribution;
    use rand::thread_rng;
    use rand_distr::Poisson;
    use std::time::Duration;

    #[tokio::test]
    async fn poisson() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Info)
            .is_test(true)
            .try_init();

        let start_time = std::time::Instant::now();
        let poisson_lambda = 10.0; // λ = 10

        loop {
            // 泊松分布生成器
            let poisson = Poisson::new(poisson_lambda).unwrap();

            // 获取每秒生成的消息数
            let num_messages: usize = poisson.sample(&mut thread_rng()) as usize;

            // 输出生成的消息
            info!(
                "[{:.3}s] [{}]Transaction generated (λ={}/s)",
                start_time.elapsed().as_secs_f64(),
                num_messages,
                poisson_lambda
            );

            tokio::time::sleep(Duration::from_secs(1)).await;
            if start_time.elapsed().as_secs() > 5 {
                break;
            }
        }
    }
}
