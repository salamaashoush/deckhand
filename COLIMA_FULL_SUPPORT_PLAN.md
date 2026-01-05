# Colima Full UI Support Plan

## Current State Analysis

### Already Implemented
- Basic VM lifecycle: start, stop, restart, delete
- VM listing with status
- Basic start options: cpus, memory, disk, runtime, kubernetes, arch, vm_type, mount_type, network_address, rosetta, ssh_agent, hostname
- SSH into VM (terminal)
- File browser in VM
- System logs, Docker logs, containerd logs
- Disk/memory/process info
- OS info display

### Missing from CLI

## Phase 1: Enhanced Start Options

### 1.1 Missing Start Options (ColimaStartOptions needs expansion)

```rust
pub struct ColimaStartOptions {
  // EXISTING
  pub name: Option<String>,
  pub cpus: Option<u32>,
  pub memory: Option<u32>,
  pub disk: Option<u32>,
  pub runtime: Option<VmRuntime>,
  pub kubernetes: bool,
  pub arch: Option<VmArch>,
  pub vm_type: Option<VmType>,
  pub mount_type: Option<MountType>,
  pub network_address: bool,
  pub rosetta: bool,
  pub ssh_agent: bool,
  pub hostname: Option<String>,
  pub edit: bool,

  // NEW - Need to add
  pub cpu_type: Option<String>,           // --cpu-type (QEMU CPU type)
  pub disk_image: Option<String>,         // --disk-image (custom disk image path)
  pub dns: Vec<String>,                   // --dns (DNS resolvers)
  pub dns_hosts: Vec<(String, String)>,   // --dns-host (custom DNS mappings)
  pub env: Vec<(String, String)>,         // --env (environment variables)
  pub mounts: Vec<MountConfig>,           // --mount (directory mounts)
  pub mount_inotify: bool,                // --mount-inotify
  pub network_mode: Option<NetworkMode>,  // --network-mode (shared, bridged)
  pub network_interface: Option<String>,  // --network-interface (for bridged)
  pub network_host_addresses: bool,       // --network-host-addresses
  pub network_preferred_route: bool,      // --network-preferred-route
  pub port_forwarder: Option<PortForwarder>, // --port-forwarder (ssh, grpc)
  pub root_disk: Option<u32>,             // --root-disk
  pub ssh_port: Option<u32>,              // --ssh-port
  pub nested_virtualization: bool,        // --nested-virtualization
  pub binfmt: bool,                       // --binfmt (default true)
  pub activate: bool,                     // --activate (default true)
  pub ssh_config: bool,                   // --ssh-config (default true)
  pub save_config: bool,                  // --save-config (default true)
  pub foreground: bool,                   // --foreground
  pub k3s_args: Vec<String>,              // --k3s-arg
  pub k3s_listen_port: Option<u32>,       // --k3s-listen-port
  pub kubernetes_version: Option<String>, // --kubernetes-version
}
```

### 1.2 New Types Needed

```rust
pub enum NetworkMode {
  Shared,
  Bridged,
}

pub enum PortForwarder {
  Ssh,
  Grpc,
}

pub struct MountConfig {
  pub location: String,
  pub writable: bool,
}
```

## Phase 2: Kubernetes Management Commands

### 2.1 New ColimaClient Methods

```rust
impl ColimaClient {
  // Kubernetes cluster management
  pub fn kubernetes_start(profile: Option<&str>) -> Result<()>;
  pub fn kubernetes_stop(profile: Option<&str>) -> Result<()>;
  pub fn kubernetes_reset(profile: Option<&str>) -> Result<()>;
  pub fn kubernetes_delete(profile: Option<&str>) -> Result<()>;
}
```

### 2.2 UI Actions
- Add K8s control buttons in machine detail view:
  - "Enable Kubernetes" (when disabled)
  - "Disable Kubernetes" (kubernetes stop)
  - "Reset Kubernetes" (kubernetes reset)
  - "Delete Kubernetes Data" (kubernetes delete)

## Phase 3: Additional Commands

### 3.1 Update Command
```rust
pub fn update(profile: Option<&str>) -> Result<()>;
```
- Updates container runtime in the VM
- Add "Update Runtime" button in machine actions

### 3.2 Prune Command
```rust
pub fn prune(all: bool, force: bool) -> Result<()>;
```
- Prunes cached downloaded assets
- Add "Clear Cache" or "Prune Assets" in settings/maintenance

### 3.3 SSH Config
```rust
pub fn ssh_config(profile: Option<&str>) -> Result<String>;
```
- Shows SSH connection config
- Display in machine info panel or copy to clipboard

### 3.4 Template Management
```rust
pub fn template_path() -> Result<String>;
pub fn read_template() -> Result<String>;
pub fn write_template(content: &str) -> Result<()>;
```
- Read/edit default configuration template
- Add "Edit Default Template" in settings

## Phase 4: Enhanced UI Components

### 4.1 Create Machine Dialog Enhancements

Current fields:
- Name, CPUs, Memory, Disk, Runtime, Kubernetes, Architecture

Add new tabs/sections:

**Basic Tab** (existing)
- Name, CPUs, Memory, Disk

**Runtime Tab**
- Runtime (Docker/Containerd/Incus)
- Kubernetes enabled
- Kubernetes version (dropdown of k3s versions)
- K3s arguments (text input)

**Virtualization Tab**
- VM Type (VZ/QEMU)
- Architecture (aarch64/x86_64)
- CPU Type (for QEMU)
- Rosetta (for amd64 emulation)
- Nested Virtualization
- Binfmt

**Storage Tab**
- Disk size
- Root disk size
- Custom disk image path
- Mount type (virtiofs/9p/sshfs)
- Mount inotify
- Directory mounts (list with add/remove)

