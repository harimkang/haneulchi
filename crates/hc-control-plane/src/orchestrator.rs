use serde::{Deserialize, Serialize};

/// Summary of the orchestrator's automation subsystem state.
///
/// This is the control-plane's authoritative view of the scheduler / retry /
/// reconcile runtime, exposed through the snapshot and ops strip.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationStatusSummary {
    pub status: String,
    pub queued_claim_count: u32,
    pub paused: bool,
    pub last_tick_at: Option<String>,
    pub last_reconcile_at: Option<String>,
}

/// Tracks the most recent tick metadata for the orchestrator runtime.
///
/// The orchestrator tick runs on a cadence (default 15 s).  After each tick the
/// caller updates this tracker so that the ops strip and snapshot can report
/// the freshness of automation state.
#[cfg(test)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct OrchestratorTickTracker {
    pub tick_count: u64,
    pub last_tick_at: String,
    pub last_reconcile_at: Option<String>,
    pub running_slots: u32,
    pub max_slots: u32,
    pub paused: bool,
}

#[cfg(test)]
impl Default for OrchestratorTickTracker {
    fn default() -> Self {
        Self {
            tick_count: 0,
            last_tick_at: hc_domain::time::now_iso8601(),
            last_reconcile_at: None,
            running_slots: 0,
            max_slots: 1,
            paused: false,
        }
    }
}

#[cfg(test)]
impl OrchestratorTickTracker {
    /// Record a new tick.  Returns the updated tick count.
    fn record_tick(&mut self) -> u64 {
        self.tick_count = self.tick_count.saturating_add(1);
        self.last_tick_at = hc_domain::time::now_iso8601();
        self.tick_count
    }

    /// Record that reconciliation ran during this tick.
    fn record_reconcile(&mut self) {
        self.last_reconcile_at = Some(hc_domain::time::now_iso8601());
    }

    /// Project the current tracker state into a summary for the ops strip.
    fn to_summary(&self) -> AutomationStatusSummary {
        AutomationStatusSummary {
            status: if self.paused {
                "paused".to_string()
            } else {
                "running".to_string()
            },
            queued_claim_count: 0,
            paused: self.paused,
            last_tick_at: Some(self.last_tick_at.clone()),
            last_reconcile_at: self.last_reconcile_at.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_tracker_increments_and_records_timestamps() {
        let mut tracker = OrchestratorTickTracker::default();
        assert_eq!(tracker.tick_count, 0);

        let count = tracker.record_tick();
        assert_eq!(count, 1);
        assert!(!tracker.last_tick_at.is_empty());

        tracker.record_reconcile();
        assert!(tracker.last_reconcile_at.is_some());

        let summary = tracker.to_summary();
        assert_eq!(summary.status, "running");
        assert!(!summary.paused);
        assert!(summary.last_tick_at.is_some());
        assert!(summary.last_reconcile_at.is_some());
    }

    #[test]
    fn paused_tracker_reports_paused_status() {
        let tracker = OrchestratorTickTracker {
            paused: true,
            ..OrchestratorTickTracker::default()
        };
        let summary = tracker.to_summary();
        assert_eq!(summary.status, "paused");
        assert!(summary.paused);
    }
}
