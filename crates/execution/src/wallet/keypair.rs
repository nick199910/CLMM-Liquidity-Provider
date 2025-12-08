//! Wallet implementation for transaction signing.

use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use std::fs;
use std::path::Path;
use tracing::info;
use zeroize::Zeroizing;

/// Wallet abstraction for signing transactions.
pub struct Wallet {
    /// The keypair.
    keypair: Keypair,
    /// Wallet label.
    label: String,
}

impl Wallet {
    /// Creates a wallet from a keypair.
    pub fn from_keypair(keypair: Keypair, label: impl Into<String>) -> Self {
        Self {
            keypair,
            label: label.into(),
        }
    }

    /// Loads a wallet from a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the keypair JSON file
    /// * `label` - Human-readable label for the wallet
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or parsed.
    pub fn from_file(path: impl AsRef<Path>, label: impl Into<String>) -> Result<Self> {
        let path = path.as_ref();
        let label = label.into();

        info!(path = %path.display(), label = %label, "Loading wallet from file");

        let contents =
            Zeroizing::new(fs::read_to_string(path).context("Failed to read keypair file")?);

        let bytes: Vec<u8> =
            serde_json::from_str(&contents).context("Failed to parse keypair JSON")?;

        let bytes_array: [u8; 32] = bytes[..32].try_into().context("Invalid keypair length")?;
        let keypair = Keypair::new_from_array(bytes_array);

        Ok(Self { keypair, label })
    }

    /// Loads a wallet from an environment variable.
    ///
    /// # Arguments
    /// * `var_name` - Name of the environment variable
    /// * `label` - Human-readable label for the wallet
    ///
    /// # Errors
    /// Returns an error if the variable is not set or cannot be parsed.
    pub fn from_env(var_name: &str, label: impl Into<String>) -> Result<Self> {
        let label = label.into();

        info!(var = var_name, label = %label, "Loading wallet from environment");

        let value = Zeroizing::new(
            std::env::var(var_name)
                .context(format!("Environment variable {} not set", var_name))?,
        );

        // Try to parse as JSON array first
        if let Ok(bytes) = serde_json::from_str::<Vec<u8>>(&value) {
            let bytes_array: [u8; 32] = bytes[..32].try_into().context("Invalid keypair length")?;
            let keypair = Keypair::new_from_array(bytes_array);
            return Ok(Self { keypair, label });
        }

        // Try to parse as base58
        let keypair = Keypair::from_base58_string(&value);

        Ok(Self { keypair, label })
    }

    /// Returns the public key.
    #[must_use]
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Returns the wallet label.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> solana_sdk::signature::Signature {
        self.keypair.sign_message(message)
    }

    /// Returns a reference to the keypair for signing transactions.
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}
