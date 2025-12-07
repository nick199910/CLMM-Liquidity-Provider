//! PnL tracking for LP positions.

use clmm_lp_domain::metrics::impermanent_loss::calculate_il_concentrated;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::collections::HashMap;
use tracing::debug;

/// Entry state for a position.
#[derive(Debug, Clone)]
pub struct PositionEntry {
    /// Entry price.
    pub entry_price: Decimal,
    /// Entry value in USD.
    pub entry_value_usd: Decimal,
    /// Entry timestamp.
    pub entry_timestamp: chrono::DateTime<chrono::Utc>,
    /// Initial token A amount.
    pub initial_amount_a: u64,
    /// Initial token B amount.
    pub initial_amount_b: u64,
    /// Lower tick at entry.
    pub tick_lower: i32,
    /// Upper tick at entry.
    pub tick_upper: i32,
}

/// PnL calculation result.
#[derive(Debug, Clone, Default)]
pub struct PnLResult {
    /// Current position value in USD.
    pub current_value_usd: Decimal,
    /// HODL value (if tokens were held instead of LP).
    pub hodl_value_usd: Decimal,
    /// Impermanent loss in USD.
    pub il_usd: Decimal,
    /// Impermanent loss percentage.
    pub il_pct: Decimal,
    /// Total fees earned in USD.
    pub fees_usd: Decimal,
    /// Net PnL in USD (value change + fees - IL).
    pub net_pnl_usd: Decimal,
    /// Net PnL percentage.
    pub net_pnl_pct: Decimal,
    /// Performance vs HODL.
    pub vs_hodl_usd: Decimal,
    /// Annualized return.
    pub apy: Decimal,
}

/// Tracks PnL for multiple positions.
pub struct PnLTracker {
    /// Entry states for positions.
    entries: HashMap<String, PositionEntry>,
}

impl PnLTracker {
    /// Creates a new PnL tracker.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Records a position entry.
    #[allow(clippy::too_many_arguments)]
    pub fn record_entry(
        &mut self,
        position_address: &str,
        entry_price: Decimal,
        entry_value_usd: Decimal,
        amount_a: u64,
        amount_b: u64,
        tick_lower: i32,
        tick_upper: i32,
    ) {
        let entry = PositionEntry {
            entry_price,
            entry_value_usd,
            entry_timestamp: chrono::Utc::now(),
            initial_amount_a: amount_a,
            initial_amount_b: amount_b,
            tick_lower,
            tick_upper,
        };

        self.entries.insert(position_address.to_string(), entry);

        debug!(
            position = position_address,
            entry_price = %entry_price,
            entry_value = %entry_value_usd,
            "Recorded position entry"
        );
    }

    /// Calculates PnL for a position.
    #[allow(clippy::too_many_arguments)]
    pub fn calculate_pnl(
        &self,
        position_address: &str,
        current_price: Decimal,
        current_amount_a: u64,
        current_amount_b: u64,
        fees_a: u64,
        fees_b: u64,
        price_a_usd: Decimal,
        price_b_usd: Decimal,
    ) -> Option<PnLResult> {
        let entry = self.entries.get(position_address)?;

        // Calculate current value
        let current_value_usd = Decimal::from(current_amount_a) * price_a_usd
            / Decimal::from(1_000_000_000u64)
            + Decimal::from(current_amount_b) * price_b_usd / Decimal::from(1_000_000u64);

        // Calculate HODL value (what if we just held the initial tokens)
        let hodl_value_usd = Decimal::from(entry.initial_amount_a) * price_a_usd
            / Decimal::from(1_000_000_000u64)
            + Decimal::from(entry.initial_amount_b) * price_b_usd / Decimal::from(1_000_000u64);

        // Calculate fees in USD
        let fees_usd = Decimal::from(fees_a) * price_a_usd / Decimal::from(1_000_000_000u64)
            + Decimal::from(fees_b) * price_b_usd / Decimal::from(1_000_000u64);

        // Calculate IL
        let lower_price = clmm_lp_protocols::prelude::tick_to_price(entry.tick_lower);
        let upper_price = clmm_lp_protocols::prelude::tick_to_price(entry.tick_upper);

        let il_pct_result =
            calculate_il_concentrated(entry.entry_price, current_price, lower_price, upper_price);
        let il_pct = il_pct_result.unwrap_or(Decimal::ZERO);

        let il_usd = entry.entry_value_usd * il_pct.abs();

        // Calculate net PnL
        let value_change = current_value_usd - entry.entry_value_usd;
        let net_pnl_usd = value_change + fees_usd;

        let net_pnl_pct = if entry.entry_value_usd.is_zero() {
            Decimal::ZERO
        } else {
            net_pnl_usd / entry.entry_value_usd * Decimal::from(100)
        };

        // Performance vs HODL
        let vs_hodl_usd = current_value_usd + fees_usd - hodl_value_usd;

        // Calculate APY
        let duration = chrono::Utc::now() - entry.entry_timestamp;
        let days = duration.num_days().max(1) as f64;
        let apy = if days > 0.0 && !entry.entry_value_usd.is_zero() {
            let daily_return = net_pnl_pct / Decimal::from_f64(days).unwrap_or(Decimal::ONE);
            daily_return * Decimal::from(365)
        } else {
            Decimal::ZERO
        };

        Some(PnLResult {
            current_value_usd,
            hodl_value_usd,
            il_usd,
            il_pct,
            fees_usd,
            net_pnl_usd,
            net_pnl_pct,
            vs_hodl_usd,
            apy,
        })
    }

    /// Gets the entry for a position.
    pub fn get_entry(&self, position_address: &str) -> Option<&PositionEntry> {
        self.entries.get(position_address)
    }

    /// Removes a position entry.
    pub fn remove_entry(&mut self, position_address: &str) {
        self.entries.remove(position_address);
    }

    /// Gets all tracked positions.
    pub fn get_all_entries(&self) -> &HashMap<String, PositionEntry> {
        &self.entries
    }
}

impl Default for PnLTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_record_entry() {
        let mut tracker = PnLTracker::new();

        tracker.record_entry(
            "position123",
            dec!(100),
            dec!(1000),
            1_000_000_000,
            100_000_000,
            -1000,
            1000,
        );

        let entry = tracker.get_entry("position123");
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.entry_price, dec!(100));
        assert_eq!(entry.entry_value_usd, dec!(1000));
    }
}
