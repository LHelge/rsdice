use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum StackError {
    #[error("Stack overflow")]
    Overflow,

    #[error("Stack underflow")]
    Underflow,
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

    pub fn is_full(&self) -> bool {
        self.count >= Self::MAX
    }

    pub fn is_single(&self) -> bool {
        self.count <= Self::MIN
    }

    pub fn increment(&mut self) -> Result<()> {
        if self.is_full() {
            Err(StackError::Overflow)
        } else {
            self.count += 1;
            Ok(())
        }
    }

    pub fn decrement(&mut self) -> Result<()> {
        if self.is_single() {
            Err(StackError::Underflow)
        } else {
            self.count -= 1;
            Ok(())
        }
    }

    pub fn defeat(&mut self) {
        self.count = Self::MIN;
    }

    pub fn split(mut self) -> Result<(Stack, Stack)> {
        if self.is_single() {
            Err(StackError::Underflow)
        } else {
            self.count -= Self::MIN;
            Ok((Stack { count: Self::MIN }, self))
        }
    }

    // Simulate defence roll with all dice in the stack.
    pub fn roll(&self) -> usize {
        // Simulate rolling `count` dice and summing the results.
        (0..self.count).map(|_| rand::random_range(1..=6)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==== Default ====

    #[test]
    fn default_stack_has_count_one() {
        let stack = Stack::default();
        assert_eq!(stack.count(), Stack::MIN);
    }

    // ==== Constants ====

    #[test]
    fn min_is_one() {
        assert_eq!(Stack::MIN, 1);
    }

    #[test]
    fn max_is_eight() {
        assert_eq!(Stack::MAX, 8);
    }

    // ==== count ====

    #[test]
    fn count_returns_current_value() {
        let stack = Stack::default();
        assert_eq!(stack.count(), 1);
    }

    // ==== is_full ====

    #[test]
    fn is_full_returns_false_for_default() {
        assert!(!Stack::default().is_full());
    }

    #[test]
    fn is_full_returns_true_at_max() {
        let mut stack = Stack::default();
        for _ in 1..Stack::MAX {
            stack.increment().unwrap();
        }
        assert!(stack.is_full());
        assert_eq!(stack.count(), Stack::MAX);
    }

    // ==== is_single ====

    #[test]
    fn is_single_returns_true_for_default() {
        assert!(Stack::default().is_single());
    }

    #[test]
    fn is_single_returns_false_after_increment() {
        let mut stack = Stack::default();
        stack.increment().unwrap();
        assert!(!stack.is_single());
    }

    // ==== increment ====

    #[test]
    fn increment_increases_count_by_one() {
        let mut stack = Stack::default();
        stack.increment().unwrap();
        assert_eq!(stack.count(), 2);
    }

    #[test]
    fn increment_up_to_max() {
        let mut stack = Stack::default();
        for i in 2..=Stack::MAX {
            stack.increment().unwrap();
            assert_eq!(stack.count(), i);
        }
        assert!(stack.is_full());
    }

    #[test]
    fn increment_past_max_returns_overflow() {
        let mut stack = Stack::default();
        for _ in 1..Stack::MAX {
            stack.increment().unwrap();
        }
        let err = stack.increment().unwrap_err();
        assert!(matches!(err, StackError::Overflow));
        // Count should remain at MAX
        assert_eq!(stack.count(), Stack::MAX);
    }

    #[test]
    fn increment_overflow_error_displays_message() {
        let err = StackError::Overflow;
        assert_eq!(err.to_string(), "Stack overflow");
    }

    // ==== decrement ====

    #[test]
    fn decrement_decreases_count_by_one() {
        let mut stack = Stack::default();
        stack.increment().unwrap();
        stack.increment().unwrap();
        stack.decrement().unwrap();
        assert_eq!(stack.count(), 2);
    }

    #[test]
    fn decrement_down_to_min() {
        let mut stack = Stack::default();
        for _ in 1..Stack::MAX {
            stack.increment().unwrap();
        }
        for _ in 1..Stack::MAX {
            stack.decrement().unwrap();
        }
        assert_eq!(stack.count(), Stack::MIN);
        assert!(stack.is_single());
    }

    #[test]
    fn decrement_past_min_returns_underflow() {
        let mut stack = Stack::default();
        let err = stack.decrement().unwrap_err();
        assert!(matches!(err, StackError::Underflow));
        // Count should remain at MIN
        assert_eq!(stack.count(), Stack::MIN);
    }

    #[test]
    fn decrement_underflow_error_displays_message() {
        let err = StackError::Underflow;
        assert_eq!(err.to_string(), "Stack underflow");
    }

    // ==== split ====

    #[test]
    fn split_single_stack_returns_underflow() {
        let stack = Stack::default();
        let err = stack.split().unwrap_err();
        assert!(matches!(err, StackError::Underflow));
    }

    #[test]
    fn split_two_stack_returns_one_and_one() {
        let mut stack = Stack::default();
        stack.increment().unwrap();
        let (left, right) = stack.split().unwrap();
        assert_eq!(left.count(), Stack::MIN);
        assert_eq!(right.count(), 1);
    }

    #[test]
    fn split_max_stack_returns_one_and_rest() {
        let mut stack = Stack::default();
        for _ in 1..Stack::MAX {
            stack.increment().unwrap();
        }
        let (left, right) = stack.split().unwrap();
        assert_eq!(left.count(), Stack::MIN);
        assert_eq!(right.count(), Stack::MAX - Stack::MIN);
    }

    #[test]
    fn split_preserves_total_count() {
        let mut stack = Stack::default();
        for _ in 1..5 {
            stack.increment().unwrap();
        }
        let original_count = stack.count();
        let (left, right) = stack.split().unwrap();
        assert_eq!(left.count() + right.count(), original_count);
    }

    #[test]
    fn split_left_is_always_min() {
        for total in 2..=Stack::MAX {
            let mut stack = Stack::default();
            for _ in 1..total {
                stack.increment().unwrap();
            }
            let (left, _) = stack.split().unwrap();
            assert_eq!(left.count(), Stack::MIN);
        }
    }

    // ==== Serialization ====

    #[test]
    fn serialize_deserialize_roundtrip() {
        let mut stack = Stack::default();
        stack.increment().unwrap();
        stack.increment().unwrap();
        let json = serde_json::to_string(&stack).unwrap();
        let deserialized: Stack = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.count(), stack.count());
    }

    #[test]
    fn clone_produces_independent_copy() {
        let mut stack = Stack::default();
        stack.increment().unwrap();
        let mut cloned = stack.clone();
        cloned.increment().unwrap();
        assert_eq!(stack.count(), 2);
        assert_eq!(cloned.count(), 3);
    }

    // ==== Increment then decrement ====

    #[test]
    fn increment_then_decrement_returns_to_original() {
        let mut stack = Stack::default();
        let original = stack.count();
        stack.increment().unwrap();
        stack.decrement().unwrap();
        assert_eq!(stack.count(), original);
    }
}
