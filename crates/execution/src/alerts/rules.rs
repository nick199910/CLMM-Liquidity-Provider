//! Alert rules engine.

use super::{Alert, AlertLevel, AlertType};
use crate::monitor::PositionPnL;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Alert rule configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name.
    pub name: String,
    /// Rule condition.
    pub condition: RuleCondition,
    /// Alert level when triggered.
    pub level: AlertLevel,
    /// Alert type when triggered.
    pub alert_type: AlertType,
    /// Message template.
    pub message_template: String,
    /// Whether the rule is enabled.
    pub enabled: bool,
    /// Cooldown between alerts in seconds.
    pub cooldown_secs: u64,
}

impl AlertRule {
    /// Creates a new alert rule.
    pub fn new(
        name: impl Into<String>,
        condition: RuleCondition,
        level: AlertLevel,
        alert_type: AlertType,
    ) -> Self {
        Self {
            name: name.into(),
            condition,
            level,
            alert_type,
            message_template: String::new(),
            enabled: true,
            cooldown_secs: 300, // 5 minutes default
        }
    }

    /// Sets the message template.
    #[must_use]
    pub fn with_message(mut self, template: impl Into<String>) -> Self {
        self.message_template = template.into();
        self
    }

    /// Sets the cooldown.
    #[must_use]
    pub fn with_cooldown(mut self, secs: u64) -> Self {
        self.cooldown_secs = secs;
        self
    }

    /// Disables the rule.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Condition for triggering an alert.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Position exits range.
    RangeExit,
    /// Position enters range.
    RangeEntry,
    /// IL exceeds threshold.
    ILExceeds(Decimal),
    /// PnL exceeds threshold (positive).
    PnLExceeds(Decimal),
    /// PnL below threshold (negative).
    PnLBelow(Decimal),
    /// Fees exceed threshold.
    FeesExceed(Decimal),
    /// Time since last rebalance exceeds hours.
    TimeSinceRebalance(u64),
    /// Compound condition (AND).
    And(Box<RuleCondition>, Box<RuleCondition>),
    /// Compound condition (OR).
    Or(Box<RuleCondition>, Box<RuleCondition>),
}

/// Context for evaluating rules.
#[derive(Debug, Clone)]
pub struct RuleContext {
    /// Whether position is in range.
    pub in_range: bool,
    /// Whether position was in range before.
    pub was_in_range: bool,
    /// Current PnL data.
    pub pnl: PositionPnL,
    /// Hours since last rebalance.
    pub hours_since_rebalance: u64,
}

/// Rules engine for evaluating alert conditions.
pub struct RulesEngine {
    /// Configured rules.
    rules: Vec<AlertRule>,
    /// Last trigger times for cooldown.
    last_triggers: std::collections::HashMap<String, chrono::DateTime<chrono::Utc>>,
}

impl RulesEngine {
    /// Creates a new rules engine.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            last_triggers: std::collections::HashMap::new(),
        }
    }

    /// Adds a rule.
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    /// Removes a rule by name.
    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|r| r.name != name);
    }

    /// Evaluates all rules and returns triggered alerts.
    pub fn evaluate(&mut self, context: &RuleContext) -> Vec<Alert> {
        let mut alerts = Vec::new();
        let now = chrono::Utc::now();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            // Check cooldown
            if let Some(last) = self.last_triggers.get(&rule.name) {
                let elapsed = (now - *last).num_seconds() as u64;
                if elapsed < rule.cooldown_secs {
                    continue;
                }
            }

            // Evaluate condition
            if self.evaluate_condition(&rule.condition, context) {
                let message = self.format_message(&rule.message_template, context);
                let alert = Alert::new(rule.level, rule.alert_type.clone(), message);
                alerts.push(alert);

                // Update last trigger time
                self.last_triggers.insert(rule.name.clone(), now);
            }
        }

        alerts
    }

    /// Evaluates a single condition.
    #[allow(clippy::only_used_in_recursion)]
    fn evaluate_condition(&self, condition: &RuleCondition, context: &RuleContext) -> bool {
        match condition {
            RuleCondition::RangeExit => context.was_in_range && !context.in_range,
            RuleCondition::RangeEntry => !context.was_in_range && context.in_range,
            RuleCondition::ILExceeds(threshold) => context.pnl.il_pct.abs() > *threshold,
            RuleCondition::PnLExceeds(threshold) => context.pnl.net_pnl_pct > *threshold,
            RuleCondition::PnLBelow(threshold) => context.pnl.net_pnl_pct < *threshold,
            RuleCondition::FeesExceed(threshold) => context.pnl.fees_usd > *threshold,
            RuleCondition::TimeSinceRebalance(hours) => context.hours_since_rebalance > *hours,
            RuleCondition::And(a, b) => {
                self.evaluate_condition(a, context) && self.evaluate_condition(b, context)
            }
            RuleCondition::Or(a, b) => {
                self.evaluate_condition(a, context) || self.evaluate_condition(b, context)
            }
        }
    }

    /// Formats a message template with context values.
    fn format_message(&self, template: &str, context: &RuleContext) -> String {
        template
            .replace("{il_pct}", &format!("{:.2}%", context.pnl.il_pct))
            .replace("{pnl_pct}", &format!("{:.2}%", context.pnl.net_pnl_pct))
            .replace("{pnl_usd}", &format!("${:.2}", context.pnl.net_pnl_usd))
            .replace("{fees_usd}", &format!("${:.2}", context.pnl.fees_usd))
            .replace("{in_range}", if context.in_range { "yes" } else { "no" })
    }

    /// Creates default rules.
    #[must_use]
    pub fn with_defaults(mut self) -> Self {
        // Range exit warning
        self.add_rule(
            AlertRule::new(
                "range_exit",
                RuleCondition::RangeExit,
                AlertLevel::Warning,
                AlertType::RangeExit,
            )
            .with_message("Position exited price range"),
        );

        // IL warning at 5%
        self.add_rule(
            AlertRule::new(
                "il_warning",
                RuleCondition::ILExceeds(Decimal::new(5, 2)),
                AlertLevel::Warning,
                AlertType::ILThreshold,
            )
            .with_message("IL exceeded 5%: {il_pct}"),
        );

        // IL critical at 10%
        self.add_rule(
            AlertRule::new(
                "il_critical",
                RuleCondition::ILExceeds(Decimal::new(10, 2)),
                AlertLevel::Critical,
                AlertType::ILThreshold,
            )
            .with_message("IL exceeded 10%: {il_pct}"),
        );

        self
    }
}

impl Default for RulesEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_creation() {
        let rule = AlertRule::new(
            "test_rule",
            RuleCondition::RangeExit,
            AlertLevel::Warning,
            AlertType::RangeExit,
        );

        assert_eq!(rule.name, "test_rule");
        assert!(rule.enabled);
    }

    #[test]
    fn test_evaluate_range_exit() {
        let mut engine = RulesEngine::new();
        engine.add_rule(
            AlertRule::new(
                "range_exit",
                RuleCondition::RangeExit,
                AlertLevel::Warning,
                AlertType::RangeExit,
            )
            .with_message("Position exited range"),
        );

        let context = RuleContext {
            in_range: false,
            was_in_range: true,
            pnl: PositionPnL::default(),
            hours_since_rebalance: 0,
        };

        let alerts = engine.evaluate(&context);
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].level, AlertLevel::Warning);
    }
}
