//! Colima machine operations

use gpui::App;

use crate::colima::{ColimaClient, ColimaStartOptions};
use crate::services::{TaskStage, advance_stage, complete_task, fail_task, start_staged_task, start_task};
use crate::state::{StateChanged, docker_state};
use crate::utils::{docker_cmd, kubectl_cmd};

use super::super::core::{DispatcherEvent, dispatcher};
use super::super::docker::refresh_containers;
use super::super::kubernetes::{refresh_deployments, refresh_namespaces, refresh_pods, refresh_services};

pub fn create_machine(options: ColimaStartOptions, cx: &mut App) {
  let machine_name = options.name.clone().unwrap_or_else(|| "default".to_string());
  let has_kubernetes = options.kubernetes;

  // Create staged task with clear progress stages
  let mut stages = vec![
    TaskStage::new("download", "Downloading VM image..."),
    TaskStage::new("create", format!("Creating VM '{machine_name}'...")),
    TaskStage::new("configure", "Configuring runtime..."),
  ];

  if has_kubernetes {
    stages.push(TaskStage::new("kubernetes", "Setting up Kubernetes..."));
  }

  stages.push(TaskStage::new("verify", "Verifying machine..."));

  let task_id = start_staged_task(cx, format!("Creating '{machine_name}'"), stages);
  let name_clone = machine_name.clone();
  let name_for_context = machine_name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        match ColimaClient::start(&options) {
          Ok(()) => {
            let vms = ColimaClient::list().unwrap_or_default();

            // If kubernetes is enabled, switch kubectl context
            if has_kubernetes {
              let kubectl_context = if machine_name == "default" {
                "colima".to_string()
              } else {
                format!("colima-{machine_name}")
              };
              // Try to switch kubectl context (don't fail if it doesn't work)
              let _ = kubectl_cmd().args(["config", "use-context", &kubectl_context]).output();
            }

            Ok(vms)
          }
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(vms) => {
        state.update(cx, |state, cx| {
          state.set_machines(vms);
          cx.emit(StateChanged::MachinesUpdated);
        });
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Machine '{name_clone}' created"),
          });
        });
        // Refresh K8s data if kubernetes was enabled
        if has_kubernetes {
          refresh_pods(cx);
          refresh_namespaces(cx);
          refresh_services(cx);
          refresh_deployments(cx);
        }
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to create '{name_for_context}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn start_machine(name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Starting '{name}'..."));
  let name_clone = name.clone();
  let name_for_context = name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        let options = ColimaStartOptions::new().with_name(name.clone());
        match ColimaClient::start(&options) {
          Ok(()) => {
            let vms = ColimaClient::list().unwrap_or_default();
            // Check if the started machine has kubernetes enabled
            let has_k8s = vms.iter().any(|vm| vm.name == name && vm.kubernetes);

            // If kubernetes is enabled, switch kubectl context
            if has_k8s {
              let kubectl_context = if name == "default" {
                "colima".to_string()
              } else {
                format!("colima-{name}")
              };
              // Try to switch kubectl context (don't fail if it doesn't work)
              let _ = kubectl_cmd().args(["config", "use-context", &kubectl_context]).output();
            }

            Ok((vms, has_k8s))
          }
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok((vms, has_k8s)) => {
        state.update(cx, |state, cx| {
          state.set_machines(vms);
          cx.emit(StateChanged::MachinesUpdated);
        });
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Machine '{name_clone}' started"),
          });
        });
        // Refresh K8s data if kubernetes is enabled
        if has_k8s {
          refresh_pods(cx);
          refresh_namespaces(cx);
          refresh_services(cx);
          refresh_deployments(cx);
        }
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to start '{name_for_context}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn stop_machine(name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Stopping '{name}'..."));
  let name_clone = name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        let name_opt = if name == "default" { None } else { Some(name.as_str()) };
        match ColimaClient::stop(name_opt) {
          Ok(()) => Ok(ColimaClient::list().unwrap_or_default()),
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(vms) => {
        state.update(cx, |state, cx| {
          state.set_machines(vms);
          cx.emit(StateChanged::MachinesUpdated);
        });
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Machine '{name_clone}' stopped"),
          });
        });
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to stop '{name_clone}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn edit_machine(options: ColimaStartOptions, cx: &mut App) {
  let name = options.name.clone().unwrap_or_else(|| "default".to_string());

  // Create staged task with clear progress stages
  let stages = vec![
    TaskStage::new("stop", format!("Stopping '{name}'...")),
    TaskStage::new("configure", "Applying new settings..."),
    TaskStage::new("start", format!("Starting '{name}' with new configuration...")),
    TaskStage::new("verify", format!("Verifying '{name}'...")),
  ];

  let task_id = start_staged_task(cx, format!("Updating '{name}'"), stages);
  let name_clone = name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  // Do NOT use --edit flag - it opens an interactive editor which hangs in subprocess
  // Just pass the new configuration directly to colima start

  cx.spawn(async move |cx| {
    // Stage 0: Stop the machine
    let stop_result = cx
      .background_executor()
      .spawn({
        let name = name.clone();
        async move {
          let name_opt = if name == "default" { None } else { Some(name.as_str()) };
          ColimaClient::stop(name_opt)
        }
      })
      .await;

    if let Err(e) = stop_result {
      cx.update(|cx| {
        fail_task(cx, task_id, format!("Failed to stop: {e}"));
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to stop '{name_clone}': {e}"),
          });
        });
      })
      .ok();
      return;
    }

    // Stage 1: Configuration applied (just advancing stage for UI feedback)
    cx.update(|cx| advance_stage(cx, task_id)).ok();

    // Brief pause to let Colima release resources
    cx.background_executor()
      .timer(std::time::Duration::from_millis(500))
      .await;

    // Stage 2: Start with new options
    cx.update(|cx| advance_stage(cx, task_id)).ok();

    let start_result = cx
      .background_executor()
      .spawn({
        let options = options.clone();
        async move { ColimaClient::start(&options) }
      })
      .await;

    if let Err(e) = start_result {
      cx.update(|cx| {
        fail_task(cx, task_id, format!("Failed to start: {e}"));
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to start '{name_clone}' with new settings: {e}"),
          });
        });
      })
      .ok();
      return;
    }

    // Stage 3: Verify and refresh list
    cx.update(|cx| advance_stage(cx, task_id)).ok();

    let has_kubernetes = options.kubernetes;
    let name_for_context = name.clone();
    let vms = cx
      .background_executor()
      .spawn(async move {
        let vms = ColimaClient::list().unwrap_or_default();

        // If kubernetes was enabled, switch kubectl context
        if has_kubernetes {
          let kubectl_context = if name_for_context == "default" {
            "colima".to_string()
          } else {
            format!("colima-{name_for_context}")
          };
          // Try to switch kubectl context (don't fail if it doesn't work)
          let _ = kubectl_cmd().args(["config", "use-context", &kubectl_context]).output();
        }

        vms
      })
      .await;

    cx.update(|cx| {
      state.update(cx, |state, cx| {
        state.set_machines(vms);
        cx.emit(StateChanged::MachinesUpdated);
      });
      complete_task(cx, task_id);
      disp.update(cx, |_, cx| {
        cx.emit(DispatcherEvent::TaskCompleted {
          message: format!("Machine '{name_clone}' updated successfully"),
        });
      });
      // Refresh K8s data if kubernetes was enabled
      if has_kubernetes {
        refresh_pods(cx);
        refresh_namespaces(cx);
        refresh_services(cx);
        refresh_deployments(cx);
      }
    })
    .ok();
  })
  .detach();
}

