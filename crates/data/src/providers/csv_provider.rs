//! CSV file provider for loading historical price data from files.
//!
//! This provider allows loading price data from CSV files for offline testing
//! and backtesting without requiring API access.

use crate::MarketDataProvider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use clmm_lp_domain::entities::price_candle::PriceCandle;
use clmm_lp_domain::entities::token::Token;
use clmm_lp_domain::value_objects::{amount::Amount, price::Price};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

/// CSV file provider for market data.
///
/// Expects CSV files with the following format:
/// ```csv
/// timestamp,open,high,low,close,volume
/// 1704067200,100.5,101.2,99.8,100.9,1000000
/// ```
///
/// The timestamp should be Unix timestamp in seconds.
/// Prices should be decimal values.
/// Volume is optional and defaults to 0 if not present.
pub struct CsvProvider {
    /// Base directory for CSV files.
    base_path: PathBuf,
}

impl CsvProvider {
    /// Creates a new CSV provider with the given base directory.
    ///
    /// # Arguments
    /// * `base_path` - Directory containing CSV files
    #[must_use]
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Creates a provider from a string path.
    #[must_use]
    pub fn from_path(path: &str) -> Self {
        Self::new(PathBuf::from(path))
    }

    /// Generates the expected filename for a token pair.
    fn get_filename(&self, token_a: &Token, token_b: &Token) -> PathBuf {
        let filename = format!(
            "{}_{}.csv",
            token_a.symbol.to_uppercase(),
            token_b.symbol.to_uppercase()
        );
        self.base_path.join(filename)
    }

    /// Parses a CSV line into components.
    fn parse_line(line: &str) -> Option<(u64, f64, f64, f64, f64, f64)> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 5 {
            return None;
        }

        let timestamp: u64 = parts[0].trim().parse().ok()?;
        let open: f64 = parts[1].trim().parse().ok()?;
        let high: f64 = parts[2].trim().parse().ok()?;
        let low: f64 = parts[3].trim().parse().ok()?;
        let close: f64 = parts[4].trim().parse().ok()?;
        let volume: f64 = parts
            .get(5)
            .and_then(|v| v.trim().parse().ok())
            .unwrap_or(0.0);

        Some((timestamp, open, high, low, close, volume))
    }
}

#[async_trait]
impl MarketDataProvider for CsvProvider {
    async fn get_price_history(
        &self,
        token_a: &Token,
        token_b: &Token,
        start_time: u64,
        end_time: u64,
        resolution: u64,
    ) -> Result<Vec<PriceCandle>> {
        let filepath = self.get_filename(token_a, token_b);

        let file = File::open(&filepath)
            .with_context(|| format!("Failed to open CSV file: {}", filepath.display()))?;

        let reader = BufReader::new(file);
        let mut candles = Vec::new();

        for (line_num, line_result) in reader.lines().enumerate() {
            let line = line_result.with_context(|| format!("Failed to read line {}", line_num))?;

            // Skip header line
            if line_num == 0 && line.to_lowercase().contains("timestamp") {
                continue;
            }

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            let Some((timestamp, open, high, low, close, volume)) = Self::parse_line(&line) else {
                tracing::warn!("Failed to parse line {}: {}", line_num, line);
                continue;
            };

            // Filter by time range
            if timestamp < start_time || timestamp > end_time {
                continue;
            }

            let open_dec = Decimal::from_f64(open).unwrap_or(Decimal::ZERO);
            let high_dec = Decimal::from_f64(high).unwrap_or(Decimal::ZERO);
            let low_dec = Decimal::from_f64(low).unwrap_or(Decimal::ZERO);
            let close_dec = Decimal::from_f64(close).unwrap_or(Decimal::ZERO);

            // Calculate volume in token terms
            let vol_dec = Decimal::from_f64(volume).unwrap_or(Decimal::ZERO);
            let vol_token = if close_dec.is_zero() {
                Decimal::ZERO
            } else {
                vol_dec / close_dec
            };

            let vol_amount = Amount::from_decimal(vol_token, token_a.decimals);

            candles.push(PriceCandle {
                token_a: token_a.clone(),
                token_b: token_b.clone(),
                start_timestamp: timestamp,
                duration_seconds: resolution,
                open: Price::new(open_dec),
                high: Price::new(high_dec),
                low: Price::new(low_dec),
                close: Price::new(close_dec),
                volume_token_a: vol_amount,
            });
        }

        // Sort by timestamp
        candles.sort_by_key(|c| c.start_timestamp);

        Ok(candles)
    }
}

