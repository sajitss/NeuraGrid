# Capability Onboarding Guide

This guide explains how to add (onboard), deploy, and consume a new capability in the NeuraGrid Worker.

## 1. Onboarding a New Capability

To add a new capability (e.g., "Face Recognition" -> `Fr`), follow these steps:

### Step 1: Create the Module
Create a new directory `src-tauri/src/capabilities/fr/` and add a `mod.rs` file.

```rust
// src-tauri/src/capabilities/fr/mod.rs
use async_trait::async_trait;
use super::Capability;

pub struct FaceRecognition;

#[async_trait]
impl Capability for FaceRecognition {
    fn code(&self) -> &'static str {
        "Fr"
    }

    async fn is_supported(&self) -> bool {
        // Implement hardware check (e.g., check for NPU)
        true 
    }

    async fn execute(&self, args: Vec<String>) -> Result<String, String> {
        // Implement core logic here
        Ok("Face Recognition executed".to_string())
    }
}
```

### Step 2: Register the Capability
Update `src-tauri/src/capabilities/mod.rs` to include the new module.

```rust
pub mod fr; // Add this line
```

### Step 3: Add to a Profile
Add the capability to the appropriate profile(s) in `src-tauri/src/profiles/`. For example, to add it to the Standard profile:

```rust
// src-tauri/src/profiles/standard.rs
use crate::capabilities::{Capability, ..., fr};

pub fn get_capabilities() -> Vec<Box<dyn Capability>> {
    vec![
        ...,
        Box::new(fr::FaceRecognition),
    ]
}
```

## 2. Deployment

Once the code is added:

1.  **Build the Worker**: Run `npx tauri build` (or `cargo build --release` in `src-tauri`).
2.  **Distribute**: The resulting binary/installer includes the new capability.
3.  **Auto-Discovery**: When the worker starts, it loads the active profile. If the `is_supported()` check passes, the capability is considered "active" for that node.

## 3. Consumption

Capabilities are consumed via the Job System.

1.  **Job Submission**: The Coordinator (or a user) submits a job with `job_type` matching the capability code (or a mapping to it).
    *   *Note: The mapping logic in `runner.rs` needs to be updated to route jobs to `Capability::execute()`.*
2.  **Execution**: The worker receives the job, looks up the capability by code, and calls its `execute()` method with the provided arguments.
3.  **Results**: The `execute()` method returns a result, which is sent back to the Coordinator via WebSocket.
