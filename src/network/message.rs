use crate::blockchain::block::Block;
use crate::blockchain::blockchain::BlockChainError;
use crate::blockchain::path::TransactionPaths;
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
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    SEND_BLOCK,
    SEND_TRANSACTION_PATHS,
    GENERATE_BLOCK,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MessageType::SEND_BLOCK => {
                write!(f, "BLOCK")
            }
            MessageType::SEND_TRANSACTION_PATHS => {
                write!(f, "TRANSACTION_PATHS")
            }
            MessageType::GENERATE_BLOCK => {
                write!(f, "GENERATE_BLOCK")
            }
        }
    }
}
