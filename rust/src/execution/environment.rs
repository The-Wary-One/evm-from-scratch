use std::fmt::Display;

use primitive_types::{U256, U512};
use thiserror::Error;

use super::code::*;
use super::memory::{MemoryError, MemoryImpl, MemoryInit};
use super::stack::*;
use crate::utils::*;

#[derive(Debug)]
pub(crate) struct ExecutionEnvImpl<'a, S: State> {
    _state: std::marker::PhantomData<S>,
    stack: StackImpl<S>,
    code: CodeImpl<'a, S>,
    memory: MemoryImpl<S>,
    result: Option<Result<'a, ()>>,
}

pub(crate) type ExecutionEnvInit<'a> = ExecutionEnvImpl<'a, Init>;
pub(crate) type ExecutionEnv<'a> = ExecutionEnvImpl<'a, Ready>;
pub(crate) type ExecutionResult<'a> = ExecutionEnvImpl<'a, Completed>;

impl<'a> ExecutionEnvInit<'a> {
    pub fn new(bytecode: &'a [u8]) -> ExecutionEnv<'a> {
        ExecutionEnv {
            _state: std::marker::PhantomData,
            stack: StackInit::new(),
            code: CodeInit::new(bytecode),
            memory: MemoryInit::new(),
            result: None,
        }
    }
}

#[derive(Error, Debug)]
pub enum ExecutionError<'a> {
    Revert(&'a [u8]),
    #[error(transparent)]
    StackError(#[from] StackError),
    #[error(transparent)]
    CodeError(#[from] CodeError),
    #[error(transparent)]
    MemoryError(#[from] MemoryError),
}

impl<'a> Display for ExecutionError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionError::Revert(bytes) => write!(f, "Execution reverted: {:?}", bytes),
            ExecutionError::StackError(e) => e.fmt(f),
            ExecutionError::CodeError(e) => e.fmt(f),
            ExecutionError::MemoryError(e) => e.fmt(f),
        }
    }
}

type Result<'a, T> = std::result::Result<T, ExecutionError<'a>>;

impl<'a> Iterator for &mut ExecutionEnv<'a> {
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
            ADD => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| {
                        // Add must overflow.
                        let (c, _) = a.overflowing_add(b);
                        c
                    })
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            MUL => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| {
                        // Mul must overflow.
                        let (c, _) = a.overflowing_mul(b);
                        c
                    })
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SUB => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| {
                        // Sub must overflow.
                        let (c, _) = a.overflowing_sub(b);
                        c
                    })
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            DIV => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)|
                         // If denominator is zero, result is 0.
                         if b.is_zero() { b } else { a / b }
                    )
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SDIV => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| {
                        // Assume a and b are signed.
                        Int256::from_raw_u256(a) / Int256::from_raw_u256(b)
                    })
                    .and_then(|c| self.stack.push(c.to_raw_u256()));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            MOD => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)|
                         // If denominator is zero, result is 0.
                         if b.is_zero() { b } else { a % b }
                    )
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
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
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            ADDMOD => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .and_then(|(a, b)| self.stack.pop().map(|n| (a, b, n)))
                    .map(|(a, b, n)|
                         // If denominator is zero, result is 0.
                         if n.is_zero() { n } else {
                             let temp = U512::from(a) + U512::from(b);
                             U256::try_from(temp % U512::from(n)).expect("safe")
                         }
                    )
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            MULMOD => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .and_then(|(a, b)| self.stack.pop().map(|n| (a, b, n)))
                    .map(|(a, b, n)|
                         // If denominator is zero, result is 0.
                         if n.is_zero() { n } else {
                             let temp = a.full_mul(b);
                             U256::try_from(temp % U512::from(n)).expect("safe")
                         }
                    )
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            EXP => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|e| (a, e)))
                    .map(|(a, e)| {
                        let (n, _) = a.overflowing_pow(e);
                        n
                    })
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SIGNEXTEND => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|b| self.stack.pop().map(|x| (b, x)))
                    .map(|(b, x)| {
                        // x assumed to be signed.
                        IntN::from_raw_u256(x, Bytesize::from(b)).sign_extend()
                    })
                    .and_then(|c| self.stack.push(c.to_raw_u256()));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            LT => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| a < b)
                    .and_then(|c| self.stack.push(c as u8));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            GT => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| a > b)
                    .and_then(|c| self.stack.push(c as u8));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SLT => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)|
                         // a and b assumed to be signed.
                         Int256::from_raw_u256(a) < Int256::from_raw_u256(b))
                    .and_then(|c| self.stack.push(c as u8));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SGT => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)|
                         // a and b assumed to be signed.
                         Int256::from_raw_u256(a) > Int256::from_raw_u256(b))
                    .and_then(|c| self.stack.push(c as u8));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            EQ => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| a == b)
                    .and_then(|c| self.stack.push(c as u8));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            AND => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| a & b)
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            OR => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| a | b)
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            XOR => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(a, b)| a ^ b)
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            ISZERO => {
                let res = self
                    .stack
                    .pop()
                    .map(|a| a.is_zero())
                    .and_then(|c| self.stack.push(c as u8));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            NOT => {
                let res = self
                    .stack
                    .pop()
                    .map(|a| !a)
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            BYTE => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|i| self.stack.pop().map(|x| (i, x)))
                    .map(|(i, x)| {
                        if i > Bytesize::MAX.into() {
                            0x00
                        } else {
                            x.byte(usize::from(Bytesize::MAX) - usize::try_from(i).expect("safe"))
                        }
                    })
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SHL => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(shift, value)| value << shift)
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SHR => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(shift, value)| value >> shift)
                    .and_then(|c| self.stack.push(c));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            SAR => {
                let res = self
                    .stack
                    .pop()
                    .and_then(|a| self.stack.pop().map(|b| (a, b)))
                    .map(|(shift, value)|
                         // value assumed to be signed.
                         Int256::from_raw_u256(value) >> shift.into())
                    .and_then(|c| self.stack.push(c.to_raw_u256()));

                match res {
                    Ok(_) => Some(()),
                    Err(e) => {
                        self.result = Some(Err(ExecutionError::StackError(e)));
                        // Stop.
                        None
                    }
                }
            }
            POP => match self.stack.pop() {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MLOAD => match self
                .stack
                .pop()
                .map_err(ExecutionError::StackError)
                .and_then(|offset| {
                    self.memory
                        .load(offset)
                        .map_err(ExecutionError::MemoryError)
                })
                .and_then(|value| self.stack.push(value).map_err(ExecutionError::StackError))
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
                .map_err(ExecutionError::StackError)
                .and_then(|(offset, b)| {
                    self.memory
                        .store(offset, b)
                        .map_err(ExecutionError::MemoryError)
                }) {
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
                .map_err(ExecutionError::StackError)
                .and_then(|(offset, b)| {
                    self.memory
                        .store8(offset, b)
                        .map_err(ExecutionError::MemoryError)
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
                .map_err(ExecutionError::StackError)
                .and_then(|counter| {
                    self.code
                        .jump_to(counter)
                        .map_err(ExecutionError::CodeError)
                }) {
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
                .map_err(ExecutionError::StackError)
                .and_then(|(counter, b)| {
                    if !b.is_zero() {
                        self.code
                            .jump_to(counter)
                            .map_err(ExecutionError::CodeError)
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
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            MSIZE => match self.stack.push(self.memory.size()) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            GAS => match self.stack.push(U256::MAX) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            JUMPDEST => Some(()),
            PUSH(n) => match self.stack.push(n) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            DUP(n) => match self.stack.dup(n) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            SWAP(n) => match self.stack.swap(n) {
                Ok(_) => Some(()),
                Err(e) => {
                    self.result = Some(Err(ExecutionError::StackError(e)));
                    // Stop.
                    None
                }
            },
            INVALID => {
                self.result = Some(Err(ExecutionError::Revert(&[])));
                // Stop.
                None
            }
        }
    }
}

impl<'a> ExecutionEnv<'a> {
    pub fn execute(mut self) -> ExecutionResult<'a> {
        log::trace!("execute(): execute the bytecode");

        let iter = &mut self.into_iter();
        while let Some(_) = iter.next() {}

        log::trace!("execution completed");
        self.into()
    }
}

impl<'a> From<ExecutionEnv<'a>> for ExecutionResult<'a> {
    fn from(env: ExecutionEnv<'a>) -> Self {
        Self {
            _state: std::marker::PhantomData,
            code: env.code.into(),
            stack: env.stack.into(),
            memory: env.memory.into(),
            result: env.result,
        }
    }
}

impl<'a> ExecutionResult<'a> {
    pub fn stack(&self) -> &StackResult {
        &self.stack
    }

    pub fn result(&self) -> &Result<()> {
        &self.result.as_ref().expect("safe")
    }
}
