# Dockside Competitive Development Plan

## Executive Summary

This plan outlines features to make Dockside competitive with Docker Desktop and OrbStack, while leveraging unique advantages: **free/open-source**, **native Rust performance**, and **Colima-based architecture**.

---

## Competitive Analysis

### Docker Desktop Strengths
- Mature ecosystem with 100+ extensions
- AI integration (Model Runner, Ask Gordon, MCP Toolkit)
- Cross-platform (Windows, macOS, Linux)
- Enterprise features (SSO, audit logs, compliance)
- Kubernetes built-in

### Docker Desktop Weaknesses
- Slow startup (20-30 seconds)
- High resource consumption (700mW+ power, heavy RAM)
- Licensing costs for businesses ($5-21/user/month)
- Bloated with features many don't need
- Electron-based (not native)

### OrbStack Strengths
- Lightning fast (2s startup, 10x faster containers)
- Native macOS (Swift/Rust/Go)
- Low resources (0.1% idle CPU, 180mW power, 60% less RAM)
- Automatic local domains (*.orb.local)
- Automatic HTTPS with local certs
- Linux VM support (15 distros)
- VPN-friendly networking
- Dynamic disk/RAM allocation

### OrbStack Weaknesses
- **$8/month for commercial use** (Dockside opportunity)
- macOS only (same as Dockside)
- Closed source
- Limited customization
- No extension system

### Dockside Current Strengths
- **100% Free and Open Source (MIT)**
- Native Rust + GPUI (GPU-accelerated, 60fps)
- Colima-based (already fast)
- Docker Compose project grouping
- Real-time activity monitoring
- 20+ themes with hot-reload
- Kubernetes support via Colima
- Professional codebase (type-safe Rust)

---

## Strategic Positioning

**Tagline:** "The free, fast, native Docker GUI for macOS"

**Key Differentiators:**
1. **Free forever** - No licensing, no subscriptions, MIT license
2. **Performance parity** - Match OrbStack speed via Colima optimizations
3. **Developer-first UX** - Keyboard-driven, command palette, shortcuts
4. **Open ecosystem** - Plugin system, community extensions
5. **Privacy-focused** - No telemetry, no AI cloud features, local-only

---

## Development Phases

## Phase 1: Performance & Core UX (Critical)
*Goal: Match OrbStack's speed and developer experience*

### 1.1 Startup Optimization
- [ ] Profile and optimize app startup time
- [ ] Lazy-load non-essential UI components
- [ ] Cache Docker state between sessions
- [ ] Background service for instant reconnection
- [ ] Target: <3 second cold start

### 1.2 Keyboard Shortcuts System
- [ ] Global shortcut to open app (configurable, e.g., Cmd+Shift+D)
- [ ] Navigation shortcuts (Cmd+1-9 for sidebar items)
- [ ] Action shortcuts:
  - `Cmd+N` - New container
  - `Cmd+R` - Restart selected
  - `Cmd+S` - Start selected
  - `Cmd+Shift+S` - Stop selected
  - `Cmd+Backspace` - Delete selected
  - `Cmd+L` - View logs
  - `Cmd+T` - Open terminal
  - `Cmd+I` - Inspect
- [ ] Vim-style navigation (j/k for list, Enter to select)
- [ ] Shortcuts overlay (press `?` to show)

### 1.3 Command Palette (Cmd+K)
- [ ] Fuzzy search across all actions
- [ ] Quick container/image/volume search
- [ ] Recent actions history
- [ ] Context-aware suggestions
- [ ] Run any command without mouse

### 1.4 Quick Actions Menu Bar
- [ ] Show running containers count in menu bar icon
- [ ] Quick access to start/stop/restart from menu
- [ ] One-click open container URL in browser
- [ ] Recent containers submenu
- [ ] Quick pull image
- [ ] System stats summary

---

## Phase 2: Networking & Developer Tools (High Priority)
*Goal: Match OrbStack's networking magic*

### 2.1 Local Domain Names (*.dockside.local)
- [ ] Automatic DNS resolution for containers
- [ ] Format: `container-name.dockside.local`
- [ ] Compose services: `service.project.dockside.local`
- [ ] Kubernetes: `service.namespace.k8s.dockside.local`
- [ ] Integration with macOS resolver (/etc/resolver/)
- [ ] Configurable domain suffix

### 2.2 Automatic HTTPS
- [ ] Local CA generation and trust
- [ ] Automatic certificate per container
- [ ] HTTPS URLs in UI (click to open)
- [ ] Certificate viewer in UI
- [ ] mkcert integration or custom CA

### 2.3 Port Management
- [ ] Port conflict detection before start
- [ ] Automatic port assignment option
- [ ] Port forwarding visualization
- [ ] Quick "Open in Browser" for web containers
- [ ] Copy URL to clipboard

