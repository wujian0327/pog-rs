use crate::blockchain::block::Block;
use crate::blockchain::path::TransactionPaths;
use crate::network::validator::{RandaoSeed, Validator};
use crate::network::world_state::SlotManager;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub msg_type: MessageType,
    pub data: Vec<u8>,
}

impl Message {
    pub fn new_block_msg(block: Block) -> Message {
        Message {
            msg_type: MessageType::SEND_BLOCK,
            data: block.to_json(),
        }
    }

    pub fn new_transaction_paths_msg(transaction_paths: TransactionPaths) -> Message {
        Message {
            msg_type: MessageType::SEND_TRANSACTION_PATHS,
            data: transaction_paths.to_json(),
        }
    }

    pub fn new_generate_block_msg() -> Message {
        Message {
            msg_type: MessageType::GENERATE_BLOCK,
            data: vec![],
        }
    }

    pub fn new_generate_transaction_path_msg(to: String) -> Message {
        Message {
            msg_type: MessageType::GENERATE_TRANSACTION_PATHS,
            data: to.into_bytes(),
        }
    }

    pub fn new_send_randao_seed_msg() -> Message {
        Message {
            msg_type: MessageType::SEND_RANDAO_SEED,
            data: vec![],
        }
    }

    pub fn new_receive_random_seed_msg(randao_seed: RandaoSeed) -> Message {
        Message {
            msg_type: MessageType::RECEIVE_RANDAO_SEED,
            data: randao_seed.to_json(),
        }
    }

    pub fn new_become_validator_msg() -> Message {
        Message {
            msg_type: MessageType::BECOME_VALIDATOR,
            data: vec![],
        }
    }

    pub fn new_receive_become_validator_msg(validator: Validator) -> Message {
        Message {
            msg_type: MessageType::RECEIVE_BECOME_VALIDATOR,
            data: validator.to_json(),
        }
    }

    pub fn new_update_slot_msg(slot: SlotManager) -> Message {
        Message {
            msg_type: MessageType::UPDATE_SLOT,
            data: slot.to_json(),
        }
    }

    pub fn new_print_blockchain_msg() -> Message {
        Message {
            msg_type: MessageType::PRINT_BLOCKCHAIN,
            data: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    SEND_BLOCK,
    SEND_TRANSACTION_PATHS,
    GENERATE_BLOCK,
    GENERATE_TRANSACTION_PATHS,
    SEND_RANDAO_SEED,
    RECEIVE_RANDAO_SEED,
    BECOME_VALIDATOR,
    RECEIVE_BECOME_VALIDATOR,
    UPDATE_SLOT,
    PRINT_BLOCKCHAIN,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MessageType::SEND_BLOCK => {
                write!(f, "SEND_BLOCK")
            }
            MessageType::SEND_TRANSACTION_PATHS => {
                write!(f, "SEND_TRANSACTION_PATHS")
            }
            MessageType::GENERATE_BLOCK => {
                write!(f, "GENERATE_BLOCK")
            }
            MessageType::SEND_RANDAO_SEED => {
                write!(f, "SEND_RANDAO_SEED")
            }
            MessageType::RECEIVE_RANDAO_SEED => {
                write!(f, "RECEIVE_RANDAO_SEED")
            }
            MessageType::BECOME_VALIDATOR => {
                write!(f, "BECOME_VALIDATOR")
            }

            MessageType::RECEIVE_BECOME_VALIDATOR => {
                write!(f, "RECEIVE_BECOME_VALIDATOR")
            }

            MessageType::UPDATE_SLOT => {
                write!(f, "UPDATE_SLOT")
            }
            MessageType::GENERATE_TRANSACTION_PATHS => {
                write!(f, "GENERATE_TRANSACTION_PATHS")
            }

            MessageType::PRINT_BLOCKCHAIN => {
                write!(f, "PRINT_BLOCKCHAIN")
            }
        }
    }
}
