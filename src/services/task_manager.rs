use gpui::{App, AppContext, Entity, Global};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_task_id() -> u64 {
  TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
  Running,
  Completed,
  Failed(String),
}

#[derive(Debug, Clone)]
pub struct Task {
  pub id: u64,
  pub name: String,
  pub description: String,
  pub status: TaskStatus,
  pub progress: Option<f32>, // 0.0 - 1.0
}

impl Task {
  pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
    Self {
      id: next_task_id(),
      name: name.into(),
      description: description.into(),
      status: TaskStatus::Running,
      progress: None,
    }
  }

  pub fn is_running(&self) -> bool {
    matches!(self.status, TaskStatus::Running)
  }
}

#[derive(Default)]
pub struct TaskManager {
  tasks: HashMap<u64, Task>,
}

impl TaskManager {
  pub fn new() -> Self {
    Self::default()
  }

  /// Start a new task and return its ID
  pub fn start_task(&mut self, name: impl Into<String>, description: impl Into<String>) -> u64 {
    let task = Task::new(name, description);
    let id = task.id;
    self.tasks.insert(id, task);
    id
  }

  /// Update task progress (0.0 - 1.0)
  pub fn update_progress(&mut self, task_id: u64, progress: f32) {
    if let Some(task) = self.tasks.get_mut(&task_id) {
      task.progress = Some(progress.clamp(0.0, 1.0));
    }
  }

  /// Mark task as completed
  pub fn complete_task(&mut self, task_id: u64) {
    if let Some(task) = self.tasks.get_mut(&task_id) {
      task.status = TaskStatus::Completed;
      task.progress = Some(1.0);
    }
    // Remove completed tasks after marking
    self.tasks.remove(&task_id);
  }

  /// Mark task as failed
  pub fn fail_task(&mut self, task_id: u64, error: impl Into<String>) {
    if let Some(task) = self.tasks.get_mut(&task_id) {
      task.status = TaskStatus::Failed(error.into());
    }
  }

  /// Remove a task (for dismissing failed tasks)
  pub fn remove_task(&mut self, task_id: u64) {
    self.tasks.remove(&task_id);
  }

  /// Get all running tasks
  pub fn running_tasks(&self) -> Vec<&Task> {
    self.tasks.values().filter(|t| t.is_running()).collect()
  }

  /// Get all tasks
  pub fn all_tasks(&self) -> Vec<&Task> {
    self.tasks.values().collect()
  }

  /// Check if any tasks are running
  pub fn has_running_tasks(&self) -> bool {
    self.tasks.values().any(|t| t.is_running())
  }

  /// Get a specific task
  pub fn get_task(&self, task_id: u64) -> Option<&Task> {
    self.tasks.get(&task_id)
  }
}

/// Global wrapper for TaskManager
pub struct GlobalTaskManager(pub Entity<TaskManager>);

impl Global for GlobalTaskManager {}

/// Initialize the global task manager
pub fn init_task_manager(cx: &mut App) {
  let manager = cx.new(|_cx| TaskManager::new());
  cx.set_global(GlobalTaskManager(manager));
}

/// Get the global task manager entity
pub fn task_manager(cx: &App) -> Entity<TaskManager> {
  cx.global::<GlobalTaskManager>().0.clone()
}

/// Helper to start a task from any context
pub fn start_task(cx: &mut App, name: impl Into<String>, description: impl Into<String>) -> u64 {
  let manager = task_manager(cx);
  manager.update(cx, |m, _| m.start_task(name, description))
}

/// Helper to complete a task from any context
pub fn complete_task(cx: &mut App, task_id: u64) {
  let manager = task_manager(cx);
  manager.update(cx, |m, _| m.complete_task(task_id));
}

/// Helper to fail a task from any context
pub fn fail_task(cx: &mut App, task_id: u64, error: impl Into<String>) {
  let manager = task_manager(cx);
  manager.update(cx, |m, _| m.fail_task(task_id, error));
}
