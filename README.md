# NeuraGrid: Distributed AI Compute Network

NeuraGrid is a high-performance, cross-platform distributed computing system designed to harness the power of idle hardware for AI workloads. It connects a central **Coordinator** to a network of **Workers** (GPUs, CPUs, NPUs) to execute tasks ranging from simple string searches to complex LLM training and inference.

![NeuraGrid Dashboard](coordinator-dashboard/public/logo.png)

## 🚀 Key Features

*   **Cross-Platform Workers**: Runs on Windows, Linux, and macOS (Apple Silicon).
*   **Hardware Agnostic**: Supports NVIDIA GPUs (CUDA), AMD/Intel (OpenCL/Vulkan), ARM NPUs (Qualcomm Hexagon), and CPUs.
*   **Real-Time Dashboard**: "Mission Control" style interface for monitoring the grid.
*   **Live System Logs**: Real-time broadcasting of job submissions, assignments, and completions to the dashboard.
*   **Capability Profiling**: Automatically detects and tags workers with capabilities like People Detection (Pd), Object Tracking (Ot), etc.
*   **Persistent State**: SQLite-backed job history and worker tracking.
*   **Secure Communication**: WebSocket-based real-time command and control.

## 💡 How It Works

![NeuraGrid System Concept](assets/infographic.jpg)

## 🏗️ Architecture

The system consists of three main components:

1.  **Coordinator (`/coordinator`)**:
    *   **Role**: The brain of the operation. Manages connections, dispatches jobs, and persists state.
    *   **Tech Stack**: Rust (Axum), SQLite (SQLx), Tokio.
    *   **API**: REST API for job submission (`POST /job`) and WebSocket (`/ws`) for worker comms.
    *   **Dashboard**: Serves the Svelte frontend directly.

2.  **Worker (`/neuragrid-worker`)**:
    *   **Role**: The muscle. Executes jobs on local hardware.
    *   **Tech Stack**: Rust (Tauri), WGPU (Graphics/Compute), Sysinfo.
    *   **Features**: Auto-discovery of GPUs, real-time status reporting, secure job execution sandbox.

3.  **Dashboard (`/coordinator-dashboard`)**:
    *   **Role**: The eyes. Visualizes grid health and activity.
    *   **Tech Stack**: Svelte, TailwindCSS, Vite.
    *   **Features**: Live worker stats, real-time system logs, capability visualization, and job queue monitoring.

## 🛠️ Installation & Usage

### Prerequisites
*   **Rust**: Latest stable toolchain (`rustup update`).
*   **Node.js**: v18+ (for Dashboard build).
*   **Git**: For version control.

### 1. Build the Dashboard
The Coordinator serves the dashboard static files, so build this first.
```bash
cd coordinator-dashboard
npm install
npm run build
```

### 2. Start the Coordinator
```bash
cd coordinator
cargo run --bin neuragrid-coordinator
```
*   The Coordinator will start on `0.0.0.0:3000`.
*   Access the Dashboard at `http://localhost:3000`.

### 3. Start a Worker
Open a new terminal.
```bash
cd neuragrid-worker
npm install # Install Tauri CLI dependencies
npx tauri build # Build release binary
# OR run in dev mode:
npx tauri dev
```
*   The Worker will auto-connect to `ws://localhost:3000/ws`.
*   You should see it appear on the Dashboard immediately.

## 🧪 Submitting Jobs

You can submit jobs via HTTP POST to the Coordinator.

**Example: Distributed String Search**
```bash
curl -X POST http://localhost:3000/job \
  -H "Content-Type: application/json" \
  -d '{
    "job_type": "string_search",
    "args": ["C:\\path\\to\\file.txt", "search_term", "context_lines"]
  }'
    "args": ["C:\\path\\to\\file.txt", "search_term", "context_lines"]
  }'
```

**PowerShell Example:**
```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:3000/job" -Method Post -Body '{"job_type": "string_search", "args": ["C:\\temp\\test.txt", "Hello", "success"]}' -ContentType "application/json"
```

## 📚 Documentation

*   **[CLI Usage Guide](docs/CLI_USAGE.md)**: Detailed instructions for using the `neuragrid-cli`.
*   [Capability Onboarding Guide](docs/CAPABILITY_ONBOARDING.md): How to add new capabilities.
*   [Directory Structure](docs/DIRECTORY_STRUCTURE.md): Codebase organization for capabilities.

## 🧩 Worker Capabilities

Workers are profiled and tagged with specific capabilities, visualized in the dashboard:

| Code | Capability | Description |
| :--- | :--- | :--- |
| **Pd** | People Detection | Timestamps of person appearances. |
| **Pr** | Person Re-ID | Tracking specific individuals across cameras. |
| **Ot** | Object Tracking | Movement tracking across frames. |
| **Vd** | Vehicle Detection | Car appearance logging. |
| **Lp** | License Plate Rec | LPR / ANPR matching. |
| **Vs** | Video Search | Semantic query matching. |

## 🖥️ Hardware Support

NeuraGrid supports a wide range of hardware profiles, from "The Titan" (H100 clusters) to "The Edge AI Starter" (Jetson Nano).

## 📄 License

Proprietary / Internal Use Only.
