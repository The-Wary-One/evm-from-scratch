use primitive_types::U256;
use thiserror::Error;

use crate::utils::{Bytesize, Completed, Init, Ready, State};

#[derive(Debug)]
pub(super) struct MemoryImpl<S: State> {
    _state: std::marker::PhantomData<S>,
    mem: Vec<u8>,
}

pub(super) type MemoryInit = MemoryImpl<Init>;
pub(super) type Memory = MemoryImpl<Ready>;
pub(super) type MemoryResult = MemoryImpl<Completed>;

impl MemoryInit {
    pub fn new() -> Memory {
        Memory {
            _state: std::marker::PhantomData,
            mem: vec![],
        }
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

    pub(super) fn load(&mut self, offset: U256) -> Result<U256> {
        log::trace!("load(): mem={:?}, offset={:?}", self.mem, offset);

        let offset = usize::try_from(offset.min(usize::MAX.into())).expect("safe");
        let max = offset + usize::from(Bytesize::MAX);
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem()?;
        }

        // Load from memory.
        let value = U256::from(self.mem.get(offset..=max).expect("safe"));
        log::trace!("result: mem={:?}, value={:?}", self.mem, value);
        Ok(value)
    }

    pub(super) fn store(&mut self, offset: U256, value: U256) -> Result<()> {
        log::trace!(
            "store(): mem={:?}, offset={:?}, value={:?}",
            self.mem,
            offset,
            value
        );

        let offset = usize::try_from(offset.min(usize::MAX.into())).expect("safe");
        let max = offset + usize::from(Bytesize::MAX);
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem()?;
        }

        // Write to memory.
        value.to_big_endian(&mut self.mem[offset..=max]);

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

        let offset = usize::try_from(offset.min(usize::MAX.into())).expect("safe");
        let max = offset + 1;
        // Expand memory if needed.
        while self.size() < max {
            self.expand_mem()?;
        }

        // Write to memory.
        let value = value.byte(0);
        self.mem[offset] = value;

        log::trace!("result: mem={:?}", self.mem);
        Ok(())
    }
}

impl From<Memory> for MemoryResult {
    fn from(mem: Memory) -> Self {
        Self {
            _state: std::marker::PhantomData,
            mem: mem.mem,
        }
    }
}

type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Error, Debug)]
pub enum MemoryError {}
