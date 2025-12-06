use serde::{Deserialize, Serialize};

/// Represents a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The mint address of the token.
    pub mint_address: String,
    /// The symbol of the token.
    pub symbol: String,
    /// The number of decimals of the token.
    pub decimals: u8,
    /// The name of the token.
    pub name: String,
    /// The CoinGecko ID of the token.
    pub coingecko_id: Option<String>,
}

impl Token {
    /// Creates a new Token.
    pub fn new(
        mint: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
        name: impl Into<String>,
    ) -> Self {
        Self {
            mint_address: mint.into(),
            symbol: symbol.into(),
            decimals,
            name: name.into(),
            coingecko_id: None,
        }
    }
}
