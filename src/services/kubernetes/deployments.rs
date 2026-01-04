//! Kubernetes deployment operations

use gpui::App;

use crate::services::{Tokio, complete_task, fail_task, start_task};
use crate::state::{StateChanged, docker_state};

use super::super::core::{DispatcherEvent, dispatcher};
use super::pods::refresh_pods;

/// Refresh deployments list
pub fn refresh_deployments(cx: &mut App) {
  let state = docker_state(cx);
  let selected_ns = state.read(cx).selected_namespace.clone();
  let namespace = if selected_ns == "all" { None } else { Some(selected_ns) };

  let tokio_task = Tokio::spawn(cx, async move {
    let client = crate::kubernetes::KubeClient::new().await?;
    client.list_deployments(namespace.as_deref()).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await.unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));

    cx.update(|cx| {
      if let Ok(deployments) = result {
        state.update(cx, |state, cx| {
          state.set_deployments(deployments);
          cx.emit(StateChanged::DeploymentsUpdated);
        });
      }
    })
  })
  .detach();
}

/// Delete a deployment
pub fn delete_deployment(name: String, namespace: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Deleting deployment '{name}'..."));
  let name_clone = name.clone();
  let _state = docker_state(cx);
  let disp = dispatcher(cx);

  let tokio_task = Tokio::spawn(cx, async move {
    let client = crate::kubernetes::KubeClient::new().await?;
    client.delete_deployment(&name, &namespace).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await.unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));

    cx.update(|cx| match result {
      Ok(()) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted {
            message: format!("Deployment '{name_clone}' deleted"),
          });
        });
        refresh_deployments(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to delete deployment '{name_clone}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

/// Scale a deployment
pub fn scale_deployment(name: String, namespace: String, replicas: i32, cx: &mut App) {
  let task_id = start_task(cx, format!("Scaling '{name}' to {replicas} replicas..."));
  let name_clone = name.clone();
  let disp = dispatcher(cx);

  let tokio_task = Tokio::spawn(cx, async move {
    let client = crate::kubernetes::KubeClient::new().await?;
    client.scale_deployment(&name, &namespace, replicas).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await.unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));

    cx.update(|cx| match result {
      Ok(msg) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted { message: msg });
        });
        refresh_deployments(cx);
        refresh_pods(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to scale '{name_clone}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}

/// Restart a deployment (rollout restart)
pub fn restart_deployment(name: String, namespace: String, cx: &mut App) {
  let task_id = start_task(cx, format!("Restarting '{name}'..."));
  let name_clone = name.clone();
  let disp = dispatcher(cx);

  let tokio_task = Tokio::spawn(cx, async move {
    let client = crate::kubernetes::KubeClient::new().await?;
    client.restart_deployment(&name, &namespace).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await.unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));

    cx.update(|cx| match result {
      Ok(msg) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted { message: msg });
        });
        refresh_deployments(cx);
        refresh_pods(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.to_string());
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

/// Get deployment YAML
pub fn get_deployment_yaml(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  let name_clone = name.clone();
  let namespace_clone = namespace.clone();

  let tokio_task = Tokio::spawn(cx, async move {
    let client = crate::kubernetes::KubeClient::new().await?;
    client.get_deployment_yaml(&name, &namespace).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await.unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));
    let yaml = match result {
      Ok(y) => y,
      Err(e) => format!("Error: {e}"),
    };

    cx.update(|cx| {
      state.update(cx, |_state, cx| {
        cx.emit(StateChanged::DeploymentYamlLoaded {
          deployment_name: name_clone,
          namespace: namespace_clone,
          yaml,
        });
      });
    })
  })
  .detach();
}

/// Create a new Kubernetes deployment
pub fn create_deployment(options: crate::kubernetes::CreateDeploymentOptions, cx: &mut App) {
  let task_id = start_task(cx, format!("Creating deployment '{}'...", options.name));
  let name = options.name.clone();
  let disp = dispatcher(cx);

  let tokio_task = Tokio::spawn(cx, async move {
    let client = crate::kubernetes::KubeClient::new().await?;
    client.create_deployment(options).await
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await.unwrap_or_else(|e| Err(anyhow::anyhow!("{e}")));

    cx.update(|cx| match result {
      Ok(msg) => {
        complete_task(cx, task_id);
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskCompleted { message: msg });
        });
        refresh_deployments(cx);
        refresh_pods(cx);
      }
      Err(e) => {
        fail_task(cx, task_id, e.to_string());
        disp.update(cx, |_, cx| {
          cx.emit(DispatcherEvent::TaskFailed {
            error: format!("Failed to create deployment '{name}': {e}"),
          });
        });
      }
    })
  })
  .detach();
}
