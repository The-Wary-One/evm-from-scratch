use ruint::aliases::U256;

use super::Address;

#[derive(Debug)]
/// Atomic operation performed on the block chain (Legacy).
pub struct Transaction {
    gas_price: U256,
    gas: U256,
    from: Address,
    to: Option<Address>,
    value: U256,
    data: Vec<u8>,
}

impl Transaction {
    pub fn new(
        gas_price: U256,
        gas: U256,
        from: Address,
        to: Option<Address>,
        value: U256,
        data: Vec<u8>,
    ) -> Self {
        Self {
            gas_price,
            gas,
            from,
            to,
            value,
            data,
        }
    }

    pub fn gas_price(&self) -> &U256 {
        &self.gas_price
    }

    pub fn gas(&self) -> &U256 {
        &self.gas
    }

    pub fn from(&self) -> &Address {
        &self.from
    }

    pub fn to(&self) -> &Option<Address> {
        &self.to
    }

    pub fn value(&self) -> &U256 {
        &self.value
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Self {
            gas_price: U256::from(10e9),
            gas: U256::MAX,
            from: Address::default(),
            to: Some(Address::default()),
            value: U256::default(),
            data: vec![],
        }
    }
}
