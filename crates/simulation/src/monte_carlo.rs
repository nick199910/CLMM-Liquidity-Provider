use crate::engine::SimulationEngine;
use crate::price_path::GeometricBrownianMotion;
use crate::volume::VolumeModel;
use clmm_lp_domain::entities::position::Position;
use clmm_lp_domain::value_objects::simulation_result::SimulationResult;
use rust_decimal::Decimal;

/// Runner for Monte Carlo simulations.
pub struct MonteCarloRunner<V: VolumeModel + Clone> {
    /// The position to simulate.
    pub position: Position,
    /// The volume model.
    pub volume_model: V,
    /// The initial price.
    pub initial_price: Decimal,
    /// The annualized drift.
    pub drift: f64,
    /// The annualized volatility.
    pub volatility: f64,
    /// The time step in years.
    pub time_step: f64,
    /// The number of steps per iteration.
    pub steps: usize,
    /// The number of iterations.
    pub iterations: usize,
}

/// Result of a Monte Carlo simulation run.
pub struct AggregateResult {
    /// Mean net PnL.
    pub mean_net_pnl: Decimal,
    /// Median net PnL.
    pub median_net_pnl: Decimal,
    /// Value at Risk (95%).
    pub var_95_net_pnl: Decimal, // Value at Risk (5th percentile)
    /// Mean fees earned.
    pub mean_fees: Decimal,
    /// Mean impermanent loss.
    pub mean_il: Decimal,
    /// Number of iterations run.
    pub iterations: usize,
}

impl<V: VolumeModel + Clone> MonteCarloRunner<V> {
    /// Runs the Monte Carlo simulation.
    pub fn run(&mut self) -> AggregateResult {
        let mut results: Vec<SimulationResult> = Vec::with_capacity(self.iterations);

        for _ in 0..self.iterations {
            let gbm = GeometricBrownianMotion::new(
                self.initial_price,
                self.drift,
                self.volatility,
                self.time_step,
            );

            // Create a fresh volume model for each run if it has state
            let vol = self.volume_model.clone();

            let mut engine = SimulationEngine::new(self.position.clone(), gbm, vol, self.steps);

            results.push(engine.run());
        }

        self.aggregate(results)
    }

    fn aggregate(&self, results: Vec<SimulationResult>) -> AggregateResult {
        let count = Decimal::from(results.len());

        let total_pnl: Decimal = results.iter().map(|r| r.net_pnl).sum();
        let total_fees: Decimal = results.iter().map(|r| r.total_fees_earned).sum();
        let total_il: Decimal = results.iter().map(|r| r.total_il).sum();

        let mean_pnl = total_pnl / count;
        let mean_fees = total_fees / count;
        let mean_il = total_il / count;

        // Sort for percentiles
        let mut pnls: Vec<Decimal> = results.iter().map(|r| r.net_pnl).collect();
        pnls.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let median_idx = results.len() / 2;
        let median_pnl = pnls[median_idx];

        // VaR 95% is the value at the 5th percentile
        let var_idx = (results.len() as f64 * 0.05).floor() as usize;
        let var_95 = pnls[var_idx.min(results.len() - 1)];

        AggregateResult {
            mean_net_pnl: mean_pnl,
            median_net_pnl: median_pnl,
            var_95_net_pnl: var_95,
            mean_fees,
            mean_il,
            iterations: results.len(),
        }
    }
}
