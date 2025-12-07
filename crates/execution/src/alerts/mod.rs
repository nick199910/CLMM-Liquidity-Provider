//! Alert system for position monitoring.
//!
//! Provides configurable alerts for:
//! - Position range exits
//! - IL thresholds
//! - PnL targets
//! - System errors

mod alert;
mod notifier;
mod rules;

pub use alert::*;
pub use notifier::*;
pub use rules::*;
