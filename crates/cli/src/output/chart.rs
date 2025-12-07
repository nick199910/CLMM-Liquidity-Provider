//! ASCII chart generation for CLI output.
//!
//! Provides simple ASCII-based visualizations for terminal output.

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;

/// Configuration for ASCII charts.
#[derive(Debug, Clone)]
pub struct ChartConfig {
    /// Chart width in characters.
    pub width: usize,
    /// Chart height in characters.
    pub height: usize,
    /// Character for filled bars.
    pub fill_char: char,
    /// Character for empty space.
    pub empty_char: char,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 60,
            height: 10,
            fill_char: '█',
            empty_char: ' ',
        }
    }
}

/// Renders a simple ASCII bar chart.
pub fn render_bar_chart(data: &[(&str, Decimal)], config: &ChartConfig) -> String {
    if data.is_empty() {
        return String::from("No data to display");
    }

    let max_value = data.iter().map(|(_, v)| *v).max().unwrap_or(Decimal::ONE);

    let max_label_len = data.iter().map(|(l, _)| l.len()).max().unwrap_or(10);

    let mut output = String::new();

    for (label, value) in data {
        let ratio = if max_value.is_zero() {
            0.0
        } else {
            (*value / max_value).to_f64().unwrap_or(0.0)
        };

        let bar_width = (ratio * config.width as f64) as usize;
        let bar: String = std::iter::repeat_n(config.fill_char, bar_width).collect();

        output.push_str(&format!(
            "{:>width$} │{} {:.2}\n",
            label,
            bar,
            value,
            width = max_label_len
        ));
    }

    output
}

/// Renders a simple ASCII line chart for price history.
pub fn render_price_chart(prices: &[Decimal], config: &ChartConfig) -> String {
    if prices.is_empty() {
        return String::from("No data to display");
    }

    let min_price = prices.iter().min().copied().unwrap_or(Decimal::ZERO);
    let max_price = prices.iter().max().copied().unwrap_or(Decimal::ONE);
    let range = max_price - min_price;

    if range.is_zero() {
        return String::from("Price range is zero");
    }

    // Sample prices to fit width
    let step = prices.len().max(1) / config.width.max(1);
    let sampled: Vec<Decimal> = if step > 1 {
        prices.iter().step_by(step).copied().collect()
    } else {
        prices.to_vec()
    };

    // Create chart grid
    let mut grid: Vec<Vec<char>> = vec![vec![' '; sampled.len()]; config.height];

    for (x, price) in sampled.iter().enumerate() {
        let normalized = ((*price - min_price) / range).to_f64().unwrap_or(0.0);
        let y = ((1.0 - normalized) * (config.height - 1) as f64) as usize;
        let y = y.min(config.height - 1);
        grid[y][x] = '●';
    }

    // Render grid
    let mut output = String::new();

    // Top label
    output.push_str(&format!("{:.2} ┤\n", max_price));

    for row in &grid {
        output.push_str("      │");
        for &c in row {
            output.push(c);
        }
        output.push('\n');
    }

    // Bottom label
    output.push_str(&format!("{:.2} ┤", min_price));
    output.push_str(&"─".repeat(sampled.len()));
    output.push('\n');

    // Time axis
    output.push_str("       ");
    output.push_str(&format!(
        "Start{:>width$}End",
        "",
        width = sampled.len().saturating_sub(8)
    ));
    output.push('\n');

    output
}

/// Renders a horizontal percentage bar.
pub fn render_percentage_bar(value: Decimal, width: usize) -> String {
    let pct = value.to_f64().unwrap_or(0.0).clamp(0.0, 100.0);
    let filled = (pct / 100.0 * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    format!("[{}{}] {:.1}%", "█".repeat(filled), "░".repeat(empty), pct)
}

/// Renders a comparison bar showing two values.
pub fn render_comparison_bar(value_a: Decimal, value_b: Decimal, width: usize) -> String {
    let total = value_a + value_b;
    if total.is_zero() {
        return format!("[{}]", "─".repeat(width));
    }

    let ratio_a = (value_a / total).to_f64().unwrap_or(0.5);
    let width_a = (ratio_a * width as f64) as usize;
    let width_b = width.saturating_sub(width_a);

    format!("[{}{}]", "▓".repeat(width_a), "░".repeat(width_b))
}

/// Prints a sparkline for a series of values.
pub fn render_sparkline(values: &[Decimal]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let chars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let min = values.iter().min().copied().unwrap_or(Decimal::ZERO);
    let max = values.iter().max().copied().unwrap_or(Decimal::ONE);
    let range = max - min;

    if range.is_zero() {
        return chars[4].to_string().repeat(values.len());
    }

    values
        .iter()
        .map(|v| {
            let normalized = ((*v - min) / range).to_f64().unwrap_or(0.0);
            let idx = (normalized * 7.0) as usize;
            chars[idx.min(7)]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_render_bar_chart() {
        let data = vec![("Fees", dec!(100)), ("IL", dec!(50)), ("PnL", dec!(50))];
        let config = ChartConfig {
            width: 20,
            ..Default::default()
        };

        let chart = render_bar_chart(&data, &config);
        assert!(!chart.is_empty());
        assert!(chart.contains("Fees"));
    }

    #[test]
    fn test_render_percentage_bar() {
        let bar = render_percentage_bar(dec!(75), 20);
        assert!(bar.contains("75.0%"));
        assert!(bar.contains("█"));
    }

    #[test]
    fn test_render_sparkline() {
        let values = vec![dec!(1), dec!(2), dec!(3), dec!(2), dec!(1)];
        let sparkline = render_sparkline(&values);
        assert_eq!(sparkline.chars().count(), 5);
    }
}
