use ruint::aliases::U256;
use thiserror::Error;

#[derive(Debug)]
pub(super) struct Code {
    bytecode: Vec<u8>,
    opcodes: Vec<Option<Opcode>>,
    pc: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Opcode {
    STOP,
    ADD,
    MUL,
    SUB,
    DIV,
    SDIV,
    MOD,
    SMOD,
    ADDMOD,
    MULMOD,
    EXP,
    SIGNEXTEND,
    LT,
    GT,
    SLT,
    SGT,
    EQ,
    ISZERO,
    AND,
    OR,
    XOR,
    NOT,
    BYTE,
    SHL,
    SHR,
    SAR,
    SHA3,
    ADDRESS,
    BALANCE,
    ORIGIN,
    CALLER,
    CALLVALUE,
    CALLDATALOAD,
    CALLDATASIZE,
    CALLDATACOPY,
    CODESIZE,
    CODECOPY,
    GASPRICE,
    EXTCODESIZE,
    EXTCODECOPY,
    EXTCODEHASH,
    BLOCKHASH,
    COINBASE,
    TIMESTAMP,
    NUMBER,
    DIFFICULTY,
    GASLIMIT,
    CHAINID,
    BASEFEE,
    SELFBALANCE,
    POP,
    MLOAD,
    MSTORE,
    MSTORE8,
    SLOAD,
    SSTORE,
    JUMP,
    JUMPI,
    PC,
    MSIZE,
    GAS,
    JUMPDEST,
    PUSH(U256),
    DUP(usize),
    SWAP(usize),
    LOG(usize),
    CALL,
    RETURN,
    REVERT,
    INVALID,
}

impl Code {
    pub fn new(bytecode: &[u8]) -> Code {
        Code {
            bytecode: bytecode.to_owned(),
            opcodes: Code::opcodes(bytecode),
            pc: 0,
        }
    }

    pub(super) fn pc(&self) -> usize {
        self.pc
    }

    pub(super) fn size(&self) -> usize {
        self.bytecode.len()
    }

    pub(super) fn jump_to(&mut self, counter: U256) -> Result<()> {
        match usize::try_from(counter)
            .ok()
            .and_then(|c| {
                self.opcodes
                    .get(c)
                    .map(|o| o.to_owned())
                    .flatten()
                    .map(|op| (c, op))
            })
            .filter(|(_, op)| *op == Opcode::JUMPDEST)
        {
            None => Err(CodeError::InvalidJumpdest),
            Some((c, _)) => {
                self.pc = c;
                Ok(())
            }
        }
    }

    pub(crate) fn load(&self, offset: usize, size: usize) -> Vec<u8> {
        let mut bytes = vec![0x00; size];
        for n in 0..size {
            let b = self.bytecode.get(offset + n).unwrap_or(&0x00);
            bytes[n] = *b;
        }
        bytes
    }

