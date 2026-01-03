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

/// A single stage/step within a task
#[derive(Debug, Clone)]
pub struct TaskStage {
  #[allow(dead_code)]
  pub name: String,
  pub description: String,
}

impl TaskStage {
  pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      description: description.into(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Task {
  pub id: u64,
  pub description: String,
  pub status: TaskStatus,
  pub progress: Option<f32>, // 0.0 - 1.0
  /// All stages for this task
  pub stages: Vec<TaskStage>,
  /// Current stage index (0-based)
  pub current_stage: usize,
  /// Current stage status message
  pub stage_status: Option<String>,
}

impl Task {
  pub fn new(description: impl Into<String>) -> Self {
    Self {
      id: next_task_id(),
      description: description.into(),
      status: TaskStatus::Running,
      progress: None,
      stages: Vec::new(),
      current_stage: 0,
      stage_status: None,
    }
  }

  pub fn with_stages(mut self, stages: Vec<TaskStage>) -> Self {
    self.stages = stages;
    self
  }

  pub fn is_running(&self) -> bool {
    matches!(self.status, TaskStatus::Running)
  }

  /// Get the current stage if stages are defined
  pub fn current_stage_info(&self) -> Option<&TaskStage> {
    self.stages.get(self.current_stage)
  }

  /// Get display text for the current state
  pub fn display_status(&self) -> String {
    if let Some(status) = &self.stage_status {
      status.clone()
    } else if let Some(stage) = self.current_stage_info() {
      stage.description.clone()
    } else {
      self.description.clone()
    }
  }

  /// Get progress as a fraction based on stages (always 0.0 to 1.0)
  #[allow(clippy::cast_precision_loss)]
  pub fn stage_progress(&self) -> f32 {
    let progress = if self.stages.is_empty() {
      self.progress.unwrap_or(0.0)
    } else if self.stages.len() <= 1 {
      0.0
    } else {
      (self.current_stage as f32) / ((self.stages.len() - 1) as f32)
    };
    // Clamp to valid range for gpui::relative()
    progress.clamp(0.0, 1.0)
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
  pub fn start_task(&mut self, description: impl Into<String>) -> u64 {
    let task = Task::new(description);
    let id = task.id;
    self.tasks.insert(id, task);
    id
  }

  /// Start a new task with predefined stages
  pub fn start_staged_task(&mut self, description: impl Into<String>, stages: Vec<TaskStage>) -> u64 {
    let task = Task::new(description).with_stages(stages);
    let id = task.id;
    self.tasks.insert(id, task);
    id
  }

  /// Advance to the next stage
  pub fn advance_stage(&mut self, task_id: u64) {
    if let Some(task) = self.tasks.get_mut(&task_id)
      && task.current_stage < task.stages.len().saturating_sub(1)
    {
      task.current_stage += 1;
      task.stage_status = None;
    }
  }

  /// Set the current stage by index
  #[allow(dead_code)]
  pub fn set_stage(&mut self, task_id: u64, stage_index: usize) {
    if let Some(task) = self.tasks.get_mut(&task_id)
      && stage_index < task.stages.len()
    {
      task.current_stage = stage_index;
      task.stage_status = None;
    }
  }

  /// Update the status message for current stage
  #[allow(dead_code)]
  pub fn set_stage_status(&mut self, task_id: u64, status: impl Into<String>) {
    if let Some(task) = self.tasks.get_mut(&task_id) {
      task.stage_status = Some(status.into());
    }
  }

  /// Mark task as completed
  pub fn complete_task(&mut self, task_id: u64) {
    if let Some(task) = self.tasks.get_mut(&task_id) {
      task.status = TaskStatus::Completed;
      task.progress = Some(1.0);
      task.current_stage = task.stages.len().saturating_sub(1);
    }
    // Remove completed tasks after marking
    self.tasks.remove(&task_id);
  }

  /// Mark task as failed
  pub fn fail_task(&mut self, task_id: u64, error: impl Into<String>) {
    if let Some(task) = self.tasks.get_mut(&task_id) {
      task.status = TaskStatus::Failed(error.into());
    }
    // Remove failed tasks
    self.tasks.remove(&task_id);
  }

  /// Get all running tasks
  pub fn running_tasks(&self) -> Vec<&Task> {
    self.tasks.values().filter(|t| t.is_running()).collect()
  }

  /// Get a specific task by ID
  #[allow(dead_code)]
  pub fn get_task(&self, task_id: u64) -> Option<&Task> {
    self.tasks.get(&task_id)
  }
}

/// Global wrapper for `TaskManager`
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
pub fn start_task(cx: &mut App, description: impl Into<String>) -> u64 {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    let id = m.start_task(description);
    cx.notify();
    id
  })
}

/// Helper to start a staged task from any context
pub fn start_staged_task(cx: &mut App, description: impl Into<String>, stages: Vec<TaskStage>) -> u64 {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    let id = m.start_staged_task(description, stages);
    cx.notify();
    id
  })
}

/// Helper to advance task to next stage
pub fn advance_stage(cx: &mut App, task_id: u64) {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    m.advance_stage(task_id);
    cx.notify();
  });
}

/// Helper to set task stage by index
#[allow(dead_code)]
pub fn set_stage(cx: &mut App, task_id: u64, stage_index: usize) {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    m.set_stage(task_id, stage_index);
    cx.notify();
  });
}

/// Helper to update stage status message
#[allow(dead_code)]
pub fn set_stage_status(cx: &mut App, task_id: u64, status: impl Into<String>) {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    m.set_stage_status(task_id, status);
    cx.notify();
  });
}

/// Helper to complete a task from any context
pub fn complete_task(cx: &mut App, task_id: u64) {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    m.complete_task(task_id);
    cx.notify();
  });
}

/// Helper to fail a task from any context
pub fn fail_task(cx: &mut App, task_id: u64, error: impl Into<String>) {
  let manager = task_manager(cx);
  manager.update(cx, |m, cx| {
    m.fail_task(task_id, error);
    cx.notify();
  });
}
