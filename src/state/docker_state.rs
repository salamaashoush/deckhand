use gpui::{App, AppContext, Entity, EventEmitter, Global};

use crate::colima::ColimaVm;
use crate::docker::{ContainerInfo, ImageInfo, NetworkInfo, VolumeInfo};
use crate::kubernetes::{DeploymentInfo, PodInfo, ServiceInfo};

use super::app_state::CurrentView;

use crate::docker::VolumeFileEntry;

/// Tab indices for machine detail view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum MachineDetailTab {
  #[default]
  Info = 0,
  Config = 1,
  Stats = 2,
  Processes = 3,
  Logs = 4,
  Terminal = 5,
  Files = 6,
}

impl MachineDetailTab {
  pub const ALL: [MachineDetailTab; 7] = [
    MachineDetailTab::Info,
    MachineDetailTab::Config,
    MachineDetailTab::Stats,
    MachineDetailTab::Processes,
    MachineDetailTab::Logs,
    MachineDetailTab::Terminal,
    MachineDetailTab::Files,
  ];

  pub fn label(self) -> &'static str {
    match self {
      MachineDetailTab::Info => "Info",
      MachineDetailTab::Config => "Config",
      MachineDetailTab::Stats => "Stats",
      MachineDetailTab::Processes => "Processes",
      MachineDetailTab::Logs => "Logs",
      MachineDetailTab::Terminal => "Terminal",
      MachineDetailTab::Files => "Files",
    }
  }
}

/// Tab indices for container detail view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum ContainerDetailTab {
  #[default]
  Info = 0,
  Logs = 1,
  Processes = 2,
  Terminal = 3,
  Files = 4,
  Inspect = 5,
}

impl ContainerDetailTab {
  pub const ALL: [ContainerDetailTab; 6] = [
    ContainerDetailTab::Info,
    ContainerDetailTab::Logs,
    ContainerDetailTab::Processes,
    ContainerDetailTab::Terminal,
    ContainerDetailTab::Files,
    ContainerDetailTab::Inspect,
  ];

  pub fn label(self) -> &'static str {
    match self {
      ContainerDetailTab::Info => "Info",
      ContainerDetailTab::Logs => "Logs",
      ContainerDetailTab::Processes => "Processes",
      ContainerDetailTab::Terminal => "Terminal",
      ContainerDetailTab::Files => "Files",
      ContainerDetailTab::Inspect => "Inspect",
    }
  }
}

/// Tab indices for pod detail view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum PodDetailTab {
  #[default]
  Info = 0,
  Logs = 1,
  Terminal = 2,
  Describe = 3,
  Yaml = 4,
}

impl PodDetailTab {
  pub const ALL: [PodDetailTab; 5] = [
    PodDetailTab::Info,
    PodDetailTab::Logs,
    PodDetailTab::Terminal,
    PodDetailTab::Describe,
    PodDetailTab::Yaml,
  ];

  pub fn label(self) -> &'static str {
    match self {
      PodDetailTab::Info => "Info",
      PodDetailTab::Logs => "Logs",
      PodDetailTab::Terminal => "Terminal",
      PodDetailTab::Describe => "Describe",
      PodDetailTab::Yaml => "YAML",
    }
  }
}

/// Tab indices for service detail view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum ServiceDetailTab {
  #[default]
  Info = 0,
  Ports = 1,
  Endpoints = 2,
  Yaml = 3,
}

/// Tab indices for deployment detail view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(usize)]
pub enum DeploymentDetailTab {
  #[default]
  Info = 0,
  Pods = 1,
  Yaml = 2,
}

/// Represents the currently selected item across all views
/// This enables keyboard shortcuts to act on the selection
#[derive(Clone, Debug, Default)]
pub enum Selection {
  #[default]
  None,
  Container(ContainerInfo),
  Image(ImageInfo),
  Volume(String),  // Volume name
  Network(String), // Network ID
  Pod {
    name: String,
    namespace: String,
  },
  Deployment {
    name: String,
    namespace: String,
  },
  Service {
    name: String,
    namespace: String,
  },
  Machine(String), // Machine name
}

