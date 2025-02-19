use crate::blockchain::block::Block;
use crate::blockchain::path::AggregatedSignedPaths;
use crate::blockchain::Blockchain;
use crate::consensus::ValidatorError::NOValidatorError;
use crate::consensus::{Validator, ValidatorError};
use log::info;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

pub struct Pog;

impl Pog {
    pub fn select(
        validators: Vec<Validator>,
        combines_seeds: [u8; 32],
        blockchain: Blockchain,
    ) -> Result<Validator, ValidatorError> {
        let last_block = blockchain.get_last_block();
        let all = last_block.count_all_paths();
        let validators: Vec<Validator> = validators
            .iter()
            .map(|x| {
                let virtual_stake = x.stake
                    * (1.0
                        + 20f64 * last_block.count_node_paths(x.address.clone()) as f64
                            / all as f64);
                Validator {
                    address: x.address.clone(),
                    stake: virtual_stake,
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

        Err(NOValidatorError)
    }
}
