//! View and tab navigation functions

use gpui::App;

use crate::state::{CurrentView, StateChanged, docker_state};

/// Set the current view
pub fn set_view(view: CurrentView, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |state, cx| {
    state.set_view(view);
    cx.emit(StateChanged::ViewChanged);
  });
}

// ==================== Container Tab Navigation ====================

/// Open a container's terminal tab
pub fn open_container_terminal(id: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::ContainerTabRequest {
      container_id: id,
      tab: 2, // Terminal is tab 2
    });
  });
}

/// Open a container's logs tab
pub fn open_container_logs(id: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::ContainerTabRequest {
      container_id: id,
      tab: 1, // Logs is tab 1
    });
  });
}

/// Open a container's inspect tab
pub fn open_container_inspect(id: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::ContainerTabRequest {
      container_id: id,
      tab: 4, // Inspect is tab 4
    });
  });
}

/// Open a container's files tab
pub fn open_container_files(id: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::ContainerTabRequest {
      container_id: id,
      tab: 3, // Files is tab 3
    });
  });
}

// ==================== Machine Tab Navigation ====================

/// Open a machine's terminal tab
pub fn open_machine_terminal(name: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::MachineTabRequest {
      machine_name: name,
      tab: 2, // Terminal is tab 2
    });
  });
}

/// Open a machine's files tab
pub fn open_machine_files(name: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::MachineTabRequest {
      machine_name: name,
      tab: 3, // Files is tab 3
    });
  });
}

// ==================== Pod Tab Navigation ====================

/// Open a pod's info tab
pub fn open_pod_info(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |state, cx| {
    state.set_view(CurrentView::Pods);
    cx.emit(StateChanged::ViewChanged);
    cx.emit(StateChanged::PodTabRequest {
      pod_name: name,
      namespace,
      tab: 0, // Info is tab 0
    });
  });
}

/// Open a pod's terminal tab
pub fn open_pod_terminal(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::PodTabRequest {
      pod_name: name,
      namespace,
      tab: 2, // Terminal is tab 2
    });
  });
}

/// Open a pod's logs tab
pub fn open_pod_logs(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::PodTabRequest {
      pod_name: name,
      namespace,
      tab: 1, // Logs is tab 1
    });
  });
}

/// Open a pod's describe tab
pub fn open_pod_describe(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::PodTabRequest {
      pod_name: name,
      namespace,
      tab: 3, // Describe is tab 3
    });
  });
}

/// Open a pod's YAML tab
pub fn open_pod_yaml(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::PodTabRequest {
      pod_name: name,
      namespace,
      tab: 4, // YAML is tab 4
    });
  });
}

// ==================== Service Tab Navigation ====================

/// Open service with YAML tab selected
pub fn open_service_yaml(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::ServiceTabRequest {
      service_name: name.clone(),
      namespace: namespace.clone(),
      tab: 2, // YAML tab
    });
  });
  // Also trigger the YAML fetch
  super::kubernetes::get_service_yaml(name, namespace, cx);
}

// ==================== Deployment Tab Navigation ====================

/// Open deployment with YAML tab selected
pub fn open_deployment_yaml(name: String, namespace: String, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::DeploymentTabRequest {
      deployment_name: name.clone(),
      namespace: namespace.clone(),
      tab: 2, // YAML tab
    });
  });
  // Also trigger the YAML fetch
  super::kubernetes::get_deployment_yaml(name, namespace, cx);
}

/// Request to open scale dialog for a deployment
pub fn request_scale_dialog(name: String, namespace: String, current_replicas: i32, cx: &mut App) {
  let state = docker_state(cx);
  state.update(cx, |_state, cx| {
    cx.emit(StateChanged::DeploymentScaleRequest {
      deployment_name: name,
      namespace,
      current_replicas,
    });
  });
}
