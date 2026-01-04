//! Initial data loading

use gpui::App;

use crate::colima::ColimaClient;
use crate::docker::DockerClient;
use crate::services::Tokio;
use crate::state::{StateChanged, docker_state, settings_state};

use super::core::docker_client;

pub fn load_initial_data(cx: &mut App) {
  let state = docker_state(cx);
  let client_handle = docker_client();

  // Get saved settings for Docker socket and Colima profile
  let settings = settings_state(cx).read(cx).settings.clone();
  let custom_socket = settings.docker_socket.clone();
  let colima_profile = settings.default_colima_profile.clone();

  // First, get colima VMs and socket path (sync operation)
  let colima_task = cx.background_executor().spawn(async move {
    let vms = ColimaClient::list().unwrap_or_default();

    // Use custom socket if provided, otherwise use colima socket with configured profile
    let socket_path = if custom_socket.is_empty() {
      let profile = if colima_profile == "default" {
        None
      } else {
        Some(colima_profile.as_str())
      };
      ColimaClient::socket_path(profile)
    } else {
      custom_socket
    };
    (vms, socket_path)
  });

  // Then spawn tokio task for Docker operations
  let tokio_task = Tokio::spawn(cx, async move {
    // Wait for colima info
    let (vms, socket_path) = colima_task.await;

    // Initialize the shared Docker client
    let mut new_client = DockerClient::new(socket_path);
    let docker_connected = new_client.connect().await.is_ok();

    // Store in the global if connected
    if docker_connected {
      let mut guard = client_handle.write().await;
      *guard = Some(new_client);
      drop(guard);

      // Now use the shared client for all queries
      let guard = client_handle.read().await;
      let docker = guard.as_ref().unwrap();

      let containers = docker.list_containers(true).await.unwrap_or_default();
      let images = docker.list_images(false).await.unwrap_or_default();
      let volumes = docker.list_volumes().await.unwrap_or_default();
      let networks = docker.list_networks().await.unwrap_or_default();

      (vms, containers, images, volumes, networks)
    } else {
      (vms, vec![], vec![], vec![], vec![])
    }
  });

  cx.spawn(async move |cx| {
    let result = tokio_task.await;
    let (vms, containers, images, volumes, networks) = result.unwrap_or_default();

    cx.update(|cx| {
      state.update(cx, |state, cx| {
        state.set_machines(vms);
        state.set_containers(containers);
        state.set_images(images);
        state.set_volumes(volumes);
        state.set_networks(networks);
        state.is_loading = false;
        cx.emit(StateChanged::MachinesUpdated);
        cx.emit(StateChanged::ContainersUpdated);
        cx.emit(StateChanged::ImagesUpdated);
        cx.emit(StateChanged::VolumesUpdated);
        cx.emit(StateChanged::NetworksUpdated);
        cx.emit(StateChanged::Loading);
      });
    })
  })
  .detach();
}