### 2.4 Network Debugging Tools
- [ ] Built-in DNS lookup
- [ ] Container-to-container ping test
- [ ] Network traffic monitor
- [ ] Connection troubleshooter

---

## Phase 3: Image Management (Complete the Basics)
*Goal: Full image lifecycle without leaving the app*

### 3.1 Build from Dockerfile
- [ ] Select Dockerfile from filesystem
- [ ] Build context selection
- [ ] Build arguments (--build-arg)
- [ ] Target stage selection
- [ ] Platform selection (amd64/arm64)
- [ ] Build progress with streaming logs
- [ ] Build cache visualization
- [ ] Save build configurations

### 3.2 Registry Integration
- [ ] Login to registries (Docker Hub, GHCR, ECR, GCR)
- [ ] Credential management (keychain)
- [ ] Browse registry images
- [ ] Tag images
- [ ] Push images with progress
- [ ] Registry search

### 3.3 Image Analysis
- [ ] Layer history visualization
- [ ] Layer size breakdown
- [ ] Dockerfile reconstruction
- [ ] Image diff (compare two images)
- [ ] Find large layers
- [ ] Security scan integration (optional)

### 3.4 Image Import/Export
- [ ] Save image to tar
- [ ] Load image from tar
- [ ] Drag & drop to import
- [ ] Export multiple images

---

## Phase 4: Container Enhancements (Power User Features)

### 4.1 Container Templates
- [ ] Save container config as template
- [ ] Template library (user-created)
- [ ] Built-in templates (nginx, postgres, redis, etc.)
- [ ] Import/export templates
- [ ] Community template sharing

### 4.2 Container Groups
- [ ] Create custom groups beyond Compose
- [ ] Start/stop groups together
- [ ] Group-level stats
- [ ] Group dashboard view

### 4.3 Advanced Logs
- [ ] Search within logs (Cmd+F)
- [ ] Filter by log level (info/warn/error)
- [ ] Timestamp toggle
- [ ] Export logs to file
- [ ] Follow mode toggle
- [ ] Multi-container log aggregation
- [ ] Log highlighting (errors in red)

### 4.4 File Operations
- [ ] Copy files to container
- [ ] Copy files from container
- [ ] Drag & drop file transfer
- [ ] Edit files in container (simple editor)
- [ ] Download folder as zip

### 4.5 Container Comparison
- [ ] Diff two containers
- [ ] Compare configs
- [ ] Compare filesystem changes

---

## Phase 5: Volume & Network Completion

### 5.1 Volume Enhancements
- [ ] Copy data to volume
- [ ] Copy data from volume
- [ ] Backup volume to tar.gz
- [ ] Restore volume from backup
- [ ] Clone volume
- [ ] Volume size statistics
- [ ] Prune confirmation with size

### 5.2 Network Enhancements
- [ ] Connect container to network (runtime)
- [ ] Disconnect container from network
- [ ] Network topology visualization
- [ ] Create overlay networks
- [ ] Network aliases management

---

## Phase 6: Kubernetes Enhancement

### 6.1 Additional Resources
- [ ] ConfigMaps view/edit
- [ ] Secrets view (masked)
- [ ] StatefulSets management
- [ ] DaemonSets management
- [ ] Jobs and CronJobs
- [ ] Ingress management
- [ ] PersistentVolumeClaims

### 6.2 Operations
- [ ] Deployment rollback
- [ ] Rolling restart
- [ ] Edit YAML inline
- [ ] Apply YAML files
- [ ] Delete with grace period

### 6.3 Cluster Overview
- [ ] Node information
- [ ] Cluster events
- [ ] Resource quotas
- [ ] Namespace management

---

## Phase 7: Dashboard & Monitoring (Differentiator)

### 7.1 Dashboard View
- [ ] System overview (CPU, RAM, disk)
- [ ] Container count by status
- [ ] Recent activity feed
- [ ] Quick action buttons
- [ ] Resource alerts
- [ ] Favorite containers

### 7.2 Enhanced Monitoring
- [ ] Historical charts (1h, 24h, 7d)
- [ ] Per-container graphs
- [ ] Network throughput graphs
- [ ] Disk I/O graphs
- [ ] Export metrics (CSV/JSON)

### 7.3 Alerts System
- [ ] CPU threshold alerts
- [ ] Memory threshold alerts
- [ ] Container stopped alerts
- [ ] Health check failure alerts
- [ ] macOS notifications integration

---

## Phase 8: Plugin System (Long-term Differentiator)

### 8.1 Plugin Architecture
- [ ] Define plugin API (Rust traits)
- [ ] Plugin manifest format
- [ ] Plugin isolation (sandboxed)
- [ ] Plugin settings UI
- [ ] Enable/disable plugins

