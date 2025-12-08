//! Types for the scheduler module.

use std::time::Duration;
use tokio::time::Instant;

/// Schedule type for task execution.
#[derive(Debug, Clone)]
pub enum Schedule {
    /// Run at fixed intervals.
    Interval(Duration),
    /// Run at specific times (hour, minute).
    Daily(Vec<(u8, u8)>),
    /// Run once after delay.
    Once(Duration),
    /// Custom schedule with cron-like expression.
    Cron(String),
}

/// A scheduled task.
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Task name.
    pub name: String,
    /// Schedule.
    pub schedule: Schedule,
    /// Whether task is enabled.
    pub enabled: bool,
    /// Last run time.
    pub last_run: Option<Instant>,
    /// Next scheduled run.
    pub next_run: Option<Instant>,
}

impl ScheduledTask {
    /// Creates a new scheduled task.
    pub fn new(name: impl Into<String>, schedule: Schedule) -> Self {
        Self {
            name: name.into(),
            schedule,
            enabled: true,
            last_run: None,
            next_run: None,
        }
    }

    /// Disables the task.
    #[must_use]
    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

/// Event sent when a task should run.
#[derive(Debug, Clone)]
pub struct TaskEvent {
    /// Task name.
    pub task_name: String,
    /// Scheduled time.
    pub scheduled_at: Instant,
    /// Actual trigger time.
    pub triggered_at: Instant,
}

/// Builder for creating common schedules.
pub struct ScheduleBuilder;

impl ScheduleBuilder {
    /// Creates an interval schedule.
    pub fn every(duration: Duration) -> Schedule {
        Schedule::Interval(duration)
    }

    /// Creates a schedule that runs every N seconds.
    pub fn every_secs(secs: u64) -> Schedule {
        Schedule::Interval(Duration::from_secs(secs))
    }

    /// Creates a schedule that runs every N minutes.
    pub fn every_mins(mins: u64) -> Schedule {
        Schedule::Interval(Duration::from_secs(mins * 60))
    }

    /// Creates a schedule that runs every N hours.
    pub fn every_hours(hours: u64) -> Schedule {
        Schedule::Interval(Duration::from_secs(hours * 60 * 60))
    }

    /// Creates a one-time schedule.
    pub fn once_after(delay: Duration) -> Schedule {
        Schedule::Once(delay)
    }

    /// Creates a daily schedule at specific times.
    pub fn daily_at(times: Vec<(u8, u8)>) -> Schedule {
        Schedule::Daily(times)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_builder() {
        let schedule = ScheduleBuilder::every_mins(5);
        assert!(matches!(schedule, Schedule::Interval(_)));

        if let Schedule::Interval(d) = schedule {
            assert_eq!(d, Duration::from_secs(300));
        }
    }

    #[test]
    fn test_scheduled_task() {
        let task = ScheduledTask::new("test", ScheduleBuilder::every_secs(60));
        assert!(task.enabled);
        assert_eq!(task.name, "test");
    }
}
