//! Event debouncer for batching rapid-fire events
//!
//! When multiple events come in rapid succession (e.g., creating multiple containers),
//! we batch them into a single UI refresh to avoid excessive re-renders.

use std::collections::HashSet;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{Instant, interval};

use crate::services::Tokio;

/// Resource types that can be debounced
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
  Container,
  Image,
  Volume,
  Network,
  Pod,
  Deployment,
  Service,
  Machine,
}

/// Simple debouncer using an async channel pattern
///
/// Events are queued and then emitted in batches after a debounce window.
/// This prevents UI thrashing when many events occur in quick succession.
pub struct EventDebouncer;

impl EventDebouncer {
  /// Create a debounced event channel
  ///
  /// Returns (sender, receiver) where:
  /// - sender: Queue resource types as they change
  /// - receiver: Receives batched updates after debounce window
  ///
  /// Note: Must be called after `gpui_tokio::init()` has been called.
  pub fn channel(debounce_ms: u64) -> (DebounceSender, mpsc::Receiver<HashSet<ResourceType>>) {
    let (event_tx, mut event_rx) = mpsc::channel::<ResourceType>(128);
    let (batch_tx, batch_rx) = mpsc::channel::<HashSet<ResourceType>>(16);

    // Spawn the debounce loop on the tokio runtime via the GPUI-tokio bridge
    let handle = Tokio::runtime_handle();
    handle.spawn(async move {
      let debounce_duration = Duration::from_millis(debounce_ms);
      let mut pending: HashSet<ResourceType> = HashSet::new();
      let mut last_event_time = Instant::now();
      let mut check_interval = interval(Duration::from_millis(50));

      loop {
        tokio::select! {
          // Receive new events
          event = event_rx.recv() => {
            match event {
              Some(resource_type) => {
                pending.insert(resource_type);
                last_event_time = Instant::now();
              }
              None => break, // Channel closed
            }
          }

          // Periodic check for debounce window
          _ = check_interval.tick() => {
            if !pending.is_empty() && last_event_time.elapsed() >= debounce_duration {
              // Debounce window passed, emit batch
              let batch = std::mem::take(&mut pending);
              if batch_tx.send(batch).await.is_err() {
                break; // Receiver closed
              }
            }
          }
        }
      }
    });

    (DebounceSender { tx: event_tx }, batch_rx)
  }
}

/// Sender for queuing resource change events
#[derive(Clone)]
pub struct DebounceSender {
  tx: mpsc::Sender<ResourceType>,
}

impl DebounceSender {
  /// Queue a resource type for debounced refresh
  ///
  /// Non-blocking - drops event if channel is full (backpressure)
  pub fn send(&self, resource: ResourceType) {
    // Use try_send to avoid blocking and provide backpressure
    let _ = self.tx.try_send(resource);
  }
}
