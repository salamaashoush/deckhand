//! Docker Compose operations

use gpui::App;

use crate::services::{complete_task, fail_task, start_task};
use crate::utils::docker_cmd;

use super::super::core::{DispatcherEvent, dispatcher};
use super::containers::refresh_containers;

pub fn compose_up(project_name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Starting '{project_name}'..."));
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let project = project_name.clone();
    let result = cx
      .background_executor()
      .spawn(async move {
        let output = docker_cmd().args(["compose", "-p", &project, "up", "-d"]).output();

        match output {
          Ok(out) if out.status.success() => Ok(()),
          Ok(out) => Err(String::from_utf8_lossy(&out.stderr).to_string()),
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(()) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Started '{project_name}'"),
          });
        });
        refresh_containers(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to start '{project_name}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn compose_down(project_name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Stopping '{project_name}'..."));
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let project = project_name.clone();
    let result = cx
      .background_executor()
      .spawn(async move {
        let output = docker_cmd().args(["compose", "-p", &project, "down"]).output();

        match output {
          Ok(out) if out.status.success() => Ok(()),
          Ok(out) => Err(String::from_utf8_lossy(&out.stderr).to_string()),
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(()) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Stopped '{project_name}'"),
          });
        });
        refresh_containers(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to stop '{project_name}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn compose_restart(project_name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Restarting '{project_name}'..."));
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let project = project_name.clone();
    let result = cx
      .background_executor()
      .spawn(async move {
        let output = docker_cmd().args(["compose", "-p", &project, "restart"]).output();

        match output {
          Ok(out) if out.status.success() => Ok(()),
          Ok(out) => Err(String::from_utf8_lossy(&out.stderr).to_string()),
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(()) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Restarted '{project_name}'"),
          });
        });
        refresh_containers(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to restart '{project_name}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}
