use crate::tools::short_hash;
use log::info;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use tokio::join;

//Barabási–Albert 模型，用于生成无标度网络
struct BANetwork {
    adjacency: HashMap<usize, HashSet<usize>>, // 邻接表：节点 -> 连接的节点
    degrees: Vec<usize>,                       // 节点度数列表（索引为节点ID）
    total_edges: usize,                        // 总边数的两倍（无向图）
}

impl BANetwork {
    fn new(m0: usize) -> Self {
        let mut adjacency = HashMap::new();
        let mut degrees = vec![0; m0];

        // 初始化为全连通
        for i in 0..m0 {
            let mut neighbors = HashSet::new();
            for j in 0..m0 {
                if i != j {
                    neighbors.insert(j);
                }
            }
            adjacency.insert(i, neighbors);
            degrees[i] = m0 - 1; // 初始每个节点度数 = m0-1
        }

        BANetwork {
            adjacency,
            degrees,
            total_edges: m0 * (m0 - 1), // 总边数（无向图每条边算两次）
        }
    }

    // 选择要连接的节点（返回选中的节点ID）
    fn choose_node(&self) -> usize {
        let mut rng = rand::thread_rng();
        let mut sum = 0;
        let target = rng.gen_range(0..self.total_edges);

        // 遍历所有节点，通过度数累计概率
        for (node, &degree) in self.degrees.iter().enumerate() {
            sum += degree;
            if sum > target {
                return node;
            }
        }
        panic!("Selection failed"); // 理论上不应触发
    }

    fn add_node(&mut self, m: usize) {
        let new_node = self.degrees.len();
        self.adjacency.insert(new_node, HashSet::new());
        self.degrees.push(0);

        for _ in 0..m {
            let target = self.choose_node();
            self.adjacency.get_mut(&new_node).unwrap().insert(target);
            self.adjacency.get_mut(&target).unwrap().insert(new_node);
            self.degrees[new_node] += 1;
            self.degrees[target] += 1;
            self.total_edges += 2; // 双向边，各加1度
        }
    }

    fn generate_ba_network(n_nodes: usize, m0: usize, m: usize) -> BANetwork {
        assert!(m <= m0, "m must be ≤ m0");
        let mut network = BANetwork::new(m0);

        for _ in m0..n_nodes {
            network.add_node(m);
        }
        network
    }
}

pub fn random_graph(nodes_address: Vec<String>, probability: f64) -> Graph<String, ()> {
    let mut graph = Graph::<String, ()>::new();
    let mut rng = rand::thread_rng();

    let nodes: Vec<NodeIndex> = nodes_address
        .iter()
        .map(|i| graph.add_node(i.clone()))
        // .map(|i| graph.add_node(short_hash(i.to_string())))
        .collect();

    // 以 p 的概率生成边
    for i in 0..nodes.len() {
        for j in (i + 1)..nodes.len() {
            if rng.gen::<f64>() < probability {
                graph.add_edge(nodes[i], nodes[j], ());
            }
        }
    }

    let mut graph_clone = graph.clone();
    graph_clone.node_indices().for_each(|i| {
        let node = graph_clone.node_weight_mut(i).unwrap();
        *node = short_hash(node.clone());
    });
    // 打印图的 DOT 表示
    info!(
        "{:?}",
        Dot::with_config(&graph_clone, &[Config::EdgeNoLabel])
    );
    print_graph(&graph_clone);
    graph
}

pub fn random_graph_with_ba_netwotk(nodes_address: Vec<String>) -> Graph<String, ()> {
    let node_number = nodes_address.len();
    let ba_network = BANetwork::generate_ba_network(node_number, 3, 2);
    let adj = ba_network.adjacency;

    let mut graph = Graph::<String, ()>::new();
    let mut node_map = HashMap::new();
    for (x, _) in adj.clone() {
        let node = graph.add_node(nodes_address[x].clone());
        node_map.insert(nodes_address[x].clone(), node);
    }
    for (x, edge) in adj {
        let from = node_map.get(&nodes_address[x].clone()).unwrap();
        for y in edge {
            let to = node_map.get(&nodes_address[y].clone()).unwrap();
            graph.add_edge(from.clone(), to.clone(), ());
        }
    }
    let mut graph_clone = graph.clone();
    graph_clone.node_indices().for_each(|i| {
        let node = graph_clone.node_weight_mut(i).unwrap();
        *node = short_hash(node.clone());
    });
    // 打印图的 DOT 表示
    info!(
        "{:?}",
        Dot::with_config(&graph_clone, &[Config::EdgeNoLabel])
    );
    print_graph(&graph_clone);
    graph
}