**Network Tab**
- Network mode (shared/bridged)
- Network interface (for bridged)
- Assign reachable IP
- Host addresses support
- Preferred route
- Port forwarder (ssh/grpc)
- DNS servers (list)
- DNS host mappings (key-value list)

**Advanced Tab**
- Hostname
- SSH agent forwarding
- SSH port
- Environment variables (key-value list)
- Auto-activate context
- Save config
- Generate SSH config

### 4.2 Machine Detail View Enhancements

Add new sections:
- **Kubernetes Panel**: Start/Stop/Reset/Delete K8s, show K8s version
- **Network Info**: IP address, DNS config, port forwarding status
- **Mounts Panel**: Show configured mounts, add/remove mounts
- **Environment Panel**: Show/edit environment variables

### 4.3 Settings/Preferences

Add Colima section:
- Edit default template
- Prune cache (with size display)
- Default values for new machines

### 4.4 Toolbar Actions

Add to machine list toolbar:
- "Update All" - update runtime on all running VMs
- "Prune Cache" - clean up downloaded assets

## Phase 5: Configuration File Support

### 5.1 Read/Write colima.yaml

```rust
pub fn read_config(profile: Option<&str>) -> Result<ColimaConfig>;
pub fn write_config(profile: Option<&str>, config: &ColimaConfig) -> Result<()>;
```

### 5.2 ColimaConfig struct (maps to YAML)

```rust
pub struct ColimaConfig {
  pub cpu: u32,
  pub memory: u32,
  pub disk: u32,
  pub arch: VmArch,
  pub runtime: VmRuntime,
  pub hostname: Option<String>,
  pub kubernetes: KubernetesConfig,
  pub auto_activate: bool,
  pub network: NetworkConfig,
  pub forward_agent: bool,
  pub docker: serde_json::Value,  // Docker daemon config
  pub vm_type: VmType,
  pub port_forwarder: PortForwarder,
  pub rosetta: bool,
  pub binfmt: bool,
  pub nested_virtualization: bool,
  pub mount_type: MountType,
  pub mount_inotify: bool,
  pub cpu_type: Option<String>,
  pub provision: Vec<ProvisionScript>,
  pub ssh_config: bool,
  pub ssh_port: u32,
  pub mounts: Vec<MountConfig>,
  pub disk_image: Option<String>,
  pub root_disk: u32,
  pub env: HashMap<String, String>,
}

pub struct KubernetesConfig {
  pub enabled: bool,
  pub version: String,
  pub k3s_args: Vec<String>,
  pub port: u32,
}

pub struct NetworkConfig {
  pub address: bool,
  pub mode: NetworkMode,
  pub interface: String,
  pub preferred_route: bool,
  pub dns: Vec<String>,
  pub dns_hosts: HashMap<String, String>,
  pub host_addresses: bool,
}

pub struct ProvisionScript {
  pub mode: ProvisionMode,  // system or user
  pub script: String,
}
```

### 5.3 Config Editor UI
- YAML editor with syntax highlighting
- Or structured form-based editor
- Validate before saving
- Show diff before applying

## Phase 6: Docker Daemon Configuration

### 6.1 Docker Config Management
```rust
pub fn get_docker_config(profile: Option<&str>) -> Result<serde_json::Value>;
pub fn set_docker_config(profile: Option<&str>, config: &serde_json::Value) -> Result<()>;
```

### 6.2 UI for Docker Daemon Settings
- Insecure registries (list)
- Registry mirrors
- BuildKit enable/disable
- Storage driver options
- Log driver configuration
- Other daemon.json options

## Phase 7: Provisioning Scripts

### 7.1 Script Management
- View existing provision scripts
- Add new scripts (system or user mode)
- Edit/delete scripts
- Test script execution

### 7.2 UI
- Script list with mode indicator
- Monaco/code editor for script content
- "Run Now" button to execute manually

## Implementation Priority

### High Priority (Core Functionality)
1. Enhanced start options (Phase 1)
2. Kubernetes management (Phase 2)
3. Create dialog enhancements (Phase 4.1)

### Medium Priority (Power User Features)
4. Configuration file support (Phase 5)
5. Machine detail view enhancements (Phase 4.2)
6. Additional commands (Phase 3)

### Lower Priority (Nice to Have)
7. Docker daemon configuration (Phase 6)
8. Provisioning scripts (Phase 7)
9. Template management
10. Settings/Preferences panel

## Files to Modify

### Types
- `src/colima/types.rs` - Add new enums and structs

### Client
- `src/colima/client.rs` - Add new methods for all commands

### Services
- `src/services/colima/machines.rs` - Update service functions
- `src/services/colima/kubernetes.rs` - Add K8s management functions

### UI
- `src/ui/machines/create_dialog.rs` - Enhanced create dialog
- `src/ui/machines/edit_dialog.rs` - Enhanced edit dialog
- `src/ui/machines/view.rs` - Add new panels and actions
- `src/ui/machines/list.rs` - Add toolbar actions
- New: `src/ui/machines/kubernetes_panel.rs`
- New: `src/ui/machines/network_panel.rs`
- New: `src/ui/machines/mounts_panel.rs`
- New: `src/ui/settings/colima_settings.rs`

## Testing Checklist

- [ ] Create VM with all basic options
- [ ] Create VM with advanced networking
- [ ] Create VM with custom mounts
- [ ] Create VM with Kubernetes and k3s args
- [ ] Start/Stop/Restart Kubernetes separately
- [ ] Reset Kubernetes cluster
- [ ] Update runtime
- [ ] Prune cache
- [ ] Edit configuration file
- [ ] Edit Docker daemon config
- [ ] Add/remove provision scripts
- [ ] VPN compatibility (localhost fallback)
