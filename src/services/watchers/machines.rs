//! Colima machine watcher
//!
//! Unlike Docker and Kubernetes, Colima doesn't have an event stream API.
//! This watcher polls for machine status changes at a reasonable interval.

use std::collections::HashMap;
use std::time::Duration;

use super::WatcherControl;
use super::debouncer::ResourceType;
use crate::colima::{ColimaClient, VmStatus};

/// Machine watcher that polls Colima for status changes
pub struct MachineWatcher {
  poll_interval: Duration,
}

impl MachineWatcher {
  pub fn new(poll_interval: Duration) -> Self {
    Self { poll_interval }
  }

  /// Watch for machine status changes via polling
  pub async fn watch<F>(&self, control: WatcherControl, mut on_change: F)
  where
    F: FnMut(ResourceType) + Send,
  {
    // Track previous state to detect changes (name -> status)
    let mut previous_state: HashMap<String, VmStatus> = HashMap::new();

    while control.is_running() {
      // Poll current machine state
      let machines = ColimaClient::list().unwrap_or_default();

      // Build current state map
      let current_state: HashMap<String, VmStatus> = machines.iter().map(|m| (m.name.clone(), m.status)).collect();

      // Check for changes
      let has_changes = if current_state.len() == previous_state.len() {
        current_state
          .iter()
          .any(|(name, status)| previous_state.get(name) != Some(status))
      } else {
        true
      };

      if has_changes && !previous_state.is_empty() {
        // Skip first poll to avoid triggering on startup
        on_change(ResourceType::Machine);
      }

      previous_state = current_state;

      // Wait for next poll interval
      tokio::time::sleep(self.poll_interval).await;
    }
  }
}
