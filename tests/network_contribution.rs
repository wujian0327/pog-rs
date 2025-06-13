use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

#[tokio::test]
async fn test_network_contribution() {
    let net_con_vec = read_network_contribution_from_file();
    let cv_list: Vec<f64> = net_con_vec
        .iter()
        .filter(|n| !n.is_empty())
        .map(|net_con| {
            let con: Vec<f64> = net_con.iter().map(|(_k, v)| v.clone()).collect();
            return cv(con);
        })
        .collect();
    let degree_map = read_topology_degree_from_file();
    let pearson_list: Vec<f64> = net_con_vec
        .iter()
        .filter(|n| !n.is_empty())
        .map(|net_con| {
            let mut contribution = vec![];
            let mut degree = vec![];
            net_con.iter().for_each(|(k, v)| {
                contribution.push(v.clone());
                degree.push(degree_map.get(&k.clone()).unwrap().clone() as f64);
            });
            let pearson = pearson_correlation(contribution, degree);
            return pearson;
        })
        .collect();

    println!("cv_list:{:?}", cv_list);
    println!("pearson_list:{:?}", pearson_list);
    println!("cv mean:{:?}", mean(cv_list));
    println!("pearson mean:{:?}", mean(pearson_list));
}

#[tokio::test]
async fn test_degree() {
    let degree = read_topology_degree_from_file();
    println!("degree:{:?}", degree);
}

fn read_network_contribution_from_file() -> Vec<HashMap<String, f64>> {
    let log = File::open("output.log").unwrap();
    let reader = BufReader::new(log);

    let mut net_con_list = Vec::new();
    for line in reader.lines() {
        let line = line.unwrap();
        if line.contains("Calculate network contribution") {
            let start = line.find('{').unwrap_or(0);
            let end = line.rfind('}').unwrap_or(line.len()) + 1;
            let json_str = &line[start..end];

            // 解析为 HashMap<String, f64>
            let result: HashMap<String, f64> = serde_json::from_str(json_str).unwrap();
            net_con_list.push(result);
        }
    }
    println!("net_con_list: {:?}", net_con_list);
    net_con_list
}

fn read_topology_degree_from_file() -> HashMap<String, usize> {
    let mut edges: HashMap<String, HashSet<String>> = HashMap::new();
    let mut graph = File::open("graph.json").unwrap();
    let mut buffer = String::new();
    graph.read_to_string(&mut buffer).unwrap();
    let edge_list: Vec<Vec<String>> = serde_json::from_str(&buffer).unwrap();
    edge_list.iter().for_each(|edge| {
        let node1 = edge.get(0).unwrap().clone();
        let node2 = edge.get(1).unwrap().clone();
        edges
            .entry(node1.clone())
            .or_insert_with(HashSet::new)
            .insert(node2.clone());
        edges
            .entry(node2.clone())
            .or_insert_with(HashSet::new)
            .insert(node1.clone());
    });
    let topology: HashMap<String, usize> = edges
        .iter()
        .map(|(node, edges)| (node.clone(), edges.len()))
        .collect();
    topology
}

fn cv(data: Vec<f64>) -> f64 {
    if data.is_empty() {
        return 0f64;
    }
    let m = mean(data.clone());
    if m == 0.0 {
        return 0f64;
    }
    let sd = std_dev(data, m);
    sd / m
}

fn mean(data: Vec<f64>) -> f64 {
    let sum: f64 = data.iter().sum();
    sum / (data.len() as f64)
}

fn std_dev(data: Vec<f64>, mean: f64) -> f64 {
    let var_sum: f64 = data
        .iter()
        .map(|value| {
            let diff = value - mean;
            diff * diff
        })
        .sum();
    let variance = var_sum / (data.len() as f64);
    variance.sqrt()
}

fn pearson_correlation(x: Vec<f64>, y: Vec<f64>) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0f64;
    }

    let mean_x = mean(x.clone());
    let mean_y = mean(y.clone());

    let mut numerator = 0.0;
    let mut denominator_x = 0.0;
    let mut denominator_y = 0.0;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        numerator += dx * dy;
        denominator_x += dx * dx;
        denominator_y += dy * dy;
    }

    let denominator = (denominator_x * denominator_y).sqrt();
    if denominator == 0.0 {
        0f64
    } else {
        numerator / denominator
    }
}

// fn read_ntd_from_file() -> HashMap<u64, u64> {
//     let log = File::open("output.log").unwrap();
//     let reader = BufReader::new(log);
//
//     let mut ntd_map = HashMap::new();
//     for line in reader.lines() {
//         let line = line.unwrap();
//         if line.contains("World State change slot") {
//             let re = Regex::new(r"epoch\[(\d+)\]\s+slot\[(\d+)\]\s+NTD\[(\d+)\]")
//                 .ok()
//                 .unwrap();
//             let caps = re.captures(&*line).unwrap();
//
//             let epoch = caps.get(1).unwrap().as_str().parse::<u64>().ok().unwrap();
//             let slot = caps.get(2).unwrap().as_str().parse::<u64>().ok().unwrap();
//             let ntd = caps.get(3).unwrap().as_str().parse::<u64>().ok().unwrap();
//             ntd_map.insert(epoch, ntd);
//             println!("epoch: {}, slot: {}, NTD: {}", epoch, slot, ntd);
//         }
//     }
//     ntd_map
// }
