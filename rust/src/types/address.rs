use ruint::aliases::U256;
use ruint::Uint;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(from = "Uint<160, 3>")]
pub struct Address(#[serde(default)] [u8; 0x14]);

impl From<[u8; 0x14]> for Address {
    fn from(b: [u8; 0x14]) -> Self {
        Self(b)
    }
}

impl From<Uint<160, 3>> for Address {
    fn from(u: Uint<160, 3>) -> Self {
        u.to_be_bytes().into()
    }
}

impl From<&Address> for U256 {
    fn from(a: &Address) -> Self {
        let temp = Uint::<160, 3>::from_be_bytes(a.0);
        U256::from(temp)
    }
}

impl Default for Address {
    fn default() -> Self {
        [0x00; 0x14].into()
    }
}