/// Image inspect data for detailed view
#[derive(Clone, Debug, Default)]
pub struct ImageInspectData {
  pub config_cmd: Option<Vec<String>>,
  pub config_workdir: Option<String>,
  pub config_env: Vec<(String, String)>,
  pub config_entrypoint: Option<Vec<String>>,
  pub config_exposed_ports: Vec<String>,
  pub used_by: Vec<String>,
}

/// Event emitted when docker state changes
#[derive(Clone, Debug)]
pub enum StateChanged {
  MachinesUpdated,
  ContainersUpdated,
  ImagesUpdated,
  VolumesUpdated,
  NetworksUpdated,
  PodsUpdated,
  NamespacesUpdated,
  ViewChanged,
  SelectionChanged,
  Loading,
  VolumeFilesLoaded {
    volume_name: String,
    path: String,
    files: Vec<VolumeFileEntry>,
  },
  VolumeFilesError {
    volume_name: String,
  },
  ImageInspectLoaded {
    image_id: String,
    data: ImageInspectData,
  },
  PodLogsLoaded {
    pod_name: String,
    namespace: String,
    logs: String,
  },
  PodDescribeLoaded {
    pod_name: String,
    namespace: String,
    describe: String,
  },
  PodYamlLoaded {
    pod_name: String,
    namespace: String,
    yaml: String,
  },
  /// Request to open a machine with a specific tab
  MachineTabRequest {
    machine_name: String,
    tab: MachineDetailTab,
  },
  /// Request to open edit dialog for a machine
  EditMachineRequest {
    machine_name: String,
  },
  /// Request to open a container with a specific tab
  ContainerTabRequest {
    container_id: String,
    tab: ContainerDetailTab,
  },
  /// Request to open rename dialog for a container
  RenameContainerRequest {
    container_id: String,
    current_name: String,
  },
  /// Request to open commit dialog for a container
  CommitContainerRequest {
    container_id: String,
    container_name: String,
  },
  /// Request to open export dialog for a container
  ExportContainerRequest {
    container_id: String,
    container_name: String,
  },
  /// Request to open a pod with a specific tab
  PodTabRequest {
    pod_name: String,
    namespace: String,
    tab: PodDetailTab,
  },
  // Services
  ServicesUpdated,
  ServiceYamlLoaded {
    service_name: String,
    namespace: String,
    yaml: String,
  },
  /// Request to open a service with a specific tab
  ServiceTabRequest {
    service_name: String,
    namespace: String,
    tab: ServiceDetailTab,
  },
  // Deployments
  DeploymentsUpdated,
  DeploymentYamlLoaded {
    deployment_name: String,
    namespace: String,
    yaml: String,
  },
  /// Request to open a deployment with a specific tab
  DeploymentTabRequest {
    deployment_name: String,
    namespace: String,
    tab: DeploymentDetailTab,
  },
  /// Request to open scale dialog for a deployment
  DeploymentScaleRequest {
    deployment_name: String,
    namespace: String,
    current_replicas: i32,
  },
}

/// Represents the load state of a resource
#[derive(Clone, Debug, Default, PartialEq)]
pub enum LoadState {
  #[default]
  NotLoaded,
  Loading,
  Loaded,
  Error(String),
}

/// Global docker state - all views subscribe to this
pub struct DockerState {
  // Docker Data
  pub colima_vms: Vec<ColimaVm>,
  pub containers: Vec<ContainerInfo>,
  pub images: Vec<ImageInfo>,
  pub volumes: Vec<VolumeInfo>,
  pub networks: Vec<NetworkInfo>,

  // Kubernetes Data
  pub pods: Vec<PodInfo>,
  pub services: Vec<ServiceInfo>,
  pub deployments: Vec<DeploymentInfo>,
  pub namespaces: Vec<String>,
  pub selected_namespace: String,
  pub k8s_available: bool,
  /// Error message for K8s connectivity issues
  pub k8s_error: Option<String>,

  // UI state
  pub current_view: CurrentView,
  pub active_detail_tab: usize,
  /// Currently selected item - used by keyboard shortcuts
  pub selection: Selection,

