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
pub(crate) struct EVM<'a, 'b, 'c>
where
    'a: 'c,
    'b: 'c,
{
    env: Environment<'a>,
    message: &'c Message<'b, 'c>,
    stack: Stack,
    memory: Memory,
    code: Code,
    logs: Vec<Log>,
    result: Option<Result<(U256, U256)>>,
}

impl<'a, 'b, 'c> EVM<'a, 'b, 'c>
where
    'a: 'c,
    'b: 'c,
{
    pub fn new(env: &'c Environment<'a>, message: &'c Message<'b, 'c>) -> EVM<'a, 'b, 'c> {
        match message {
            Message::Call { target, .. } => {
                let code = Code::new(env.state().get_account(target).code().clone());

                Self {
                    env: env.clone(),
                    message,
                    stack: Stack::new(),
                    memory: Memory::new(),
                    code,
                    logs: vec![],
                    result: None,
                }
            }
            _ => todo!(),
        }
    }
}

#[derive(Error, Debug)]
pub enum EVMError {
    Revert(U256, U256),
    StateModificationDisallowed,
    #[error(transparent)]
    StackError(#[from] StackError),
    #[error(transparent)]
    CodeError(#[from] CodeError),
    #[error(transparent)]
    MemoryError(#[from] MemoryError),
}

impl<'a> Display for EVMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EVMError::Revert(_, _) => write!(f, "EVM reverted: {:?}", self),
            EVMError::StateModificationDisallowed => {
                write!(f, "Cannot modify state in a staticcall")
            }
            EVMError::StackError(e) => e.fmt(f),
            EVMError::CodeError(e) => e.fmt(f),
            EVMError::MemoryError(e) => e.fmt(f),
        }
    }
}

type Result<T> = std::result::Result<T, EVMError>;

impl<'a, 'b, 'c> Iterator for &mut EVM<'a, 'b, 'c> {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!("next(): get the next opcode");
        use Opcode::*;

        match self.code.next().expect("safe") {
            STOP => {
                self.result = Some(Ok((U256::ZERO, U256::ZERO)));
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
                .map(|(offset, size)| {
                    let offset = offset.saturating_to();
                    let size = size.saturating_to();
                    self.memory.load(offset, size)
                })
                .map(|value| {
                    let mut hasher = sha3::Keccak256::new();
                    hasher.update(value.to_vec());
                    hasher.finalize()
                })
                .map(|hash| U256::try_from_be_slice(&hash[..]).expect("safe"))
                .and_then(|c| self.stack.push(c).map_err(EVMError::StackError))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            ADDRESS => match self.message {
                Message::Create { .. } => todo!(),
                _ => {
                    match self
                        .stack
                        .push(<U256 as From<&Address>>::from(self.message.target()))
                        .map_err(EVMError::StackError)
                    {
                        Ok(_) => Some(()),
                        Err(e) => {
                            self.result = Some(Err(e));
                            // Stop.
                            None
                        }
                    }
                }
            },
            BALANCE => match self
                .stack
                .pop()
                .map(|addr| self.env.state().get_account(&addr.into()).balance())
                .and_then(|balance| self.stack.push(*balance))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            ORIGIN => match self
                .stack
                .push(<U256 as From<&Address>>::from(self.env.caller()))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CALLER => match self
                .stack
                .push(<U256 as From<&Address>>::from(self.message.caller()))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CALLVALUE => match self
                .stack
                .push(*self.message.value())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CALLDATALOAD => match self
                .stack
                .pop()
                .map(|i| self.message.data().load_word(i.saturating_to()))
                .and_then(|data| self.stack.push(U256::from_be_bytes(data)))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CALLDATASIZE => match self
                .stack
                .push(self.message.data().size())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CALLDATACOPY => match self
                .stack
                .pop()
                .and_then(|dest_offset| self.stack.pop().map(|offset| (dest_offset, offset)))
                .and_then(|(dest_offset, offset)| {
                    self.stack.pop().map(|size| (dest_offset, offset, size))
                })
                .map_err(EVMError::StackError)
                .map(|(dest_offset, offset, size)| {
                    let dest_offset = dest_offset.saturating_to::<usize>();
                    let offset = offset.saturating_to::<usize>();
                    let size = size.saturating_to::<usize>();

                    self.memory.store(
                        dest_offset,
                        size,
                        self.message.data().load(offset, size).as_ref(),
                    )
                }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CODESIZE => match self.stack.push(self.code.size()) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            CODECOPY => match self
                .stack
                .pop()
                .and_then(|dest_offset| self.stack.pop().map(|offset| (dest_offset, offset)))
                .and_then(|(dest_offset, offset)| {
                    self.stack.pop().map(|size| (dest_offset, offset, size))
                })
                .map_err(EVMError::StackError)
                .map(|(dest_offset, offset, size)| {
                    let dest_offset = dest_offset.saturating_to();
                    let offset = offset.saturating_to();
                    let size = size.saturating_to();

                    self.memory
                        .store(dest_offset, size, self.code.load(offset, size).as_ref())
                }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            GASPRICE => match self
                .stack
                .push(*self.env.gas_price())
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            EXTCODESIZE => match self.stack.pop().map(Address::from).and_then(|addr| {
                self.stack
                    .push(self.env.state().get_account(&addr).code().len())
            }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(EVMError::StackError(e)));
                    // Stop.
                    None
                }
            },
            EXTCODECOPY => match self
                .stack
                .pop()
                .map(Address::from)
                .and_then(|addr| self.stack.pop().map(|dest_offset| (addr, dest_offset)))
                .and_then(|(addr, dest_offset)| {
                    self.stack.pop().map(|offset| (addr, dest_offset, offset))
                })
                .and_then(|(addr, dest_offset, offset)| {
                    self.stack
                        .pop()
                        .map(|size| (addr, dest_offset, offset, size))
                })
                .map_err(EVMError::StackError)
                .map(|(addr, dest_offset, offset, size)| {
                    let dest_offset = dest_offset.saturating_to();
                    let offset = offset.saturating_to();
                    let size = size.saturating_to();
                    let code = Code::new(self.env.state().get_account(&addr).code());

                    self.memory
                        .store(dest_offset, size, code.load(offset, size).as_ref())
                }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            EXTCODEHASH => match self
                .stack
                .pop()
                .map(|addr| self.env.state().get_account(&addr.into()).code_hash())
                .and_then(|hash| self.stack.push(hash))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            SELFBALANCE => match self
                .stack
                .push(
                    self.env
                        .state()
                        .get_account(self.message.target())
                        .balance()
                        .clone(),
                )
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            POP => match self.stack.pop().map(|_| ()) {
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
                .map(|offset| self.memory.load_u256(offset.saturating_to()))
                .and_then(|value| self.stack.push(value))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            MSTORE => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|b| (offset, b)))
                .map_err(EVMError::StackError)
                .map(|(offset, b)| self.memory.store_u256(offset.saturating_to(), b))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            MSTORE8 => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|b| (offset, b)))
                .map_err(EVMError::StackError)
                .map(|(offset, b)| {
                    self.memory
                        .store_u8(offset.saturating_to(), b.saturating_to())
                }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            SLOAD => match self
                .stack
                .pop()
                .map(|key| {
                    self.env
                        .state()
                        .get_account(self.message.target())
                        .load(&key)
                        .clone()
                })
                .and_then(|v| self.stack.push(v))
                .map_err(EVMError::StackError)
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            SSTORE => match (if self.message.is_staticcall() {
                Err(EVMError::StateModificationDisallowed)
            } else {
                Ok(())
            })
            .and_then(|_| self.stack.pop().map_err(EVMError::StackError))
            .and_then(|key| {
                self.stack
                    .pop()
                    .map_err(EVMError::StackError)
                    .map(|value| (key, value))
            })
            .map(|(key, value)| {
                self.env
                    .state_mut()
                    .update_account(self.message.target(), |mut account| {
                        account.store(key, value);
                        Ok(account)
                    })
                    .expect("safe")
            }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
                Err(e) => {
                    self.result = Some(Err(e));
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
            LOG(n) => match (if self.message.is_staticcall() {
                Err(EVMError::StateModificationDisallowed)
            } else {
                Ok(())
            })
            .and_then(|_| self.stack.pop().map_err(EVMError::StackError))
            .and_then(|offset| {
                self.stack
                    .pop()
                    .map(|size| {
                        (
                            offset.saturating_to::<usize>(),
                            size.saturating_to::<usize>(),
                        )
                    })
                    .map_err(EVMError::StackError)
            })
            .and_then(|(offset, size)| {
                let address = self.message.target().clone();
                let data = self.memory.load(offset, size).to_vec();

                let res = match n {
                    0 => Ok(Log::log0(address, data)),
                    1 => {
                        let topic1 = self.stack.pop()?;
                        Ok(Log::log1(address, [topic1], data))
                    }
                    2 => {
                        let topic1 = self.stack.pop()?;
                        let topic2 = self.stack.pop()?;
                        Ok(Log::log2(address, [topic1, topic2], data))
                    }
                    3 => {
                        let topic1 = self.stack.pop()?;
                        let topic2 = self.stack.pop()?;
                        let topic3 = self.stack.pop()?;
                        Ok(Log::log3(address, [topic1, topic2, topic3], data))
                    }
                    _ => {
                        let topic1 = self.stack.pop()?;
                        let topic2 = self.stack.pop()?;
                        let topic3 = self.stack.pop()?;
                        let topic4 = self.stack.pop()?;
                        Ok(Log::log4(address, [topic1, topic2, topic3, topic4], data))
                    }
                };

                let log = res.map_err(EVMError::StackError)?;
                self.logs.push(log);
                Ok(())
            }) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            CALL => match (if self.message.is_staticcall() {
                Err(EVMError::StateModificationDisallowed)
            } else {
                Ok(())
            })
            .and_then(|_| {
                let args = {
                    Ok((
                        self.stack.pop()?,
                        self.stack.pop()?,
                        self.stack.pop()?,
                        self.stack.pop()?,
                        self.stack.pop()?,
                        self.stack.pop()?,
                        self.stack.pop()?,
                    ))
                };
                let (gas, address, value, args_offset, args_size, ret_offset, ret_size) =
                    args.map_err(EVMError::StackError)?;
                let target = address.into();
                let args_offset = args_offset.saturating_to();
                let args_size = args_size.saturating_to();
                let ret_offset = ret_offset.saturating_to();
                let ret_size = ret_size.saturating_to();

                // Instanciate a new EVM.
                let bytes = self.memory.load(args_offset, args_size);
                let data = Calldata::new(&bytes[..]);
                let message = Message::call(self.message.target(), &target, &gas, &value, &data);
                let evm = EVM::new(&self.env, &message);
                let result = EVM::execute(evm);

                match result {
                    // Call succeded.
                    EVMResult {
                        memory,
                        logs,
                        result: Ok((offset, size)),
                        ..
                    } => {
                        // Copy the returned data to memory.
                        {
                            let bytes = memory.load(offset.saturating_to(), size.saturating_to());
                            self.memory
                                .store(ret_offset, ret_size, bytes.to_vec().as_ref());
                            Ok(())
                        }
                        .map_err(EVMError::MemoryError)?;
                        // Add result logs to logs.
                        self.logs
                            .append(&mut logs.into_iter().map(|l| l.into()).collect::<Vec<Log>>());
                        // Continue.
                        Ok(true)
                    }
                    // Call failed.
                    EVMResult {
                        memory,
                        result: Err(e),
                        ..
                    } => {
                        // Copy returned revert data into memory.
                        let (offset, size) = match e {
                            EVMError::Revert(o, s) => (o, s),
                            _ => (U256::ZERO, U256::ZERO),
                        };
                        {
                            let bytes = memory.load(offset.saturating_to(), size.saturating_to());
                            self.memory
                                .store(ret_offset, ret_size, bytes.to_vec().as_ref());
                            Ok(())
                        }
                        .map_err(EVMError::MemoryError)?;
                        // Revert.
                        Ok(false)
                    }
                }
            })
            .and_then(|status| self.stack.push(status as u8).map_err(EVMError::StackError))
            {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            RETURN => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|size| (offset, size)))
                .map_err(EVMError::StackError)
            {
                Ok((offset, size)) => {
                    self.result = Some(Ok((offset, size)));
                    // Stop.
                    None
                }
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            REVERT => match self
                .stack
                .pop()
                .and_then(|offset| self.stack.pop().map(|size| (offset, size)))
                .map_err(EVMError::StackError)
            {
                Ok((offset, size)) => {
                    self.result = Some(Err(EVMError::Revert(offset, size)));
                    // Stop.
                    None
                }
                Err(e) => {
                    self.result = Some(Err(e));
                    // Stop.
                    None
                }
            },
            INVALID => {
                self.result = Some(Err(EVMError::Revert(U256::ZERO, U256::ZERO)));
                // Stop.
                None
            }
        }
    }
}

impl<'a, 'b, 'c> EVM<'a, 'b, 'c> {
    pub fn execute(mut self) -> EVMResult {
        log::trace!("execute(): execute the bytecode");

        // State snapshot.
        //let env = self.env.clone();

        // Send Eth.
        if *self.message.value() != U256::ZERO {
            match self.message {
                // Check if it is a staticcall
                Message::Staticcall { .. } => {
                    self.result = Some(Err(EVMError::StateModificationDisallowed));
                    return self.into();
                }
                _ => {
                    self.env
                        .state_mut()
                        .send_eth(
                            self.message.caller(),
                            self.message.target(),
                            self.message.value(),
                        )
                        .expect("not handled");
                }
            }
        }

        let iter = &mut self.into_iter();
        while let Some(_) = iter.next() {}

        // Restore previous state snapshot if the call reverted.
        // TODO

        log::trace!("execution completed");
        self.into()
    }
}

#[derive(Debug)]
pub(crate) struct EVMResult {
    stack: StackResult,
    memory: MemoryResult,
    logs: Vec<LogResult>,
    result: Result<(U256, U256)>,
}

impl<'a, 'b, 'c> From<EVM<'a, 'b, 'c>> for EVMResult {
    fn from(evm: EVM<'a, 'b, 'c>) -> Self {
        Self {
            stack: evm.stack.into(),
            memory: evm.memory.into(),
            logs: evm.logs.into_iter().map(From::from).collect(),
            result: evm.result.expect("safe"),
        }
    }
}

impl EVMResult {
    pub fn stack(&self) -> &StackResult {
        &self.stack
    }

    pub fn logs(&self) -> &Vec<LogResult> {
        &self.logs
    }

    pub fn result(&self) -> &Result<(U256, U256)> {
        &self.result
    }
}
