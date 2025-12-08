//! Wallet manager for handling multiple wallets.

use super::Wallet;
use std::collections::HashMap;

/// Wallet manager for handling multiple wallets.
pub struct WalletManager {
    /// Available wallets.
    wallets: HashMap<String, Wallet>,
    /// Default wallet label.
    default_wallet: Option<String>,
}

impl WalletManager {
    /// Creates a new wallet manager.
    #[must_use]
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
            default_wallet: None,
        }
    }

    /// Adds a wallet.
    pub fn add_wallet(&mut self, wallet: Wallet) {
        let label = wallet.label().to_string();
        self.wallets.insert(label.clone(), wallet);

        if self.default_wallet.is_none() {
            self.default_wallet = Some(label);
        }
    }

    /// Gets a wallet by label.
    pub fn get_wallet(&self, label: &str) -> Option<&Wallet> {
        self.wallets.get(label)
    }

    /// Gets the default wallet.
    pub fn get_default(&self) -> Option<&Wallet> {
        self.default_wallet
            .as_ref()
            .and_then(|label| self.wallets.get(label))
    }

    /// Sets the default wallet.
    ///
    /// Returns true if the wallet exists and was set as default.
    pub fn set_default(&mut self, label: &str) -> bool {
        if self.wallets.contains_key(label) {
            self.default_wallet = Some(label.to_string());
            true
        } else {
            false
        }
    }

    /// Lists all wallet labels.
    pub fn list_wallets(&self) -> Vec<&str> {
        self.wallets.keys().map(String::as_str).collect()
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::signature::Keypair;

    #[test]
    fn test_wallet_manager() {
        let mut manager = WalletManager::new();

        let keypair = Keypair::new();
        let wallet = Wallet::from_keypair(keypair, "test");

        manager.add_wallet(wallet);

        assert!(manager.get_wallet("test").is_some());
        assert!(manager.get_default().is_some());
    }
}
