use crate::types::{Account, AccountError, Address, EMPTY_ACCOUNT};
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
        log::trace!("new(): accounts={:?}", accounts);
        Self { accounts }
    }

    pub(crate) fn get_account(&self, addr: &Address) -> &Account {
        self.accounts.get(addr).unwrap_or_else(|| &EMPTY_ACCOUNT)
    }

    pub(crate) fn update_account(
        &mut self,
        addr: &Address,
        f: impl FnOnce(Account) -> Result<Account>,
    ) -> Result<()> {
        log::trace!("update_account(): account={:?}", self.get_account(&addr));

        let updated = f(self.get_account(addr).clone())?;
        self.accounts.insert(addr.clone(), updated);

        log::trace!("result: account={:?}", self);
        Ok(())
    }

    pub(crate) fn delete_account(&mut self, addr: &Address) -> Result<()> {
        log::trace!("delete_account(): address={:?}", addr);
        self.update_account(addr, |_| Ok(Account::Empty))
    }

    pub(crate) fn send_eth(&mut self, from: &Address, to: &Address, amount: &U256) -> Result<()> {
        log::trace!(
            "send_eth(): from={:?}, to={:?}, amount={:02X?}",
            from,
            to,
            amount
        );

        // ⚠️ Do not check the sender amount because of the invalid state data.
        //self.update_account(from, |from_account| {
        //    from_account
        //        .decrease_balance(amount)
        //        .map_err(StateError::AccountError)
        //})
        //.and_then(|_| {
        self.update_account(to, |to_account| {
            to_account
                .increase_balance(amount)
                .map_err(StateError::AccountError)
        })
        //})
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            accounts: HashMap::default(),
        }
    }
}

#[derive(Error, Debug, Clone)]
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
