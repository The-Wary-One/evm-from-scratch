use ruint::{aliases::U256, uint};
use sha3::Digest;
use thiserror::Error;

#[derive(Debug, Clone)]
/// State associated with an address.
pub enum Account {
    Empty,
    ExternallyOwned { balance: U256 },
    Contract { balance: U256, code: Vec<u8> },
}

pub static EMPTY_ACCOUNT: Account = Account::Empty;

impl Account {
    pub fn new(balance: Option<U256>, code: Option<Vec<u8>>) -> Self {
        match (balance, code) {
            (None, None) => Account::Empty,
            (Some(b), None) => Account::ExternallyOwned { balance: b },
            (None, Some(c)) => Account::Contract {
                balance: U256::ZERO,
                code: c,
            },
            (Some(b), Some(c)) => Account::Contract {
                balance: b,
                code: c,
            },
        }
    }

    pub fn balance(&self) -> &U256 {
        match self {
            Account::Empty => &U256::ZERO,
            Account::ExternallyOwned { balance } | Account::Contract { balance, .. } => balance,
        }
    }

    pub fn code(&self) -> &[u8] {
        match self {
            Account::Empty => &[],
            Account::ExternallyOwned { .. } => &[],
            Account::Contract { code, .. } => code,
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

    pub fn increase_balance(self, amount: &U256) -> Result<Self> {
        match self {
            Account::Empty => Ok(self),
            Account::ExternallyOwned { balance } => balance
                .checked_add(*amount)
                .map(|balance| Self::ExternallyOwned { balance })
                // Improbable.
                .ok_or(AccountError::TooMuchMoney),
            Account::Contract { balance, code } => balance
                .checked_add(*amount)
                .map(|balance| Self::Contract { balance, code })
                // Improbable.
                .ok_or(AccountError::TooMuchMoney),
        }
    }

    pub fn decrease_balance(self, amount: &U256) -> Result<Self> {
        match self {
            Account::Empty => Ok(self),
            Account::ExternallyOwned { balance } => balance
                .checked_sub(*amount)
                .map(|balance| Self::ExternallyOwned { balance })
                // Improbable.
                .ok_or(AccountError::NotEnoughBalance),
            Account::Contract { balance, code } => balance
                .checked_sub(*amount)
                .map(|balance| Self::Contract { balance, code })
                // Improbable.
                .ok_or(AccountError::NotEnoughBalance),
        }
    }
}

impl<'a> Default for Account {
    fn default() -> Self {
        EMPTY_ACCOUNT.clone()
    }
}

#[derive(Error, Debug)]
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
