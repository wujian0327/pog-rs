use log::error;
use pog::consensus::RandaoSeed;
use pog::tools;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

///
/// 准备测试一下，不同的真实权益(stake_real)占比和不同的网络贡献度(c_total)占比，对选举的影响
/// 这块用测试用例仿真，因为用区块链去跑，网络贡献度具有随机性，不好对比
/// 主要内容如下：
/// 1. c_total一致，随着stake_real占比的提高，选举的成功率的变化(这就是纯POS，应该是线性变化）
/// 2. stake_real一致，c_total占比提高，选举的成功率的变化(根据虚拟权益stake_virtual的公式，会缓慢提高)
/// 3. 两个结合在一起，就是c_total和stake_real一起提高，选举的成功率的变化()
///

#[tokio::test]
async fn test_stake_real_c_total_both_increase() {
    let mut result: Vec<Vec<f64>> = Vec::new();
    //真实权益增长，网络贡献度增长
    let mut validators = generate_validators();
    //每次选举rounds来计算选举的平均概率
    let rounds = 5000;
    for i in (0..500).step_by(2) {
        let name = validators[0].address.clone();
        validators[0].stake_real = i as f64;
        for j in 0..20 {
            validators[0].c_total_rate = j as f64 / 100f64;
            validators[0].update();
            let total_stake_real: f64 = validators.iter().map(|v| v.stake_real).sum();
            let total_stake_virtual: f64 = validators.iter().map(|v| v.stake_virtual).sum();
            let counter = select_rounds(validators.clone(), rounds);
            let stake_read_rate = validators[0].stake_real / total_stake_real;
            let stake_virtual_rate = validators[0].stake_virtual / total_stake_virtual;
            let minner_counts = counter.get(&name).unwrap_or_else(|| &0).clone();
            let minner_rate = minner_counts as f64 / rounds as f64;
            println!(
                "name: {:?}, stake_real:{}, stake_virtual: {:.2}, stake_real_rate:{:.2}%, c_total_rate:{:.2}%, stake_virtual_rate: {:.2}%, minner_rate: {:.2}%",
                name, validators[0].stake_real, validators[0].stake_virtual,stake_read_rate * 100f64, validators[0].c_total_rate* 100f64,stake_virtual_rate* 100f64, minner_rate* 100f64
            );
            result.push(vec![
                stake_read_rate,
                validators[0].c_total_rate,
                minner_rate,
            ])
        }
    }
    let path = "selection.json";
    let json = serde_json::to_string_pretty(&result).unwrap();
    match tokio::fs::write(path, json).await {
        Ok(_) => {}
        Err(e) => {
            error!("Error writing json file: {}", e);
        }
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: String,
    pub stake_real: f64,
    pub stake_virtual: f64,
    pub c_total_rate: f64,
}

impl Validator {
    fn new(address: String, stake_real: f64) -> Self {
        Validator {
            address,
            stake_real,
            stake_virtual: stake_real,
            c_total_rate: 0.0,
        }
    }

    fn update(&mut self) {
        self.stake_virtual = self.stake_real * (1.0 + 5f64 * self.c_total_rate);
    }
}

fn generate_validators() -> Vec<Validator> {
    let mut validators = Vec::<Validator>::new();
    for i in 0..100 {
        let validator = Validator::new(format!("node_{}", i), 32f64);
        validators.push(validator);
    }
    validators
}

fn generate_randao(v: Validator) -> RandaoSeed {
    RandaoSeed {
        address: v.address,
        seed: RandaoSeed::generate_seed(),
        signature: "".to_string(),
    }
}

fn select_rounds(validators: Vec<Validator>, rounds: i32) -> HashMap<String, i32> {
    let mut counter: HashMap<String, i32> = HashMap::new();
    for i in 0..rounds {
        let randao_seeds = validators
            .iter()
            .map(|v| generate_randao(v.clone()))
            .collect::<Vec<_>>();
        let global_seed = combine_seed_no_check(randao_seeds);
        let winner = select(validators.clone(), global_seed);
        if let Some(n) = counter.get(&winner) {
            counter.insert(winner, n + 1);
        } else {
            counter.insert(winner, 0);
        }
    }
    counter
}

fn select(validators: Vec<Validator>, combines_seeds: [u8; 32]) -> String {
    let total_stake: f64 = validators.iter().map(|v| v.stake_virtual).sum();

    // 使用combine seed
    let mut rng = StdRng::from_seed(combines_seeds);

    let random_value = rng.gen_range(0.0..total_stake);

    // 选择符合条件的第一个验证者
    let mut accumulated_weight = 0f64;
    for validator in validators.clone() {
        accumulated_weight += validator.stake_virtual;
        if accumulated_weight > random_value {
            return validator.address;
        }
    }
    panic!("No validator found");
}

fn combine_seed_no_check(seeds: Vec<RandaoSeed>) -> [u8; 32] {
    let mut result = [0u8; 32];
    for v in seeds.clone() {
        for i in 0..32 {
            result[i] ^= v.seed[i];
        }
    }
    tools::Hasher::hash(Vec::from(result))
}
