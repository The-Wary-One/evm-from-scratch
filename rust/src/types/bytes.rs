use ruint::{aliases::U256, FromUintError};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
/// A U256 value between 0 and 255
pub struct Bitsize(usize);

impl Bitsize {
    pub const MIN: Self = Self(0x00);

    pub const MAX: Self = Self(0xFF);
}

impl From<U256> for Bitsize {
    fn from(u: U256) -> Self {
        let s: U256 = u.clamp(U256::from(Self::MIN.0), U256::from(Self::MAX.0));
        Self(usize::try_from(s).expect("safe"))
    }
}

impl From<Bitsize> for U256 {
    fn from(s: Bitsize) -> Self {
        U256::from(s.0)
    }
}

impl From<&Bitsize> for U256 {
    fn from(s: &Bitsize) -> Self {
        U256::from(s.0)
    }
}

impl From<Bitsize> for usize {
    fn from(s: Bitsize) -> Self {
        s.0
    }
}

impl From<Bytesize> for Bitsize {
    fn from(s: Bytesize) -> Self {
        Self(usize::from(s) * 8 + 7)
    }
}

impl From<Bitsize> for Bytesize {
    fn from(s: Bitsize) -> Self {
        Self(usize::from(s) / 8)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
/// A U256 value between 0 and 31.
pub struct Bytesize(usize);

impl Bytesize {
    pub const MIN: Self = Self(0x00);

    pub const MAX: Self = Self(0x1F);
}

//impl From<U256> for Bytesize {
//    fn from(u: U256) -> Self {
//        let s: U256 = u.clamp(U256::from(Self::MIN.0), U256::from(Self::MAX.0));
//        Self(usize::try_from(s).expect("safe"))
//    }
//}

impl From<Bytesize> for U256 {
    fn from(s: Bytesize) -> Self {
        U256::from(s.0)
    }
}

impl From<Bytesize> for usize {
    fn from(s: Bytesize) -> Self {
        s.0
    }
}

impl TryFrom<&U256> for Bytesize {
    type Error = FromUintError<Bytesize>;

    fn try_from(value: &U256) -> Result<Self, Self::Error> {
        if value > &Bytesize::MAX.into() {
            Err(FromUintError::Overflow(
                256,
                Bytesize(usize::try_from(value % U256::from(0x20)).expect("safe")),
                Bytesize::MAX,
            ))
        } else {
            Ok(Bytesize(usize::try_from(value).expect("safe")))
        }
    }
}
