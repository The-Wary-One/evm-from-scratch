use ruint::aliases::{U160, U256};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(from = "U160")]
pub struct Address(#[serde(default)] [u8; 0x14]);

impl From<[u8; 0x14]> for Address {
    fn from(b: [u8; 0x14]) -> Self {
        Self(b)
    }
}

impl From<U160> for Address {
    fn from(u: U160) -> Self {
        u.to_be_bytes().into()
    }
}

impl From<U256> for Address {
    fn from(u: U256) -> Self {
        u.wrapping_to::<U160>().into()
    }
}

impl From<&Address> for U256 {
    fn from(a: &Address) -> Self {
        let temp = U160::from_be_bytes(a.0);
        U256::from(temp)
    }
}

impl Default for Address {
    fn default() -> Self {
        [0x00; 0x14].into()
    }
}