    fn opcodes(bytecode: &[u8]) -> Vec<Option<Opcode>> {
        let mut opcodes = vec![None; bytecode.len()];
        let mut pc = 0;

        while pc < opcodes.len() {
            let byte = bytecode[pc];
            let mut counter = pc + 1;

            use Opcode::*;
            let opcode = match byte {
                0x00 => STOP,
                0x01 => ADD,
                0x02 => MUL,
                0x03 => SUB,
                0x04 => DIV,
                0x05 => SDIV,
                0x06 => MOD,
                0x07 => SMOD,
                0x08 => ADDMOD,
                0x09 => MULMOD,
                0x0A => EXP,
                0x0B => SIGNEXTEND,
                0x10 => LT,
                0x11 => GT,
                0x12 => SLT,
                0x13 => SGT,
                0x14 => EQ,
                0x15 => ISZERO,
                0x16 => AND,
                0x17 => OR,
                0x18 => XOR,
                0x19 => NOT,
                0x1A => BYTE,
                0x1B => SHL,
                0x1C => SHR,
                0x1D => SAR,
                0x20 => SHA3,
                0x30 => ADDRESS,
                0x31 => BALANCE,
                0x32 => ORIGIN,
                0x33 => CALLER,
                0x34 => CALLVALUE,
                0x35 => CALLDATALOAD,
                0x36 => CALLDATASIZE,
                0x37 => CALLDATACOPY,
                0x38 => CODESIZE,
                0x39 => CODECOPY,
                0x3A => GASPRICE,
                0x3B => EXTCODESIZE,
                0x3C => EXTCODECOPY,
                0x3F => EXTCODEHASH,
                0x40 => BLOCKHASH,
                0x41 => COINBASE,
                0x42 => TIMESTAMP,
                0x43 => NUMBER,
                0x44 => DIFFICULTY,
                0x45 => GASLIMIT,
                0x46 => CHAINID,
                0x47 => SELFBALANCE,
                0x48 => BASEFEE,
                0x50 => POP,
                0x51 => MLOAD,
                0x52 => MSTORE,
                0x53 => MSTORE8,
                0x54 => SLOAD,
                0x55 => SSTORE,
                0x56 => JUMP,
                0x57 => JUMPI,
                0x58 => PC,
                0x59 => MSIZE,
                0x5A => GAS,
                0x5B => JUMPDEST,
                0x60..=0x7F => {
                    // 1 <= n <= 32
                    let n: usize = (byte - 0x5F).into();
                    // Check for bad bytecode length.
                    let bytes = &bytecode[counter..std::cmp::min(counter + n, bytecode.len())];
                    // The end of the number in the bytecode.
                    counter += n;
                    PUSH(U256::try_from_be_slice(&bytes).expect("safe"))
                }
                0x80..=0x8F => {
                    // 1 <= n <= 16
                    let n: usize = (byte - 0x7F).into();
                    DUP(n)
                }
                0x90..=0x9F => {
                    // 1 <= n <= 16
                    let n: usize = (byte - 0x8F).into();
                    SWAP(n)
                }
                0xA0..=0xA4 => {
                    // 0 <= n <= 4
                    let n: usize = (byte - 0xA0).into();
                    LOG(n)
                }
                0xF1 => CALL,
                0xF3 => RETURN,
                0xFD => REVERT,
                0xFE | _ => INVALID,
            };

            opcodes[pc] = Some(opcode);
            pc = counter;
        }

        opcodes
    }
}

#[derive(Error, Debug)]
pub enum CodeError {
    InvalidJumpdest,
}

pub(super) type Result<T> = std::result::Result<T, CodeError>;

impl std::fmt::Display for CodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeError::InvalidJumpdest => write!(f, "invalid jumpdest"),
        }
    }
}

impl Iterator for Code {
    type Item = Opcode;

    fn next(&mut self) -> Option<Self::Item> {
        log::trace!(
            "next(): bytecode={:02X?}, pc={:?}, opcodes={:?}",
            self.bytecode,
            self.pc,
            self.opcodes
        );

        let mut pc = self.pc;

        // Get the next opcode by filtering the empty push data slots.
        let opcode = loop {
            let o = self
                .opcodes
                .get(pc)
                // STOP if there are no opcode to execute.
                .unwrap_or(&Some(Opcode::STOP));

            pc += 1;

            if let Some(op) = o {
                break op.clone();
            }
        };

        // Increment the pc.
        self.pc = pc;

        log::trace!("result: opcode={:02X?}, pc={:?}", opcode, self.pc);
        Some(opcode)
    }
}

#[derive(Debug)]
pub(super) struct CodeResult {
    bytecode: Vec<u8>,
    opcodes: Vec<Option<Opcode>>,
    pc: usize,
}

impl From<Code> for CodeResult {
    fn from(code: Code) -> Self {
        Self {
            bytecode: code.bytecode,
            opcodes: code.opcodes,
            pc: code.pc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_iterate_over_bytecode() {
        let raw = [0x00, 0xFE];
        let mut code = Code::new(&raw);
        assert_eq!(Some(Opcode::STOP), code.next());
        assert_eq!(Some(Opcode::INVALID), code.next());
    }
}
