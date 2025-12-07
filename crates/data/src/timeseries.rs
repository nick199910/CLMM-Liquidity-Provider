//! Time series data structures for market data.
//!
//! This module provides efficient data structures for storing and querying
//! OHLCV (Open, High, Low, Close, Volume) data with time-based indexing.

use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// A single OHLCV candle.
#[derive(Debug, Clone, PartialEq)]
pub struct OhlcvCandle {
    /// Timestamp in seconds since epoch.
    pub timestamp: u64,
    /// Opening price.
    pub open: Decimal,
    /// Highest price during the period.
    pub high: Decimal,
    /// Lowest price during the period.
    pub low: Decimal,
    /// Closing price.
    pub close: Decimal,
    /// Trading volume during the period.
    pub volume: Decimal,
}

impl OhlcvCandle {
    /// Creates a new OHLCV candle.
    #[must_use]
    pub fn new(
        timestamp: u64,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
    ) -> Self {
        Self {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
        }
    }

    /// Returns the typical price (HLC average).
    #[must_use]
    pub fn typical_price(&self) -> Decimal {
        (self.high + self.low + self.close) / Decimal::from(3)
    }

    /// Returns the price range (high - low).
    #[must_use]
    pub fn range(&self) -> Decimal {
        self.high - self.low
    }

    /// Returns true if this is a bullish candle (close > open).
    #[must_use]
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Returns true if this is a bearish candle (close < open).
    #[must_use]
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }
}

/// Time series of OHLCV data with efficient indexing.
#[derive(Debug, Clone, Default)]
pub struct TimeSeries {
    /// Candles indexed by timestamp.
    candles: BTreeMap<u64, OhlcvCandle>,
    /// Interval between candles in seconds.
    interval_seconds: u64,
}

impl TimeSeries {
    /// Creates a new empty time series.
    #[must_use]
    pub fn new(interval_seconds: u64) -> Self {
        Self {
            candles: BTreeMap::new(),
            interval_seconds,
        }
    }

    /// Creates a time series from a vector of candles.
    #[must_use]
    pub fn from_candles(candles: Vec<OhlcvCandle>, interval_seconds: u64) -> Self {
        let mut ts = Self::new(interval_seconds);
        for candle in candles {
            ts.insert(candle);
        }
        ts
    }

    /// Inserts a candle into the time series.
    pub fn insert(&mut self, candle: OhlcvCandle) {
        self.candles.insert(candle.timestamp, candle);
    }

    /// Returns the number of candles in the series.
    #[must_use]
    pub fn len(&self) -> usize {
        self.candles.len()
    }

