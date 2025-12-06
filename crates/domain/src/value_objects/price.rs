use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// The `Price` struct represents a monetary value using a `Decimal` type for precision.
///
/// This struct derives several traits to enhance its usability:
/// - `Debug`: Allows for the `Price` struct to be formatted using the `{:?}` formatter,
///   useful for debugging purposes.
/// - `Clone`: Enables creating an exact duplicate of a `Price` instance.
/// - `Copy`: Enables creating duplicates of the `Price` instance without requiring an explicit call to `clone`.
/// - `PartialEq` and `Eq`: Allow for equality comparisons between two `Price` instances.
/// - `PartialOrd` and `Ord`: Allow for ordering comparisons between `Price` instances,
///   enabling sorting, minimum, and maximum operations.
/// - `Serialize` and `Deserialize`: Provide support for serializing and deserializing
///   `Price` instances, useful for formats like JSON.
///
/// # Fields
///
/// * `value` (`Decimal`): The numeric representation of the price. The `Decimal` type
///   is used to ensure precision, making the `Price` struct suitable for use cases
///   involving monetary calculations without the common pitfalls of floating-point inaccuracies.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price {
    /// The underlying `Decimal` value representing the price.
    pub value: Decimal,
}

impl Price {
    /// Creates a new Price.
    pub fn new(value: Decimal) -> Self {
        Self { value }
    }

    /// Inverts the price (1/price).
    pub fn invert(&self) -> Self {
        if self.value.is_zero() {
            // Handle zero price appropriately, maybe return zero or infinity representation
            // For now, just return zero to avoid panic
            return Self {
                value: Decimal::ZERO,
            };
        }
        Self {
            value: Decimal::ONE / self.value,
        }
    }
}
