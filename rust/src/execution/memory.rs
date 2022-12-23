use std::cell::{Ref, RefCell};

use ruint::aliases::U256;
use thiserror::Error;

use crate::types::Bytesize;

#[derive(Debug, Clone)]
pub(super) struct Memory {
    mem: RefCell<Vec<u8>>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            mem: RefCell::new(vec![]),
        }
    }
}

impl Memory {
    pub(super) fn size(&self) -> usize {
        self.mem.borrow().len()
    }

    fn expand_mem(&self) {
        let length = self.mem.borrow().len();
        self.mem
            .borrow_mut()
            .resize(length + usize::from(Bytesize::MAX) + 1, 0x00);
    }

    pub(super) fn load(&self, offset: usize, size: usize) -> Box<[u8]> {
        log::trace!(
            "load(): mem={:02X?}, offset={:02X?}, size={:02X?}",
            self.mem,
            offset,
            size
        );

        let max = offset + size;
        let value = if max == 0 {
            Box::new([])
        } else {
            // Expand memory if needed.
            while self.size() < max {
                self.expand_mem();
            }

            // Load from memory.
            let r = Ref::map(self.mem.borrow(), |r| r.get(offset..max).expect("safe"));
            r.to_owned().into_boxed_slice()
        };

        log::trace!("result: mem={:02X?}, value={:02X?}", self.mem, value);
        value
    }

    pub(super) fn load_u256(&self, offset: usize) -> U256 {
        let b = self.load(offset, 0x20);
        U256::try_from_be_slice(&b).expect("safe")
    }

    pub(super) fn store(&mut self, offset: usize, size: usize, value: &[u8]) {
        log::trace!(
            "store(): mem={:02X?}, offset={:02X?}, size={:02X?}, value={:02X?}",
            self.mem,
            offset,
            size,
            value
        );

        let max = offset + size;
        if max != 0 {
            // Expand memory if needed.
            while self.size() < max {
                self.expand_mem();
            }

            // Write to memory.
            let mem = self.mem.get_mut();
            for i in 0..size {
                mem[offset + i] = value.get(i).map(|&b| b).unwrap_or_default();
            }
        }

        log::trace!("result: mem={:02X?}", self.mem);
    }

    pub(super) fn store_u256(&mut self, offset: usize, value: U256) {
        self.store(offset, 0x20, &value.to_be_bytes::<0x20>())
    }

    pub(super) fn store_u8(&mut self, offset: usize, value: u8) {
        self.store(offset, 0x01, &[value; 0x01])
    }
}

#[derive(Debug, Clone)]
pub(super) struct MemoryResult(Memory);

impl MemoryResult {}

impl From<Memory> for MemoryResult {
    fn from(mem: Memory) -> Self {
        Self(mem)
    }
}

//pub(super) type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Error, Debug, Clone)]
pub enum MemoryError {
    OffsetHigherThanSize,
}

impl std::fmt::Display for MemoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryError::OffsetHigherThanSize => write!(f, "offset higher than size"),
        }
    }
}
