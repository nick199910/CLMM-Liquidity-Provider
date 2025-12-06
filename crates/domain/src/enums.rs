use serde::{Deserialize, Serialize};

/// Supported protocols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    /// Raydium protocol.
    Raydium,
    /// Orca Whirlpools protocol.
    OrcaWhirlpools,
    /// Orca Legacy protocol.
    OrcaLegacy,
    /// Meteora DLMM protocol.
    MeteoraDLMM,
    /// Meteora Stable protocol.
    MeteoraStable,
}

/// Types of pools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolType {
    /// Constant product pool (AMM).
    ConstantProduct,
    /// Concentrated liquidity pool (CLMM).
    ConcentratedLiquidity,
    /// Stable swap pool.
    StableSwap,
}

/// Status of a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionStatus {
    /// The position is open.
    Open,
    /// The position is closed.
    Closed,
    /// The position is out of range.
    OutOfRange,
}

/// Objectives for optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationObjective {
    /// Maximize fee yield.
    MaximizeFeeYield,
    /// Minimize impermanent loss.
    MinimizeImpermanentLoss,
    /// Maximize Sharpe ratio.
    MaximizeSharpeRatio,
    /// Maximize net return.
    MaximizeNetReturn,
}

/// Time horizon for analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeHorizon {
    /// Horizon in days.
    Days(u32),
    /// Horizon in weeks.
    Weeks(u32),
    /// Horizon in months.
    Months(u32),
}
