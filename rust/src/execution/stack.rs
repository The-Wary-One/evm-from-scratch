use ruint::{aliases::U256, UintTryFrom};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct Stack {
    /// The index of the stack's top.
    top: Option<usize>,
    arr: [U256; 1024],
}

#[derive(Error, Debug, Clone)]
pub enum StackError {
    StackOverflow,
    NotEnoughValuesOnStack,
}

pub(super) type Result<T> = std::result::Result<T, StackError>;

impl std::fmt::Display for StackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StackError::StackOverflow => write!(f, "stack overflow"),
            StackError::NotEnoughValuesOnStack => write!(f, "not enough values on stack"),
        }
    }
}

impl Stack {
    pub(super) fn new() -> Self {
        Self {
            top: None,
            arr: [U256::default(); 1024],
        }
    }

    pub(super) fn push<T>(&mut self, n: T) -> Result<()>
    where
        U256: UintTryFrom<T>,
    {
        let n = U256::saturating_from(n);
        log::trace!(
            "push(n={:02X?}): top={:02X?}, arr={:02X?}",
            n,
            self.top,
            &self.arr[..=self.top.unwrap_or_default()]
        );

        let top = self.top.map_or(0, |t| t + 1);

        let res = if top >= 1024 {
            Err(StackError::StackOverflow)
        } else {
            self.arr[top] = n;
            self.top = Some(top);
            Ok(())
        };

        log::trace!(
            "result: top={:?}, arr={:02X?}",
            self.top,
            &self.arr[..=self.top.unwrap_or_default()]
        );
        res
    }

    pub(super) fn pop(&mut self) -> Result<U256> {
        log::trace!(
            "pop(): top={:?}, arr={:02X?}",
            self.top,
            &self.arr[..=self.top.unwrap_or_default()]
        );

        let res = match self.top {
            None => Err(StackError::NotEnoughValuesOnStack),
            Some(top) => {
                let value = self.arr[top];
                self.top = if top > 0 { Some(top - 1) } else { None };
                Ok(value)
            }
        };

        log::trace!(
            "result: top={:?}, arr={:02X?}, res={:02X?}",
            self.top,
            &self.arr[..=self.top.unwrap_or_default()],
            res
        );
        res
    }

    pub(super) fn dup(&mut self, n: usize) -> Result<()> {
        let index_to_dup = n - 1;
        if self.top.is_none() || self.top.expect("safe") < index_to_dup {
            Err(StackError::NotEnoughValuesOnStack)
        } else {
            let value = self.arr[self.top.expect("safe") - index_to_dup];
            self.push(value)
        }
    }

    pub(super) fn swap(&mut self, n: usize) -> Result<()> {
        if self.top.is_none() || self.top.expect("safe") < n {
            Err(StackError::NotEnoughValuesOnStack)
        } else {
            let top = self.top.expect("safe");
            let temp = self.arr[top - n];
            self.arr[top - n] = self.arr[top];
            self.arr[top] = temp;
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StackResult {
    /// The index of the stack's top.
    top: Option<usize>,
    arr: [U256; 1024],
}

impl From<Stack> for StackResult {
    fn from(stack: Stack) -> Self {
        Self {
            top: stack.top,
            arr: stack.arr,
        }
    }
}

impl StackResult {
    pub fn top(&self) -> Option<usize> {
        self.top
    }
}

impl From<&StackResult> for Box<[U256]> {
    fn from(s: &StackResult) -> Self {
        match s.top() {
            None => Box::default(),
            Some(top) => s
                .arr
                .into_iter()
                .take(top + 1)
                .rev()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}
