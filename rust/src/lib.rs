use ruint::aliases::U256;

mod execution;
pub mod types;
use execution::*;
use types::*;

pub struct TestResult {
    pub stack: Vec<U256>,
    pub logs: Vec<LogResult>,
    pub success: bool,
}

impl<'a> From<EVMResult> for TestResult {
    fn from(result: EVMResult) -> Self {
        Self {
            stack: result.stack().into(),
            logs: result.logs().to_owned(),
            success: result.result().is_ok(),
        }
    }
}

impl Transaction {
    pub fn process<'a>(&'a self, env: &'a mut Environment<'a>) -> TestResult {
        let data = Calldata::new(self.data());
        let message = Message::new(self.from(), self.to(), self.gas(), self.value(), &data);
        Message::process(&message, env).into()
    }
}
