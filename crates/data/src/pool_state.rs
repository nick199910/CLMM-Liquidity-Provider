//! Historical pool state structures.
//!
//! This module provides data structures for storing and querying
//! historical pool states for simulation and backtesting.

use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// A snapshot of pool state at a point in time.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolStateSnapshot {
    /// Timestamp in seconds since epoch.
    pub timestamp: u64,
    /// Current price (token1/token0).
    pub price: Decimal,
    /// Current sqrt price (Q64.64 format as Decimal).
    pub sqrt_price: Option<Decimal>,
    /// Current tick (for CLMM pools).
    pub current_tick: Option<i32>,
    /// Total liquidity in the pool.
    pub liquidity: u128,
    /// Reserve of token A.
    pub reserve_a: Decimal,
    /// Reserve of token B.
    pub reserve_b: Decimal,
    /// Fee rate as decimal (e.g., 0.003 for 0.3%).
    pub fee_rate: Decimal,
    /// 24-hour volume at this snapshot.
    pub volume_24h: Option<Decimal>,
    /// TVL in USD at this snapshot.
    pub tvl_usd: Option<Decimal>,
}

impl PoolStateSnapshot {
    /// Creates a new pool state snapshot.
    #[must_use]
    pub fn new(
        timestamp: u64,
        price: Decimal,
        liquidity: u128,
        reserve_a: Decimal,
        reserve_b: Decimal,
        fee_rate: Decimal,
    ) -> Self {
        Self {
            timestamp,
            price,
            sqrt_price: None,
            current_tick: None,
            liquidity,
            reserve_a,
            reserve_b,
            fee_rate,
            volume_24h: None,
            tvl_usd: None,
        }
    }

    /// Sets the sqrt price.
    #[must_use]
    pub fn with_sqrt_price(mut self, sqrt_price: Decimal) -> Self {
        self.sqrt_price = Some(sqrt_price);
        self
    }

    /// Sets the current tick.
    #[must_use]
    pub fn with_tick(mut self, tick: i32) -> Self {
        self.current_tick = Some(tick);
        self
    }

    /// Sets the 24-hour volume.
    #[must_use]
    pub fn with_volume(mut self, volume: Decimal) -> Self {
        self.volume_24h = Some(volume);
        self
    }

    /// Sets the TVL in USD.
    #[must_use]
    pub fn with_tvl(mut self, tvl: Decimal) -> Self {
        self.tvl_usd = Some(tvl);
        self
    }

    /// Checks if price is within a given range.
    #[must_use]
    pub fn is_price_in_range(&self, lower: Decimal, upper: Decimal) -> bool {
        self.price >= lower && self.price <= upper
    }

    /// Calculates the implied constant product K.
    #[must_use]
    pub fn constant_product_k(&self) -> Decimal {
        self.reserve_a * self.reserve_b
    }
}

/// Historical pool state storage with time-based indexing.
#[derive(Debug, Clone, Default)]
pub struct PoolStateHistory {
    /// Pool address or identifier.
    pub pool_id: String,
    /// Snapshots indexed by timestamp.
    snapshots: BTreeMap<u64, PoolStateSnapshot>,
}

impl PoolStateHistory {
    /// Creates a new empty pool state history.
    #[must_use]
    pub fn new(pool_id: String) -> Self {
        Self {
            pool_id,
            snapshots: BTreeMap::new(),
        }
    }

    /// Creates a pool state history from a vector of snapshots.
    #[must_use]
    pub fn from_snapshots(pool_id: String, snapshots: Vec<PoolStateSnapshot>) -> Self {
        let mut history = Self::new(pool_id);
        for snapshot in snapshots {
            history.insert(snapshot);
        }
        history
    }

    /// Inserts a snapshot into the history.
    pub fn insert(&mut self, snapshot: PoolStateSnapshot) {
        self.snapshots.insert(snapshot.timestamp, snapshot);
    }

    /// Returns the number of snapshots.
    #[must_use]
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns true if the history is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Returns a snapshot at the exact timestamp, if it exists.
    #[must_use]
    pub fn get(&self, timestamp: u64) -> Option<&PoolStateSnapshot> {
        self.snapshots.get(&timestamp)
    }