/// Writes price candles to a CSV file.
///
/// # Arguments
/// * `candles` - The candles to write
/// * `filepath` - Path to the output file
///
/// # Errors
/// Returns an error if file operations fail.
pub fn write_candles_to_csv(candles: &[PriceCandle], filepath: &std::path::Path) -> Result<()> {
    use std::io::Write;

    let mut file = File::create(filepath)
        .with_context(|| format!("Failed to create {}", filepath.display()))?;

    // Write header
    writeln!(file, "timestamp,open,high,low,close,volume")?;

    // Write data
    for candle in candles {
        writeln!(
            file,
            "{},{},{},{},{},{}",
            candle.start_timestamp,
            candle.open.value,
            candle.high.value,
            candle.low.value,
            candle.close.value,
            candle.volume_token_a.to_decimal()
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_test_token(symbol: &str) -> Token {
        Token::new("mint_address", symbol, 9, symbol)
    }

    #[tokio::test]
    async fn test_csv_provider_reads_file() {
        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("SOL_USDC.csv");

        // Create test CSV
        let mut file = File::create(&csv_path).unwrap();
        writeln!(file, "timestamp,open,high,low,close,volume").unwrap();
        writeln!(file, "1704067200,100.0,101.0,99.0,100.5,1000000").unwrap();
        writeln!(file, "1704070800,100.5,102.0,100.0,101.5,1500000").unwrap();
        writeln!(file, "1704074400,101.5,103.0,101.0,102.5,2000000").unwrap();

        let provider = CsvProvider::new(dir.path().to_path_buf());
        let token_a = create_test_token("SOL");
        let token_b = create_test_token("USDC");

        let candles = provider
            .get_price_history(&token_a, &token_b, 0, u64::MAX, 3600)
            .await
            .unwrap();

        assert_eq!(candles.len(), 3);
        assert_eq!(candles[0].start_timestamp, 1704067200);
        assert_eq!(candles[0].close.value, Decimal::from_f64(100.5).unwrap());
    }

    #[tokio::test]
    async fn test_csv_provider_filters_by_time() {
        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("SOL_USDC.csv");

        let mut file = File::create(&csv_path).unwrap();
        writeln!(file, "timestamp,open,high,low,close,volume").unwrap();
        writeln!(file, "1704067200,100.0,101.0,99.0,100.5,1000000").unwrap();
        writeln!(file, "1704070800,100.5,102.0,100.0,101.5,1500000").unwrap();
        writeln!(file, "1704074400,101.5,103.0,101.0,102.5,2000000").unwrap();

        let provider = CsvProvider::new(dir.path().to_path_buf());
        let token_a = create_test_token("SOL");
        let token_b = create_test_token("USDC");

        // Only get middle candle
        let candles = provider
            .get_price_history(&token_a, &token_b, 1704070000, 1704072000, 3600)
            .await
            .unwrap();

        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].start_timestamp, 1704070800);
    }

    #[tokio::test]
    async fn test_csv_provider_handles_missing_volume() {
        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("SOL_USDC.csv");

        let mut file = File::create(&csv_path).unwrap();
        writeln!(file, "timestamp,open,high,low,close").unwrap();
        writeln!(file, "1704067200,100.0,101.0,99.0,100.5").unwrap();

        let provider = CsvProvider::new(dir.path().to_path_buf());
        let token_a = create_test_token("SOL");
        let token_b = create_test_token("USDC");

        let candles = provider
            .get_price_history(&token_a, &token_b, 0, u64::MAX, 3600)
            .await
            .unwrap();

        assert_eq!(candles.len(), 1);
    }

    #[test]
    fn test_write_candles_to_csv() {
        let dir = tempdir().unwrap();
        let csv_path = dir.path().join("output.csv");

        let token_a = create_test_token("SOL");
        let token_b = create_test_token("USDC");

        let candles = vec![PriceCandle {
            token_a: token_a.clone(),
            token_b: token_b.clone(),
            start_timestamp: 1704067200,
            duration_seconds: 3600,
            open: Price::new(Decimal::from(100)),
            high: Price::new(Decimal::from(101)),
            low: Price::new(Decimal::from(99)),
            close: Price::new(Decimal::from(100)),
            volume_token_a: Amount::from_decimal(Decimal::from(1000), 9),
        }];

        write_candles_to_csv(&candles, &csv_path).unwrap();

        // Verify file was created and has content
        let content = std::fs::read_to_string(&csv_path).unwrap();
        assert!(content.contains("timestamp,open,high,low,close,volume"));
        assert!(content.contains("1704067200"));
    }
}
