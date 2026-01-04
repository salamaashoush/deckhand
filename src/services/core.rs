//! Core dispatcher types and Docker client management

use gpui::{App, AppContext, Entity, EventEmitter, Global};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::docker::DockerClient;

/// Shared Docker client - initialized once in `load_initial_data`
static DOCKER_CLIENT: std::sync::OnceLock<Arc<RwLock<Option<DockerClient>>>> = std::sync::OnceLock::new();

/// Get the shared Docker client handle
pub fn docker_client() -> Arc<RwLock<Option<DockerClient>>> {
  DOCKER_CLIENT.get_or_init(|| Arc::new(RwLock::new(None))).clone()
}

/// Event emitted when a task completes (for UI to show notifications)
#[derive(Clone, Debug)]
pub enum DispatcherEvent {
  TaskCompleted { message: String },
  TaskFailed { error: String },
}

/// Central action dispatcher - handles all async operations
pub struct ActionDispatcher;

impl ActionDispatcher {
  pub fn new() -> Self {
    Self
  }
}

impl Default for ActionDispatcher {
  fn default() -> Self {
    Self::new()
  }
}

impl EventEmitter<DispatcherEvent> for ActionDispatcher {}

/// Global wrapper
pub struct GlobalActionDispatcher(pub Entity<ActionDispatcher>);

impl Global for GlobalActionDispatcher {}

/// Initialize the global action dispatcher
pub fn init_dispatcher(cx: &mut App) -> Entity<ActionDispatcher> {
  let dispatcher = cx.new(|_cx| ActionDispatcher::new());
  cx.set_global(GlobalActionDispatcher(dispatcher.clone()));
  dispatcher
}

/// Get the global dispatcher
pub fn dispatcher(cx: &App) -> Entity<ActionDispatcher> {
  cx.global::<GlobalActionDispatcher>().0.clone()
}