pub fn print_graph(graph: &Graph<String, ()>) {
    {
        let dot_string = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
        let mut file = File::create("graph.dot").expect("Unable to create file");
        file.write_all(dot_string.as_bytes())
            .expect("Unable to write data to file");

        file.flush().expect("Unable to flush data to file");
        println!("DOT format written to 'graph.dot'");
    }

    let output = Command::new("cmd")
        .arg("/C")
        .arg("dot")
        .arg("-Tpng")
        .arg("graph.dot")
        .arg("-o")
        .arg("graph.png")
        .output();

    match output {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to eprint graph: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::network::graph::{print_graph, BANetwork};
    use log::info;
    use petgraph::dot::{Config, Dot};
    use petgraph::graph::NodeIndex;
    use petgraph::visit::IntoEdges;
    use petgraph::Graph;
    use rand::Rng;
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;

    #[test]
    fn BANetwork() {
        let ba_network = BANetwork::generate_ba_network(100, 3, 2);
        let adj = ba_network.adjacency;

        let mut graph = Graph::<String, ()>::new();
        let mut node_map = HashMap::new();
        for (x, _) in adj.clone() {
            let node = graph.add_node(x.to_string());
            node_map.insert(x, node);
        }
        for (x, edge) in adj {
            let from = node_map.get(&x).unwrap();
            for y in edge {
                let to = node_map.get(&y).unwrap();
                graph.add_edge(from.clone(), to.clone(), ());
            }
        }
        print_graph(&graph);
    }

    #[test]
    fn graph() {
        let mut graph = Graph::<&str, &str>::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "edge_1");
        graph.add_edge(b, c, "edge_2");

        // 打印图的 DOT 表示
        info!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    }

    #[test]
    fn random_graph() {
        let mut graph = Graph::<String, ()>::new();
        let mut rng = rand::thread_rng();

        // 随机生成 5 个节点
        let nodes: Vec<NodeIndex> = (0..10)
            .map(|i| graph.add_node(format!("node{}", i)))
            .collect();

        // 以 30% 的概率生成边
        let probability = 0.3;
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                // 只检查一半的组合，避免重复添加边
                if rng.gen::<f64>() < probability {
                    // 生成 [0.0, 1.0) 范围的随机浮点数
                    graph.add_edge(nodes[i], nodes[j], ());
                }
            }
        }

        // 打印图的 DOT 表示
        info!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));

        // 打印图的节点和边
        for node in graph.node_indices() {
            info!("Node: {:?}", graph[node]);
        }

        for edge in graph.edge_indices() {
            let (source, target) = graph.edge_endpoints(edge).unwrap();
            info!("Edge: {} -> {}", graph[source], graph[target]);
        }
    }

    #[test]
    fn print_with_dot() {
        let mut graph = Graph::<&str, &str>::new();

        let a = graph.add_node("A");
        let b = graph.add_node("B");
        let c = graph.add_node("C");

        graph.add_edge(a, b, "edge_1");
        graph.add_edge(b, c, "edge_2");

        {
            let dot_string = format!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
            let mut file = File::create("graph.dot").expect("Unable to create file");
            file.write_all(dot_string.as_bytes())
                .expect("Unable to write data to file");

            file.flush().expect("Unable to flush data to file");
            println!("DOT format written to 'graph.dot'");
        }

        let output = Command::new("cmd")
            .arg("/C")
            .arg("dot")
            .arg("-Tpng")
            .arg("graph.dot")
            .arg("-o")
            .arg("graph.png")
            .output();

        match output {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    println!("Output:\n{}", String::from_utf8_lossy(&output.stdout));
                }
                if !output.stderr.is_empty() {
                    eprintln!("Error:\n{}", String::from_utf8_lossy(&output.stderr));
                }
            }
            Err(e) => {
                eprintln!("Failed to execute command: {}", e);
            }
        }
    }
}
