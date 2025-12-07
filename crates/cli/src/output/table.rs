//! Table formatting for CLI output.
//!
//! Uses prettytable-rs for rich table formatting.

use super::{AnalysisReport, BacktestReport, OptimizationReport};
use prettytable::{Table, row};
use rust_decimal::Decimal;

/// Prints an analysis report as a formatted table.
pub fn print_analysis_report(report: &AnalysisReport) {
    println!("\n📊 Pool Analysis: {}", report.pair);
    println!("═══════════════════════════════════════════════════════════════");

    // Price summary table
    let mut price_table = Table::new();
    price_table.add_row(row!["Metric", "Value"]);
    price_table.add_row(row![
        "Current Price",
        format!("${:.4}", report.current_price)
    ]);
    price_table.add_row(row!["High Price", format!("${:.4}", report.high_price)]);
    price_table.add_row(row!["Low Price", format!("${:.4}", report.low_price)]);
    price_table.add_row(row!["Average Price", format!("${:.4}", report.avg_price)]);

    println!(
        "\n📈 Price Summary ({} days, {} data points)",
        report.period_days, report.data_points
    );
    price_table.printstd();

    // Volatility table
    let mut vol_table = Table::new();
    vol_table.add_row(row!["Metric", "Value"]);
    vol_table.add_row(row![
        "Daily Volatility",
        format!("{:.2}%", report.volatility_daily * Decimal::from(100))
    ]);
    vol_table.add_row(row![
        "Annual Volatility",
        format!("{:.2}%", report.volatility_annual * Decimal::from(100))
    ]);

    println!("\n📉 Volatility");
    vol_table.printstd();

    // Recommendations table
    let mut rec_table = Table::new();
    rec_table.add_row(row!["Metric", "Value"]);
    rec_table.add_row(row![
        "Recommended Lower",
        format!("${:.4}", report.recommended_lower)
    ]);
    rec_table.add_row(row![
        "Recommended Upper",
        format!("${:.4}", report.recommended_upper)
    ]);
    rec_table.add_row(row![
        "Range Width",
        format!("{:.1}%", report.recommended_width * Decimal::from(100))
    ]);
    rec_table.add_row(row![
        "Est. Time in Range",
        format!("{:.1}%", report.estimated_time_in_range)
    ]);

    println!("\n💡 Recommendations");
    rec_table.printstd();
}

/// Prints a backtest report as a formatted table.
pub fn print_backtest_report(report: &BacktestReport) {
    println!("\n📊 Backtest Results: {}", report.pair);
    println!("═══════════════════════════════════════════════════════════════");

    // Configuration table
    let mut config_table = Table::new();
    config_table.add_row(row!["Parameter", "Value"]);
    config_table.add_row(row!["Period", format!("{} days", report.period_days)]);
    config_table.add_row(row!["Entry Price", format!("${:.4}", report.entry_price)]);
    config_table.add_row(row!["Exit Price", format!("${:.4}", report.exit_price)]);
    config_table.add_row(row![
        "Range",
        format!("${:.2} - ${:.2}", report.range_lower, report.range_upper)
    ]);
    config_table.add_row(row![
        "Initial Capital",
        format!("${:.2}", report.initial_capital)
    ]);
    config_table.add_row(row!["Strategy", &report.strategy]);

    println!("\n⚙️  Configuration");
    config_table.printstd();

    // Performance table
    let mut perf_table = Table::new();
    perf_table.add_row(row!["Metric", "Value"]);
    perf_table.add_row(row!["Final Value", format!("${:.2}", report.final_value)]);
    perf_table.add_row(row![
        "Total Return",
        format_pct_colored(report.total_return)
    ]);
    perf_table.add_row(row![
        "Fee Earnings",
        format!("+${:.2}", report.fee_earnings)
    ]);
    perf_table.add_row(row![
        "Impermanent Loss",
        format!("-${:.2}", report.impermanent_loss.abs())
    ]);
    perf_table.add_row(row!["vs HODL", format_pct_colored(report.vs_hodl)]);

    if let Some(sharpe) = report.sharpe_ratio {
        perf_table.add_row(row!["Sharpe Ratio", format!("{:.2}", sharpe)]);
    }

    println!("\n💰 Performance");
    perf_table.printstd();

    // Risk table
    let mut risk_table = Table::new();
    risk_table.add_row(row!["Metric", "Value"]);
    risk_table.add_row(row![
        "Time in Range",
        format!("{:.1}%", report.time_in_range)
    ]);
    risk_table.add_row(row![
        "Max Drawdown",
        format!("-{:.2}%", report.max_drawdown.abs())
    ]);
    risk_table.add_row(row!["Rebalances", report.rebalance_count.to_string()]);
    risk_table.add_row(row![
        "Transaction Costs",
        format!("${:.2}", report.total_tx_costs)
    ]);

    println!("\n⚠️  Risk Metrics");
    risk_table.printstd();

    // Summary
    println!("\n───────────────────────────────────────────────────────────────");
    let emoji = if report.total_return > Decimal::ZERO {
        "✅"
    } else {
        "❌"
    };
    println!(
        "{} Net Result: {} ({} vs HODL)",
        emoji,
        format_pct_colored(report.total_return),
        format_pct_colored(report.vs_hodl)
    );
}

