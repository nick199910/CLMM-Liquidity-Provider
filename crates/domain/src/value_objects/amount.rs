use primitive_types::U256;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

/// Represents an amount with decimals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Amount {
    /// The raw amount.
    pub raw: U256,
    /// The number of decimals.
    pub decimals: u8,
}

impl Amount {
    /// Creates a new Amount.
    pub fn new(raw: U256, decimals: u8) -> Self {
        Self { raw, decimals }
    }

    /// Creates an Amount from a decimal.
    pub fn from_decimal(d: Decimal, decimals: u8) -> Self {
        let multiplier = Decimal::from(10u64.pow(decimals as u32));
        let raw_decimal = d * multiplier;
        // This conversion is simplistic and might panic on overflow or negative
        let raw_u128 = raw_decimal.to_u128().unwrap_or(0);
        Self {
            raw: U256::from(raw_u128),
            decimals,
        }
    }

    /// Converts the Amount to a decimal.
    pub fn to_decimal(&self) -> Decimal {
        let raw_u128 = self.raw.low_u128(); // Truncates if > u128::MAX, careful
        let d = Decimal::from(raw_u128);
        let divisor = Decimal::from(10u64.pow(self.decimals as u32));
        d / divisor
    }
}
