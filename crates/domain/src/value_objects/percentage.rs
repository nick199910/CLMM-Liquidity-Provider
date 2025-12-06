use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

/// A wrapper struct representing a percentage value, built on top of a `Decimal` type.
///
/// The `Percentage` struct is primarily used to represent percentage values in a clear
/// and type-safe manner. This struct encapsulates a `Decimal` value, ensuring that it
/// can be used for precise arithmetic operations without losing accuracy, as percentages
/// often require precision beyond simple floating-point representations.
///
/// # Derives
/// - `Debug`: Allows this struct to be formatted using the `{:?}` formatter for debugging purposes.
/// - `Clone`: Enables the creation of a duplicate of a `Percentage` value.
/// - `Copy`: Allows the `Percentage` struct to be copied rather than moved,
///   making it convenient and efficient for use in computations.
/// - `Serialize`: Allows a `Percentage` value to be serialized, which is especially useful
///   for saving or transferring the value as part of an external format (e.g., JSON).
/// - `Deserialize`: Enables deserialization of a `Percentage` value from an external format,
///   allowing it to be reconstructed from serialized data.
///
/// # Fields
/// - `0` (`Decimal`): A wrapped `Decimal` value representing the percentage value.
///
/// The `Percentage` struct is designed to be flexible and precise for use cases
/// involving financial computations, statistics, or any domain requiring accurate
/// representation of percentages.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Percentage(pub Decimal);

impl Percentage {
    /// Creates a new instance of the type from a given value in basis points (bps).
    ///
    /// # Parameters
    /// - `bps`: A `u32` value representing the number of basis points. Basis points
    ///   are a unit of measure used in finance, where 1 basis point equals 0.01% (1/100th of a percent).
    ///
    /// # Returns
    /// A new instance of the type with the value converted from basis points to its equivalent
    /// decimal representation. The conversion is performed by dividing the input by 10,000
    /// (as 10,000 basis points equals 1 in decimal form).
    ///
    pub fn from_bps(bps: u32) -> Self {
        Self(Decimal::from(bps) / Decimal::from(10000))
    }

    /// Converts the Percentage to basis points (bps).
    ///
    /// Basis points are calculated by multiplying the internal value by 10,000. The conversion
    /// is performed using the `Decimal::from(10000)` multiplier and then casting the resulting
    /// value to a `u32`.
    ///
    /// # Returns
    ///
    /// * A `u32` value representing the basis points (bps). If the conversion to `u32` fails,
    ///   it will return `0` as a fallback.
    ///
    /// # Panics
    ///
    /// * The method does not panic but will return `0` if the conversion to `u32` is unsuccessful.
    ///
    pub fn to_bps(&self) -> u32 {
        (self.0 * Decimal::from(10000)).to_u32().unwrap_or(0)
    }
}
