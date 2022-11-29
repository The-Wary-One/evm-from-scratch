use ruint::aliases::U256;

mod execution;
pub mod types;
use execution::*;
use types::*;

pub struct TestResult {
    pub stack: Vec<U256>,
    pub success: bool,
}

impl<'a> From<EVMResult<'_>> for TestResult {
    fn from(result: EVMResult) -> Self {
        Self {
            stack: result.stack().into(),
            success: result.result().is_ok(),
        }
    }
}

impl Transaction {
    pub fn process(&self, env: &Environment) -> TestResult {
        let message = Message::new(
            self.from(),
            self.to(),
            self.gas(),
            self.value(),
            self.data(),
        );
        Message::process(&message, &env).into()
    }
}

impl<'a> Message<'a> {
    fn process(&'a self, env: &'a Environment<'a>) -> EVMResult<'a> {
        // Prepare the next state.
        let mut state = env.state().clone();

        match self {
            // Executes a call to an account.
            Message::Call { target, .. } => {
                // Send Eth.
                if *self.value() != U256::ZERO {
                    state
                        .send_eth(self.caller(), target, self.value())
                        .expect("not handled");
                }
                // Execute code.
                let evm = EVM::new(&env, &self);
                EVM::execute(evm).into()
            }
            // Executes a create a smart contract account.
            Message::Create { .. } => {
                todo!()
            }
        }

        // Should save the new state.
    }
}
