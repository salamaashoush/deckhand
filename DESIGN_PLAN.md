# Docker UI - Feature Design Plan

## Current Implementation Status

### Containers (Well Implemented)
- [x] List containers (all/running)
- [x] Start/Stop/Restart/Delete
- [x] Create with options (image, name, platform, restart policy, command, entrypoint, workdir, privileged, read-only, init, auto-remove)
- [x] View logs (with tail)
- [x] Terminal/Exec
- [x] File browser
- [x] Inspect (JSON)
- [x] Port mappings display
- [x] Labels display

### Images (Partially Implemented)
- [x] List images (with In Use/Unused sections)
- [x] Pull with platform selection
- [x] Delete
- [x] Inspect details (config, env, ports, containers using it)
- [ ] Build from Dockerfile
- [ ] Tag/Push
- [ ] History/Layers view
- [ ] Save/Load to file

### Volumes (Partially Implemented)
- [x] List volumes
- [x] Create with name, driver, labels
- [x] Delete
- [x] File browser (via temp container)
- [ ] Copy files to/from
- [ ] Backup/Export
- [ ] Prune unused

### Networks (Partially Implemented)
- [x] List networks
- [x] Create with name, IPv6, subnet
- [x] Delete
- [x] View details (driver, IPAM, connected containers)
- [ ] Connect/Disconnect containers
- [ ] Prune unused

### Colima/Machines (Well Implemented)
- [x] List VMs with status
- [x] Create/Start with full options (CPU, memory, disk, runtime, arch, VM type, mount type, K8s, Rosetta)
- [x] Stop/Restart/Delete
- [x] SSH into VM
- [x] View OS info, logs, disk usage, memory, processes
- [x] File browser
- [x] Docker context switching
- [ ] Edit VM config after creation
- [ ] Snapshots

### Activity Monitor (Implemented)
- [x] Real-time stats (CPU, Memory, Network, Disk)
- [x] Per-container breakdown
- [x] History tracking (60 samples)
- [x] Mini charts
- [ ] Full historical graphs
- [ ] Export metrics

### Kubernetes (Not Implemented)
- [ ] Pods list/management
- [ ] Services list/management
- [ ] Deployments
- [ ] Pod logs/exec

### Global UI Features
- [x] Sidebar navigation
- [x] Task bar for running operations
- [x] Notifications
- [x] Dialog system
- [x] Terminal view
- [x] Dark theme
- [ ] Search/Filter
- [ ] Settings panel
- [ ] Keyboard shortcuts
- [ ] Prune tools
- [ ] Docker Compose support

---

## Phase 1: Polish & Complete Core Features

### 1.1 Container Create Dialog Enhancements
**Priority: High**
**Files:** `src/ui/containers/create_dialog.rs`

Add missing options to container creation:
- Environment variables (key-value pairs with add/remove)
- Port mappings (host:container with protocol)
- Volume mounts (source:dest with mode)
- Network selection dropdown
- Resource limits (CPU, memory)
- Labels (key-value pairs)

```
+------------------------------------------+
| New Container                            |
+------------------------------------------+
| Image: [nginx:latest        ] [Browse]   |
| Name:  [my-container        ]            |
|                                          |
| [General] [Ports] [Volumes] [Env] [Net]  |
+------------------------------------------+
| Environment Variables                    |
| +--------------------------------------+ |
| | KEY          | VALUE                 | |
| | NODE_ENV     | production            | |
| | API_URL      | http://api.local      | |
| +--------------------------------------+ |
| [+ Add Variable]                         |
+------------------------------------------+
```

### 1.2 Search & Filter
**Priority: High**
**Files:** New `src/ui/components/search.rs`, modify all list views

Add global search bar that filters:
- Containers by name, image, status
- Images by repository, tag
- Volumes by name, driver
- Networks by name, driver

```rust
pub struct SearchBar {
    query: String,
    filter_type: FilterType, // All, Containers, Images, etc.
}
```

### 1.3 Prune Tools
**Priority: High**
**Files:** `src/services/dispatcher.rs`, new `src/ui/prune_dialog.rs`

Add prune functionality accessible from menu or toolbar:
- Prune stopped containers
- Prune unused images (dangling)
- Prune unused volumes
- Prune unused networks
- System prune (all)

Docker API calls needed:
```rust
// In docker client
pub async fn prune_containers(&self) -> Result<PruneInfo>
pub async fn prune_images(&self, dangling_only: bool) -> Result<PruneInfo>
pub async fn prune_volumes(&self) -> Result<PruneInfo>
pub async fn prune_networks(&self) -> Result<PruneInfo>
```

### 1.4 Settings Panel
**Priority: High**
**Files:** New `src/ui/settings/mod.rs`

Settings for:
- Docker socket path
- Default Colima profile
- Theme selection (if we add light mode)
- Refresh intervals
- Log line limits
- Terminal font/size

---

## Phase 2: Docker Compose Support

### 2.1 Compose Projects View
**Priority: High**
**Files:** New `src/ui/compose/mod.rs`, `src/docker/compose.rs`

Features:
- Detect compose projects from labels
- Group containers by project
- Show project status (all running, partial, stopped)
- Up/Down/Restart project
- View aggregated logs
- Scale services

