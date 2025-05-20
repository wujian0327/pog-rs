use petgraph::visit::Walker;
use pog::blockchain::block::Block;
use pog::blockchain::Blockchain;
use std::collections::{HashMap, HashSet};
use tokio::fs;

#[tokio::test]
async fn test_network_contribution() {
    let json_str = fs::read_to_string("blockchain.json").await.unwrap();
    let blocks: Vec<Block> = serde_json::from_str(&json_str).unwrap();
    let bc = Blockchain {
        blocks,
        transactions_hash_set: HashSet::new(),
    };
    let blocks = bc.get_last_slot_block();

    blocks.iter().for_each(|block| {
        println!("block: {:?}", block);
    })
}
