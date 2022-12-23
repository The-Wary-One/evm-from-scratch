use ruint::aliases::U256;
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
    pub(super) env: &'c mut Environment<'a>,
    pub(super) message: &'c Message<'b, 'c>,
    pub(super) stack: Stack,
    pub(super) memory: Memory,
    pub(super) code: Code,
    pub(super) logs: Vec<Log>,
    pub(super) result: Option<Result<(U256, U256)>>,
    pub(super) last_inner_call: Option<EVMResult>,
}

impl<'a, 'b, 'c> EVM<'a, 'b, 'c>
where
    'a: 'c,
    'b: 'c,
{
    pub fn new(env: &'c mut Environment<'a>, message: &'c Message<'b, 'c>) -> EVM<'a, 'b, 'c> {
        match message {
            Message::Call { target, .. } | Message::Staticcall { target, .. } => {
                let code = Code::new(env.state().get_account(target).code().clone());

                Self {
                    env,
                    message,
                    stack: Stack::new(),
                    memory: Memory::new(),
                    code,
                    logs: vec![],
                    result: None,
                    last_inner_call: None,
                }
            }
            Message::Delegatecall { delegate, .. } => {
                let code = Code::new(env.state().get_account(delegate).code().clone());

                Self {
                    env,
                    message,
                    stack: Stack::new(),
                    memory: Memory::new(),
                    code,
                    logs: vec![],
                    result: None,
                    last_inner_call: None,
                }
            }
            Message::Create { .. } => todo!(),
        }
    }
}

#[derive(Error, Debug, Clone)]
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

impl<'a, 'b, 'c> EVM<'a, 'b, 'c> {
    pub fn execute(mut self) -> EVMResult {
        log::trace!("execute(): execute the bytecode");

        // State snapshot.
        let env = self.env.state().clone();

        // Send Eth.
        if *self.message.value() != U256::ZERO {
            match self.message {
                // Check if it is a staticcall
                Message::Staticcall { .. } => {
                    self.result = Some(Err(EVMError::StateModificationDisallowed));
                    return self.into();
                }
                // Do not send ETH again when doing a delegate call.
                Message::Delegatecall { .. } => {}
                Message::Call { .. } | Message::Create { .. } => {
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
        if let Some(Err(_)) = &self.result {
            self.env.set_state(env);
        }

        log::trace!("execution completed");
        self.into()
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EVMResult {
    pub(super) stack: StackResult,
    pub(super) return_data: Box<[u8]>,
    pub(super) logs: Box<[LogResult]>,
    pub(super) status: bool,
}

impl<'a, 'b, 'c> From<EVM<'a, 'b, 'c>> for EVMResult {
    fn from(evm: EVM<'a, 'b, 'c>) -> Self {
        let (offset, size) = match evm.result {
            Some(Ok((o, s))) => (o, s),
            Some(Err(EVMError::Revert(o, s))) => (o, s),
            _ => (U256::ZERO, U256::ZERO),
        };
        let return_data = evm
            .memory
            .load(offset.saturating_to(), size.saturating_to());
        Self {
            stack: evm.stack.into(),
            return_data,
            logs: evm.logs.into_iter().map(From::from).collect(),
            status: evm.result.map_or(false, |r| r.is_ok()),
        }
    }
}

impl EVMResult {
    pub fn stack(&self) -> &StackResult {
        &self.stack
    }

    pub fn return_data(&self) -> &Box<[u8]> {
        &self.return_data
    }

    pub fn logs(&self) -> &Box<[LogResult]> {
        &self.logs
    }

    pub fn status(&self) -> bool {
        self.status
    }
}