pub fn restart_machine(name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Restarting '{name}'..."));
  let name_clone = name.clone();
  let name_for_context = name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        let name_opt = if name == "default" { None } else { Some(name.as_str()) };
        match ColimaClient::restart(name_opt) {
          Ok(()) => {
            let vms = ColimaClient::list().unwrap_or_default();
            // Check if the restarted machine has kubernetes enabled
            let has_k8s = vms.iter().any(|vm| vm.name == name && vm.kubernetes);

            // If kubernetes is enabled, switch kubectl context
            if has_k8s {
              let kubectl_context = if name == "default" {
                "colima".to_string()
              } else {
                format!("colima-{name}")
              };
              // Try to switch kubectl context (don't fail if it doesn't work)
              let _ = kubectl_cmd().args(["config", "use-context", &kubectl_context]).output();
            }

            Ok((vms, has_k8s))
          }
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok((vms, has_k8s)) => {
        state.update(cx, |state, cx| {
          state.set_machines(vms);
          cx.emit(StateChanged::MachinesUpdated);
        });
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Machine '{name_clone}' restarted"),
          });
        });
        // Refresh K8s data if kubernetes is enabled
        if has_k8s {
          refresh_pods(cx);
          refresh_namespaces(cx);
          refresh_services(cx);
          refresh_deployments(cx);
        }
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to restart '{name_for_context}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

