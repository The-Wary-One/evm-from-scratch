use ruint::aliases::U256;
use thiserror::Error;

use crate::types::Bytesize;

#[derive(Debug)]
pub(super) struct Memory {
    mem: Vec<u8>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory { mem: vec![] }
    }
}

impl Memory {
    pub(super) fn size(&self) -> usize {
        self.mem.len()
    }

    fn expand_mem(&mut self) -> Result<()> {
        self.mem
            .resize(self.mem.len() + usize::from(Bytesize::MAX) + 1, 0x00);
        Ok(())
    }

    pub(super) fn load(&mut self, offset: usize, size: usize) -> Result<&[u8]> {
        log::trace!(
            "load(): mem={:?}, offset={:?}, size={:?}",
            self.mem,
            offset,
            size
        );

        let max = offset + size - 1;
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem()?;
        }

        // Load from memory.
        let value = self.mem.get(offset..=max).expect("safe");
        log::trace!("result: mem={:?}, value={:?}", self.mem, value);
        Ok(value)
    }

    pub(super) fn load_u256(&mut self, offset: usize) -> Result<U256> {
        log::trace!("load_u256(): mem={:?}, offset={:?}", self.mem, offset);
        self.load(offset, 0x20)
            .map(|b| U256::try_from_be_slice(b).expect("safe"))
    }

    pub(super) fn store(&mut self, offset: usize, size: usize, value: &[u8]) -> Result<()> {
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
            self.expand_mem()?;
        }

        // Write to memory.
        let _ = &mut self.mem[offset..max].copy_from_slice(value);

        log::trace!("result: mem={:?}", self.mem);
        Ok(())
    }

    pub(super) fn store_u256(&mut self, offset: usize, value: U256) -> Result<()> {
        self.store(offset, 0x20, &value.to_be_bytes::<0x20>())
    }

    pub(super) fn store_u8(&mut self, offset: usize, value: u8) -> Result<()> {
        self.store(offset, 0x01, vec![value; 0x01].as_ref())
    }
}

#[derive(Debug)]
pub(super) struct MemoryResult {
    mem: Vec<u8>,
}

impl From<Memory> for MemoryResult {
    fn from(mem: Memory) -> Self {
        Self { mem: mem.mem }
    }
}

pub(super) type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Error, Debug)]
pub enum MemoryError {}
