// Allow precision loss for display formatting of byte sizes
#![allow(clippy::cast_precision_loss)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum VmStatus {
  Running,
  Stopped,
  #[default]
  Unknown,
}

impl VmStatus {
  pub fn is_running(self) -> bool {
    matches!(self, VmStatus::Running)
  }
}

impl std::fmt::Display for VmStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VmStatus::Running => write!(f, "Running"),
      VmStatus::Stopped => write!(f, "Stopped"),
      VmStatus::Unknown => write!(f, "Unknown"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VmRuntime {
  #[default]
  Docker,
  Containerd,
  Incus,
}

impl std::fmt::Display for VmRuntime {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VmRuntime::Docker => write!(f, "docker"),
      VmRuntime::Containerd => write!(f, "containerd"),
      VmRuntime::Incus => write!(f, "incus"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VmArch {
  #[default]
  #[serde(alias = "host")]
  Host,
  #[serde(alias = "aarch64")]
  Aarch64,
  #[serde(rename = "x86_64", alias = "x86_64")]
  X86_64,
}

impl VmArch {
  pub fn display_name(self) -> &'static str {
    match self {
      VmArch::Host => "Host (native)",
      VmArch::Aarch64 => "ARM64 (aarch64)",
      VmArch::X86_64 => "x86_64",
    }
  }
}

impl std::fmt::Display for VmArch {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VmArch::Host => write!(f, "host"),
      VmArch::Aarch64 => write!(f, "aarch64"),
      VmArch::X86_64 => write!(f, "x86_64"),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum VmType {
  #[default]
  Qemu,
  Vz,
}

impl std::fmt::Display for VmType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      VmType::Vz => write!(f, "vz"),
      VmType::Qemu => write!(f, "qemu"),
    }
  }
}

impl VmType {
  pub fn display_name(self) -> &'static str {
    match self {
      VmType::Vz => "Apple Virtualization (VZ)",
      VmType::Qemu => "QEMU",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MountType {
  #[default]
  Sshfs,
  Virtiofs,
  #[serde(rename = "9p")]
  NineP,
}

impl std::fmt::Display for MountType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MountType::Virtiofs => write!(f, "virtiofs"),
      MountType::Sshfs => write!(f, "sshfs"),
      MountType::NineP => write!(f, "9p"),
    }
  }
}

/// Network mode for the VM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NetworkMode {
  #[default]
  Shared,
  Bridged,
}

impl std::fmt::Display for NetworkMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      NetworkMode::Shared => write!(f, "shared"),
      NetworkMode::Bridged => write!(f, "bridged"),
    }
  }
}

/// Port forwarder method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PortForwarder {
  #[default]
  Ssh,
  Grpc,
}

impl std::fmt::Display for PortForwarder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      PortForwarder::Ssh => write!(f, "ssh"),
      PortForwarder::Grpc => write!(f, "grpc"),
    }
  }
}

/// Configuration for a directory mount
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MountConfig {
  pub location: String,
  pub writable: bool,
}

impl MountConfig {
  pub fn new(location: impl Into<String>, writable: bool) -> Self {
    Self {
      location: location.into(),
      writable,
    }
  }
}

/// Provision script mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProvisionMode {
  #[default]
  System,
  User,
}

impl std::fmt::Display for ProvisionMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ProvisionMode::System => write!(f, "system"),
      ProvisionMode::User => write!(f, "user"),
    }
  }
}

/// A provisioning script configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionScript {
  pub mode: ProvisionMode,
  pub script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColimaVm {
  pub name: String,
  pub status: VmStatus,
  pub runtime: VmRuntime,
  pub arch: VmArch,
  pub cpus: u32,
  pub memory: u64,
  pub disk: u64,
  pub kubernetes: bool,
  pub address: Option<String>,
  // Extended fields from status --json
  pub driver: Option<String>,
  pub vm_type: Option<VmType>,
  pub mount_type: Option<MountType>,
  pub docker_socket: Option<String>,
  pub containerd_socket: Option<String>,
  pub hostname: Option<String>,
  pub rosetta: bool,
  pub ssh_agent: bool,
}

