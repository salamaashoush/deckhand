//! Real-time resource watchers
//!
//! This module provides event-driven watchers for Docker and Kubernetes resources.
//! Instead of polling, these watchers subscribe to event streams and emit state
//! changes as they occur.
//!
//! - `docker_events` - Watches Docker daemon events (container start/stop, image pull, etc.)
//! - `kubernetes` - Watches Kubernetes resources using the Watch API
//! - `machines` - Polls Colima for machine status changes (no event API available)
//! - `manager` - Coordinates all watchers with debouncing

mod debouncer;
mod docker_events;
mod kubernetes;
mod machines;
mod manager;

pub use manager::WatcherManager;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Shared watcher control handle for stopping watchers
#[derive(Clone, Default)]
pub struct WatcherControl {
  running: Arc<AtomicBool>,
}

impl WatcherControl {
  pub fn new() -> Self {
    Self {
      running: Arc::new(AtomicBool::new(true)),
    }
  }

  pub fn stop(&self) {
    self.running.store(false, Ordering::SeqCst);
  }

  pub fn is_running(&self) -> bool {
    self.running.load(Ordering::SeqCst)
  }
}