### 8.2 Built-in Plugins
- [ ] Database browser (postgres, mysql, redis)
- [ ] Log aggregator
- [ ] Backup scheduler
- [ ] Portainer-lite (remote Docker)

### 8.3 Plugin Marketplace
- [ ] GitHub-based distribution
- [ ] Plugin discovery
- [ ] Version management
- [ ] Community plugins

---

## Phase 9: Quality of Life

### 9.1 Context Menus
- [ ] Right-click on containers
- [ ] Right-click on images
- [ ] Right-click on volumes/networks
- [ ] Consistent actions across app

### 9.2 Multi-Select Operations
- [ ] Select multiple containers
- [ ] Bulk start/stop/restart/delete
- [ ] Bulk export
- [ ] Select all / deselect

### 9.3 Search & Filter
- [ ] Global search (Cmd+/)
- [ ] Filter by status
- [ ] Filter by labels
- [ ] Filter by Compose project
- [ ] Save filter presets

### 9.4 Preferences
- [ ] Configurable refresh intervals
- [ ] Default terminal shell
- [ ] Default log tail lines
- [ ] Startup behavior
- [ ] Notification preferences

---

## Phase 10: Polish & Ecosystem

### 10.1 Documentation
- [ ] In-app help tooltips
- [ ] Keyboard shortcut cheatsheet
- [ ] First-run onboarding
- [ ] Video tutorials

### 10.2 CLI Companion
- [ ] `dockside` CLI tool
- [ ] Open container in app from terminal
- [ ] Quick actions from CLI
- [ ] Integration with docker CLI

### 10.3 Integrations
- [ ] VS Code extension (open in Dockside)
- [ ] Alfred/Raycast workflow
- [ ] AppleScript support
- [ ] URL scheme (dockside://container/xxx)

---

## Implementation Priority Matrix

| Priority | Feature | Effort | Impact | Competitive Advantage |
|----------|---------|--------|--------|----------------------|
| P0 | Keyboard Shortcuts | Medium | High | Table stakes |
| P0 | Command Palette | Medium | High | Major UX win |
| P0 | Quick Actions Menu Bar | Low | High | OrbStack parity |
| P1 | Local Domain Names | High | Very High | OrbStack killer feature |
| P1 | Automatic HTTPS | High | High | OrbStack parity |
| P1 | Build from Dockerfile | Medium | High | Basic completeness |
| P1 | Container Templates | Medium | High | Unique feature |
| P2 | Dashboard View | Medium | Medium | Differentiator |
| P2 | Log Search/Export | Low | Medium | Power users |
| P2 | Registry Integration | Medium | Medium | Completeness |
| P2 | Volume Backup | Medium | Medium | Unique feature |
| P3 | Plugin System | Very High | Very High | Long-term moat |
| P3 | Historical Charts | High | Medium | Nice to have |
| P3 | Network Visualization | High | Low | Nice to have |

---

## Quick Wins (Implement First)

1. **Keyboard shortcuts** - Low effort, high impact
2. **Command palette** - Medium effort, very high impact
3. **Menu bar quick actions** - Low effort, high visibility
4. **Context menus** - Low effort, expected feature
5. **Log search** - Low effort, frequently needed
6. **Container templates** - Medium effort, unique value

---

## Competitive Messaging

### vs Docker Desktop
- "10x faster startup, zero licensing fees"
- "Native performance without the bloat"
- "All the features you need, none you don't"
- "Free for everyone, forever"

### vs OrbStack
- "Same speed, completely free"
- "Open source - audit, contribute, customize"
- "No subscription, no vendor lock-in"
- "Community-driven development"

---

## Success Metrics

1. **Startup time**: <3 seconds (match OrbStack)
2. **Idle CPU**: <0.5% (match OrbStack)
3. **Memory usage**: <200MB idle
4. **Feature parity**: 90% of common Docker Desktop features
5. **GitHub stars**: Community validation

---

## Resource Estimates

| Phase | Complexity | Approximate Scope |
|-------|------------|-------------------|
| Phase 1 | Medium | Core UX foundation |
| Phase 2 | High | Networking (requires system integration) |
| Phase 3 | Medium | Image management completion |
| Phase 4 | Medium | Container power features |
| Phase 5 | Low | Volume/network completion |
| Phase 6 | Medium | Kubernetes enhancement |
| Phase 7 | Medium | Dashboard and monitoring |
| Phase 8 | Very High | Plugin architecture |
| Phase 9 | Low | Quality of life |
| Phase 10 | Medium | Polish and ecosystem |

---

## Recommended Starting Point

Start with **Phase 1** (Performance & Core UX) as it provides:
1. Immediate user experience improvement
2. Foundation for all other features
3. Quick wins to show progress
4. Competitive parity on basics

Then prioritize **Phase 2.1** (Local Domain Names) as it's OrbStack's most praised feature and a major differentiator.
