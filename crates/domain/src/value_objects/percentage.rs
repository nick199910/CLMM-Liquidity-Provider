use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

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

    /// Converts the stored `Decimal` value into basis points (bps) and returns it as a `u32`.
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