  // Loading states - general loading indicator
  pub is_loading: bool,

  // Per-resource load states (tracks loading, loaded, and error)
  pub containers_state: LoadState,
  pub images_state: LoadState,
  pub volumes_state: LoadState,
  pub networks_state: LoadState,
  pub pods_state: LoadState,
  pub services_state: LoadState,
  pub deployments_state: LoadState,
  pub machines_state: LoadState,
}

impl DockerState {
  pub fn new() -> Self {
    Self {
      colima_vms: Vec::new(),
      containers: Vec::new(),
      images: Vec::new(),
      volumes: Vec::new(),
      networks: Vec::new(),
      pods: Vec::new(),
      services: Vec::new(),
      deployments: Vec::new(),
      namespaces: vec!["default".to_string()],
      selected_namespace: "default".to_string(),
      k8s_available: false,
      k8s_error: None,
      current_view: CurrentView::default(),
      active_detail_tab: 0,
      selection: Selection::None,
      is_loading: true,
      // Per-resource load states
      containers_state: LoadState::NotLoaded,
      images_state: LoadState::NotLoaded,
      volumes_state: LoadState::NotLoaded,
      networks_state: LoadState::NotLoaded,
      pods_state: LoadState::NotLoaded,
      services_state: LoadState::NotLoaded,
      deployments_state: LoadState::NotLoaded,
      machines_state: LoadState::NotLoaded,
    }
  }

  // Selection management
  pub fn set_selection(&mut self, selection: Selection) {
    self.selection = selection;
  }

  // Machines
  pub fn set_machines(&mut self, vms: Vec<ColimaVm>) {
    self.colima_vms = vms;
    self.machines_state = LoadState::Loaded;
  }

  // Containers
  pub fn set_containers(&mut self, containers: Vec<ContainerInfo>) {
    self.containers = containers;
    self.containers_state = LoadState::Loaded;
  }

  // Images
  pub fn set_images(&mut self, images: Vec<ImageInfo>) {
    self.images = images;
    self.images_state = LoadState::Loaded;
  }

  // Volumes
  pub fn set_volumes(&mut self, volumes: Vec<VolumeInfo>) {
    self.volumes = volumes;
    self.volumes_state = LoadState::Loaded;
  }

  // Networks
  pub fn set_networks(&mut self, networks: Vec<NetworkInfo>) {
    self.networks = networks;
    self.networks_state = LoadState::Loaded;
  }

  // Pods (Kubernetes)
  pub fn set_pods(&mut self, pods: Vec<PodInfo>) {
    self.pods = pods;
    self.pods_state = LoadState::Loaded;
  }

  pub fn set_pods_loading(&mut self) {
    self.pods_state = LoadState::Loading;
  }

  pub fn set_pods_error(&mut self, error: String) {
    self.pods_state = LoadState::Error(error);
  }

  pub fn get_pod(&self, name: &str, namespace: &str) -> Option<&PodInfo> {
    self.pods.iter().find(|p| p.name == name && p.namespace == namespace)
  }

  pub fn set_namespaces(&mut self, namespaces: Vec<String>) {
    self.namespaces = namespaces;
  }

  pub fn set_selected_namespace(&mut self, namespace: String) {
    self.selected_namespace = namespace;
  }

  pub fn set_k8s_available(&mut self, available: bool) {
    self.k8s_available = available;
    if available {
      self.k8s_error = None;
    }
  }

  pub fn set_k8s_error(&mut self, error: Option<String>) {
    self.k8s_error = error;
    if self.k8s_error.is_some() {
      self.k8s_available = false;
    }
  }

  // Services (Kubernetes)
  pub fn set_services(&mut self, services: Vec<ServiceInfo>) {
    self.services = services;
    self.services_state = LoadState::Loaded;
  }

  pub fn set_services_loading(&mut self) {
    self.services_state = LoadState::Loading;
  }

  pub fn set_services_error(&mut self, error: String) {
    self.services_state = LoadState::Error(error);
  }