```
+------------------------------------------+
| Docker Compose Projects                  |
+------------------------------------------+
| [+] my-app (3/3 running)                 |
|     |- web (nginx)          Running      |
|     |- api (node:18)        Running      |
|     |- db (postgres:15)     Running      |
|                                          |
| [-] another-project (0/2 running)        |
|     |- frontend             Stopped      |
|     |- backend              Stopped      |
+------------------------------------------+
```

### 2.2 Compose File Editor
**Priority: Medium**
**Files:** New `src/ui/compose/editor.rs`

- Load/parse docker-compose.yml
- Syntax highlighting
- Validation
- Apply changes

---

## Phase 3: Kubernetes Support

### 3.1 Enable K8s in Colima
**Priority: Medium**
**Files:** `src/colima/client.rs` (already has k8s option)

- Add toggle in Machines view to enable K8s
- Show K8s status in machine detail

### 3.2 Pods View
**Priority: Medium**
**Files:** New `src/ui/kubernetes/pods.rs`, `src/kubernetes/mod.rs`

Features:
- List pods (namespace filter)
- Pod status with ready/restart counts
- Pod logs
- Pod exec/terminal
- Delete pod
- Describe pod

### 3.3 Services View
**Priority: Medium**
**Files:** New `src/ui/kubernetes/services.rs`

Features:
- List services
- Service type (ClusterIP, NodePort, LoadBalancer)
- Endpoints
- Port mappings

### 3.4 Deployments View
**Priority: Low**
**Files:** New `src/ui/kubernetes/deployments.rs`

Features:
- List deployments
- Scale up/down
- Rollout status
- Rollback

---

## Phase 4: Advanced Image Features

### 4.1 Image Build
**Priority: Medium**
**Files:** New `src/ui/images/build_dialog.rs`, `src/docker/build.rs`

Features:
- Select Dockerfile location
- Build context path
- Build arguments
- Target stage
- Platform selection
- Progress streaming
- Build cache options

### 4.2 Image Layers View
**Priority: Low**
**Files:** `src/ui/images/detail.rs`

- Show layer history
- Layer sizes
- Commands that created each layer

### 4.3 Registry Operations
**Priority: Low**
**Files:** New `src/docker/registry.rs`

- Tag image
- Push to registry
- Registry authentication
- Save/Load to tar

---

## Phase 5: OrbStack-like Features

### 5.1 Local Domain Names
**Priority: Medium**
**Files:** New `src/services/dns.rs`

- Assign `.local` domains to containers
- Update /etc/hosts or use dnsmasq
- Format: `container-name.docker.local`

### 5.2 HTTPS for Containers
**Priority: Low**
**Files:** New `src/services/https_proxy.rs`

- Generate self-signed certificates
- Reverse proxy for HTTPS
- Trust CA in system keychain

---

## Phase 6: UI/UX Improvements

### 6.1 Keyboard Shortcuts
**Priority: Medium**
**Files:** `src/app.rs`, keybinding system

| Shortcut | Action |
|----------|--------|
| Cmd+1-6 | Switch views |
| Cmd+N | New (container/volume/etc based on view) |
| Cmd+F | Focus search |
| Cmd+R | Refresh current view |
| Delete | Delete selected item |
| Enter | Open detail/expand |

### 6.2 Context Menus
**Priority: Medium**
**Files:** All list views

Right-click context menus for quick actions.

### 6.3 Multi-Select & Bulk Actions
**Priority: Low**
**Files:** All list views

- Shift+click for range select
- Cmd+click for multi-select
- Bulk delete, start, stop

### 6.4 Drag & Drop
**Priority: Low**

- Drag Dockerfile to build
- Drag image to create container
- Drag files into volume browser

---

## Implementation Priority Order

### Sprint 1 (Core Polish)
1. Container Create Dialog - add env vars, ports, volumes
2. Search/Filter across all views
3. Prune tools dialog
4. Settings panel

### Sprint 2 (Compose & K8s Basics)
1. Docker Compose projects detection
2. Compose up/down/restart
3. Basic Kubernetes pods list
4. Pod logs/exec

### Sprint 3 (Advanced Features)
1. Image build from Dockerfile
2. Image layers/history view
3. K8s services and deployments
4. Keyboard shortcuts

### Sprint 4 (Polish & OrbStack Features)
1. Local domain names for containers
2. Context menus
3. Multi-select operations
4. Performance optimizations

---

## Technical Notes

### Adding New Docker Operations
1. Add method to `src/docker/client.rs` or relevant module
2. Add service function in `src/services/dispatcher.rs`
3. Update UI to call the service
4. Handle events/notifications

### Adding New Views
1. Create `src/ui/{feature}/mod.rs` with view, list, detail
2. Add to `src/ui/mod.rs`
3. Add `CurrentView` variant in `src/state/app_state.rs`
4. Add sidebar item in `src/app.rs`
5. Add route in `render_content()` in `src/app.rs`

### State Updates
- Use `StateChanged` events for reactive updates
- Subscribe to state in view constructors
- Call `cx.notify()` after state changes