/// Prints an optimization report as a formatted table.
pub fn print_optimization_report(report: &OptimizationReport) {
    println!("\n🎯 Optimization Results: {}", report.pair);
    println!("═══════════════════════════════════════════════════════════════");

    // Context table
    let mut ctx_table = Table::new();
    ctx_table.add_row(row!["Parameter", "Value"]);
    ctx_table.add_row(row![
        "Current Price",
        format!("${:.4}", report.current_price)
    ]);
    ctx_table.add_row(row![
        "Volatility",
        format!("{:.1}%", report.volatility * Decimal::from(100))
    ]);
    ctx_table.add_row(row!["Capital", format!("${:.2}", report.capital)]);
    ctx_table.add_row(row!["Objective", &report.objective]);

    println!("\n⚙️  Optimization Context");
    ctx_table.printstd();

    // Candidates table
    let mut cand_table = Table::new();
    cand_table.add_row(row![
        "Rank", "Width", "Range", "Fees", "IL", "PnL", "Time%", "Score"
    ]);

    for c in &report.candidates {
        cand_table.add_row(row![
            format!("#{}", c.rank),
            format!("{:.1}%", c.range_width_pct),
            format!("${:.2}-${:.2}", c.lower_price, c.upper_price),
            format!("{:.2}", c.expected_fees),
            format!("{:.2}", c.expected_il),
            format!("{:.2}", c.expected_pnl),
            format!("{:.0}%", c.time_in_range),
            format!("{:.2}", c.score)
        ]);
    }

    println!("\n📊 Top Range Candidates");
    cand_table.printstd();

    // Strategy recommendations
    if !report.strategy_recommendations.is_empty() {
        let mut strat_table = Table::new();
        strat_table.add_row(row!["Strategy", "Parameters", "Rebalances", "Score"]);

        for s in &report.strategy_recommendations {
            strat_table.add_row(row![
                &s.strategy_type,
                &s.params,
                s.expected_rebalances.to_string(),
                format!("{:.2}", s.score)
            ]);
        }

        println!("\n🔄 Strategy Recommendations");
        strat_table.printstd();
    }

    // Best recommendation
    if let Some(best) = report.candidates.first() {
        println!("\n───────────────────────────────────────────────────────────────");
        println!(
            "💡 Recommended: {:.1}% range (${:.2} - ${:.2})",
            best.range_width_pct, best.lower_price, best.upper_price
        );
        println!(
            "   Expected: +{:.2} fees, -{:.2} IL = {:.2} net PnL",
            best.expected_fees, best.expected_il, best.expected_pnl
        );
    }
}

/// Formats a percentage with color indicator.
fn format_pct_colored(value: Decimal) -> String {
    if value > Decimal::ZERO {
        format!("+{:.2}%", value)
    } else {
        format!("{:.2}%", value)
    }
}

/// Prints a simple key-value table.
pub fn print_kv_table(title: &str, items: &[(&str, String)]) {
    println!("\n{}", title);
    let mut table = Table::new();
    table.add_row(row!["Key", "Value"]);
    for (key, value) in items {
        table.add_row(row![key, value]);
    }
    table.printstd();
}