/// Start Colima with optional profile name (None = default profile)
pub fn start_colima(profile: Option<&str>, cx: &mut App) {
  let name = profile.unwrap_or("default").to_string();
  start_machine(name, cx);
}

/// Stop Colima with optional profile name (None = default profile)
pub fn stop_colima(profile: Option<&str>, cx: &mut App) {
  let name = profile.unwrap_or("default").to_string();
  stop_machine(name, cx);
}

/// Restart Colima with optional profile name (None = default profile)
pub fn restart_colima(profile: Option<&str>, cx: &mut App) {
  let name = profile.unwrap_or("default").to_string();
  let task_id = start_task(cx, format!("Restarting '{name}'..."));
  let name_clone = name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        let name_opt = if name == "default" { None } else { Some(name.as_str()) };
        match ColimaClient::restart(name_opt) {
          Ok(()) => Ok(ColimaClient::list().unwrap_or_default()),
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(vms) => {
        state.update(cx, |state, cx| {
          state.set_machines(vms);
          cx.emit(StateChanged::MachinesUpdated);
        });
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Machine '{name_clone}' restarted"),
          });
        });
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to restart '{name_clone}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn delete_machine(name: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Deleting '{name}'..."));
  let name_clone = name.clone();

  let state = docker_state(cx);
  let disp = dispatcher(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        let name_opt = if name == "default" { None } else { Some(name.as_str()) };
        match ColimaClient::delete(name_opt, true) {
          Ok(()) => Ok(ColimaClient::list().unwrap_or_default()),
          Err(e) => Err(e.to_string()),
        }
      })
      .await;

    cx.update(|cx| match result {
      Ok(vms) => {
        state.update(cx, |state, cx| {
          state.set_machines(vms);
          cx.emit(StateChanged::MachinesUpdated);
        });
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Machine '{name_clone}' deleted"),
          });
        });
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to delete '{name_clone}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

/// Refresh the list of Colima machines
pub fn refresh_machines(cx: &mut App) {
  let state = docker_state(cx);

  let task = cx
    .background_executor()
    .spawn(async move { ColimaClient::list().unwrap_or_default() });

  cx.spawn(async move |cx| {
    let vms = task.await;
    cx.update(|cx| {
      state.update(cx, |state, cx| {
        state.set_machines(vms);
        cx.emit(StateChanged::MachinesUpdated);
      });
    })
  })
  .detach();
}

/// Set a machine as the default by switching docker and k8s contexts
pub fn set_default_machine(name: String, has_kubernetes: bool, cx: &mut App) {
  let task_id = start_task(cx, format!("Setting '{name}' as default..."));

  let disp = dispatcher(cx);
  let state = docker_state(cx);

  cx.spawn(async move |cx| {
    let result = cx
      .background_executor()
      .spawn(async move {
        // Docker context name for colima is "colima" for default or "colima-<profile>" for others
        let context_name = if name == "default" {
          "colima".to_string()
        } else {
          format!("colima-{name}")
        };

        // Switch docker context
        let docker_output = docker_cmd().args(["context", "use", &context_name]).output();

        match &docker_output {
          Err(e) => return Err(format!("Failed to switch docker context: {e}")),
          Ok(out) if !out.status.success() => {
            return Err(format!(
              "Failed to switch docker context: {}",
              String::from_utf8_lossy(&out.stderr)
            ));
          }
          Ok(_) => {}
        }

        // If machine has kubernetes, switch kubectl context
        if has_kubernetes {
          // kubectl context for colima is "colima" for default or "colima-<profile>" for others
          let kubectl_context = context_name.clone();

          // k8s context switch is optional - don't fail if kubectl isn't available
          let _ = kubectl_cmd().args(["config", "use-context", &kubectl_context]).output();
        }

        Ok((context_name, has_kubernetes))
      })
      .await;

    cx.update(|cx| match result {
      Ok((context_name, switched_k8s)) => {
        complete_task(cx, task_id);

        let msg = if switched_k8s {
          format!("'{context_name}' is now the default (Docker + Kubernetes)")
        } else {
          format!("'{context_name}' is now the default")
        };

        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted { message: msg });
        });

        // Refresh data to reflect new context
        refresh_containers(cx);
        if switched_k8s {
          refresh_pods(cx);
          refresh_namespaces(cx);
          refresh_services(cx);
          refresh_deployments(cx);
        }

        // Notify that default machine changed
        state.update(cx, |_, cx| {
          cx.emit(StateChanged::MachinesUpdated);
        });
      }
      Err(e) => {
        fail_task(cx, task_id, e.clone());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to set default: {e}"),
          });
        });
      }
    })
  })
  .detach();
}
