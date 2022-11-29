use ruint::aliases::U256;
use thiserror::Error;

#[derive(Debug, Clone)]
/// State associated with an address.
pub struct Account {
    balance: U256,
    code: Vec<u8>,
}

pub static ACCOUNT_DEFAULT: Account = Account {
    balance: U256::ZERO,
    code: vec![],
};

impl Account {
    pub fn new(balance: U256, code: Vec<u8>) -> Self {
        Self { balance, code }
    }

    pub fn balance(&self) -> &U256 {
        &self.balance
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn increase_balance(self, amount: &U256) -> Result<Self> {
        self.balance
            .checked_add(*amount)
            .map(|balance| Self {
                balance,
                code: self.code,
            })
            // Improbable.
            .ok_or(AccountError::TooMuchMoney)
    }
    pub fn decrease_balance(self, amount: &U256) -> Result<Self> {
        self.balance
            .checked_sub(*amount)
            .map(|balance| Self {
                balance,
                code: self.code,
            })
            .ok_or(AccountError::NotEnoughBalance)
    }
}

impl<'a> Default for Account {
    fn default() -> Self {
        ACCOUNT_DEFAULT.clone()
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