  pub fn get_service(&self, name: &str, namespace: &str) -> Option<&ServiceInfo> {
    self
      .services
      .iter()
      .find(|s| s.name == name && s.namespace == namespace)
  }

  // Deployments (Kubernetes)
  pub fn set_deployments(&mut self, deployments: Vec<DeploymentInfo>) {
    self.deployments = deployments;
    self.deployments_state = LoadState::Loaded;
  }

  pub fn set_deployments_loading(&mut self) {
    self.deployments_state = LoadState::Loading;
  }

  pub fn set_deployments_error(&mut self, error: String) {
    self.deployments_state = LoadState::Error(error);
  }

  pub fn get_deployment(&self, name: &str, namespace: &str) -> Option<&DeploymentInfo> {
    self
      .deployments
      .iter()
      .find(|d| d.name == name && d.namespace == namespace)
  }

  // Navigation
  pub fn set_view(&mut self, view: CurrentView) {
    self.current_view = view;
    self.active_detail_tab = 0;
  }
}

impl Default for DockerState {
  fn default() -> Self {
    Self::new()
  }
}

// Enable event emission for reactive updates
impl EventEmitter<StateChanged> for DockerState {}

/// Global wrapper for `DockerState`
pub struct GlobalDockerState(pub Entity<DockerState>);

impl Global for GlobalDockerState {}

/// Initialize the global docker state
pub fn init_docker_state(cx: &mut App) -> Entity<DockerState> {
  let state = cx.new(|_cx| DockerState::new());
  cx.set_global(GlobalDockerState(state.clone()));
  state
}

/// Get the global docker state entity
pub fn docker_state(cx: &App) -> Entity<DockerState> {
  cx.global::<GlobalDockerState>().0.clone()
}

#[cfg(test)]
mod tests {
  use super::super::app_state::CurrentView;
  use super::*;

  #[test]
  fn test_docker_state_initialization() {
    let state = DockerState::new();

    assert!(state.containers.is_empty());
    assert!(state.images.is_empty());
    assert!(state.volumes.is_empty());
    assert!(state.networks.is_empty());
    assert!(state.colima_vms.is_empty());
    assert!(matches!(state.selection, Selection::None));
    assert!(state.is_loading);
    assert!(!state.k8s_available);
  }

  #[test]
  fn test_docker_state_load_states() {
    let state = DockerState::new();

    // All states should start as NotLoaded
    assert!(matches!(state.containers_state, LoadState::NotLoaded));
    assert!(matches!(state.images_state, LoadState::NotLoaded));
    assert!(matches!(state.volumes_state, LoadState::NotLoaded));
    assert!(matches!(state.networks_state, LoadState::NotLoaded));
    assert!(matches!(state.pods_state, LoadState::NotLoaded));
    assert!(matches!(state.services_state, LoadState::NotLoaded));
    assert!(matches!(state.deployments_state, LoadState::NotLoaded));
    assert!(matches!(state.machines_state, LoadState::NotLoaded));
  }

  #[test]
  fn test_docker_state_selection() {
    let mut state = DockerState::new();

    // Initial selection is None
    assert!(matches!(state.selection, Selection::None));

    // Set volume selection
    state.set_selection(Selection::Volume("my-volume".to_string()));
    assert!(matches!(state.selection, Selection::Volume(_)));
    if let Selection::Volume(ref name) = state.selection {
      assert_eq!(name, "my-volume");
    }

    // Set network selection
    state.set_selection(Selection::Network("network-123".to_string()));
    assert!(matches!(state.selection, Selection::Network(_)));

    // Set machine selection
    state.set_selection(Selection::Machine("default".to_string()));
    assert!(matches!(state.selection, Selection::Machine(_)));

    // Set pod selection
    state.set_selection(Selection::Pod {
      name: "my-pod".to_string(),
      namespace: "default".to_string(),
    });
    assert!(matches!(state.selection, Selection::Pod { .. }));

    // Clear selection
    state.set_selection(Selection::None);
    assert!(matches!(state.selection, Selection::None));
  }

