use crate::tools::short_hash;
use log::info;
use petgraph::dot::{Config, Dot};
use petgraph::graph::NodeIndex;
use petgraph::Graph;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::process::Command;

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
    use log::info;
    use petgraph::dot::{Config, Dot};
    use petgraph::graph::NodeIndex;
    use petgraph::visit::IntoEdges;
    use petgraph::Graph;
    use rand::Rng;
    use std::fs::File;
    use std::io::Write;
    use std::process::Command;

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
