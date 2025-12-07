//! Export functionality for CLI output.
//!
//! Provides export to various formats including JSON, CSV, and HTML.

use super::{AnalysisReport, BacktestReport, OptimizationReport};
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Export format options.
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    /// JSON format.
    Json,
    /// CSV format.
    Csv,
    /// HTML format.
    Html,
    /// Markdown format.
    Markdown,
}

/// Exports an analysis report to a file.
pub fn export_analysis_report(
    report: &AnalysisReport,
    path: &Path,
    format: ExportFormat,
) -> Result<()> {
    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(report)?,
        ExportFormat::Csv => analysis_to_csv(report),
        ExportFormat::Html => analysis_to_html(report),
        ExportFormat::Markdown => analysis_to_markdown(report),
    };

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// Exports a backtest report to a file.
pub fn export_backtest_report(
    report: &BacktestReport,
    path: &Path,
    format: ExportFormat,
) -> Result<()> {
    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(report)?,
        ExportFormat::Csv => backtest_to_csv(report),
        ExportFormat::Html => backtest_to_html(report),
        ExportFormat::Markdown => backtest_to_markdown(report),
    };

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

/// Exports an optimization report to a file.
pub fn export_optimization_report(
    report: &OptimizationReport,
    path: &Path,
    format: ExportFormat,
) -> Result<()> {
    let content = match format {
        ExportFormat::Json => serde_json::to_string_pretty(report)?,
        ExportFormat::Csv => optimization_to_csv(report),
        ExportFormat::Html => optimization_to_html(report),
        ExportFormat::Markdown => optimization_to_markdown(report),
    };

    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

// CSV formatters

fn analysis_to_csv(report: &AnalysisReport) -> String {
    let mut csv = String::from("metric,value\n");
    csv.push_str(&format!("pair,{}\n", report.pair));
    csv.push_str(&format!("period_days,{}\n", report.period_days));
    csv.push_str(&format!("current_price,{}\n", report.current_price));
    csv.push_str(&format!("high_price,{}\n", report.high_price));
    csv.push_str(&format!("low_price,{}\n", report.low_price));
    csv.push_str(&format!("avg_price,{}\n", report.avg_price));
    csv.push_str(&format!("volatility_daily,{}\n", report.volatility_daily));
    csv.push_str(&format!("volatility_annual,{}\n", report.volatility_annual));
    csv.push_str(&format!("recommended_lower,{}\n", report.recommended_lower));
    csv.push_str(&format!("recommended_upper,{}\n", report.recommended_upper));
    csv.push_str(&format!("recommended_width,{}\n", report.recommended_width));
    csv.push_str(&format!(
        "estimated_time_in_range,{}\n",
        report.estimated_time_in_range
    ));
    csv.push_str(&format!("data_points,{}\n", report.data_points));
    csv
}

fn backtest_to_csv(report: &BacktestReport) -> String {
    let mut csv = String::from("metric,value\n");
    csv.push_str(&format!("pair,{}\n", report.pair));
    csv.push_str(&format!("period_days,{}\n", report.period_days));
    csv.push_str(&format!("entry_price,{}\n", report.entry_price));
    csv.push_str(&format!("exit_price,{}\n", report.exit_price));
    csv.push_str(&format!("range_lower,{}\n", report.range_lower));
    csv.push_str(&format!("range_upper,{}\n", report.range_upper));
    csv.push_str(&format!("initial_capital,{}\n", report.initial_capital));
    csv.push_str(&format!("final_value,{}\n", report.final_value));
    csv.push_str(&format!("total_return,{}\n", report.total_return));
    csv.push_str(&format!("fee_earnings,{}\n", report.fee_earnings));
    csv.push_str(&format!("impermanent_loss,{}\n", report.impermanent_loss));
    csv.push_str(&format!("vs_hodl,{}\n", report.vs_hodl));
    csv.push_str(&format!("time_in_range,{}\n", report.time_in_range));
    csv.push_str(&format!("max_drawdown,{}\n", report.max_drawdown));
    csv.push_str(&format!("rebalance_count,{}\n", report.rebalance_count));
    csv.push_str(&format!("total_tx_costs,{}\n", report.total_tx_costs));
    csv.push_str(&format!("strategy,{}\n", report.strategy));
    if let Some(sharpe) = report.sharpe_ratio {
        csv.push_str(&format!("sharpe_ratio,{}\n", sharpe));
    }
    csv
}

fn optimization_to_csv(report: &OptimizationReport) -> String {
    let mut csv = String::from(
        "rank,width_pct,lower,upper,expected_fees,expected_il,expected_pnl,time_in_range,score\n",
    );
    for c in &report.candidates {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            c.rank,
            c.range_width_pct,
            c.lower_price,
            c.upper_price,
            c.expected_fees,
            c.expected_il,
            c.expected_pnl,
            c.time_in_range,
            c.score
        ));
    }
    csv
}

