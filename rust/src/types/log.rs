use ruint::aliases::U256;
use serde::Deserialize;

use super::Address;

#[derive(Debug)]
pub enum Log {
    Log0 {
        address: Address,
        data: Vec<u8>,
    },
    Log1 {
        address: Address,
        topics: [U256; 1],
        data: Vec<u8>,
    },
    Log2 {
        address: Address,
        topics: [U256; 2],
        data: Vec<u8>,
    },
    Log3 {
        address: Address,
        topics: [U256; 3],
        data: Vec<u8>,
    },
    Log4 {
        address: Address,
        topics: [U256; 4],
        data: Vec<u8>,
    },
}

impl Log {
    pub(crate) fn log0(address: Address, data: Vec<u8>) -> Log {
        Log::Log0 { address, data }
    }

    pub(crate) fn log1(address: Address, topics: [U256; 1], data: Vec<u8>) -> Log {
        Log::Log1 {
            address,
            topics,
            data,
        }
    }

    pub(crate) fn log2(address: Address, topics: [U256; 2], data: Vec<u8>) -> Log {
        Log::Log2 {
            address,
            topics,
            data,
        }
    }

    pub(crate) fn log3(address: Address, topics: [U256; 3], data: Vec<u8>) -> Log {
        Log::Log3 {
            address,
            topics,
            data,
        }
    }

    pub(crate) fn log4(address: Address, topics: [U256; 4], data: Vec<u8>) -> Log {
        Log::Log4 {
            address,
            topics,
            data,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct LogResult {
    address: Address,
    topics: Vec<U256>,
    #[serde(with = "hex::serde", default)]
    data: Vec<u8>,
}

impl From<Log> for LogResult {
    fn from(log: Log) -> Self {
        use super::Log::*;

        match log {
            Log0 { address, data } => LogResult {
                address,
                topics: vec![],
                data,
            },
            Log1 {
                address,
                topics,
                data,
            } => LogResult {
                address,
                topics: topics.to_vec(),
                data,
            },
            Log2 {
                address,
                topics,
                data,
            } => LogResult {
                address,
                topics: topics.to_vec(),
                data,
            },
            Log3 {
                address,
                topics,
                data,
            } => LogResult {
                address,
                topics: topics.to_vec(),
                data,
            },
            Log4 {
                address,
                topics,
                data,
            } => LogResult {
                address,
                topics: topics.to_vec(),
                data,
            },
        }
    }
}
