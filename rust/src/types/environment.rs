use ruint::aliases::U256;

use crate::types::Address;

use super::{State, U256_DEFAULT};

#[derive(Debug)]
/// Items external to the virtual machine itself, provided by the environment.
pub struct Environment<'a> {
    caller: &'a Address,
    block_hashes: &'a [U256],
    coinbase: &'a Address,
    number: &'a U256,
    base_fee_per_gas: &'a U256,
    gas_limit: &'a U256,
    gas_price: &'a U256,
    time: &'a U256,
    difficulty: &'a U256,
    state: &'a State,
    chain_id: &'a U256,
}

impl<'a> Environment<'a> {
    pub fn new(
        caller: &'a Address,
        block_hashes: &'a [U256],
        coinbase: &'a Address,
        number: &'a U256,
        base_fee_per_gas: &'a U256,
        gas_limit: &'a U256,
        gas_price: &'a U256,
        time: &'a U256,
        difficulty: &'a U256,
        state: &'a State,
        chain_id: &'a U256,
    ) -> Self {
        Self {
            caller,
            block_hashes,
            coinbase,
            number,
            base_fee_per_gas,
            gas_limit,
            gas_price,
            time,
            difficulty,
            state,
            chain_id,
        }
    }

    pub fn caller(&self) -> &Address {
        &self.caller
    }

    pub fn block_hash(&self, block_number: usize) -> &U256 {
        &self
            .block_hashes
            .get(block_number)
            .unwrap_or_else(|| &U256_DEFAULT)
    }

    pub fn coinbase(&self) -> &Address {
        &self.coinbase
    }

    pub fn number(&self) -> &U256 {
        &self.number
    }

    pub fn base_fee_per_gas(&self) -> &U256 {
        &self.base_fee_per_gas
    }

    pub fn gas_limit(&self) -> &U256 {
        &self.gas_limit
    }

    pub fn gas_price(&self) -> &U256 {
        &self.gas_price
    }

    pub fn time(&self) -> &U256 {
        &self.time
    }

    pub fn difficulty(&self) -> &U256 {
        &self.difficulty
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn chain_id(&self) -> &U256 {
        &self.chain_id
    }
}
