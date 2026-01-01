use gpui::{App, AppContext, Entity, EventEmitter, Global};

use crate::colima::{ColimaClient, ColimaStartOptions};
use crate::docker::DockerClient;
use crate::state::{docker_state, StateChanged, CurrentView};
use crate::services::{complete_task, fail_task, start_task};

/// Event emitted when a task completes (for UI to show notifications)
#[derive(Clone, Debug)]
pub enum DispatcherEvent {
    TaskCompleted { name: String, message: String },
    TaskFailed { name: String, error: String },
}

/// Central action dispatcher - handles all async operations
pub struct ActionDispatcher {
    pub show_create_dialog: bool,
}

impl ActionDispatcher {
    pub fn new() -> Self {
        Self {
            show_create_dialog: false,
        }
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

// ==================== Action Handlers ====================
// These are standalone functions that can be called from anywhere

pub fn create_machine(options: ColimaStartOptions, cx: &mut App) {
    let machine_name = options.name.clone().unwrap_or_else(|| "default".to_string());
    let task_id = start_task(cx, "create_machine", format!("Creating '{}'...", machine_name));
    let name_clone = machine_name.clone();

    let state = docker_state(cx);
    let disp = dispatcher(cx);

    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                match colima_client.start(options) {
                    Ok(_) => Ok(colima_client.list().unwrap_or_default()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;

        cx.update(|cx| {
            match result {
                Ok(vms) => {
                    state.update(cx, |state, cx| {
                        state.set_machines(vms);
                        cx.emit(StateChanged::MachinesUpdated);
                    });
                    complete_task(cx, task_id);
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskCompleted {
                            name: "create_machine".to_string(),
                            message: format!("Machine '{}' created", name_clone),
                        });
                    });
                }
                Err(e) => {
                    fail_task(cx, task_id, e.clone());
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskFailed {
                            name: "create_machine".to_string(),
                            error: format!("Failed to create '{}': {}", name_clone, e),
                        });
                    });
                }
            }
        })
    })
    .detach();
}

pub fn start_machine(name: String, cx: &mut App) {
    let task_id = start_task(cx, "start_machine", format!("Starting '{}'...", name));
    let name_clone = name.clone();

    let state = docker_state(cx);
    let disp = dispatcher(cx);

    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                let options = ColimaStartOptions::new().with_name(name.clone());
                match colima_client.start(options) {
                    Ok(_) => Ok(colima_client.list().unwrap_or_default()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;

        cx.update(|cx| {
            match result {
                Ok(vms) => {
                    state.update(cx, |state, cx| {
                        state.set_machines(vms);
                        cx.emit(StateChanged::MachinesUpdated);
                    });
                    complete_task(cx, task_id);
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskCompleted {
                            name: "start_machine".to_string(),
                            message: format!("Machine '{}' started", name_clone),
                        });
                    });
                }
                Err(e) => {
                    fail_task(cx, task_id, e.clone());
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskFailed {
                            name: "start_machine".to_string(),
                            error: format!("Failed to start '{}': {}", name_clone, e),
                        });
                    });
                }
            }
        })
    })
    .detach();
}

pub fn stop_machine(name: String, cx: &mut App) {
    let task_id = start_task(cx, "stop_machine", format!("Stopping '{}'...", name));
    let name_clone = name.clone();

    let state = docker_state(cx);
    let disp = dispatcher(cx);

    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                let name_opt = if name == "default" { None } else { Some(name.as_str()) };
                match colima_client.stop(name_opt) {
                    Ok(_) => Ok(colima_client.list().unwrap_or_default()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;

        cx.update(|cx| {
            match result {
                Ok(vms) => {
                    state.update(cx, |state, cx| {
                        state.set_machines(vms);
                        cx.emit(StateChanged::MachinesUpdated);
                    });
                    complete_task(cx, task_id);
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskCompleted {
                            name: "stop_machine".to_string(),
                            message: format!("Machine '{}' stopped", name_clone),
                        });
                    });
                }
                Err(e) => {
                    fail_task(cx, task_id, e.clone());
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskFailed {
                            name: "stop_machine".to_string(),
                            error: format!("Failed to stop '{}': {}", name_clone, e),
                        });
                    });
                }
            }
        })
    })
    .detach();
}

pub fn restart_machine(name: String, cx: &mut App) {
    let task_id = start_task(cx, "restart_machine", format!("Restarting '{}'...", name));
    let name_clone = name.clone();

    let state = docker_state(cx);
    let disp = dispatcher(cx);

    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                let name_opt = if name == "default" { None } else { Some(name.as_str()) };
                match colima_client.restart(name_opt) {
                    Ok(_) => Ok(colima_client.list().unwrap_or_default()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;

        cx.update(|cx| {
            match result {
                Ok(vms) => {
                    state.update(cx, |state, cx| {
                        state.set_machines(vms);
                        cx.emit(StateChanged::MachinesUpdated);
                    });
                    complete_task(cx, task_id);
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskCompleted {
                            name: "restart_machine".to_string(),
                            message: format!("Machine '{}' restarted", name_clone),
                        });
                    });
                }
                Err(e) => {
                    fail_task(cx, task_id, e.clone());
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskFailed {
                            name: "restart_machine".to_string(),
                            error: format!("Failed to restart '{}': {}", name_clone, e),
                        });
                    });
                }
            }
        })
    })
    .detach();
}

