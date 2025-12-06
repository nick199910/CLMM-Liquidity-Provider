use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price {
    pub value: Decimal,
}

impl Price {
    /// Creates a new instance of the struct with the given `Decimal` value.
    ///
    /// # Parameters
    /// - `value`: A `Decimal` representing the desired value to initialize the struct.
    ///
    /// # Returns
    /// - A new instance of the struct initialized with the provided `Decimal` value.
    ///
    pub fn new(value: Decimal) -> Self {
        Self { value }
    }

    ///
    /// Inverts the value of the current object.
    ///
    /// This function calculates the reciprocal (1/value) of the `self.value`.
    /// If the value is equal to zero, the function returns a new instance of
    /// the object with the value set to `Decimal::ZERO`.
    ///
    /// # Returns
    ///
    /// A new instance of `Self` with the inverted value. If `self.value` is zero,
    /// the returned instance will have a value of `Decimal::ZERO` to avoid
    /// division by zero.
    ///
    ///
    pub fn invert(&self) -> Self {
        if self.value.is_zero() {
            return Self {
                value: Decimal::ZERO,
            };
        }
        Self {
            value: Decimal::ONE / self.value,
        }
    }
}
