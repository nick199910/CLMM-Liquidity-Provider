//! Scheduler implementation for task execution timing.

use super::{Schedule, ScheduledTask, TaskEvent};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{Instant, interval};
use tracing::{debug, info, warn};

/// Scheduler for managing task execution timing.
pub struct Scheduler {
    /// Scheduled tasks.
    tasks: Vec<ScheduledTask>,
    /// Event sender.
    event_tx: mpsc::Sender<TaskEvent>,
    /// Event receiver.
    event_rx: Option<mpsc::Receiver<TaskEvent>>,
    /// Running flag.
    running: Arc<AtomicBool>,
}

impl Scheduler {
    /// Creates a new scheduler.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tasks: Vec::new(),
            event_tx: tx,
            event_rx: Some(rx),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Adds a task to the scheduler.
    pub fn add_task(&mut self, task: ScheduledTask) {
        info!(task = %task.name, "Adding task to scheduler");
        self.tasks.push(task);
    }

    /// Removes a task by name.
    pub fn remove_task(&mut self, name: &str) {
        self.tasks.retain(|t| t.name != name);
    }

    /// Enables a task by name.
    pub fn enable_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == name) {
            task.enabled = true;
        }
    }

    /// Disables a task by name.
    pub fn disable_task(&mut self, name: &str) {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.name == name) {
            task.enabled = false;
        }
    }

    /// Takes the event receiver for processing events.
    pub fn take_receiver(&mut self) -> Option<mpsc::Receiver<TaskEvent>> {
        self.event_rx.take()
    }

    /// Starts the scheduler.
    pub async fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);

        info!(tasks = self.tasks.len(), "Starting scheduler");

        // Initialize next run times
        let now = Instant::now();
        for task in &mut self.tasks {
            let next = Self::calculate_next_run_static(&task.schedule, now);
            task.next_run = Some(next);
        }

        // Main scheduler loop
        let mut check_interval = interval(Duration::from_secs(1));

        while self.running.load(Ordering::SeqCst) {
            check_interval.tick().await;

            let now = Instant::now();

            // Collect events to send
            let mut events_to_send = Vec::new();

            for task in &mut self.tasks {
                if !task.enabled {
                    continue;
                }

                if let Some(next_run) = task.next_run
                    && now >= next_run
                {
                    // Task should run
                    let event = TaskEvent {
                        task_name: task.name.clone(),
                        scheduled_at: next_run,
                        triggered_at: now,
                    };

                    events_to_send.push((task.name.clone(), event));

                    task.last_run = Some(now);
                    let next = Self::calculate_next_run_static(&task.schedule, now);
                    task.next_run = Some(next);

                    debug!(
                        task = %task.name,
                        next_run = ?task.next_run,
                        "Task triggered"
                    );
                }
            }

            // Send events outside the mutable borrow
            for (task_name, event) in events_to_send {
                if let Err(e) = self.event_tx.send(event).await {
                    warn!(task = %task_name, error = %e, "Failed to send task event");
                }
            }
        }

        info!("Scheduler stopped");
    }

    /// Stops the scheduler.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Calculates the next run time for a schedule (static version).
    fn calculate_next_run_static(schedule: &Schedule, from: Instant) -> Instant {
        match schedule {
            Schedule::Interval(duration) => from + *duration,
            Schedule::Once(delay) => from + *delay,
            Schedule::Daily(_times) => {
                // Simplified: just run in 24 hours
                // A real implementation would calculate based on wall clock time
                from + Duration::from_secs(24 * 60 * 60)
            }
            Schedule::Cron(_expr) => {
                // Simplified: just run in 1 hour
                // A real implementation would parse the cron expression
                from + Duration::from_secs(60 * 60)
            }
        }
    }

    /// Gets all tasks.
    pub fn tasks(&self) -> &[ScheduledTask] {
        &self.tasks
    }

    /// Checks if the scheduler is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scheduler::ScheduleBuilder;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let mut scheduler = Scheduler::new();
        scheduler.add_task(ScheduledTask::new("test", ScheduleBuilder::every_secs(1)));

        assert_eq!(scheduler.tasks().len(), 1);
    }
}
