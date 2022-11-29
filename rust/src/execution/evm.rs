use ruint::aliases::U256;
use sha3::Digest;
use std::fmt::Display;
use thiserror::Error;

use super::code::*;
use super::memory::*;
use super::stack::*;
use crate::types::*;

#[derive(Debug)]
/// The internal state of the virtual machine.
pub(crate) struct EVM<'a> {
    env: &'a Environment<'a>,
    message: &'a Message<'a>,
    stack: Stack,
    memory: Memory,
    code: Code<'a>,
    result: Option<Result<'a, ()>>,
}

impl<'a> EVM<'a> {
    pub fn new(env: &'a Environment<'a>, message: &'a Message<'a>) -> EVM<'a> {
        match message {
            Message::Call { target, .. } => {
                let code = Code::new(env.state().get_account(target).code());
                Self {
                    env,
                    message,
                    stack: Stack::new(),
                    memory: Memory::new(),
                    code,
                    result: None,
                }
            }
            _ => todo!(),
        }
    }
}

#[derive(Error, Debug)]
pub enum EVMError<'a> {
    Revert(&'a [u8]),
    #[error(transparent)]
    StackError(#[from] StackError),
    #[error(transparent)]
    CodeError(#[from] CodeError),
    #[error(transparent)]
    MemoryError(#[from] MemoryError),
}

impl<'a> Display for EVMError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EVMError::Revert(bytes) => write!(f, "EVM reverted: {:?}", bytes),
            EVMError::StackError(e) => e.fmt(f),
            EVMError::CodeError(e) => e.fmt(f),
            EVMError::MemoryError(e) => e.fmt(f),
        }
    }
}

type Result<'a, T> = std::result::Result<T, EVMError<'a>>;

