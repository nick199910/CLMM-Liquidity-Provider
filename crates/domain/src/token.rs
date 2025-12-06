use primitive_types::U256;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The address of the token.
    pub address: String,
    /// The symbol of the token.
    pub symbol: String,
    /// The decimals of the token.
    pub decimals: u8,
    /// The name of the token.
    pub name: String,
}

impl Token {
    /// Creates a new Token.
    ///
    /// # Arguments
    ///
    /// * `address`: The address of the token.
    /// * `symbol`: The symbol of the token.
    /// * `decimals`: The decimals of the token.
    /// * `name`: The name of the token.
    pub fn new(
        address: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
        name: impl Into<String>,
    ) -> Self {
        Self {
            address: address.into(),
            symbol: symbol.into(),
            decimals,
            name: name.into(),
        }
    }
}

/// Represents a token amount.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TokenAmount(pub U256);

impl TokenAmount {
    /// Creates a new TokenAmount.
    ///
    /// # Arguments
    ///
    /// * `amount`: The amount of the token.
    pub fn new(amount: impl Into<U256>) -> Self {
        Self(amount.into())
    }

    /// Creates a zero TokenAmount.
    pub fn zero() -> Self {
        Self(U256::zero())
    }

    /// Returns the amount as U256.
    pub fn as_u256(&self) -> U256 {
        self.0
    }
}

impl From<u64> for TokenAmount {
    /// Converts a u64 into a TokenAmount.
    fn from(v: u64) -> Self {
        Self(U256::from(v))
    }
}

impl From<u128> for TokenAmount {
    /// Converts a u128 into a TokenAmount.
    fn from(v: u128) -> Self {
        Self(U256::from(v))
    }
}

impl fmt::Display for TokenAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A structure representing a monetary value, encapsulated as a single field tuple.
///
/// The `Price` struct wraps around a `Decimal` value, providing support for precise
/// representation of monetary amounts or other values requiring decimal precision.
///
/// # Derive Attributes
/// - `Debug`: Enables formatting using the `{:?}` formatter.
/// - `Clone`: Allows for creating a copy of the `Price` value.
/// - `Copy`: Marks the `Price` values as copyable, meaning they can be duplicated implicitly
///   without explicitly calling the `Clone` trait.
/// - `PartialEq` and `Eq`: Allows for equality comparison between `Price` instances.
/// - `PartialOrd` and `Ord`: Supports comparison and ordering of `Price` values.
/// - `Serialize` and `Deserialize`: Enables conversion of `Price` to and from data formats
///   (e.g., JSON) for persistence or communication.
///
/// # Fields
/// - `0: Decimal`
///   - The underlying `Decimal` value which stores the exact monetary amount.
///
/// # Usage
/// This struct is designed to represent and manipulate financial amounts, ensuring
/// high levels of precision and avoiding floating-point inaccuracies.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Price(pub Decimal);

impl Price {
    /// Creates a new instance of the struct with the given price.
    ///
    /// # Arguments
    ///
    /// * `price` - A `Decimal` value representing the price to initialize the struct with.
    ///
    /// # Returns
    ///
    /// A new instance of the struct containing the specified price.
    ///
    pub fn new(price: Decimal) -> Self {
        Self(price)
    }
}