  #[test]
  fn test_docker_state_kubernetes() {
    let mut state = DockerState::new();

    // Initially k8s should not be available
    assert!(!state.k8s_available);
    assert!(state.k8s_error.is_none());

    // Set k8s as available
    state.set_k8s_available(true);
    assert!(state.k8s_available);
    assert!(state.k8s_error.is_none());

    // Set k8s error - should mark as unavailable
    state.set_k8s_error(Some("Connection refused".to_string()));
    assert!(!state.k8s_available);
    assert_eq!(state.k8s_error, Some("Connection refused".to_string()));

    // Clear error by setting available
    state.set_k8s_available(true);
    assert!(state.k8s_available);
    assert!(state.k8s_error.is_none());
  }

  #[test]
  fn test_docker_state_namespaces() {
    let mut state = DockerState::new();

    // Default namespace should be "default"
    assert_eq!(state.selected_namespace, "default");
    assert_eq!(state.namespaces, vec!["default".to_string()]);

    // Set namespaces
    state.set_namespaces(vec![
      "default".to_string(),
      "kube-system".to_string(),
      "production".to_string(),
    ]);
    assert_eq!(state.namespaces.len(), 3);

    // Change selected namespace
    state.set_selected_namespace("production".to_string());
    assert_eq!(state.selected_namespace, "production");
  }

  #[test]
  fn test_docker_state_view_navigation() {
    let mut state = DockerState::new();

    // Set different views
    state.set_view(CurrentView::Containers);
    assert!(matches!(state.current_view, CurrentView::Containers));
    assert_eq!(state.active_detail_tab, 0); // Tab resets on view change

    state.active_detail_tab = 2;
    state.set_view(CurrentView::Images);
    assert!(matches!(state.current_view, CurrentView::Images));
    assert_eq!(state.active_detail_tab, 0); // Tab resets
  }

  #[test]
  fn test_load_state_enum() {
    let not_loaded = LoadState::NotLoaded;
    let loading = LoadState::Loading;
    let loaded = LoadState::Loaded;
    let error = LoadState::Error("Test error".to_string());

    assert!(matches!(not_loaded, LoadState::NotLoaded));
    assert!(matches!(loading, LoadState::Loading));
    assert!(matches!(loaded, LoadState::Loaded));
    assert!(matches!(error, LoadState::Error(_)));
  }

  #[test]
  fn test_container_detail_tab() {
    assert_eq!(ContainerDetailTab::ALL.len(), 6);
    assert_eq!(ContainerDetailTab::Info.label(), "Info");
    assert_eq!(ContainerDetailTab::Logs.label(), "Logs");
    assert_eq!(ContainerDetailTab::Processes.label(), "Processes");
    assert_eq!(ContainerDetailTab::Terminal.label(), "Terminal");
    assert_eq!(ContainerDetailTab::Files.label(), "Files");
    assert_eq!(ContainerDetailTab::Inspect.label(), "Inspect");
  }

  #[test]
  fn test_machine_detail_tab() {
    assert_eq!(MachineDetailTab::ALL.len(), 7);
    assert_eq!(MachineDetailTab::Info.label(), "Info");
    assert_eq!(MachineDetailTab::Config.label(), "Config");
    assert_eq!(MachineDetailTab::Stats.label(), "Stats");
    assert_eq!(MachineDetailTab::Processes.label(), "Processes");
    assert_eq!(MachineDetailTab::Logs.label(), "Logs");
    assert_eq!(MachineDetailTab::Terminal.label(), "Terminal");
    assert_eq!(MachineDetailTab::Files.label(), "Files");
  }

  #[test]
  fn test_pod_detail_tab() {
    assert_eq!(PodDetailTab::ALL.len(), 5);
    assert_eq!(PodDetailTab::Info.label(), "Info");
    assert_eq!(PodDetailTab::Logs.label(), "Logs");
    assert_eq!(PodDetailTab::Terminal.label(), "Terminal");
    assert_eq!(PodDetailTab::Describe.label(), "Describe");
    assert_eq!(PodDetailTab::Yaml.label(), "YAML");
  }
}
