use crate::blockchain::block::Block;
use crate::blockchain::Blockchain;
use crate::network::node::Node;
use crate::tools;
use crate::wallet::Wallet;
use clap::{Subcommand, ValueEnum};
use log::error;
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

pub mod pog;
pub mod pos;

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum ConsensusType {
    POS,
    POG,
}

impl Display for ConsensusType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ConsensusType::POS => {
                write!(f, "pos")
            }
            ConsensusType::POG => {
                write!(f, "pog")
            }
        }
    }
}

pub trait Consensus: Send + Sync {
    fn name(&self) -> &'static str;
    fn select_proposer(
        &mut self,
        validators: &[Validator],
        combines_seed: [u8; 32],
        blockchain: &Blockchain,
    ) -> Result<Validator, ValidatorError>;
    fn on_epoch_end(&mut self, blocks: &[Block]);
    fn apply_block_feedback(&mut self, _block: &Block) {}
    fn state_summary(&self) -> String {
        String::new()
    }
}

pub fn combine_seed(validators: Vec<Validator>, vdf_seeds: Vec<RandaoSeed>) -> [u8; 32] {
    let mut result = [0u8; 32];
    for v in vdf_seeds.clone() {
        if !validators
            .iter()
            .any(|validator| validator.address.eq(&v.address))
        {
            error!("Randao combine seed warning: this seed is not from validators");
            continue;
        }
        let valid = Wallet::verify_by_address(Vec::from(v.seed), v.signature, v.address);
        if valid {
            for i in 0..32 {
                result[i] ^= v.seed[i];
            }
        } else {
            error!("Randao combine seed warning: invalid seed");
        }
    }
    tools::Hasher::hash(Vec::from(result))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validator {
    pub address: String,
    pub stake: f64,
}

impl Validator {
    pub fn new(address: String, stake: f64) -> Self {
        Validator { address, stake }
    }

    pub fn from_node(node: Node, stake: f64) -> Self {
        Validator::new(node.wallet.address.clone(), stake)
    }

    pub fn from_json(json: Vec<u8>) -> Result<Validator, ValidatorError> {
        let randao_seed: Validator = serde_json::from_slice(json.as_slice())?;
        Ok(randao_seed)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug)]
pub enum ValidatorError {
    JSONError,
    NOValidatorError,
}
impl fmt::Display for ValidatorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ValidatorError::JSONError => {
                write!(f, "Invalid Json Error")
            }

            ValidatorError::NOValidatorError => {
                write!(f, "NoValidatorError")
            }
        }
    }
}
impl From<serde_json::error::Error> for ValidatorError {
    fn from(_: serde_json::error::Error) -> Self {
        ValidatorError::JSONError
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RandaoSeed {
    pub address: String,
    pub seed: [u8; 32],
    pub signature: String,
}

impl RandaoSeed {
    pub fn new(wallet: Wallet) -> Self {
        let seed = RandaoSeed::generate_seed();
        let signature = wallet.sign(Vec::from(seed));
        RandaoSeed {
            address: wallet.address,
            seed,
            signature,
        }
    }

    pub fn generate_seed() -> [u8; 32] {
        let mut rng = OsRng;
        let mut seed = [0u8; 32];
        rng.fill_bytes(&mut seed);
        seed
    }

    pub fn from_json(json: Vec<u8>) -> Result<RandaoSeed, ValidatorError> {
        let randao_seed: RandaoSeed = serde_json::from_slice(json.as_slice())?;
        Ok(randao_seed)
    }

    pub fn to_json(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
