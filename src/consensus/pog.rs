use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::consensus::{Consensus, Validator, ValidatorError};
use log::{debug, info, trace};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

pub struct PogConsensus {
    ntd: usize,
}

impl PogConsensus {
    pub fn new(initial_ntd: usize) -> Self {
        PogConsensus { ntd: initial_ntd }
    }

    fn k(&self) -> usize {
        self.ntd * 2
    }

    fn select_internal(
        &self,
        validators: Vec<Validator>,
        combines_seeds: [u8; 32],
        blockchain: Blockchain,
    ) -> Result<Validator, ValidatorError> {
        let k = self.k();
        let last_block = blockchain.get_last_block();
        let paths = last_block.get_all_paths();
        let c_n = self.cal_network_contribution(paths, validators.clone());
        info!(
            "Calculate network contribution: {}",
            serde_json::to_string(&c_n)?
        );
        let s_real_map: HashMap<String, f64> = validators
            .iter()
            .map(|x| (x.address.to_string(), x.stake))
            .collect();
        let s_virtual_map = self.cal_virtual_stake(s_real_map, c_n);
        info!(
            "Calculate virtual stake: {}",
            serde_json::to_string(&s_virtual_map)?
        );
        let validators: Vec<Validator> = validators
            .iter()
            .map(|x| {
                let virtual_stake = s_virtual_map.get(&x.address.to_string()).unwrap();
                Validator {
                    address: x.address.clone(),
                    stake: *virtual_stake,
                }
            })
            .collect();

        // 计算总的权重
        let total_stake: f64 = validators.clone().iter().map(|v| v.stake).sum();

        // 使用combine seed
        let mut rng = StdRng::from_seed(combines_seeds);

        let random_value = rng.gen_range(0.0..total_stake);

        // 选择符合条件的第一个验证者
        let mut accumulated_weight = 0f64;
        for validator in validators.clone() {
            accumulated_weight += validator.stake;
            if accumulated_weight > random_value {
                info!(
                    "Miner {} has virtual stake {}",
                    validator.address, validator.stake
                );
                return Ok(validator);
            }
        }

        Err(ValidatorError::NOValidatorError)
    }

    fn cal_network_contribution(
        &self,
        paths: Vec<Vec<String>>,
        validators: Vec<Validator>,
    ) -> HashMap<String, f64> {
        let mut c_n: HashMap<String, f64> = HashMap::new();

        paths.iter().for_each(|p| {
            if p.is_empty() {
                return;
            }
            //去掉miner
            let p = p[..p.len() - 1].to_vec();
            let c_p = if p.len() <= self.ntd {
                1.0
            } else {
                1.0 / (1 + p.len() - self.ntd) as f64
            };
            let sum_s = p
                .iter()
                .map(|x| Self::get_real_stake(x.clone(), validators.clone()))
                .sum::<f64>();
            p.iter().for_each(|x| {
                let c_n_p = c_p * Self::get_real_stake(x.clone(), validators.clone()) / sum_s;
                *c_n.entry(x.clone()).or_insert(0.0) += c_n_p;
            });
        });
        c_n
    }

    fn get_real_stake(n: String, validators: Vec<Validator>) -> f64 {
        if let Some(v) = validators.iter().find(|x| x.address == n) {
            return v.stake;
        }
        0f64
    }

