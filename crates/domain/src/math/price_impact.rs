//! Price impact estimation for swaps in CLMM pools.
//!
//! This module provides functions to estimate the price impact of swaps
//! in concentrated liquidity pools, which is crucial for optimizing
//! trade execution and understanding slippage.

use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

/// Estimates price impact for a swap in a constant product AMM.
///
/// Uses the formula: price_impact = swap_amount / (reserve + swap_amount)
///
/// # Arguments
/// * `swap_amount` - Amount being swapped
/// * `reserve` - Reserve of the token being swapped into
///
/// # Returns
/// Price impact as a decimal (e.g., 0.01 = 1%)
#[must_use]
pub fn estimate_price_impact_constant_product(swap_amount: Decimal, reserve: Decimal) -> Decimal {
    if reserve.is_zero() {
        return Decimal::ONE; // 100% impact if no liquidity
    }

    swap_amount / (reserve + swap_amount)
}

/// Estimates price impact for a swap in a concentrated liquidity pool.
///
/// In CLMM, price impact depends on the liquidity distribution across ticks.
/// This simplified model assumes uniform liquidity in the current tick range.
///
/// # Arguments
/// * `swap_amount` - Amount being swapped (in token terms)
/// * `liquidity` - Active liquidity in the current tick range
/// * `sqrt_price` - Current sqrt price (Q64.64 format as f64)
/// * `fee_rate` - Fee rate as decimal (e.g., 0.003)
///
/// # Returns
/// Estimated price impact as a decimal
#[must_use]
pub fn estimate_price_impact_clmm(
    swap_amount: Decimal,
    liquidity: u128,
    sqrt_price: f64,
    fee_rate: Decimal,
) -> Decimal {
    if liquidity == 0 || sqrt_price <= 0.0 {
        return Decimal::ONE;
    }

    // Effective swap amount after fees
    let effective_amount = swap_amount * (Decimal::ONE - fee_rate);

    // Approximate virtual reserves from liquidity and sqrt_price
    // For a CLMM: x = L / sqrt(P), y = L * sqrt(P)
    let liquidity_dec = Decimal::from(liquidity);
    let sqrt_price_dec = Decimal::from_f64(sqrt_price).unwrap_or(Decimal::ONE);

    if sqrt_price_dec.is_zero() {
        return Decimal::ONE;
    }

    let virtual_reserve = liquidity_dec / sqrt_price_dec;

    if virtual_reserve.is_zero() {
        return Decimal::ONE;
    }

    // Price impact approximation
    effective_amount / (virtual_reserve + effective_amount)
}

/// Calculates the execution price for a swap given price impact.
///
/// # Arguments
/// * `spot_price` - Current spot price
/// * `price_impact` - Price impact as decimal (e.g., 0.01 = 1%)
/// * `is_buy` - True if buying the base token (price goes up)
///
/// # Returns
/// Expected execution price
#[must_use]
pub fn calculate_execution_price(
    spot_price: Decimal,
    price_impact: Decimal,
    is_buy: bool,
) -> Decimal {
    if is_buy {
        spot_price * (Decimal::ONE + price_impact)
    } else {
        spot_price * (Decimal::ONE - price_impact)
    }
}

/// Calculates slippage from spot price to execution price.
///
/// # Arguments
/// * `spot_price` - Current spot price
/// * `execution_price` - Actual execution price
///
/// # Returns
/// Slippage as a decimal (positive means worse execution)
#[must_use]
pub fn calculate_slippage(spot_price: Decimal, execution_price: Decimal) -> Decimal {
    if spot_price.is_zero() {
        return Decimal::ZERO;
    }

    ((execution_price - spot_price) / spot_price).abs()
}

/// Estimates the maximum swap size for a given maximum price impact.
///
/// # Arguments
/// * `max_impact` - Maximum acceptable price impact (e.g., 0.01 = 1%)
/// * `liquidity` - Active liquidity
/// * `sqrt_price` - Current sqrt price
///
/// # Returns
/// Maximum swap amount to stay within impact limit
#[must_use]
pub fn estimate_max_swap_for_impact(
    max_impact: Decimal,
    liquidity: u128,
    sqrt_price: f64,
) -> Decimal {
    if liquidity == 0 || sqrt_price <= 0.0 || max_impact >= Decimal::ONE {
        return Decimal::ZERO;
    }

    let liquidity_dec = Decimal::from(liquidity);
    let sqrt_price_dec = Decimal::from_f64(sqrt_price).unwrap_or(Decimal::ONE);

    if sqrt_price_dec.is_zero() {
        return Decimal::ZERO;
    }

    let virtual_reserve = liquidity_dec / sqrt_price_dec;

    // From: impact = amount / (reserve + amount)
    // Solving for amount: amount = impact * reserve / (1 - impact)
    let denominator = Decimal::ONE - max_impact;
    if denominator.is_zero() {
        return Decimal::ZERO;
    }

    max_impact * virtual_reserve / denominator
}

