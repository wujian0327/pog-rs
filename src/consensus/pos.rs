use crate::blockchain::Blockchain;
use crate::consensus::ValidatorError::NOValidatorError;
use crate::consensus::{Validator, ValidatorError};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;

pub struct Pos;

impl Pos {
    pub fn select(
        validators: Vec<Validator>,
        combines_seeds: [u8; 32],
        _blockchain: Blockchain,
    ) -> Result<Validator, ValidatorError> {
        if validators.is_empty() {
            return Err(NOValidatorError);
        }
        // 计算总的权重
        let total_stake: f64 = validators.iter().map(|v| v.stake).sum();

        // 使用combine seed
        let mut rng = StdRng::from_seed(combines_seeds);

        let random_value = rng.gen_range(0.0..total_stake);

        // 选择符合条件的第一个验证者
        let mut accumulated_weight = 0f64;
        for validator in validators.clone() {
            accumulated_weight += validator.stake;
            if accumulated_weight > random_value {
                return Ok(validator);
            }
        }

        Err(NOValidatorError)
    }
}
