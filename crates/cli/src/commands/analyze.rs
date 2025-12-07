//! Analyze command implementation.
//!
//! Provides pool analysis functionality including volatility,
//! volume statistics, and optimal range recommendations.

use crate::output::{AnalysisReport, print_analysis_report};
use anyhow::Result;
use clmm_lp_data::prelude::*;
use clmm_lp_domain::entities::token::Token;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tracing::info;

/// Arguments for the analyze command.
#[derive(Debug, Clone)]
pub struct AnalyzeArgs {
    /// Token A symbol.
    pub symbol_a: String,
    /// Token A mint address.
    pub mint_a: String,
    /// Token B symbol.
    pub symbol_b: String,
    /// Token B mint address.
    pub mint_b: String,
    /// Number of days to analyze.
    pub days: u64,
    /// Output format.
    pub format: OutputFormat,
}

/// Output format for analysis.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable table format.
    #[default]
    Table,
    /// JSON format.
    Json,
    /// CSV format.
    Csv,
}

impl Default for AnalyzeArgs {
    fn default() -> Self {
        Self {
            symbol_a: "SOL".to_string(),
            mint_a: "So11111111111111111111111111111111111111112".to_string(),
            symbol_b: "USDC".to_string(),
            mint_b: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            days: 30,
            format: OutputFormat::Table,
        }
    }
}

/// Runs the analyze command.
pub async fn run_analyze(args: AnalyzeArgs) -> Result<()> {
    info!(
        "Analyzing {}/{} pool for {} days",
        args.symbol_a, args.symbol_b, args.days
    );

    // Create tokens
    let token_a = Token::new(&args.mint_a, &args.symbol_a, 9, &args.symbol_a);
    let token_b = Token::new(&args.mint_b, &args.symbol_b, 6, &args.symbol_b);

    // Try to fetch data from provider
    let api_key = std::env::var("BIRDEYE_API_KEY").ok();

    let report = if let Some(key) = api_key {
        let provider = BirdeyeProvider::new(key);

        let end_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let start_time = end_time - (args.days * 24 * 3600);

        match provider
            .get_price_history(&token_a, &token_b, start_time, end_time, 3600)
            .await
        {
            Ok(candles) => {
                info!("Fetched {} candles", candles.len());
                analyze_candles(&candles, &args)
            }
            Err(e) => {
                info!("Failed to fetch data: {}. Using mock data.", e);
                generate_mock_report(&args)
            }
        }
    } else {
        info!("No API key found. Using mock data for demonstration.");
        generate_mock_report(&args)
    };

    // Output the report
    match args.format {
        OutputFormat::Table => print_analysis_report(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Csv => print_csv_report(&report),
    }

    Ok(())
}

/// Analyzes price candles and generates a report.
fn analyze_candles(
    candles: &[clmm_lp_domain::entities::price_candle::PriceCandle],
    args: &AnalyzeArgs,
) -> AnalysisReport {
    if candles.is_empty() {
        return generate_mock_report(args);
    }

    // Calculate statistics
    let prices: Vec<Decimal> = candles.iter().map(|c| c.close.value).collect();

    let current_price = prices.last().copied().unwrap_or(Decimal::from(100));
    let high_price = prices.iter().max().copied().unwrap_or(current_price);
    let low_price = prices.iter().min().copied().unwrap_or(current_price);

    let avg_price = prices.iter().copied().sum::<Decimal>() / Decimal::from(prices.len().max(1));

    // Calculate volatility (standard deviation of returns)
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

    let volatility = if returns.is_empty() {
        Decimal::from_f64(0.05).unwrap()
    } else {
        let mean: Decimal = returns.iter().copied().sum::<Decimal>() / Decimal::from(returns.len());
        let variance: Decimal = returns
            .iter()
            .map(|r| {
                let diff = *r - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(returns.len());

        let var_f64 = variance.to_string().parse::<f64>().unwrap_or(0.0);
        Decimal::from_f64(var_f64.sqrt()).unwrap_or(Decimal::from_f64(0.05).unwrap())
    };

    // Calculate recommended range based on volatility
    let range_width = (volatility * Decimal::from(2)).max(Decimal::from_f64(0.05).unwrap());
    let recommended_lower = current_price * (Decimal::ONE - range_width);
    let recommended_upper = current_price * (Decimal::ONE + range_width);

    // Estimate time in range
    let in_range_count = prices
        .iter()
        .filter(|p| **p >= recommended_lower && **p <= recommended_upper)
        .count();
    let time_in_range = Decimal::from(in_range_count * 100) / Decimal::from(prices.len().max(1));

    AnalysisReport {
        pair: format!("{}/{}", args.symbol_a, args.symbol_b),
        period_days: args.days,
        current_price,
        high_price,
        low_price,
        avg_price,
        volatility_daily: volatility,
        volatility_annual: volatility * Decimal::from_f64(365.0_f64.sqrt()).unwrap(),
        recommended_lower,
        recommended_upper,
        recommended_width: range_width,
        estimated_time_in_range: time_in_range,
        data_points: prices.len(),
    }
}

/// Generates a mock report for demonstration.
fn generate_mock_report(args: &AnalyzeArgs) -> AnalysisReport {
    let current_price = Decimal::from(100);
    let volatility = Decimal::from_f64(0.03).unwrap();

    AnalysisReport {
        pair: format!("{}/{}", args.symbol_a, args.symbol_b),
        period_days: args.days,
        current_price,
        high_price: Decimal::from(115),
        low_price: Decimal::from(88),
        avg_price: Decimal::from(102),
        volatility_daily: volatility,
        volatility_annual: volatility * Decimal::from_f64(365.0_f64.sqrt()).unwrap(),
        recommended_lower: Decimal::from(94),
        recommended_upper: Decimal::from(106),
        recommended_width: Decimal::from_f64(0.06).unwrap(),
        estimated_time_in_range: Decimal::from(72),
        data_points: args.days as usize * 24,
    }
}

/// Prints the report in CSV format.
fn print_csv_report(report: &AnalysisReport) {
    println!("metric,value");
    println!("pair,{}", report.pair);
    println!("period_days,{}", report.period_days);
    println!("current_price,{}", report.current_price);
    println!("high_price,{}", report.high_price);
    println!("low_price,{}", report.low_price);
    println!("avg_price,{}", report.avg_price);
    println!("volatility_daily,{}", report.volatility_daily);
    println!("volatility_annual,{}", report.volatility_annual);
    println!("recommended_lower,{}", report.recommended_lower);
    println!("recommended_upper,{}", report.recommended_upper);
    println!("recommended_width,{}", report.recommended_width);
    println!("estimated_time_in_range,{}", report.estimated_time_in_range);
    println!("data_points,{}", report.data_points);
}
