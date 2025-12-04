# Directory Structure for Worker Capabilities

This document outlines the directory structure for integrating worker capabilities (Pd, Pr, Ot, etc.) into the NeuraGrid Worker codebase.

## 1. Principles

*   **Modularity**: Each capability (e.g., People Detection) is a self-contained module.
*   **Pseudonyms**: Use the 2-letter codes (Pd, Pr, Ot) as directory names for consistency with the UI and protocol.
*   **Node Alignment**: Define "Node Profiles" that group capabilities for specific hardware or deployment scenarios (e.g., "Edge Camera", "Compute Cluster").

## 2. Structure

The worker capabilities are organized under `src-tauri/src`:

```text
neuragrid-worker/
└── src-tauri/
    └── src/
        ├── main.rs                 # Entry point
        ├── runner.rs               # Job runner
        ├── hardware.rs             # Hardware detection
        ├── connection.rs           # WebSocket connection
        │
        ├── capabilities/           # Core logic for each capability
        │   ├── mod.rs              # Exports modules and defines Capability trait
        │   ├── pd/                 # People Detection (Pd)
        │   ├── pr/                 # Person Re-ID (Pr)
        │   ├── ot/                 # Object Tracking (Ot)
        │   ├── vd/                 # Vehicle Detection (Vd)
        │   ├── lp/                 # License Plate Rec (Lp)
        │   └── vs/                 # Video Search (Vs)
        │
        └── profiles/               # Node Type Definitions
            ├── mod.rs
            ├── standard.rs         # Standard Desktop (All capabilities enabled)
            ├── edge_jetson.rs      # Edge Node (Optimized for Jetson)
            └── server_gpu.rs       # Server Node (High throughput)
```

## 3. Implementation Details

### `capabilities/mod.rs`
Defines a common trait that all capabilities must implement.

```rust
// src/capabilities/mod.rs
use async_trait::async_trait;

#[async_trait]
pub trait Capability: Send + Sync {
    /// Returns the 2-letter code (e.g., "Pd")
    fn code(&self) -> &'static str;
    
    /// Checks if the hardware supports this capability
    async fn is_supported(&self) -> bool;
    
    /// Executes a job for this capability
    async fn execute(&self, args: Vec<String>) -> Result<String, String>;
}
```

### `profiles/mod.rs`
Selects the active profile based on configuration or hardware detection.

```rust
// src/profiles/mod.rs
pub fn get_active_capabilities() -> Vec<Box<dyn Capability>> {
    // Logic to return the set of capabilities for this node
    // e.g., if running on Jetson, return edge_jetson::capabilities()
}
```
