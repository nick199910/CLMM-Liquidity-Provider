//! Optimize command implementation.
//!
//! Provides optimization functionality for finding optimal
//! LP position parameters.

use crate::output::{OptimizationReport, RangeCandidate, print_optimization_report};
use anyhow::Result;
use clmm_lp_optimization::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use tracing::info;

/// Arguments for the optimize command.
#[derive(Debug, Clone)]
pub struct OptimizeArgs {
    /// Token A symbol.
    pub symbol_a: String,
    /// Token B symbol.
    pub symbol_b: String,
    /// Current price.
    pub current_price: Decimal,
    /// Volatility estimate (annual).
    pub volatility: f64,
    /// Initial capital.
    pub capital: Decimal,
    /// Optimization objective.
    pub objective: ObjectiveType,
    /// Number of top candidates to show.
    pub top_n: usize,
    /// Output format.
    pub format: OutputFormat,
}

/// Optimization objective type.
#[derive(Debug, Clone, Copy, Default)]
pub enum ObjectiveType {
    /// Maximize net PnL.
    #[default]
    Pnl,
    /// Maximize fees.
    Fees,
    /// Maximize Sharpe ratio.
    Sharpe,
    /// Minimize IL.
    MinIL,
    /// Maximize time in range.
    TimeInRange,
}

/// Output format.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable table.
    #[default]
    Table,
    /// JSON format.
    Json,
    /// CSV format.
    Csv,
}

impl Default for OptimizeArgs {
    fn default() -> Self {
        Self {
            symbol_a: "SOL".to_string(),
            symbol_b: "USDC".to_string(),
            current_price: Decimal::from(100),
            volatility: 0.5,
            capital: Decimal::from(1000),
            objective: ObjectiveType::Pnl,
            top_n: 5,
            format: OutputFormat::Table,
        }
    }
}

/// Runs the optimize command.
pub async fn run_optimize(args: OptimizeArgs) -> Result<()> {
    info!(
        "Optimizing {}/{} position at price {}",
        args.symbol_a, args.symbol_b, args.current_price
    );
    info!(
        "Objective: {:?}, Volatility: {:.1}%",
        args.objective,
        args.volatility * 100.0
    );

    // Create optimization config
    let config = OptimizationConfig::new()
        .with_iterations(100)
        .with_steps(30)
        .with_volatility(args.volatility)
        .with_price(args.current_price);

    // Create optimizer
    let optimizer = AnalyticalOptimizer::new();

    // Run optimization based on objective
    let candidates = match args.objective {
        ObjectiveType::Pnl => optimizer.optimize(&config, &MaximizeNetPnL),
        ObjectiveType::Fees => optimizer.optimize(&config, &MaximizeFees),
        ObjectiveType::Sharpe => optimizer.optimize(&config, &MaximizeSharpeRatio::default()),
        ObjectiveType::MinIL => optimizer.optimize(&config, &MinimizeIL::default()),
        ObjectiveType::TimeInRange => optimizer.optimize(&config, &MaximizeTimeInRange),
    };

    // Convert to report format
    let range_candidates: Vec<RangeCandidate> = candidates
        .iter()
        .take(args.top_n)
        .enumerate()
        .map(|(i, c)| {
            let lower = args.current_price * (Decimal::ONE - c.range_width);
            let upper = args.current_price * (Decimal::ONE + c.range_width);

            RangeCandidate {
                rank: i + 1,
                range_width_pct: c.range_width * Decimal::from(100),
                lower_price: lower,
                upper_price: upper,
                expected_fees: c.expected_fees,
                expected_il: c.expected_il,
                expected_pnl: c.net_pnl,
                time_in_range: c.time_in_range,
                score: c.score,
            }
        })
        .collect();

    // Also run parameter optimization for the best range
    let best_width = candidates
        .first()
        .map(|c| c.range_width)
        .unwrap_or(Decimal::from_f64(0.10).unwrap());

    let param_optimizer = ParameterOptimizer::new();

    let threshold_candidates =
        param_optimizer.optimize_threshold(&config, best_width, &MaximizeNetPnL);
    let periodic_candidates =
        param_optimizer.optimize_periodic(&config, best_width, &MaximizeNetPnL);

    let best_threshold = threshold_candidates
        .first()
        .map(|c| StrategyRecommendation {
            strategy_type: "Threshold".to_string(),
            params: format!(
                "price_threshold={:.1}%, il_threshold={:.1}%",
                c.params.price_threshold * Decimal::from(100),
                c.params.il_threshold * Decimal::from(100)
            ),
            expected_rebalances: c.expected_rebalances,
            score: c.score,
        });

    let best_periodic = periodic_candidates.first().map(|c| StrategyRecommendation {
        strategy_type: "Periodic".to_string(),
        params: format!("interval={}h", c.params.interval),
        expected_rebalances: c.expected_rebalances,
        score: c.score,
    });

    let report = OptimizationReport {
        pair: format!("{}/{}", args.symbol_a, args.symbol_b),
        current_price: args.current_price,
        volatility: Decimal::from_f64(args.volatility).unwrap(),
        capital: args.capital,
        objective: format!("{:?}", args.objective),
        candidates: range_candidates,
        strategy_recommendations: vec![best_threshold, best_periodic]
            .into_iter()
            .flatten()
            .collect(),
    };

    // Output the report
    match args.format {
        OutputFormat::Table => print_optimization_report(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
        OutputFormat::Csv => print_csv_optimization(&report),
    }

    Ok(())
}

/// Strategy recommendation from parameter optimization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StrategyRecommendation {
    /// Strategy type name.
    pub strategy_type: String,
    /// Strategy parameters.
    pub params: String,
    /// Expected number of rebalances.
    pub expected_rebalances: u32,
    /// Optimization score.
    pub score: Decimal,
}

/// Prints optimization report in CSV format.
fn print_csv_optimization(report: &OptimizationReport) {
    println!(
        "rank,width_pct,lower,upper,expected_fees,expected_il,expected_pnl,time_in_range,score"
    );
    for c in &report.candidates {
        println!(
            "{},{},{},{},{},{},{},{},{}",
            c.rank,
            c.range_width_pct,
            c.lower_price,
            c.upper_price,
            c.expected_fees,
            c.expected_il,
            c.expected_pnl,
            c.time_in_range,
            c.score
        );
    }
}
