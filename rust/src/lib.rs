use primitive_types::U256;

mod execution;
pub(crate) mod utils;
use execution::*;

pub struct EvmResult {
    pub stack: Vec<U256>,
    pub success: bool,
}

impl<'a> From<ExecutionResult<'_>> for EvmResult {
    fn from(result: ExecutionResult) -> Self {
        Self {
            stack: result.stack().into(),
            success: result.result().is_ok(),
        }
    }
}

pub fn evm(_code: impl AsRef<[u8]> + std::fmt::Debug) -> EvmResult {
    let env = ExecutionEnvInit::new(_code.as_ref());
    let result = ExecutionEnv::execute(env);
    result.into()
}
