/*!
 * EVM From Scratch
 * Rust template
 *
 * To work on EVM From Scratch in Rust:
 *
 * - Install Rust: https://www.rust-lang.org/tools/install
 * - Edit `rust/lib.rs`
 * - Run `cd rust && cargo run` to run the tests
 *
 * Hint: most people who were trying to learn Rust and EVM at the same
 * gave up and switched to JavaScript, Python, or Go. If you are new
 * to Rust, implement EVM in another programming language first.
 */

use evm::types::{Account, Address, Environment, LogResult, State, Transaction};
use ruint::{aliases::U256, uint};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct Evmtest {
    name: String,
    hint: String,
    #[serde(default)]
    block: Block,
    #[serde(default)]
    tx: Tx,
    #[serde(default)]
    state: HashMap<Address, AccountTest>,
    code: Code,
    expect: Expect,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct Block {
    #[serde(default)]
    basefee: U256,
    #[serde(default)]
    coinbase: Address,
    #[serde(default)]
    chainid: U256,
    #[serde(default)]
    gaslimit: U256,
    #[serde(default)]
    difficulty: U256,
    #[serde(default)]
    number: U256,
    #[serde(default)]
    timestamp: U256,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct Tx {
    #[serde(default)]
    from: Address,
    #[serde(default)]
    origin: Address,
    #[serde(default, with = "::serde_with::rust::double_option")]
    to: Option<Option<Address>>,
    #[serde(default)]
    value: U256,
    #[serde(with = "hex::serde", default)]
    data: Vec<u8>,
    #[serde(default)]
    gasprice: U256,
}

#[derive(Debug, Deserialize, Clone)]
struct AccountTest {
    balance: Option<U256>,
    code: Option<Code>,
}

#[derive(Debug, Deserialize, Clone, Default)]
struct Code {
    #[serde(deserialize_with = "deserialize_null_default")]
    asm: String,
    #[serde(with = "hex::serde", default)]
    bin: Vec<u8>,
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct Expect {
    #[serde(default)]
    stack: Vec<U256>,
    #[serde(default)]
    logs: Vec<LogResult>,
    success: bool,
    // #[serde(rename = "return")]
    // ret: Option<String>,
}

fn main() {
    env_logger::init();

    let text = std::fs::read_to_string("../evm.json").unwrap();
    let deserializer = &mut serde_json::Deserializer::from_str(&text);
    let res: Result<Vec<Evmtest>, _> = serde_path_to_error::deserialize(deserializer);
    let data = res.unwrap();

    let total = data.len();

    let default_origin: Address = uint!(0x1E79B045DC29EAE9FDC69673C9DCD7C53E5E159D_U160).into();
    let default_caller: Address = uint!(0x0000000000000000000000000000000000001337_U160).into();
    let default_contract: Address = uint!(0x000000000000000000000000000000000000dead_U160).into();

    for (index, test) in data.iter().enumerate() {
        println!("Test {} of {}: {}", index + 1, total, test.name);

        // Get the transaction data.
        let from = if test.tx.from != Address::default() {
            test.tx.from.clone()
        } else {
            default_caller.clone()
        };
        let to = test.tx.to.clone().unwrap_or(Some(default_contract.clone()));
        let caller = if test.tx.origin != Address::default() {
            test.tx.origin.clone()
        } else {
            default_origin.clone()
        };
        let transaction = Transaction::new(
            test.tx.gasprice,
            U256::default(),
            from.clone(),
            to.clone(),
            test.tx.value.clone(),
            test.tx.data.clone(),
        );

        // Setup the chain state.
        let mut accounts = test
            .state
            .clone()
            .into_iter()
            .map(|(k, v)| (k.clone(), Account::new(v.balance, v.code.map(|c| c.bin))))
            .collect::<HashMap<Address, Account>>();
        // Give from ETH.
        accounts.insert(from, Account::new(Some(test.tx.value), None));
        // Code to execute should be the to account code.
        accounts.insert(
            to.clone().expect("safe"),
            Account::new(
                accounts
                    .get(&to.expect("safe"))
                    .map(|a| a.balance().clone()),
                Some(test.code.bin.clone()),
            ),
        );
        let state = State::new(accounts);
        // Setup the chain environment.
        let mut env = Environment::new(
            &caller,
            &[],
            &test.block.coinbase,
            &test.block.number,
            &test.block.basefee,
            &test.block.gaslimit,
            &transaction.gas_price(),
            &test.block.timestamp,
            &test.block.difficulty,
            state,
            &test.block.chainid,
        );

        let result = transaction.process(&mut env);

        let is_expected_status = result.success == test.expect.success;

        let is_expected_stack = test.expect.stack == result.stack.to_vec();
        let is_expected_logs = test.expect.logs == result.logs.to_vec();

        let test_passed = is_expected_status && is_expected_stack && is_expected_logs;

        if !test_passed {
            println!("Instructions: \n{}\n", test.code.asm);

            println!("Expected success: {:?}", test.expect.success);
            println!("Expected stack: [");
            for v in &test.expect.stack {
                println!("  {:#X},", v);
            }
            println!("]\n");
            println!("Expected logs: [");
            for v in &test.expect.logs {
                println!("  {:?},", v);
            }
            println!("]\n");

            println!("Actual success: {:?}", result.success);
            println!("Actual stack: [");
            for v in result.stack.as_ref() {
                println!("  {:#X},", v);
            }
            println!("]\n");
            println!("Actual logs: [");
            for v in result.logs.as_ref() {
                println!("  {:?},", v);
            }
            println!("]\n");

            println!("\nHint: {}\n", test.hint);
            println!("Progress: {}/{}\n\n", index, total);
            panic!("Test failed");
        }
        println!("PASS");
    }
    println!("Congratulations!");
}
