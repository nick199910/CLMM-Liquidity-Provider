//! Fee earnings models and calculations.
//!
//! This module provides functions for calculating fee earnings,
//! APY projections, and breakeven analysis for LP positions.

use crate::token::TokenAmount;
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

/// Calculates total fees earned by the pool given volume and fee tier.
///
/// # Arguments
/// * `volume` - Trading volume
/// * `fee_bps` - Fee in basis points (e.g., 30 = 0.30%)
///
/// # Returns
/// Fee amount as `TokenAmount`
///
/// # Errors
/// Returns error if conversion fails or overflow occurs.
pub fn calculate_pool_fees(volume: TokenAmount, fee_bps: u32) -> Result<TokenAmount, &'static str> {
    let vol = Decimal::from_str(&volume.0.to_string()).map_err(|_| "Conversion error")?;
    let bps = Decimal::from(fee_bps);
    let ten_thousand = Decimal::from(10000);

    let fees = vol * (bps / ten_thousand);

    let fees_u128 = fees.to_u128().ok_or("Overflow")?;
    Ok(TokenAmount::from(fees_u128))
}

/// Calculates simple APY based on fees earned over a period.
///
/// # Arguments
/// * `fees_earned` - Total fees earned during the period
/// * `principal` - Initial capital invested
/// * `days` - Number of days in the period
///
/// # Returns
/// Annualized return as decimal (e.g., 0.25 = 25% APY)
///
/// # Errors
/// Returns error if principal or days is zero.
pub fn calculate_apy(
    fees_earned: Decimal,
    principal: Decimal,
    days: u32,
) -> Result<Decimal, &'static str> {
    if principal.is_zero() {
        return Err("Principal cannot be zero");
    }
    if days == 0 {
        return Err("Days cannot be zero");
    }

    let days_dec = Decimal::from(days);
    let year_days = Decimal::from(365);

    let roi = fees_earned / principal;
    let annualized = roi * (year_days / days_dec);

    Ok(annualized)
}

/// Calculates compound APY (APR to APY conversion).
///
/// # Arguments
/// * `apr` - Annual percentage rate as decimal
/// * `compounds_per_year` - Number of compounding periods per year
///
/// # Returns
/// APY as decimal
#[must_use]
pub fn apr_to_apy(apr: Decimal, compounds_per_year: u32) -> Decimal {
    if compounds_per_year == 0 {
        return apr;
    }

    let n = Decimal::from(compounds_per_year);
    let rate_per_period = apr / n;

    // APY = (1 + r/n)^n - 1
    // Using approximation for Decimal: (1 + r/n)^n ≈ e^r for large n
    // For exact calculation with small n, we iterate
    let mut result = Decimal::ONE + rate_per_period;
    for _ in 1..compounds_per_year {
        result *= Decimal::ONE + rate_per_period;
    }

    result - Decimal::ONE
}

/// Fee projection model types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeeProjectionModel {
    /// Constant fee rate based on historical average.
    Constant,
    /// Linear decay from current rate.
    LinearDecay,
    /// Exponential decay from current rate.
    ExponentialDecay,
}

/// Projects future fee earnings based on historical data.
///
/// # Arguments
/// * `historical_daily_fees` - Average daily fees from historical data
/// * `projection_days` - Number of days to project
/// * `model` - Projection model to use
/// * `decay_rate` - Decay rate per day (for decay models, e.g., 0.01 = 1% daily decay)
///
/// # Returns
/// Projected total fees over the period
#[must_use]
pub fn project_fees(
    historical_daily_fees: Decimal,
    projection_days: u32,
    model: FeeProjectionModel,
    decay_rate: Decimal,
) -> Decimal {
    match model {
        FeeProjectionModel::Constant => historical_daily_fees * Decimal::from(projection_days),

        FeeProjectionModel::LinearDecay => {
            let mut total = Decimal::ZERO;
            for day in 0..projection_days {
                let decay_factor = Decimal::ONE - (decay_rate * Decimal::from(day));
                let daily_fee = historical_daily_fees * decay_factor.max(Decimal::ZERO);
                total += daily_fee;
            }
            total
        }

        FeeProjectionModel::ExponentialDecay => {
            let mut total = Decimal::ZERO;
            let mut current_rate = historical_daily_fees;
            for _ in 0..projection_days {
                total += current_rate;
                current_rate *= Decimal::ONE - decay_rate;
            }
            total
        }
    }
}

/// Calculates the breakeven time for an LP position.
///
/// Determines how long it takes for fee earnings to offset impermanent loss.
///
/// # Arguments
/// * `impermanent_loss` - Current IL as positive decimal (e.g., 0.05 = 5% loss)
/// * `daily_fee_rate` - Daily fee earnings as percentage of position (e.g., 0.001 = 0.1%/day)
///
/// # Returns
/// Number of days to breakeven, or None if fees are zero
#[must_use]
pub fn calculate_breakeven_days(impermanent_loss: Decimal, daily_fee_rate: Decimal) -> Option<u32> {
    if daily_fee_rate.is_zero() || daily_fee_rate.is_sign_negative() {
        return None;
    }

    let days = impermanent_loss / daily_fee_rate;
    days.to_u32()
}

/// Calculates the minimum required fee rate to be profitable.
///
/// # Arguments
/// * `impermanent_loss` - Expected IL as positive decimal
/// * `holding_days` - Expected holding period in days
/// * `target_profit` - Target profit margin as decimal (e.g., 0.05 = 5%)
///
/// # Returns
/// Required daily fee rate as decimal
#[must_use]
pub fn calculate_required_fee_rate(
    impermanent_loss: Decimal,
    holding_days: u32,
    target_profit: Decimal,
) -> Decimal {
    if holding_days == 0 {
        return Decimal::MAX;
    }

    let total_required = impermanent_loss + target_profit;
    total_required / Decimal::from(holding_days)
}

