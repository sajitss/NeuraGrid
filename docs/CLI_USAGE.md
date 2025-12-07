# NeuraGrid CLI Usage Guide

The `neuragrid-cli` is a command-line tool for interacting with the NeuraGrid system. It allows you to submit jobs, tag them for organization, and monitor their progress in real-time.

## Installation

If you haven't built the CLI yet:

```bash
cd neuragrid-cli
cargo build --release
# The binary will be at target/release/neuragrid-cli.exe
```

## Global Options

These options apply to all commands:

*   `--url <URL>`: The base URL of the Coordinator.
    *   **Default**: `http://localhost:3000`
    *   **Example**: `--url "http://192.168.1.50:3000"`

## Commands

### 1. `submit`

Submits a new job to the grid.

**Syntax:**
```bash
neuragrid-cli submit --type <TYPE> --args <ARGS>... [OPTIONS]
```

**Options:**
*   `--type <TYPE>`: The type of job to run (e.g., `string_search`, `prime_search`, `Lp`, `Pd`).
*   `--args <ARGS>...`: A list of arguments required by the job type.
*   `--tags <TAG>...`: One or more hash tags to categorize the job (e.g., `#urgent`, `#batch-1`).
*   `--wait`: If specified, the CLI will wait for the job to complete and print the result.
*   `--target <WORKER>`: (Optional) Target a specific worker by name.

**Examples:**

**Submit a String Search Job:**
```powershell
.\neuragrid-cli submit --type string_search --args "C:\logs\error.log" "CRITICAL" --tags "#logs"
```

**Submit a License Plate Recognition (LPR) Job and Wait:**
```powershell
.\neuragrid-cli submit --type Lp --args "KA01AB1234" "http://example.com/video.mp4" --wait
```

**Submit with Multiple Tags:**
```powershell
.\neuragrid-cli submit --type prime_search --args 1 10000 --tags "#math" --tags "#demo"
```

---

### 2. `listen`

Monitors the grid for jobs with a specific tag and waits until all of them are completed. This is useful for scripting batch workflows.

**Syntax:**
```bash
neuragrid-cli listen --tag <TAG>
```

**Options:**
*   `--tag <TAG>`: The tag to listen for. **Note:** In PowerShell, you must quote the tag (e.g., `"#urgent"`) to prevent it from being interpreted as a comment.

**Example:**

1.  **Submit a batch of jobs:**
    ```powershell
    .\neuragrid-cli submit --type Lp --args "..." --tags "#batch-001"
    .\neuragrid-cli submit --type Lp --args "..." --tags "#batch-001"
    ```

2.  **Wait for the batch to finish:**
    ```powershell
    .\neuragrid-cli listen --tag "#batch-001"
    ```

**Output:**
```text
Listening for jobs with tag: #batch-001
Found 2 pending jobs for tag '#batch-001'
Connected to event stream. Waiting for updates...
[PROCESSING] Job picked up by Worker-Titan
[COMPLETED] Job ... completed by Worker-Titan
Remaining jobs for '#batch-001': 1
[PROCESSING] Job picked up by Worker-Alpha
[COMPLETED] Job ... completed by Worker-Alpha
Remaining jobs for '#batch-001': 0
All jobs for tag '#batch-001' finished. Exiting.
```