    /// Returns snapshots in a time range (inclusive).
    #[must_use]
    pub fn range(&self, from: u64, to: u64) -> Vec<&PoolStateSnapshot> {
        self.snapshots
            .range(from..=to)
            .map(|(_, snapshot)| snapshot)
            .collect()
    }

    /// Returns all snapshots as a vector.
    #[must_use]
    pub fn all(&self) -> Vec<&PoolStateSnapshot> {
        self.snapshots.values().collect()
    }

    /// Returns the first snapshot, if any.
    #[must_use]
    pub fn first(&self) -> Option<&PoolStateSnapshot> {
        self.snapshots.values().next()
    }

    /// Returns the last snapshot, if any.
    #[must_use]
    pub fn last(&self) -> Option<&PoolStateSnapshot> {
        self.snapshots.values().next_back()
    }

    /// Returns the snapshot at or before the given timestamp.
    #[must_use]
    pub fn get_at_or_before(&self, timestamp: u64) -> Option<&PoolStateSnapshot> {
        self.snapshots
            .range(..=timestamp)
            .next_back()
            .map(|(_, s)| s)
    }

    /// Returns the snapshot at or after the given timestamp.
    #[must_use]
    pub fn get_at_or_after(&self, timestamp: u64) -> Option<&PoolStateSnapshot> {
        self.snapshots.range(timestamp..).next().map(|(_, s)| s)
    }

    /// Interpolates the price at a given timestamp.
    #[must_use]
    pub fn interpolate_price(&self, timestamp: u64) -> Option<Decimal> {
        // Exact match
        if let Some(snapshot) = self.snapshots.get(&timestamp) {
            return Some(snapshot.price);
        }

        // Find surrounding snapshots
        let before = self.snapshots.range(..timestamp).next_back();
        let after = self.snapshots.range(timestamp..).next();

        match (before, after) {
            (Some((t1, s1)), Some((t2, s2))) => {
                let t1 = *t1 as f64;
                let t2 = *t2 as f64;
                let t = timestamp as f64;

                let ratio = (t - t1) / (t2 - t1);
                let p1 = s1.price;
                let p2 = s2.price;

                let diff = p2 - p1;
                let interpolated = p1 + diff * Decimal::try_from(ratio).unwrap_or(Decimal::ZERO);
                Some(interpolated)
            }
            (Some((_, s)), None) => Some(s.price),
            (None, Some((_, s))) => Some(s.price),
            (None, None) => None,
        }
    }

    /// Returns the price history as a vector of (timestamp, price) pairs.
    #[must_use]
    pub fn price_history(&self) -> Vec<(u64, Decimal)> {
        self.snapshots
            .iter()
            .map(|(ts, s)| (*ts, s.price))
            .collect()
    }

    /// Returns the liquidity history as a vector of (timestamp, liquidity) pairs.
    #[must_use]
    pub fn liquidity_history(&self) -> Vec<(u64, u128)> {
        self.snapshots
            .iter()
            .map(|(ts, s)| (*ts, s.liquidity))
            .collect()
    }

    /// Calculates the average liquidity over the history.
    #[must_use]
    pub fn average_liquidity(&self) -> u128 {
        if self.snapshots.is_empty() {
            return 0;
        }

        let sum: u128 = self.snapshots.values().map(|s| s.liquidity).sum();
        sum / self.snapshots.len() as u128
    }