pub fn delete_machine(name: String, cx: &mut App) {
    let task_id = start_task(cx, "delete_machine", format!("Deleting '{}'...", name));
    let name_clone = name.clone();

    let state = docker_state(cx);
    let disp = dispatcher(cx);

    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                let name_opt = if name == "default" { None } else { Some(name.as_str()) };
                match colima_client.delete(name_opt, true) {
                    Ok(_) => Ok(colima_client.list().unwrap_or_default()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;

        cx.update(|cx| {
            match result {
                Ok(vms) => {
                    state.update(cx, |state, cx| {
                        state.set_machines(vms);
                        state.clear_selection();
                        cx.emit(StateChanged::MachinesUpdated);
                        cx.emit(StateChanged::SelectionChanged);
                    });
                    complete_task(cx, task_id);
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskCompleted {
                            name: "delete_machine".to_string(),
                            message: format!("Machine '{}' deleted", name_clone),
                        });
                    });
                }
                Err(e) => {
                    fail_task(cx, task_id, e.clone());
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskFailed {
                            name: "delete_machine".to_string(),
                            error: format!("Failed to delete '{}': {}", name_clone, e),
                        });
                    });
                }
            }
        })
    })
    .detach();
}

pub fn refresh_machines(cx: &mut App) {
    let state = docker_state(cx);

    cx.spawn(async move |cx| {
        let vms = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                colima_client.list().unwrap_or_default()
            })
            .await;

        cx.update(|cx| {
            state.update(cx, |state, cx| {
                state.set_machines(vms);
                cx.emit(StateChanged::MachinesUpdated);
            });
        })
    })
    .detach();
}

pub fn select_machine(name: String, cx: &mut App) {
    let state = docker_state(cx);
    state.update(cx, |state, cx| {
        state.select_machine(&name);
        cx.emit(StateChanged::SelectionChanged);
    });
}

pub fn set_view(view: CurrentView, cx: &mut App) {
    let state = docker_state(cx);
    state.update(cx, |state, cx| {
        state.set_view(view);
        cx.emit(StateChanged::ViewChanged);
    });
}

pub fn show_create_dialog(cx: &mut App) {
    let disp = dispatcher(cx);
    disp.update(cx, |d, _| {
        d.show_create_dialog = true;
    });
}

pub fn hide_create_dialog(cx: &mut App) {
    let disp = dispatcher(cx);
    disp.update(cx, |d, _| {
        d.show_create_dialog = false;
    });
}

pub fn set_docker_context(name: String, cx: &mut App) {
    let task_id = start_task(cx, "set_context", format!("Switching to '{}'...", name));

    let disp = dispatcher(cx);

    cx.spawn(async move |cx| {
        let result = cx
            .background_executor()
            .spawn(async move {
                use std::process::Command;
                // Docker context name for colima is "colima" for default or "colima-<profile>" for others
                let context_name = if name == "default" {
                    "colima".to_string()
                } else {
                    format!("colima-{}", name)
                };

                let output = Command::new("docker")
                    .args(["context", "use", &context_name])
                    .output();

                match output {
                    Ok(out) if out.status.success() => Ok(context_name),
                    Ok(out) => Err(String::from_utf8_lossy(&out.stderr).to_string()),
                    Err(e) => Err(e.to_string()),
                }
            })
            .await;

        cx.update(|cx| {
            match result {
                Ok(context_name) => {
                    complete_task(cx, task_id);
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskCompleted {
                            name: "set_context".to_string(),
                            message: format!("Docker context switched to '{}'", context_name),
                        });
                    });
                }
                Err(e) => {
                    fail_task(cx, task_id, e.clone());
                    disp.update(cx, |_, cx| {
                        cx.emit(DispatcherEvent::TaskFailed {
                            name: "set_context".to_string(),
                            error: format!("Failed to switch context: {}", e),
                        });
                    });
                }
            }
        })
    })
    .detach();
}

// ==================== Initial Data Loading ====================

pub fn load_initial_data(cx: &mut App) {
    let state = docker_state(cx);

    cx.spawn(async move |cx| {
        let (vms, containers, images, volumes, networks) = cx
            .background_executor()
            .spawn(async move {
                let colima_client = ColimaClient::new();
                let vms = colima_client.list().unwrap_or_default();
                let socket_path = colima_client.socket_path(None);

                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("Failed to create tokio runtime");

                let mut docker_client = DockerClient::new(socket_path);
                let docker_connected = rt.block_on(async {
                    docker_client.connect().await.is_ok()
                });

                let containers = if docker_connected {
                    rt.block_on(async { docker_client.list_containers(true).await.unwrap_or_default() })
                } else {
                    vec![]
                };

                let images = if docker_connected {
                    rt.block_on(async { docker_client.list_images(false).await.unwrap_or_default() })
                } else {
                    vec![]
                };

                let volumes = if docker_connected {
                    rt.block_on(async { docker_client.list_volumes().await.unwrap_or_default() })
                } else {
                    vec![]
                };

                let networks = if docker_connected {
                    rt.block_on(async { docker_client.list_networks().await.unwrap_or_default() })
                } else {
                    vec![]
                };

                (vms, containers, images, volumes, networks)
            })
            .await;

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
                cx.emit(StateChanged::Loading(false));
            });
        })
    })
    .detach();
}