// HTML formatters

fn analysis_to_html(report: &AnalysisReport) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Analysis Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #4CAF50; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        h1 {{ color: #333; }}
    </style>
</head>
<body>
    <h1>Pool Analysis: {}</h1>
    <h2>Price Summary</h2>
    <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Current Price</td><td>${}</td></tr>
        <tr><td>High Price</td><td>${}</td></tr>
        <tr><td>Low Price</td><td>${}</td></tr>
        <tr><td>Average Price</td><td>${}</td></tr>
    </table>
    <h2>Volatility</h2>
    <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Daily Volatility</td><td>{}%</td></tr>
        <tr><td>Annual Volatility</td><td>{}%</td></tr>
    </table>
    <h2>Recommendations</h2>
    <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Recommended Range</td><td>${} - ${}</td></tr>
        <tr><td>Range Width</td><td>{}%</td></tr>
        <tr><td>Est. Time in Range</td><td>{}%</td></tr>
    </table>
</body>
</html>"#,
        report.pair,
        report.pair,
        report.current_price,
        report.high_price,
        report.low_price,
        report.avg_price,
        report.volatility_daily * rust_decimal::Decimal::from(100),
        report.volatility_annual * rust_decimal::Decimal::from(100),
        report.recommended_lower,
        report.recommended_upper,
        report.recommended_width * rust_decimal::Decimal::from(100),
        report.estimated_time_in_range
    )
}

fn backtest_to_html(report: &BacktestReport) -> String {
    let sharpe_row = report
        .sharpe_ratio
        .map(|s| format!("<tr><td>Sharpe Ratio</td><td>{}</td></tr>", s))
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Backtest Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #2196F3; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
        .positive {{ color: green; }}
        .negative {{ color: red; }}
    </style>
</head>
<body>
    <h1>Backtest Results: {}</h1>
    <h2>Configuration</h2>
    <table>
        <tr><th>Parameter</th><th>Value</th></tr>
        <tr><td>Period</td><td>{} days</td></tr>
        <tr><td>Range</td><td>${} - ${}</td></tr>
        <tr><td>Initial Capital</td><td>${}</td></tr>
        <tr><td>Strategy</td><td>{}</td></tr>
    </table>
    <h2>Performance</h2>
    <table>
        <tr><th>Metric</th><th>Value</th></tr>
        <tr><td>Final Value</td><td>${}</td></tr>
        <tr><td>Total Return</td><td>{}%</td></tr>
        <tr><td>Fee Earnings</td><td>${}</td></tr>
        <tr><td>Impermanent Loss</td><td>${}</td></tr>
        <tr><td>vs HODL</td><td>{}%</td></tr>
        {}
    </table>
</body>
</html>"#,
        report.pair,
        report.pair,
        report.period_days,
        report.range_lower,
        report.range_upper,
        report.initial_capital,
        report.strategy,
        report.final_value,
        report.total_return,
        report.fee_earnings,
        report.impermanent_loss,
        report.vs_hodl,
        sharpe_row
    )
}