    /// Calculates the price volatility over the history.
    #[must_use]
    pub fn price_volatility(&self) -> Option<Decimal> {
        let prices: Vec<Decimal> = self.snapshots.values().map(|s| s.price).collect();
        if prices.len() < 2 {
            return None;
        }

        // Calculate returns
        let returns: Vec<Decimal> = prices
            .windows(2)
            .filter_map(|w| {
                if w[0].is_zero() {
                    None
                } else {
                    Some((w[1] - w[0]) / w[0])
                }
            })
            .collect();

        if returns.is_empty() {
            return None;
        }

        // Calculate mean
        let n = Decimal::from(returns.len());
        let mean: Decimal = returns.iter().copied().sum::<Decimal>() / n;

        // Calculate variance
        let variance: Decimal = returns
            .iter()
            .map(|r| {
                let diff = *r - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / n;

        // Return standard deviation
        let var_f64 = variance.to_string().parse::<f64>().unwrap_or(0.0);
        Decimal::try_from(var_f64.sqrt()).ok()
    }

    /// Returns the time range covered by this history.
    #[must_use]
    pub fn time_range(&self) -> Option<(u64, u64)> {
        let start = self.snapshots.keys().next()?;
        let end = self.snapshots.keys().next_back()?;
        Some((*start, *end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_snapshots() -> Vec<PoolStateSnapshot> {
        vec![
            PoolStateSnapshot::new(
                1000,
                dec!(100),
                1_000_000,
                dec!(10000),
                dec!(1000000),
                dec!(0.003),
            ),
            PoolStateSnapshot::new(
                2000,
                dec!(105),
                1_100_000,
                dec!(9500),
                dec!(997500),
                dec!(0.003),
            ),
            PoolStateSnapshot::new(
                3000,
                dec!(102),
                1_050_000,
                dec!(9800),
                dec!(999600),
                dec!(0.003),
            ),
            PoolStateSnapshot::new(
                4000,
                dec!(110),
                1_200_000,
                dec!(9000),
                dec!(990000),
                dec!(0.003),
            ),
            PoolStateSnapshot::new(
                5000,
                dec!(108),
                1_150_000,
                dec!(9200),
                dec!(993600),
                dec!(0.003),
            ),
        ]
    }

    #[test]
    fn test_snapshot_in_range() {
        let snapshot = PoolStateSnapshot::new(
            1000,
            dec!(100),
            1_000_000,
            dec!(10000),
            dec!(1000000),
            dec!(0.003),
        );

        assert!(snapshot.is_price_in_range(dec!(90), dec!(110)));
        assert!(snapshot.is_price_in_range(dec!(100), dec!(100)));
        assert!(!snapshot.is_price_in_range(dec!(101), dec!(110)));
    }

    #[test]
    fn test_history_range_query() {
        let snapshots = create_test_snapshots();
        let history = PoolStateHistory::from_snapshots("pool1".to_string(), snapshots);

        let range = history.range(2000, 4000);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].timestamp, 2000);
        assert_eq!(range[2].timestamp, 4000);
    }

    #[test]
    fn test_history_interpolation() {
        let snapshots = create_test_snapshots();
        let history = PoolStateHistory::from_snapshots("pool1".to_string(), snapshots);

        // Exact match
        let exact = history.interpolate_price(2000);
        assert_eq!(exact, Some(dec!(105)));

        // Interpolation between 2000 (105) and 3000 (102)
        let interp = history.interpolate_price(2500);
        assert!(interp.is_some());
        let price = interp.unwrap();
        assert!(price > dec!(102) && price < dec!(105));
    }

    #[test]
    fn test_history_get_at_or_before() {
        let snapshots = create_test_snapshots();
        let history = PoolStateHistory::from_snapshots("pool1".to_string(), snapshots);

        let snapshot = history.get_at_or_before(2500);
        assert!(snapshot.is_some());
        assert_eq!(snapshot.unwrap().timestamp, 2000);
    }

    #[test]
    fn test_history_average_liquidity() {
        let snapshots = create_test_snapshots();
        let history = PoolStateHistory::from_snapshots("pool1".to_string(), snapshots);

        let avg = history.average_liquidity();
        // (1_000_000 + 1_100_000 + 1_050_000 + 1_200_000 + 1_150_000) / 5 = 1_100_000
        assert_eq!(avg, 1_100_000);
    }

    #[test]
    fn test_history_time_range() {
        let snapshots = create_test_snapshots();
        let history = PoolStateHistory::from_snapshots("pool1".to_string(), snapshots);

        let range = history.time_range();
        assert_eq!(range, Some((1000, 5000)));
    }
}