impl<'a> Iterator for &mut EVM<'a> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("next(): get the next opcode");
        use Opcode::*;

        match self.code.next().expect("safe") {
            STOP => {
                self.result = Some(Ok(()));
                // Stop.
                None
            }
            ADD => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| {
                    // Add must overflow.
                    let (c, _) = a.overflowing_add(b);
                    c
                })
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MUL => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| {
                    // Mul must overflow.
                    let (c, _) = a.overflowing_mul(b);
                    c
                })
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SUB => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| {
                    // Sub must overflow.
                    let (c, _) = a.overflowing_sub(b);
                    c
                })
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            DIV => match self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)|
                         // If denominator is zero, result is 0.
                         if b == U256::ZERO { b } else { a / b }
                    )
                    .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SDIV => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| {
                    // Assume a and b are signed.
                    Int256::from_raw_u256(a) / Int256::from_raw_u256(b)
                })
                .and_then(|c| self.stack.push(c.to_raw_u256()))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MOD => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)|
                        // If denominator is zero, result is 0.
                        if b == U256::ZERO { b } else { a % b }
                )
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SMOD => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| {
                        // Assume a and b are signed.
                        Int256::from_raw_u256(a) % Int256::from_raw_u256(b)
                    })
                    .and_then(|c| self.stack.push(c.to_raw_u256()));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(EVMError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            ADDMOD => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .and_then(|(a, b)| self.stack.pop().map(|n| (a, b, n)))
                .map(|(a, b, n)| a.add_mod(b, n))
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MULMOD => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .and_then(|(a, b)| self.stack.pop().map(|n| (a, b, n)))
                .map(|(a, b, n)| a.mul_mod(b, n))
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            EXP => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|e| (a, e)))
                .map(|(a, e)| {
                    let (n, _) = a.overflowing_pow(e);
                    n
                })
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SIGNEXTEND => match self
                .stack
                .pop()
                .and_then(|b| self.stack.pop().map(|x| (b, x)))
                .map(|(b, x)| {
                    // x assumed to be signed.
                    IntN::from_raw_u256(x, b.saturating_to()).sign_extend()
                })
                .and_then(|c| self.stack.push(c.to_raw_u256()))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            LT => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| a < b)
                .and_then(|c| self.stack.push(U256::from(c as u8)))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            GT => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| a > b)
                .and_then(|c| self.stack.push(c as u8))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SLT => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)|
                         // a and b assumed to be signed.
                         Int256::from_raw_u256(a) < Int256::from_raw_u256(b))
                .and_then(|c| self.stack.push(c as u8))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SGT => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)|
                         // a and b assumed to be signed.
                         Int256::from_raw_u256(a) > Int256::from_raw_u256(b))
                .and_then(|c| self.stack.push(c as u8))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            EQ => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| a == b)
                .and_then(|c| self.stack.push(c as u8))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            AND => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| a & b)
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            OR => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| a | b)
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            XOR => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(a, b)| a ^ b)
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            ISZERO => match self
                .stack
                .pop()
                .map(|a| a == U256::ZERO)
                .and_then(|c| self.stack.push(c as u8))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            NOT => match self
                .stack
                .pop()
                .map(|a| !a)
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            BYTE => match self
                .stack
                .pop()
                .and_then(|i| self.stack.pop().map(|x| (i, x)))
                .map(|(i, x)| {
                    if i > Bytesize::MAX.into() {
                        0x00
                    } else {
                        x.to_be_bytes::<0x20>()[usize::from(i.saturating_to::<Bytesize>())]
                    }
                })
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SHL => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(shift, value)| value << shift.saturating_to::<usize>())
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SHR => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(shift, value)| value >> shift.saturating_to::<usize>())
                .and_then(|c| self.stack.push(c))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SAR => match self
                .stack
                .pop()
                .and_then(|a| self.stack.pop().map(|b| (a, b)))
                .map(|(shift, value)|
                    // value assumed to be signed.
                    Int256::from_raw_u256(value) >> shift.into())
                .and_then(|c| self.stack.push(c.to_raw_u256()))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SHA3 => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|size| (offset, size)))
                .map_err(EVMError::StackError)
                .and_then(|(offset, size)| {
                    self.memory
                        .load(offset, size)
                        .map_err(EVMError::MemoryError)
                })
                .map(|value| {
                    let mut hasher = sha3::Keccak256::new();
                    hasher.update(value);
                    hasher.finalize()
                })
                .map(|hash| U256::try_from_be_slice(&hash[..]).expect("safe"))
                .and_then(|c| self.stack.push(c).map_err(EVMError::StackError))
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            ADDRESS => match self.message {
                Message::Call { target, .. } => {
                    match self
                        .stack
                        .push(<U256 as From<&Address>>::from(target))
                        .map_err(EVMError::StackError)
                    {
                        Ok(_) => Some(()),
                        e => {
                            self.result = Some(e);
                            // Stop.
                            None
                        }
                    }
                }
                Message::Create { .. } => todo!(),
            },
            ORIGIN => match self
                .stack
                .push(<U256 as From<&Address>>::from(self.env.caller()))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            CALLER => match self.message {
                Message::Call { caller, .. } => {
                    match self
                        .stack
                        .push(<U256 as From<&Address>>::from(caller))
                        .map_err(EVMError::StackError)
                    {
                        Ok(_) => Some(()),
                        e => {
                            self.result = Some(e);
                            // Stop.
                            None
                        }
                    }
                }
                Message::Create { .. } => todo!(),
            },
            GASPRICE => match self
                .stack
                .push(*self.env.gas_price())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            BLOCKHASH => match self
                .stack
                .pop()
                .map(|number| self.env.block_hash(number.saturating_to::<usize>()))
                .and_then(|c| self.stack.push(c.clone()))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            COINBASE => match self
                .stack
                .push(<U256 as From<&Address>>::from(self.env.coinbase()))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            TIMESTAMP => match self
                .stack
                .push(*self.env.time())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            NUMBER => match self
                .stack
                .push(*self.env.number())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            DIFFICULTY => match self
                .stack
                .push(*self.env.difficulty())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            GASLIMIT => match self
                .stack
                .push(*self.env.gas_limit())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            CHAINID => match self
                .stack
                .push(*self.env.chain_id())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            BASEFEE => match self
                .stack
                .push(*self.env.base_fee_per_gas())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            POP => match self.stack.pop() {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MLOAD => match self
                .stack
                .pop()
                .map_err(EVMError::StackError)
                .and_then(|offset| self.memory.load_u256(offset).map_err(EVMError::MemoryError))
                .and_then(|value| self.stack.push(value).map_err(EVMError::StackError))
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            MSTORE => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|b| (offset, b)))
                .map_err(EVMError::StackError)
                .and_then(|(offset, b)| self.memory.store(offset, b).map_err(EVMError::MemoryError))
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            MSTORE8 => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|b| (offset, b)))
                .map_err(EVMError::StackError)
                .and_then(|(offset, b)| {
                    self.memory.store8(offset, b).map_err(EVMError::MemoryError)
                }) {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            JUMP => match self
                .stack
                .pop()
                .map_err(EVMError::StackError)
                .and_then(|counter| self.code.jump_to(counter).map_err(EVMError::CodeError))
            {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            JUMPI => match self
                .stack
                .pop()
                .and_then(|counter| self.stack.pop().map(|b| (counter, b)))
                .map_err(EVMError::StackError)
                .and_then(|(counter, b)| {
                    if b != U256::ZERO {
                        self.code.jump_to(counter).map_err(EVMError::CodeError)
                    } else {
                        Ok(())
                    }
                }) {
                Ok(_) => Some(()),
                e => {
                    self.result = Some(e);
                    // Stop.
                    None
                }
            },
            PC => match self.stack.push(self.code.pc() - 1) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MSIZE => match self.stack.push(self.memory.size()) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            GAS => match self.stack.push(U256::MAX) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            JUMPDEST => Some(()),
            PUSH(n) => match self.stack.push(n) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            DUP(n) => match self.stack.dup(n) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SWAP(n) => match self.stack.swap(n) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            INVALID => {
                self.result = Some(Err(EVMError::Revert(&[])));
                // Stop.
                None
            }
        }
    }
}

impl<'a> EVM<'a> {
    pub fn execute(mut self) -> EVMResult<'a> {
        log::trace!("execute(): execute the bytecode");

        let iter = &mut self.into_iter();
        while let Some(_) = iter.next() {}

        log::trace!("execution completed");
        self.into()
    }
}

#[derive(Debug)]
pub(crate) struct EVMResult<'a> {
    stack: StackResult,
    memory: MemoryResult,
    result: Option<Result<'a, ()>>,
}

impl<'a> From<EVM<'a>> for EVMResult<'a> {
    fn from(env: EVM<'a>) -> Self {
        Self {
            stack: env.stack.into(),
            memory: env.memory.into(),
            result: env.result,
        }
    }
}

impl<'a> EVMResult<'a> {
    pub fn stack(&self) -> &StackResult {
        &self.stack
    }

    pub fn result(&self) -> &Result<()> {
        &self.result.as_ref().expect("safe")
    }
}
