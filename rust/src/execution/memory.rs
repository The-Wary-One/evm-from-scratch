use std::cell::{Ref, RefCell};

use ruint::aliases::U256;
use thiserror::Error;

use crate::types::Bytesize;

#[derive(Debug)]
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

    pub(super) fn load(&self, offset: usize, size: usize) -> Vec<u8> {
        log::trace!(
            "load(): mem={:?}, offset={:?}, size={:?}",
            self.mem,
            offset,
            size
        );

        let value = if offset + size == 0 {
            vec![]
        } else {
            let max = offset + size - 1;
            // Expand memory if needed.
            while self.size() < max {
                self.expand_mem();
            }

            // Load from memory.
            let r = Ref::map(self.mem.borrow(), |r| r.get(offset..=max).expect("safe"));
            r.to_owned()
        };

        log::trace!("result: mem={:?}, value={:?}", self.mem, value);
        value
    }

    pub(super) fn load_u256(&self, offset: usize) -> U256 {
        log::trace!("load_u256(): mem={:?}, offset={:?}", self.mem, offset);
        let b = self.load(offset, 0x20);
        U256::try_from_be_slice(&b).expect("safe")
    }

    pub(super) fn store(&mut self, offset: usize, size: usize, value: &[u8]) {
        log::trace!(
            "store(): mem={:?}, offset={:?}, size={:?}, value={:?}",
            self.mem,
            offset,
            size,
            value
        );

        let max = offset + size;
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem();
        }

        // Write to memory.
        let _ = &mut self.mem.get_mut()[offset..max].copy_from_slice(value);

        log::trace!("result: mem={:?}", self.mem);
    }

    pub(super) fn store_u256(&mut self, offset: usize, value: U256) {
        self.store(offset, 0x20, &value.to_be_bytes::<0x20>())
    }

    pub(super) fn store_u8(&mut self, offset: usize, value: u8) {
        self.store(offset, 0x01, &[value; 0x01])
    }
}

#[derive(Debug)]
pub(super) struct MemoryResult(Memory);

impl MemoryResult {
    pub(super) fn load(&self, offset: usize, size: usize) -> Vec<u8> {
        self.0.load(offset, size)
    }
}

impl From<Memory> for MemoryResult {
    fn from(mem: Memory) -> Self {
        Self(mem)
    }
}

pub(super) type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Error, Debug)]
pub enum MemoryError {}