/// Analyzes fee sustainability for a position.
///
/// # Arguments
/// * `current_daily_fees` - Current daily fee earnings
/// * `position_value` - Total position value
/// * `estimated_il` - Estimated IL over holding period
/// * `holding_days` - Expected holding period
///
/// # Returns
/// Tuple of (projected_net_return, is_profitable, breakeven_days)
#[must_use]
pub fn analyze_fee_sustainability(
    current_daily_fees: Decimal,
    position_value: Decimal,
    estimated_il: Decimal,
    holding_days: u32,
) -> (Decimal, bool, Option<u32>) {
    let total_fees = current_daily_fees * Decimal::from(holding_days);
    let il_amount = position_value * estimated_il;
    let net_return = total_fees - il_amount;
    let is_profitable = net_return > Decimal::ZERO;

    let daily_fee_rate = if position_value.is_zero() {
        Decimal::ZERO
    } else {
        current_daily_fees / position_value
    };

    let breakeven = calculate_breakeven_days(estimated_il, daily_fee_rate);

    (net_return, is_profitable, breakeven)
}

/// Calculates the fee tier efficiency score.
///
/// Compares actual fee earnings to theoretical maximum based on volume.
///
/// # Arguments
/// * `actual_fees` - Actual fees earned
/// * `volume` - Trading volume during period
/// * `fee_bps` - Fee tier in basis points
/// * `time_in_range_pct` - Percentage of time position was in range
///
/// # Returns
/// Efficiency score from 0.0 to 1.0
#[must_use]
pub fn calculate_fee_efficiency(
    actual_fees: Decimal,
    volume: Decimal,
    fee_bps: u32,
    time_in_range_pct: Decimal,
) -> Decimal {
    if volume.is_zero() || time_in_range_pct.is_zero() {
        return Decimal::ZERO;
    }

    let theoretical_max =
        volume * Decimal::from(fee_bps) / Decimal::from(10_000) * time_in_range_pct;

    if theoretical_max.is_zero() {
        return Decimal::ZERO;
    }

    (actual_fees / theoretical_max).min(Decimal::ONE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_calculate_apy() {
        // $100 fees on $1000 over 30 days
        let apy = calculate_apy(dec!(100), dec!(1000), 30).unwrap();
        // ROI = 10%, annualized = 10% * (365/30) ≈ 121.67%
        assert!(apy > dec!(1.2) && apy < dec!(1.25));
    }

    #[test]
    fn test_calculate_apy_zero_principal() {
        let result = calculate_apy(dec!(100), Decimal::ZERO, 30);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_apy_zero_days() {
        let result = calculate_apy(dec!(100), dec!(1000), 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_apr_to_apy() {
        // 10% APR compounded daily
        let apy = apr_to_apy(dec!(0.10), 365);
        // Should be slightly higher than 10%
        assert!(apy > dec!(0.10));
        assert!(apy < dec!(0.11));
    }

    #[test]
    fn test_project_fees_constant() {
        let daily = dec!(10);
        let projected = project_fees(daily, 30, FeeProjectionModel::Constant, Decimal::ZERO);
        assert_eq!(projected, dec!(300));
    }

    #[test]
    fn test_project_fees_linear_decay() {
        let daily = dec!(10);
        let decay = dec!(0.01); // 1% decay per day
        let projected = project_fees(daily, 10, FeeProjectionModel::LinearDecay, decay);
        // Day 0: 10, Day 1: 9.9, Day 2: 9.8, etc.
        assert!(projected < dec!(100));
        assert!(projected > dec!(90));
    }

    #[test]
    fn test_project_fees_exponential_decay() {
        let daily = dec!(10);
        let decay = dec!(0.05); // 5% decay per day
        let projected = project_fees(daily, 10, FeeProjectionModel::ExponentialDecay, decay);
        assert!(projected < dec!(100));
    }

    #[test]
    fn test_calculate_breakeven_days() {
        // 5% IL, 0.1% daily fees
        let days = calculate_breakeven_days(dec!(0.05), dec!(0.001));
        assert_eq!(days, Some(50));
    }

    #[test]
    fn test_calculate_breakeven_days_zero_fees() {
        let days = calculate_breakeven_days(dec!(0.05), Decimal::ZERO);
        assert_eq!(days, None);
    }

    #[test]
    fn test_calculate_required_fee_rate() {
        // 5% IL, 30 days, 5% profit target
        let rate = calculate_required_fee_rate(dec!(0.05), 30, dec!(0.05));
        // Need 10% total over 30 days = 0.33% per day
        assert!((rate - dec!(0.003333)).abs() < dec!(0.0001));
    }

    #[test]
    fn test_analyze_fee_sustainability() {
        let daily_fees = dec!(10);
        let position_value = dec!(10000);
        let estimated_il = dec!(0.05); // 5%
        let holding_days = 30;

        let (net_return, is_profitable, breakeven) =
            analyze_fee_sustainability(daily_fees, position_value, estimated_il, holding_days);

        // Total fees: $300, IL: $500, Net: -$200
        assert_eq!(net_return, dec!(-200));
        assert!(!is_profitable);
        assert!(breakeven.is_some());
    }

    #[test]
    fn test_calculate_fee_efficiency() {
        let actual = dec!(90);
        let volume = dec!(100000);
        let fee_bps = 30; // 0.3%
        let time_in_range = dec!(0.5); // 50%

        let efficiency = calculate_fee_efficiency(actual, volume, fee_bps, time_in_range);

        // Theoretical max: 100000 * 0.003 * 0.5 = 150
        // Efficiency: 90/150 = 0.6
        assert_eq!(efficiency, dec!(0.6));
    }
}
