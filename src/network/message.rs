use crate::blockchain::block::Block;
use crate::blockchain::path::TransactionPaths;
use crate::consensus::{RandaoSeed, Validator};
use crate::network::world_state::SlotManager;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub msg_type: MessageType,
    pub data: Vec<u8>,
    pub from: String,
}

impl Message {
    pub fn new_block_msg(block: Block, from: String) -> Message {
        Message {
            msg_type: MessageType::SendBlock,
            data: block.to_json(),
            from,
        }
    }

    pub fn new_transaction_paths_msg(transaction_paths: TransactionPaths, from: String) -> Message {
        Message {
            msg_type: MessageType::SendTransactionPaths,
            data: transaction_paths.to_json(),
            from,
        }
    }

    pub fn new_generate_block_msg() -> Message {
        Message {
            msg_type: MessageType::GenerateBlock,
            data: vec![],
            from: "".to_string(),
        }
    }

    pub fn new_generate_transaction_path_msg(to: String) -> Message {
        Message {
            msg_type: MessageType::GenerateTransactionPaths,
            data: to.into_bytes(),
            from: "".to_string(),
        }
    }

    pub fn new_send_randao_seed_msg() -> Message {
        Message {
            msg_type: MessageType::SendRandaoSeed,
            data: vec![],
            from: "".to_string(),
        }
    }

    pub fn new_receive_random_seed_msg(randao_seed: RandaoSeed) -> Message {
        Message {
            msg_type: MessageType::ReceiveRandaoSeed,
            data: randao_seed.to_json(),
            from: "".to_string(),
        }
    }

    pub fn new_become_validator_msg(stake_json: Vec<u8>) -> Message {
        Message {
            msg_type: MessageType::BecomeValidator,
            data: stake_json,
            from: "".to_string(),
        }
    }

    pub fn new_receive_become_validator_msg(validator: Validator) -> Message {
        Message {
            msg_type: MessageType::ReceiveBecomeValidator,
            data: validator.to_json(),
            from: "".to_string(),
        }
    }

    pub fn new_update_slot_msg(slot: SlotManager) -> Message {
        Message {
            msg_type: MessageType::UpdateSlot,
            data: slot.to_json(),
            from: "".to_string(),
        }
    }

    pub fn new_print_blockchain_msg() -> Message {
        Message {
            msg_type: MessageType::PrintBlockchain,
            data: vec![],
            from: "".to_string(),
        }
    }

    pub fn new_request_block_sync_msg(last_block_index: u64, from: String) -> Message {
        Message {
            msg_type: MessageType::RequestBlockSync,
            data: last_block_index.to_le_bytes().to_vec(),
            from: from,
        }
    }

    pub fn new_response_block_sync_msg(blocks: Vec<Block>, from: String) -> Message {
        let blocks_json = serde_json::to_string(&blocks).unwrap_or_default();
        Message {
            msg_type: MessageType::ResponseBlockSync,
            data: blocks_json.into_bytes(),
            from,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    SendBlock,
    SendTransactionPaths,
    GenerateBlock,
    GenerateTransactionPaths,
    SendRandaoSeed,
    ReceiveRandaoSeed,
    BecomeValidator,
    ReceiveBecomeValidator,
    UpdateSlot,
    PrintBlockchain,
    RequestBlockSync,
    ResponseBlockSync,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MessageType::SendBlock => {
                write!(f, "SendBlock")
            }
            MessageType::SendTransactionPaths => {
                write!(f, "SendTransactionPaths")
            }
            MessageType::GenerateBlock => {
                write!(f, "GenerateBlock")
            }
            MessageType::SendRandaoSeed => {
                write!(f, "SendRandaoSeed")
            }
            MessageType::ReceiveRandaoSeed => {
                write!(f, "ReceiveRandaoSeed")
            }
            MessageType::BecomeValidator => {
                write!(f, "BecomeValidator")
            }

            MessageType::ReceiveBecomeValidator => {
                write!(f, "ReceiveBecomeValidator")
            }

            MessageType::UpdateSlot => {
                write!(f, "UpdateSlot")
            }
            MessageType::GenerateTransactionPaths => {
                write!(f, "GenerateTransactionPaths")
            }

            MessageType::PrintBlockchain => {
                write!(f, "PrintBlockchain")
            }
            MessageType::RequestBlockSync => {
                write!(f, "RequestBlockSync")
            }
            MessageType::ResponseBlockSync => {
                write!(f, "ResponseBlockSync")
            }
        }
    }
}