    /// Returns true if the series is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.candles.is_empty()
    }

    /// Returns the interval between candles in seconds.
    #[must_use]
    pub fn interval(&self) -> u64 {
        self.interval_seconds
    }

    /// Returns a candle at the exact timestamp, if it exists.
    #[must_use]
    pub fn get(&self, timestamp: u64) -> Option<&OhlcvCandle> {
        self.candles.get(&timestamp)
    }

    /// Returns candles in a time range (inclusive).
    #[must_use]
    pub fn range(&self, from: u64, to: u64) -> Vec<&OhlcvCandle> {
        self.candles
            .range(from..=to)
            .map(|(_, candle)| candle)
            .collect()
    }

    /// Returns all candles as a vector.
    #[must_use]
    pub fn all(&self) -> Vec<&OhlcvCandle> {
        self.candles.values().collect()
    }

    /// Returns the first candle, if any.
    #[must_use]
    pub fn first(&self) -> Option<&OhlcvCandle> {
        self.candles.values().next()
    }

    /// Returns the last candle, if any.
    #[must_use]
    pub fn last(&self) -> Option<&OhlcvCandle> {
        self.candles.values().next_back()
    }

    /// Returns the first timestamp in the series.
    #[must_use]
    pub fn start_time(&self) -> Option<u64> {
        self.candles.keys().next().copied()
    }

    /// Returns the last timestamp in the series.
    #[must_use]
    pub fn end_time(&self) -> Option<u64> {
        self.candles.keys().next_back().copied()
    }

    /// Returns the closing prices as a vector.
    #[must_use]
    pub fn close_prices(&self) -> Vec<Decimal> {
        self.candles.values().map(|c| c.close).collect()
    }

    /// Returns the volumes as a vector.
    #[must_use]
    pub fn volumes(&self) -> Vec<Decimal> {
        self.candles.values().map(|c| c.volume).collect()
    }

    /// Interpolates a price at a given timestamp.
    /// If the exact timestamp exists, returns that price.
    /// Otherwise, linearly interpolates between surrounding candles.
    #[must_use]
    pub fn interpolate_price(&self, timestamp: u64) -> Option<Decimal> {
        // Exact match
        if let Some(candle) = self.candles.get(&timestamp) {
            return Some(candle.close);
        }

        // Find surrounding candles
        let before = self.candles.range(..timestamp).next_back();
        let after = self.candles.range(timestamp..).next();

        match (before, after) {
            (Some((t1, c1)), Some((t2, c2))) => {
                // Linear interpolation
                let t1 = *t1 as f64;
                let t2 = *t2 as f64;
                let t = timestamp as f64;

                let ratio = (t - t1) / (t2 - t1);
                let p1 = c1.close;
                let p2 = c2.close;

                let diff = p2 - p1;
                let interpolated = p1 + diff * Decimal::try_from(ratio).unwrap_or(Decimal::ZERO);
                Some(interpolated)
            }
            (Some((_, c)), None) => Some(c.close), // Use last known price
            (None, Some((_, c))) => Some(c.close), // Use first known price
            (None, None) => None,
        }
    }

    /// Calculates the simple moving average of closing prices.
    #[must_use]
    pub fn sma(&self, period: usize) -> Vec<Decimal> {
        let closes: Vec<Decimal> = self.close_prices();
        if closes.len() < period || period == 0 {
            return Vec::new();
        }

        let mut result = Vec::with_capacity(closes.len() - period + 1);
        let period_dec = Decimal::from(period);

        for i in (period - 1)..closes.len() {
            let sum: Decimal = closes[(i + 1 - period)..=i].iter().copied().sum();
            result.push(sum / period_dec);
        }

        result
    }

    /// Calculates the volatility (standard deviation of returns).
    #[must_use]
    pub fn volatility(&self) -> Option<Decimal> {
        let closes = self.close_prices();
        if closes.len() < 2 {
            return None;
        }

        // Calculate returns
        let returns: Vec<Decimal> = closes
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

        // Return standard deviation (approximate sqrt using f64)
        let var_f64 = variance.to_string().parse::<f64>().unwrap_or(0.0);
        Decimal::try_from(var_f64.sqrt()).ok()
    }

    /// Calculates the total volume over the series.
    #[must_use]
    pub fn total_volume(&self) -> Decimal {
        self.candles.values().map(|c| c.volume).sum()
    }

    /// Calculates the average volume per candle.
    #[must_use]
    pub fn average_volume(&self) -> Decimal {
        if self.candles.is_empty() {
            return Decimal::ZERO;
        }
        self.total_volume() / Decimal::from(self.candles.len())
    }

    /// Returns the highest price in the series.
    #[must_use]
    pub fn highest_price(&self) -> Option<Decimal> {
        self.candles.values().map(|c| c.high).max()
    }

    /// Returns the lowest price in the series.
    #[must_use]
    pub fn lowest_price(&self) -> Option<Decimal> {
        self.candles.values().map(|c| c.low).min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_candles() -> Vec<OhlcvCandle> {
        vec![
            OhlcvCandle::new(1000, dec!(100), dec!(105), dec!(98), dec!(102), dec!(1000)),
            OhlcvCandle::new(2000, dec!(102), dec!(108), dec!(101), dec!(106), dec!(1200)),
            OhlcvCandle::new(3000, dec!(106), dec!(110), dec!(104), dec!(108), dec!(1100)),
            OhlcvCandle::new(4000, dec!(108), dec!(112), dec!(107), dec!(110), dec!(1300)),
            OhlcvCandle::new(5000, dec!(110), dec!(115), dec!(109), dec!(114), dec!(1500)),
        ]
    }

    #[test]
    fn test_candle_typical_price() {
        let candle = OhlcvCandle::new(1000, dec!(100), dec!(110), dec!(95), dec!(105), dec!(1000));
        // (110 + 95 + 105) / 3 = 310 / 3 ≈ 103.33
        let typical = candle.typical_price();
        assert!(typical > dec!(103) && typical < dec!(104));
    }

    #[test]
    fn test_candle_bullish_bearish() {
        let bullish = OhlcvCandle::new(1000, dec!(100), dec!(110), dec!(95), dec!(105), dec!(1000));
        let bearish = OhlcvCandle::new(1000, dec!(100), dec!(110), dec!(95), dec!(95), dec!(1000));

        assert!(bullish.is_bullish());
        assert!(!bullish.is_bearish());
        assert!(bearish.is_bearish());
        assert!(!bearish.is_bullish());
    }

    #[test]
    fn test_timeseries_range_query() {
        let candles = create_test_candles();
        let ts = TimeSeries::from_candles(candles, 1000);

        let range = ts.range(2000, 4000);
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].timestamp, 2000);
        assert_eq!(range[2].timestamp, 4000);
    }

    #[test]
    fn test_timeseries_interpolation() {
        let candles = create_test_candles();
        let ts = TimeSeries::from_candles(candles, 1000);

        // Exact match
        let exact = ts.interpolate_price(2000);
        assert_eq!(exact, Some(dec!(106)));

        // Interpolation between 2000 (106) and 3000 (108)
        let interp = ts.interpolate_price(2500);
        assert!(interp.is_some());
        let price = interp.unwrap();
        assert!(price > dec!(106) && price < dec!(108));
    }

    #[test]
    fn test_timeseries_sma() {
        let candles = create_test_candles();
        let ts = TimeSeries::from_candles(candles, 1000);

        let sma = ts.sma(3);
        // SMA(3) should have 3 values for 5 candles
        assert_eq!(sma.len(), 3);

        // First SMA: (102 + 106 + 108) / 3 ≈ 105.33
        assert!(sma[0] > dec!(105) && sma[0] < dec!(106));
    }

    #[test]
    fn test_timeseries_volatility() {
        let candles = create_test_candles();
        let ts = TimeSeries::from_candles(candles, 1000);

        let vol = ts.volatility();
        assert!(vol.is_some());
        // Should be a small positive number
        assert!(vol.unwrap() > Decimal::ZERO);
    }

    #[test]
    fn test_timeseries_volume_stats() {
        let candles = create_test_candles();
        let ts = TimeSeries::from_candles(candles, 1000);

        assert_eq!(ts.total_volume(), dec!(6100));
        assert_eq!(ts.average_volume(), dec!(1220));
    }

    #[test]
    fn test_timeseries_price_extremes() {
        let candles = create_test_candles();
        let ts = TimeSeries::from_candles(candles, 1000);

        assert_eq!(ts.highest_price(), Some(dec!(115)));
        assert_eq!(ts.lowest_price(), Some(dec!(98)));
    }
}