impl Default for ColimaVm {
  fn default() -> Self {
    Self {
      name: "default".to_string(),
      status: VmStatus::Unknown,
      runtime: VmRuntime::Docker,
      arch: VmArch::Aarch64,
      cpus: 2,
      memory: 2 * 1024 * 1024 * 1024,
      disk: 60 * 1024 * 1024 * 1024,
      kubernetes: false,
      address: None,
      driver: None,
      vm_type: None,
      mount_type: None,
      docker_socket: None,
      containerd_socket: None,
      hostname: None,
      rosetta: false,
      ssh_agent: false,
    }
  }
}

impl ColimaVm {
  pub fn memory_gb(&self) -> f64 {
    self.memory as f64 / (1024.0 * 1024.0 * 1024.0)
  }

  pub fn disk_gb(&self) -> f64 {
    self.disk as f64 / (1024.0 * 1024.0 * 1024.0)
  }

  pub fn display_driver(&self) -> String {
    self.driver.clone().unwrap_or_else(|| {
      self
        .vm_type
        .map_or_else(|| "Unknown".to_string(), |v| v.display_name().to_string())
    })
  }

  pub fn display_mount_type(&self) -> String {
    self
      .mount_type
      .map_or_else(|| "virtiofs".to_string(), |m| m.to_string())
  }
}

/// Information about the VM's operating system
#[derive(Debug, Clone, Default)]
pub struct VmOsInfo {
  pub pretty_name: String,
  pub name: String,
  pub version: String,
  pub version_id: String,
  pub id: String,
  pub kernel: String,
  pub arch: String,
}

/// A file entry in the VM filesystem
#[derive(Debug, Clone)]
pub struct VmFileEntry {
  pub name: String,
  pub path: String,
  pub is_dir: bool,
  pub is_symlink: bool,
  pub size: u64,
  pub permissions: String,
  pub owner: String,
  pub modified: String,
}