/// Estimates price impact across multiple tick ranges.
///
/// This is a more accurate model for large swaps that cross tick boundaries.
///
/// # Arguments
/// * `swap_amount` - Total amount to swap
/// * `tick_liquidities` - Vector of (tick, liquidity) pairs in order
/// * `current_sqrt_price` - Current sqrt price
/// * `tick_spacing` - Tick spacing for the pool
///
/// # Returns
/// Total price impact considering tick crossings
#[must_use]
pub fn estimate_price_impact_multi_tick(
    swap_amount: Decimal,
    tick_liquidities: &[(i32, u128)],
    current_sqrt_price: f64,
    tick_spacing: i32,
) -> Decimal {
    if tick_liquidities.is_empty() || swap_amount.is_zero() {
        return Decimal::ZERO;
    }

    let mut remaining_amount = swap_amount;
    let mut total_impact = Decimal::ZERO;
    let mut current_price =
        Decimal::from_f64(current_sqrt_price * current_sqrt_price).unwrap_or(Decimal::ONE);

    for (tick, liquidity) in tick_liquidities {
        if remaining_amount.is_zero() {
            break;
        }

        if *liquidity == 0 {
            continue;
        }

        // Calculate how much can be swapped in this tick range
        let tick_range_size = Decimal::from_f64(
            (1.0001_f64.powi(tick_spacing) - 1.0) * current_sqrt_price * current_sqrt_price,
        )
        .unwrap_or(Decimal::ZERO);

        let liquidity_dec = Decimal::from(*liquidity);
        let max_in_tick = liquidity_dec * tick_range_size / current_price;

        let amount_in_tick = remaining_amount.min(max_in_tick);
        let tick_impact = if liquidity_dec.is_zero() {
            Decimal::ZERO
        } else {
            amount_in_tick / liquidity_dec
        };

        // Weight impact by proportion of swap
        let weight = amount_in_tick / swap_amount;
        total_impact += tick_impact * weight;

        remaining_amount -= amount_in_tick;

        // Update price for next tick (simplified)
        let price_change = 1.0001_f64.powi(*tick);
        current_price = Decimal::from_f64(price_change).unwrap_or(current_price);
    }

    total_impact
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_price_impact_constant_product() {
        // Swapping 100 into a pool with 10000 reserve
        let impact = estimate_price_impact_constant_product(dec!(100), dec!(10000));
        // Expected: 100 / (10000 + 100) = 0.0099...
        assert!(impact > dec!(0.009) && impact < dec!(0.01));
    }

    #[test]
    fn test_price_impact_constant_product_zero_reserve() {
        let impact = estimate_price_impact_constant_product(dec!(100), Decimal::ZERO);
        assert_eq!(impact, Decimal::ONE);
    }

    #[test]
    fn test_price_impact_clmm() {
        let swap_amount = dec!(1000);
        let liquidity = 1_000_000_u128;
        let sqrt_price = 100.0; // sqrt(10000) = 100
        let fee_rate = dec!(0.003);

        let impact = estimate_price_impact_clmm(swap_amount, liquidity, sqrt_price, fee_rate);

        // Should be a small positive number
        assert!(impact > Decimal::ZERO);
        assert!(impact < dec!(0.1)); // Less than 10%
    }

    #[test]
    fn test_price_impact_clmm_zero_liquidity() {
        let impact = estimate_price_impact_clmm(dec!(1000), 0, 100.0, dec!(0.003));
        assert_eq!(impact, Decimal::ONE);
    }

    #[test]
    fn test_calculate_execution_price_buy() {
        let spot = dec!(100);
        let impact = dec!(0.01); // 1%

        let exec_price = calculate_execution_price(spot, impact, true);
        assert_eq!(exec_price, dec!(101));
    }

    #[test]
    fn test_calculate_execution_price_sell() {
        let spot = dec!(100);
        let impact = dec!(0.01); // 1%

        let exec_price = calculate_execution_price(spot, impact, false);
        assert_eq!(exec_price, dec!(99));
    }

    #[test]
    fn test_calculate_slippage() {
        let spot = dec!(100);
        let exec = dec!(101);

        let slippage = calculate_slippage(spot, exec);
        assert_eq!(slippage, dec!(0.01));
    }

    #[test]
    fn test_estimate_max_swap_for_impact() {
        let max_impact = dec!(0.01); // 1%
        let liquidity = 1_000_000_u128;
        let sqrt_price = 100.0;

        let max_swap = estimate_max_swap_for_impact(max_impact, liquidity, sqrt_price);

        // Verify the impact of this swap is approximately the max
        let actual_impact =
            estimate_price_impact_clmm(max_swap, liquidity, sqrt_price, Decimal::ZERO);
        assert!((actual_impact - max_impact).abs() < dec!(0.001));
    }

    #[test]
    fn test_estimate_max_swap_zero_liquidity() {
        let max_swap = estimate_max_swap_for_impact(dec!(0.01), 0, 100.0);
        assert_eq!(max_swap, Decimal::ZERO);
    }
}
