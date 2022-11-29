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

    pub(super) fn load(&mut self, offset: U256, size: U256) -> Result<&[u8]> {
        log::trace!(
            "load(): mem={:?}, offset={:?}, size={:?}",
            self.mem,
            offset,
            size
        );

        let offset = usize::try_from(offset.min(U256::from(usize::MAX))).expect("safe");
        let size = usize::try_from(size.min(U256::from(usize::MAX))).expect("safe");
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

    pub(super) fn load_u256(&mut self, offset: U256) -> Result<U256> {
        log::trace!("load_u256(): mem={:?}, offset={:?}", self.mem, offset);
        self.load(
            offset,
            <U256 as From<Bytesize>>::from(Bytesize::MAX) + U256::from(1),
        )
        .map(|b| U256::try_from_be_slice(b).expect("safe"))
    }

    pub(super) fn store(&mut self, offset: U256, value: U256) -> Result<()> {
        log::trace!(
            "store(): mem={:?}, offset={:?}, value={:?}",
            self.mem,
            offset,
            value
        );

        let offset = offset.saturating_to::<usize>();
        let max = offset + usize::from(Bytesize::MAX);
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem()?;
        }

        // Write to memory.
        &mut self.mem[offset..=max].copy_from_slice(&value.to_be_bytes::<0x20>());

        log::trace!("result: mem={:?}", self.mem);
        Ok(())
    }

    pub(super) fn store8(&mut self, offset: U256, value: U256) -> Result<()> {
        log::trace!(
            "store8(): mem={:?}, offset={:?}, value={:?}",
            self.mem,
            offset,
            value
        );

        let offset = offset.saturating_to::<usize>();
        let max = offset + 1;
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem()?;
        }

        // Write to memory.
        let value = value.to_be_bytes::<0x20>()[usize::from(Bytesize::MAX)];
        self.mem[offset] = value;

        log::trace!("result: mem={:?}", self.mem);
        Ok(())
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

type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Error, Debug)]
pub enum MemoryError {}