    fn cal_virtual_stake(
        &self,
        real_stake_map: HashMap<String, f64>,
        c_n: HashMap<String, f64>,
    ) -> HashMap<String, f64> {
        let k = self.k();
        let c_sum = c_n.iter().map(|(_n, v)| v).sum::<f64>();
        let mut c_mao = HashMap::new();
        c_n.iter().for_each(|(k, v)| {
            c_mao.insert(k.clone(), v / c_sum);
        });
        trace!("c_mao: {:#?}", c_mao);
        let s_sum = real_stake_map.iter().map(|(_n, v)| v).sum::<f64>();
        let mut s_mao = HashMap::new();
        real_stake_map.iter().for_each(|(k, v)| {
            s_mao.insert(k.clone(), v / s_sum);
        });
        trace!("s_mao: {:#?}", s_mao);
        let s_phi: HashMap<String, f64> = s_mao
            .iter()
            .map(|(n, v)| {
                let mut x = 1f64 / 2f64 - v;
                if x < 0f64 {
                    x = 0f64;
                }
                return (n.clone(), x);
            })
            .collect();
        trace!("s_phi: {:#?}", s_phi);
        let s_virtual_map: HashMap<String, f64> = real_stake_map
            .iter()
            .map(|(n, v)| {
                let c = c_mao.get(n).unwrap_or(&0f64);
                let s_p = s_phi.get(n).unwrap_or(&0f64);
                let s_v = v * (1f64 + k as f64 * c * s_p);
                return (n.clone(), s_v);
            })
            .collect();
        trace!("s_virtual_map: {:#?}", s_virtual_map);
        s_virtual_map
    }
}

impl Consensus for PogConsensus {
    fn name(&self) -> &'static str {
        "POG"
    }

    fn select_proposer(
        &mut self,
        validators: &[Validator],
        combines_seed: [u8; 32],
        blockchain: &Blockchain,
    ) -> Result<Validator, ValidatorError> {
        self.select_internal(validators.to_vec(), combines_seed, blockchain.clone())
    }

    fn on_epoch_end(&mut self, blocks: &[Block]) {
        let paths: Vec<Vec<String>> = blocks.iter().flat_map(|b| b.get_all_paths()).collect();
        self.adjust_ntd(&paths);
    }

    fn state_summary(&self) -> String {
        format!("pog(ntd={})", self.ntd)
    }
}

impl PogConsensus {
    fn adjust_ntd(&mut self, paths: &[Vec<String>]) {
        if paths.is_empty() {
            return;
        }
        let p_ave = paths
            .iter()
            .map(|path| path.len().saturating_sub(1))
            .sum::<usize>() as f64
            / paths.len() as f64;
        let target = p_ave.ceil() as usize;
        if self.ntd > target {
            self.ntd -= 1;
        } else if self.ntd < target {
            self.ntd += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::blockchain::path::{AggregatedSignedPaths, TransactionPaths};
    use crate::blockchain::transaction::Transaction;
    use crate::consensus::pog::PogConsensus;
    use crate::consensus::Validator;
    use crate::wallet::Wallet;
    use log::info;
    use std::collections::HashMap;

    #[tokio::test]
    async fn cal_network_contribution() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .is_test(true)
            .try_init();

        let wallet = Wallet::new();
        let wallet2 = Wallet::new();
        let wallet3 = Wallet::new();
        let miner = Wallet::new();
        let transaction = Transaction::new("123".to_string(), 32, wallet.clone());
        let mut transaction_paths = TransactionPaths::new(transaction.clone());
        transaction_paths.add_path(wallet2.address.clone(), wallet.clone());
        transaction_paths.add_path(wallet3.address.clone(), wallet2.clone());
        transaction_paths.add_path(miner.address.clone(), wallet3.clone());

        //check aggregated_signed_paths
        let aggregated_signed_paths =
            AggregatedSignedPaths::from_transaction_paths(transaction_paths);

        let paths = vec![aggregated_signed_paths.paths];
        let v1 = Validator::new(wallet.address, 1f64);
        let v2 = Validator::new(wallet2.address, 2f64);
        let v3 = Validator::new(wallet3.address, 3f64);
        let miner = Validator::new(miner.address, 4f64);
        let validators = vec![v1, v2, v3, miner];
        let pog = PogConsensus::new(3);
        let c_n = pog.cal_network_contribution(paths, validators.clone());
        info!("c_n: {:#?}", c_n);

        let s_real_map: HashMap<String, f64> = validators
            .iter()
            .map(|x| (x.address.to_string(), x.stake))
            .collect();
        let s_v = pog.cal_virtual_stake(s_real_map, c_n);
        info!("s_v: {:#?}", s_v);
    }
}
