//! Container operations

use gpui::App;

use crate::docker::{ContainerCreateConfig, ContainerFlags};
use crate::services::{Tokio, complete_task, fail_task, start_task};
use crate::state::{StateChanged, docker_state};

use super::super::core::{DispatcherEvent, dispatcher, docker_client};

pub fn start_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Starting container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.start_container(&id).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container started".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to start container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn stop_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Stopping container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.stop_container(&id).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container stopped".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to stop container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn restart_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Restarting container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.restart_container(&id).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container restarted".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to restart container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn delete_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Deleting container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.remove_container(&id, true).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container deleted".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to delete container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn pause_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Pausing container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.pause_container(&id).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container paused".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to pause container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn unpause_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Resuming container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.unpause_container(&id).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container resumed".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to resume container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn kill_container(id: String, cx: &mut App) {
  let task_id = start_task(cx, "Killing container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.kill_container(&id, None).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container killed".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to kill container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn rename_container(id: String, new_name: String, cx: &mut App) {
  let task_id = start_task(cx, "Renaming container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.rename_container(&id, &new_name).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container renamed".to_string(),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to rename container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn commit_container(
  id: String,
  repo: String,
  tag: String,
  comment: Option<String>,
  author: Option<String>,
  cx: &mut App,
) {
  let task_id = start_task(cx, "Committing container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker
      .commit_container(&id, &repo, &tag, comment.as_deref(), author.as_deref())
      .await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(image_id)) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Container committed as image: {}", &image_id[..12.min(image_id.len())]),
          });
        });
        super::images::refresh_images(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to commit container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

pub fn export_container(id: String, output_path: String, cx: &mut App) {
  let task_id = start_task(cx, "Exporting container...".to_string());
  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;
    docker.export_container(&id, &output_path).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: "Container exported".to_string(),
          });
        });
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to export container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}

/// Request to open rename dialog for a container
pub fn request_rename_container(id: String, current_name: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::RenameContainerRequest {
      container_id: id,
      current_name,
    });
  });
}

/// Request to open commit dialog for a container
pub fn request_commit_container(id: String, container_name: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::CommitContainerRequest {
      container_id: id,
      container_name,
    });
  });
}

/// Request to open export dialog for a container
pub fn request_export_container(id: String, container_name: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::ExportContainerRequest {
      container_id: id,
      container_name,
    });
  });
}

pub fn refresh_containers(cx: &mut App) {
  let state = docker_state(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    match guard.as_ref() {
      Some(docker) => docker.list_containers(true).await.unwrap_or_default(),
      None => vec![],
    }
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    let containers = result.unwrap_or_default();
    cx.update(|cx| {
      state.update(cx, |state, cx| {
        state.set_containers(containers);
        cx.emit(StateChanged::ContainersUpdated);
      });
    })
  })
  .detach();
}

pub fn create_container(options: crate::ui::containers::CreateContainerOptions, cx: &mut App) {
  let image_name = options.image.clone();
  let start_after = options.start_after_create;
  let task_id = start_task(cx, format!("Creating container from {image_name}..."));

  let disp = dispatcher(cx);
  let client = docker_client();

  let tokio_task = Tokio::spawn(cx, async move {
    let guard = client.read().await;
    let docker = guard
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("Docker client not connected"))?;

    // Ensure image exists locally, pull if necessary
    docker
      .ensure_image(&options.image, options.platform.as_docker_arg())
      .await?;

    // Parse command and entrypoint if provided
    let command: Option<Vec<String>> = options
      .command
      .as_ref()
      .map(|c| c.split_whitespace().map(String::from).collect());
    let entrypoint: Option<Vec<String>> = options
      .entrypoint
      .as_ref()
      .map(|e| e.split_whitespace().map(String::from).collect());

    let config = ContainerCreateConfig {
      image: options.image,
      name: options.name,
      platform: options.platform.as_docker_arg().map(String::from),
      command,
      entrypoint,
      working_dir: options.workdir,
      restart_policy: options.restart_policy.as_docker_arg().map(String::from),
      flags: ContainerFlags {
        auto_remove: options.remove_after_stop,
        privileged: options.privileged,
        read_only: options.read_only,
        init: options.docker_init,
      },
      env_vars: options.env_vars,
      ports: options.ports,
      volumes: options.volumes,
      network: options.network,
    };

    let container_id = docker.create_container(config).await?;

    // Start the container if requested
    if start_after {
      docker.start_container(&container_id).await?;
    }

    Ok::<_, anyhow::Error>(())
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    cx.update(|cx| match result {
      Ok(Ok(())) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Container created from {image_name}"),
          });
        });
        refresh_containers(cx);
      }
      Ok(Err(e)) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to create container: {e}"),
          });
        });
      }
      Err(join_err) => {
        fail_task(cx, task_id, join_err.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Task failed: {join_err}"),
          });
        });
      }
    })
  })
  .detach();
}
