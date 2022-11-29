use crate::types::{Account, AccountError, Address, ACCOUNT_DEFAULT};
use ruint::aliases::U256;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone)]
/// Contains all information that is preserved between transactions.
pub struct State {
    accounts: HashMap<Address, Account>,
}

impl<'a> State {
    pub fn new(accounts: HashMap<Address, Account>) -> Self {
        Self { accounts }
    }

    pub(crate) fn get_account(&self, addr: &Address) -> &Account {
        self.accounts.get(addr).unwrap_or_else(|| &ACCOUNT_DEFAULT)
    }

    fn update_account(
        &mut self,
        addr: &Address,
        f: impl FnOnce(Account) -> Result<Account>,
    ) -> Result<()> {
        let updated = f(self.get_account(addr).clone())?;
        self.accounts.insert(addr.clone(), updated);
        Ok(())
    }

    pub(crate) fn send_eth(&mut self, from: &Address, to: &Address, amount: &U256) -> Result<()> {
        self.update_account(from, |from_account| {
            from_account
                .decrease_balance(amount)
                .map_err(StateError::AccountError)
        })
        .and_then(|_| {
            self.update_account(to, |to_account| {
                to_account
                    .increase_balance(amount)
                    .map_err(StateError::AccountError)
            })
        })
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            accounts: HashMap::default(),
        }
    }
}

#[derive(Error, Debug)]
pub enum StateError {
    #[error(transparent)]
    AccountError(#[from] AccountError),
}

pub type Result<T> = std::result::Result<T, StateError>;

//impl<'a> Display for StateError {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        match self {
//            Self::AccountError(e) => e.fmt(f),
//        }
//    }
//}
