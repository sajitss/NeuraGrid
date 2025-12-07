# NeuraGrid Worker Guide

The NeuraGrid Worker is the computational node of the network. It executes jobs dispatched by the Coordinator.

## 🚀 Key Features
- **Auto-Discovery**: Automatically detects available GPUs (CUDA, Vulkan, DX12).
- **Silent Mode**: Temporarily pause job acceptance without disconnecting.
- **Weekly Scheduler**: Define allowed operating hours.
- **Persistent Config**: Remembers your settings and identity.

## 🛠️ Running the Worker

### Quick Start (Release Binary)
The most efficient way to run the worker is using the compiled release binary.

```powershell
# Navigate to the release folder
cd c:\work2\ACX\neuragrid-worker\src-tauri\target\release

# Run with custom name and coordinator URL
.\neuragrid-worker.exe --name "Titan-Worker" --url "ws://127.0.0.1:3000/ws"
```

### CLI Arguments
| Argument | Description | Default |
| :--- | :--- | :--- |
| `--name` | Custom name for this worker shown in the dashboard. | `Worker-{HardwareID}` |
| `--url` | WebSocket URL of the Coordinator. | `ws://localhost:3000/ws` |
| `--help` | Show all available options. | - |

> **Note**: These values are prioritized over the saved configuration file.

---

##  UI Features

### 1. Silent Mode
Located in the header.
- **Active State**: Toggle is **Green** (Emerald) with text "ACTIVE". The worker is ready to accept jobs.
- **Silent State**: Toggle is **Grey** with text "SILENT". The worker will reject new jobs with a "Busy" status but maintains the connection.

### 2. Weekly Scheduler
Click the **Calendar Icon** <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" class="inline-block align-text-bottom"><rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect><line x1="16" y1="2" x2="16" y2="6"></line><line x1="8" y1="2" x2="8" y2="6"></line><line x1="3" y1="10" x2="21" y2="10"></line></svg> in the header to open the grid.
- **Grid**: 7 days x 24 hours.
- **Green Cell**: Allowed time.
- **Empty Cell**: Silent time (auto-reject jobs).
- **Save**: Changes are persisted to `worker_config.json`.

### 3. Status Badge
Displays the real-time connection status to the Coordinator (`Connected`, `Reconnecting`, `Disconnected`).

---

## ⚙️ Configuration
The worker creates a `worker_config.json` file in its app data directory to store:
- `name`
- `coordinator_url`
- `silent_mode` state
- `schedule` grid

This allows your settings to persist across restarts.