fn optimization_to_html(report: &OptimizationReport) -> String {
    let mut rows = String::new();
    for c in &report.candidates {
        rows.push_str(&format!(
            "<tr><td>#{}</td><td>{}%</td><td>${} - ${}</td><td>{}</td><td>{}</td><td>{}</td><td>{}%</td><td>{}</td></tr>\n",
            c.rank, c.range_width_pct, c.lower_price, c.upper_price,
            c.expected_fees, c.expected_il, c.expected_pnl, c.time_in_range, c.score
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Optimization Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #9C27B0; color: white; }}
        tr:nth-child(even) {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <h1>Optimization Results: {}</h1>
    <h2>Top Candidates</h2>
    <table>
        <tr><th>Rank</th><th>Width</th><th>Range</th><th>Fees</th><th>IL</th><th>PnL</th><th>Time%</th><th>Score</th></tr>
        {}
    </table>
</body>
</html>"#,
        report.pair, report.pair, rows
    )
}

// Markdown formatters

fn analysis_to_markdown(report: &AnalysisReport) -> String {
    format!(
        r#"# Pool Analysis: {}

## Price Summary

| Metric | Value |
|--------|-------|
| Current Price | ${} |
| High Price | ${} |
| Low Price | ${} |
| Average Price | ${} |

## Volatility

| Metric | Value |
|--------|-------|
| Daily Volatility | {}% |
| Annual Volatility | {}% |

## Recommendations

| Metric | Value |
|--------|-------|
| Recommended Range | ${} - ${} |
| Range Width | {}% |
| Est. Time in Range | {}% |
"#,
        report.pair,
        report.current_price,
        report.high_price,
        report.low_price,
        report.avg_price,
        report.volatility_daily * rust_decimal::Decimal::from(100),
        report.volatility_annual * rust_decimal::Decimal::from(100),
        report.recommended_lower,
        report.recommended_upper,
        report.recommended_width * rust_decimal::Decimal::from(100),
        report.estimated_time_in_range
    )
}

fn backtest_to_markdown(report: &BacktestReport) -> String {
    let sharpe_row = report
        .sharpe_ratio
        .map(|s| format!("| Sharpe Ratio | {} |\n", s))
        .unwrap_or_default();

    format!(
        r#"# Backtest Results: {}

## Configuration

| Parameter | Value |
|-----------|-------|
| Period | {} days |
| Range | ${} - ${} |
| Initial Capital | ${} |
| Strategy | {} |

## Performance

| Metric | Value |
|--------|-------|
| Final Value | ${} |
| Total Return | {}% |
| Fee Earnings | ${} |
| Impermanent Loss | ${} |
| vs HODL | {}% |
{}
"#,
        report.pair,
        report.period_days,
        report.range_lower,
        report.range_upper,
        report.initial_capital,
        report.strategy,
        report.final_value,
        report.total_return,
        report.fee_earnings,
        report.impermanent_loss,
        report.vs_hodl,
        sharpe_row
    )
}

fn optimization_to_markdown(report: &OptimizationReport) -> String {
    let mut rows = String::new();
    for c in &report.candidates {
        rows.push_str(&format!(
            "| #{} | {}% | ${} - ${} | {} | {} | {} | {}% | {} |\n",
            c.rank,
            c.range_width_pct,
            c.lower_price,
            c.upper_price,
            c.expected_fees,
            c.expected_il,
            c.expected_pnl,
            c.time_in_range,
            c.score
        ));
    }

    format!(
        r#"# Optimization Results: {}

## Top Candidates

| Rank | Width | Range | Fees | IL | PnL | Time% | Score |
|------|-------|-------|------|----|----|-------|-------|
{}
"#,
        report.pair, rows
    )
}
