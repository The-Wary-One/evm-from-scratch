use super::{Bitsize, Bytesize};
use ruint::{aliases::U256, uint};
use std::{cmp, ops};

#[derive(Debug, Clone)]
/// Signed 256 bits integers.
pub struct Int256(U256);

impl Int256 {
    pub fn zero() -> Self {
        Int256(U256::ZERO)
    }
    pub fn negative_one() -> Self {
        Int256(uint!(0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_U256))
    }

    pub fn max_negative_value() -> Self {
        Int256(uint!(0x8000000000000000000000000000000000000000000000000000000000000000_U256))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == U256::ZERO
    }

    pub fn is_negative(&self) -> bool {
        self.0.bit(0xFF)
    }

    pub fn abs(&self) -> U256 {
        if self.is_negative() {
            !self.0 + U256::from(1)
        } else {
            self.0
        }
    }

    pub fn from_u256(u: U256, is_negative: bool) -> Self {
        if is_negative {
            // Two's complement.
            Int256(!u + U256::from(1))
        } else {
            Int256(u)
        }
    }

    pub fn from_raw_u256(u: U256) -> Self {
        // Assume u is already signed.
        Int256(u)
    }

    pub fn to_raw_u256(self) -> U256 {
        self.0
    }
}

impl cmp::PartialEq for Int256 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }

    fn ne(&self, other: &Self) -> bool {
        !(self == other)
    }
}

impl cmp::PartialOrd for Int256 {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }

    fn lt(&self, other: &Self) -> bool {
        match (self.is_negative(), other.is_negative()) {
            (true, false) => true,
            (false, true) => false,
            _ => self.0 < other.0,
        }
    }

    fn gt(&self, other: &Self) -> bool {
        match (self.is_negative(), other.is_negative()) {
            (false, true) => true,
            (true, false) => false,
            _ => self.0 > other.0,
        }
    }

    fn le(&self, other: &Self) -> bool {
        !(self > other)
    }

    fn ge(&self, other: &Self) -> bool {
        !(self < other)
    }
}

impl cmp::Eq for Int256 {}

impl cmp::Ord for Int256 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        if self == other {
            cmp::Ordering::Equal
        } else if self < other {
            cmp::Ordering::Less
        } else {
            cmp::Ordering::Greater
        }
    }

    fn max(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }

    fn min(self, other: Self) -> Self {
        if self <= other {
            self
        } else {
            other
        }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        max.min(min.max(self))
    }
}

impl ops::Div for Int256 {
    type Output = Self;

    fn div(self, divisor: Self) -> Self::Output {
        let dividend = self;
        // If divisor is zero, quotient is 0.
        if divisor.is_zero() {
            return Int256::zero();
        }

        if dividend == Int256::max_negative_value() && divisor == Int256::negative_one() {
            dividend
        } else {
            let is_negative = dividend.is_negative() ^ divisor.is_negative();
            let c = dividend.abs() / divisor.abs();
            Int256::from_u256(c, is_negative)
        }
    }
}

impl ops::Rem for Int256 {
    type Output = Self;

    fn rem(self, divisor: Self) -> Self::Output {
        let dividend = self;
        // If divisor is zero, quotient is 0.
        if divisor.is_zero() {
            return Int256::zero();
        }

        if dividend == Int256::max_negative_value() && divisor == Int256::negative_one() {
            dividend
        } else {
            let is_negative = dividend.is_negative();
            let c = dividend.abs() % divisor.abs();
            Int256::from_u256(c, is_negative)
        }
    }
}

impl ops::Shr<Bitsize> for Int256 {
    type Output = Self;

    fn shr(self, shift: Bitsize) -> Self::Output {
        match (self.is_negative(), shift == Bitsize::MAX) {
            (false, true) => Int256::zero(),
            (true, true) => Int256::negative_one(),
            (is_negative, _) => {
                let raw = self.0 >> &shift.clone().into();
                if is_negative {
                    let int = IntN::from_raw_u256(raw, shift.into());
                    int.sign_extend()
                } else {
                    Int256(raw)
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
/// Signed `size` bytes integers.
pub struct IntN {
    raw: U256,
    size: Bytesize,
}

impl IntN {
    fn is_negative(&self) -> bool {
        let size = Bitsize::from(self.size.clone());
        self.raw.bit(size.into())
    }

    pub fn sign_extend(self) -> Int256 {
        // Is it a negative integer ?
        if self.is_negative() {
            // Replace the leading zeros with 0xFF.
            let size: usize = self.size.into();
            let mut bytes = self.raw.to_be_bytes::<0x20>();
            (0..0x1F - size).for_each(|b| {
                bytes[b] = 0xFF;
            });
            Int256(U256::from_be_bytes(bytes))
        } else {
            // Do nothing.
            Int256::from_raw_u256(self.raw)
        }
    }

    pub fn from_raw_u256(raw: U256, size: Bytesize) -> Self {
        // Assume u is already signed.
        IntN { raw, size }
    }
}
