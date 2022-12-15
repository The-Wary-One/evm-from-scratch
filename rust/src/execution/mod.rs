mod code;
mod evm;
mod memory;
mod stack;

use crate::types::{Environment, Message};
pub(super) use evm::*;

impl<'a, 'b> Message<'a, 'b>
where
    'a: 'b,
{
    pub(crate) fn process(&'b self, env: &'b mut Environment<'a>) -> EVMResult {
        match self {
            // Executes a call to an account.
            Message::Call { .. } |
            // Executes a staticcall to an account.
            Message::Staticcall { .. } => {
                // Execute code.
                let evm = EVM::new(env, self);
                EVM::execute(evm).into()
            }
            // Executes a create a smart contract account.
            Message::Create { .. } => todo!(),
        }
    }
}
