use super::U256_DEFAULT;
use ruint::{aliases::U256, uint};
use sha3::Digest;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
/// State associated with an address.
pub enum Account {
    Empty,
    ExternallyOwned {
        nonce: usize,
        balance: U256,
    },
    Contract {
        nonce: usize,
        balance: U256,
        code: Box<[u8]>,
        storage: HashMap<U256, U256>,
    },
}

pub static EMPTY_ACCOUNT: Account = Account::Empty;

impl Account {
    pub fn new(balance: Option<U256>, code: Option<Box<[u8]>>) -> Self {
        log::trace!("new(): balance={:?}, code={:?}", balance, code);

        let res = match (balance, code) {
            (None, None) => Account::Empty,
            (Some(b), None) => Account::ExternallyOwned {
                nonce: 0,
                balance: b,
            },
            (None, Some(c)) | (Some(_), Some(c)) => Account::Contract {
                nonce: 0,
                balance: balance.unwrap_or_default(),
                code: c,
                storage: HashMap::new(),
            },
        };

        log::trace!("result: {:?}", res);
        res
    }

    pub fn nonce(&self) -> &usize {
        match self {
            Account::Empty => &0,
            Account::ExternallyOwned { nonce, .. } | Account::Contract { nonce, .. } => &nonce,
        }
    }

    pub fn balance(&self) -> &U256 {
        match self {
            Account::Empty => &U256::ZERO,
            Account::ExternallyOwned { balance, .. } | Account::Contract { balance, .. } => balance,
        }
    }

    pub fn increase_balance(self, amount: &U256) -> Result<Self> {
        match self {
            Account::Empty => Ok(Self::ExternallyOwned {
                nonce: 0,
                balance: *amount,
            }),
            Account::ExternallyOwned { balance, nonce } => balance
                .checked_add(*amount)
                .map(|balance| Self::ExternallyOwned { nonce, balance })
                // Improbable.
                .ok_or(AccountError::TooMuchMoney),
            Account::Contract {
                nonce,
                balance,
                code,
                storage,
            } => balance
                .checked_add(*amount)
                .map(|balance| Self::Contract {
                    nonce,
                    balance,
                    code,
                    storage,
                })
                // Improbable.
                .ok_or(AccountError::TooMuchMoney),
        }
    }

    pub fn decrease_balance(self, amount: &U256) -> Result<Self> {
        match self {
            Account::Empty => Err(AccountError::NotEnoughBalance),
            Account::ExternallyOwned { nonce, balance } => balance
                .checked_sub(*amount)
                .map(|balance| Self::ExternallyOwned { nonce, balance })
                // Improbable.
                .ok_or(AccountError::NotEnoughBalance),
            Account::Contract {
                nonce,
                balance,
                code,
                storage,
            } => balance
                .checked_sub(*amount)
                .map(|balance| Self::Contract {
                    nonce,
                    balance,
                    code,
                    storage,
                })
                // Improbable.
                .ok_or(AccountError::NotEnoughBalance),
        }
    }

    pub fn code(&self) -> &[u8] {
        match self {
            Account::Empty | Account::ExternallyOwned { .. } => &[],
            Account::Contract { code, .. } => code,
        }
    }

    pub fn set_code(self, code: Box<[u8]>) -> Result<Self> {
        match self {
            Account::Empty => Ok(Self::Contract {
                nonce: 0,
                balance: U256::ZERO,
                code,
                storage: HashMap::new(),
            }),
            Account::ExternallyOwned { nonce, balance } => Ok(Self::Contract {
                nonce,
                balance,
                code,
                storage: HashMap::new(),
            }),
            Account::Contract {
                nonce,
                balance,
                storage,
                ..
            } => Ok(Self::Contract {
                nonce,
                balance,
                code,
                storage,
            }),
        }
    }

    pub fn code_hash(&self) -> U256 {
        match self {
            Account::Empty => U256::ZERO,
            Account::ExternallyOwned { .. } => {
                uint!(0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470_U256)
            }
            Account::Contract { code, .. } => {
                let mut hasher = sha3::Keccak256::new();
                hasher.update(code);
                let hash = hasher.finalize();
                U256::try_from_be_slice(&hash[..]).expect("safe")
            }
        }
    }

    pub(crate) fn load(&self, key: &U256) -> &U256 {
        match self {
            Account::Contract { storage, .. } => {
                log::trace!("load(): key={:?}, storage={:?}", key, storage);
                let v = storage.get(key).unwrap_or_else(|| &U256_DEFAULT);
                log::trace!("result: key={:?}, value={:?}", key, v);
                v
            }
            _ => panic!("impossible"),
        }
    }

    pub(crate) fn store(&mut self, key: U256, value: U256) {
        log::trace!("store(): key={:?}, value={:?}", key, value);
        match self {
            Account::Contract { storage, .. } => {
                if value == U256_DEFAULT {
                    storage.remove(&key);
                } else {
                    storage.insert(key, value);
                }
                log::trace!("result: storage={:?}", storage);
            }
            _ => (),
        };
    }
}

impl<'a> Default for Account {
    fn default() -> Self {
        EMPTY_ACCOUNT.clone()
    }
}

#[derive(Error, Debug, Clone)]
pub enum AccountError {
    TooMuchMoney,
    NotEnoughBalance,
}

pub(super) type Result<T> = std::result::Result<T, AccountError>;

impl std::fmt::Display for AccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooMuchMoney => write!(f, "too much money"),
            Self::NotEnoughBalance => write!(f, "not enough balance"),
        }
    }
}