impl VmFileEntry {
  pub fn display_size(&self) -> String {
    if self.is_dir {
      "-".to_string()
    } else if self.size < 1024 {
      format!("{} B", self.size)
    } else if self.size < 1024 * 1024 {
      format!("{:.1} KB", self.size as f64 / 1024.0)
    } else if self.size < 1024 * 1024 * 1024 {
      format!("{:.1} MB", self.size as f64 / (1024.0 * 1024.0))
    } else {
      format!("{:.1} GB", self.size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
  }
}

// ============================================================================
// Configuration file structures (for reading/writing colima.yaml)
// ============================================================================

/// Kubernetes cluster configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KubernetesConfig {
  pub enabled: bool,
  #[serde(default)]
  pub version: String,
  #[serde(default, rename = "k3sArgs")]
  pub k3s_args: Vec<String>,
  #[serde(default = "default_k8s_port")]
  pub port: u32,
}

fn default_k8s_port() -> u32 {
  6443
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
  #[serde(default)]
  pub address: bool,
  #[serde(default = "default_network_mode")]
  pub mode: NetworkMode,
  #[serde(default = "default_interface", rename = "interface")]
  pub interface: String,
  #[serde(default, rename = "preferredRoute")]
  pub preferred_route: bool,
  #[serde(default)]
  pub dns: Vec<String>,
  #[serde(default = "default_dns_hosts", rename = "dnsHosts")]
  pub dns_hosts: HashMap<String, String>,
  #[serde(default, rename = "hostAddresses")]
  pub host_addresses: bool,
}

fn default_network_mode() -> NetworkMode {
  NetworkMode::Shared
}

fn default_interface() -> String {
  "en0".to_string()
}

fn default_dns_hosts() -> HashMap<String, String> {
  let mut hosts = HashMap::new();
  hosts.insert("host.docker.internal".to_string(), "host.lima.internal".to_string());
  hosts
}

impl Default for NetworkConfig {
  fn default() -> Self {
    Self {
      address: false,
      mode: NetworkMode::Shared,
      interface: "en0".to_string(),
      preferred_route: false,
      dns: Vec::new(),
      dns_hosts: default_dns_hosts(),
      host_addresses: false,
    }
  }
}

/// Full colima configuration (maps to colima.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColimaConfig {
  #[serde(default = "default_cpu")]
  pub cpu: u32,
  #[serde(default = "default_disk")]
  pub disk: u32,
  #[serde(default = "default_memory")]
  pub memory: u32,
  #[serde(default)]
  pub arch: VmArch,
  #[serde(default)]
  pub runtime: VmRuntime,
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub hostname: String,
  #[serde(default)]
  pub kubernetes: KubernetesConfig,
  #[serde(default = "default_true", rename = "autoActivate")]
  pub auto_activate: bool,
  #[serde(default)]
  pub network: NetworkConfig,
  #[serde(default, rename = "forwardAgent")]
  pub forward_agent: bool,
  #[serde(default, skip_serializing_if = "is_empty_object")]
  pub docker: serde_json::Value,
  #[serde(default, rename = "vmType")]
  pub vm_type: VmType,
  #[serde(default, rename = "portForwarder")]
  pub port_forwarder: PortForwarder,
  #[serde(default)]
  pub rosetta: bool,
  #[serde(default = "default_true")]
  pub binfmt: bool,
  #[serde(default, rename = "nestedVirtualization")]
  pub nested_virtualization: bool,
  #[serde(default, rename = "mountType")]
  pub mount_type: MountType,
  #[serde(default, rename = "mountInotify")]
  pub mount_inotify: bool,
  #[serde(default = "default_cpu_type", rename = "cpuType")]
  pub cpu_type: String,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub provision: Vec<ProvisionScript>,
  #[serde(default = "default_true", rename = "sshConfig")]
  pub ssh_config: bool,
  #[serde(default, rename = "sshPort")]
  pub ssh_port: u32,
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub mounts: Vec<MountConfig>,
  #[serde(default, skip_serializing_if = "String::is_empty", rename = "diskImage")]
  pub disk_image: String,
  #[serde(default = "default_root_disk", rename = "rootDisk")]
  pub root_disk: u32,
  #[serde(default, skip_serializing_if = "HashMap::is_empty")]
  pub env: HashMap<String, String>,
}

fn is_empty_object(v: &serde_json::Value) -> bool {
  v.as_object().is_some_and(serde_json::Map::is_empty)
}

fn default_cpu() -> u32 {
  2
}

fn default_memory() -> u32 {
  2
}

fn default_disk() -> u32 {
  100
}

fn default_root_disk() -> u32 {
  20
}

fn default_cpu_type() -> String {
  "host".to_string()
}

fn default_true() -> bool {
  true
}

impl Default for ColimaConfig {
  fn default() -> Self {
    Self {
      cpu: default_cpu(),
      disk: default_disk(),
      memory: default_memory(),
      arch: VmArch::default(),
      runtime: VmRuntime::default(),
      hostname: String::new(),
      kubernetes: KubernetesConfig::default(),
      auto_activate: true,
      network: NetworkConfig::default(),
      forward_agent: false,
      docker: serde_json::Value::Object(serde_json::Map::new()),
      vm_type: VmType::default(),
      port_forwarder: PortForwarder::default(),
      rosetta: false,
      binfmt: true,
      nested_virtualization: false,
      mount_type: MountType::default(),
      mount_inotify: false,
      cpu_type: default_cpu_type(),
      provision: Vec::new(),
      ssh_config: true,
      ssh_port: 0,
      mounts: Vec::new(),
      disk_image: String::new(),
      root_disk: default_root_disk(),
      env: HashMap::new(),
    }
  }
}
