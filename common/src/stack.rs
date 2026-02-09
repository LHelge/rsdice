use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StackError {
    #[error("Stack overflow")]
    Overflow,

    #[error("Stack underflow")]
    Empty,
}

type Result<T> = std::result::Result<T, StackError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    count: usize,
}

impl Default for Stack {
    fn default() -> Self {
        Stack { count: 1 }
    }
}

impl Stack {
    pub const MAX: usize = 8;
    pub const MIN: usize = 1;

    pub fn count(&self) -> usize {
        self.count
    }

    pub fn increment(&mut self) -> Result<()> {
        if self.count >= Self::MAX {
            Err(StackError::Overflow)
        } else {
            self.count += 1;
            Ok(())
        }
    }

    pub fn decrement(&mut self) -> Result<()> {
        if self.count <= Self::MIN {
            Err(StackError::Empty)
        } else {
            self.count -= 1;
            Ok(())
        }
    }

    pub fn split(mut self) -> Result<(Stack, Stack)> {
        if self.count <= Self::MIN {
            Err(StackError::Empty)
        } else {
            self.count -= 1;
            Ok((Stack { count: 1 }, self))
        }
    }
}
