//! Data command implementation.
//!
//! Provides data management functionality including fetching,
//! caching, and exporting market data.

use anyhow::Result;
use clmm_lp_data::prelude::*;
use clmm_lp_domain::entities::token::Token;
use rust_decimal::Decimal;
use std::path::PathBuf;
use tracing::info;

/// Arguments for the data command.
#[derive(Debug, Clone)]
pub struct DataArgs {
    /// Subcommand to execute.
    pub action: DataAction,
}

/// Data subcommand actions.
#[derive(Debug, Clone)]
pub enum DataAction {
    /// Fetch market data from API.
    Fetch(FetchArgs),
    /// Export data to file.
    Export(ExportArgs),
    /// Show cache status.
    CacheStatus,
    /// Clear cache.
    ClearCache,
}

/// Arguments for fetch action.
#[derive(Debug, Clone)]
pub struct FetchArgs {
    /// Token A symbol.
    pub symbol_a: String,
    /// Token A mint address.
    pub mint_a: String,
    /// Token B symbol.
    pub symbol_b: String,
    /// Token B mint address.
    pub mint_b: String,
    /// Hours of history to fetch.
    pub hours: u64,
    /// Resolution in minutes.
    pub resolution_minutes: u64,
}

/// Arguments for export action.
#[derive(Debug, Clone)]
pub struct ExportArgs {
    /// Token A symbol.
    pub symbol_a: String,
    /// Token A mint address.
    pub mint_a: String,
    /// Token B symbol.
    pub symbol_b: String,
    /// Token B mint address.
    pub mint_b: String,
    /// Hours of history.
    pub hours: u64,
    /// Output file path.
    pub output: PathBuf,
    /// Output format.
    pub format: ExportFormat,
}

/// Export format.
#[derive(Debug, Clone, Copy, Default)]
pub enum ExportFormat {
    /// CSV format.
    #[default]
    Csv,
    /// JSON format.
    Json,
}

impl Default for FetchArgs {
    fn default() -> Self {
        Self {
            symbol_a: "SOL".to_string(),
            mint_a: "So11111111111111111111111111111111111111112".to_string(),
            symbol_b: "USDC".to_string(),
            mint_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            hours: 24,
            resolution_minutes: 60,
        }
    }
}

/// Runs the data command.
pub async fn run_data(args: DataArgs) -> Result<()> {
    match args.action {
        DataAction::Fetch(fetch_args) => run_fetch(fetch_args).await,
        DataAction::Export(export_args) => run_export(export_args).await,
        DataAction::CacheStatus => run_cache_status().await,
        DataAction::ClearCache => run_clear_cache().await,
    }
}

/// Fetches market data and displays summary.
async fn run_fetch(args: FetchArgs) -> Result<()> {
    info!(
        "Fetching {}/{} data for {} hours",
        args.symbol_a, args.symbol_b, args.hours
    );

    let api_key =
        std::env::var("BIRDEYE_API_KEY").map_err(|_| anyhow::anyhow!("BIRDEYE_API_KEY not set"))?;

    let provider = BirdeyeProvider::new(api_key);

    let token_a = Token::new(&args.mint_a, &args.symbol_a, 9, &args.symbol_a);
    let token_b = Token::new(&args.mint_b, &args.symbol_b, 6, &args.symbol_b);

    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let start_time = end_time - (args.hours * 3600);
    let resolution = args.resolution_minutes * 60;

    let candles = provider
        .get_price_history(&token_a, &token_b, start_time, end_time, resolution)
        .await?;

    println!("\n📊 Data Fetch Summary");
    println!("═══════════════════════════════════════");
    println!("Pair:        {}/{}", args.symbol_a, args.symbol_b);
    println!("Period:      {} hours", args.hours);
    println!("Resolution:  {} minutes", args.resolution_minutes);
    println!("Candles:     {}", candles.len());

    if !candles.is_empty() {
        let first = &candles[0];
        let last = candles.last().unwrap();

        println!("\n📈 Price Summary");
        println!("───────────────────────────────────────");
        println!("First Price: {}", first.close.value);
        println!("Last Price:  {}", last.close.value);

        let high = candles.iter().map(|c| c.high.value).max().unwrap();
        let low = candles.iter().map(|c| c.low.value).min().unwrap();

        println!("High:        {}", high);
        println!("Low:         {}", low);

        let change =
            (last.close.value - first.close.value) / first.close.value * Decimal::from(100);
        println!("Change:      {:.2}%", change);
    }

    Ok(())
}

/// Exports market data to a file.
async fn run_export(args: ExportArgs) -> Result<()> {
    info!(
        "Exporting {}/{} data to {:?}",
        args.symbol_a, args.symbol_b, args.output
    );

    let api_key =
        std::env::var("BIRDEYE_API_KEY").map_err(|_| anyhow::anyhow!("BIRDEYE_API_KEY not set"))?;

    let provider = BirdeyeProvider::new(api_key);

    let token_a = Token::new(&args.mint_a, &args.symbol_a, 9, &args.symbol_a);
    let token_b = Token::new(&args.mint_b, &args.symbol_b, 6, &args.symbol_b);

    let end_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let start_time = end_time - (args.hours * 3600);

    let candles = provider
        .get_price_history(&token_a, &token_b, start_time, end_time, 3600)
        .await?;

    match args.format {
        ExportFormat::Csv => {
            write_candles_to_csv(&candles, &args.output)?;
            println!("✅ Exported {} candles to {:?}", candles.len(), args.output);
        }
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(&candles)?;
            std::fs::write(&args.output, json)?;
            println!("✅ Exported {} candles to {:?}", candles.len(), args.output);
        }
    }

    Ok(())
}

/// Shows cache status.
async fn run_cache_status() -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clmm-lp");

    println!("\n💾 Cache Status");
    println!("═══════════════════════════════════════");
    println!("Cache Directory: {:?}", cache_dir);

    if cache_dir.exists() {
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        if let Ok(entries) = std::fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;
                }
            }
        }

        println!("Files:           {}", file_count);
        println!("Total Size:      {} KB", total_size / 1024);
    } else {
        println!("Status:          No cache directory found");
    }

    Ok(())
}

/// Clears the cache.
async fn run_clear_cache() -> Result<()> {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("clmm-lp");

    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
        println!("✅ Cache cleared");
    } else {
        println!("ℹ️  No cache to clear");
    }

    Ok(())
}
