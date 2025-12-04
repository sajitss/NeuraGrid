use serde::{Serialize, Deserialize};
use tokio::process::Command;
use std::process::Stdio;
use tauri::Emitter;
use tracing::{info, error};
use wgpu;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Job {
    #[serde(default)]
    pub id: String,
    pub job_type: String, // "inference", "training", "custom"
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

pub struct JobRunner {
    app_handle: tauri::AppHandle,
}

impl JobRunner {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    pub async fn run_job(&self, job: Job) {
        info!("Starting job: {}", job.id);
        let _ = self.app_handle.emit("job-status", format!("Starting job {} ({})", job.id, job.job_type));

        if job.job_type == "prime_search" {
            let start: u64 = job.args.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
            let end: u64 = job.args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            
            let _ = self.app_handle.emit("job-status", format!("Job {}: Searching primes {}-{}", job.id, start, end));
            
            // Simulate heavy compute
            let count = tokio::task::spawn_blocking(move || {
                let mut c = 0;
                for i in start..end {
                    if is_prime(i) {
                        c += 1;
                    }
                }
                c
            }).await.unwrap_or(0);

            let _ = self.app_handle.emit("job-status", format!("Job {} finished: Found {} primes", job.id, count));
            info!("Job {} finished: Found {} primes", job.id, count);
            return;
        }

        if job.job_type == "gpu_test" {
            let _ = self.app_handle.emit("job-status", format!("Job {}: Initializing GPU test...", job.id));
            
            // Spawn blocking to avoid freezing main thread during GPU init
            let result = tokio::task::spawn_blocking(move || {
                futures::executor::block_on(async {
                    let instance = wgpu::Instance::default();
                    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
                        power_preference: wgpu::PowerPreference::HighPerformance,
                        force_fallback_adapter: false,
                        compatible_surface: None,
                    }).await.ok_or("No suitable GPU adapter found")?;

                    let info = adapter.get_info();
                    let result_msg = format!("Selected GPU: {:?} ({:?})", info.name, info.backend);
                    
                    // Simple validation: request device
                    let _ = adapter.request_device(&wgpu::DeviceDescriptor::default(), None).await?;
                    
                    Ok::<String, Box<dyn std::error::Error + Send + Sync>>(result_msg)
                })
            }).await.unwrap_or_else(|e| Err(format!("Join error: {}", e).into()));

            match result {
                Ok(msg) => {
                    let _ = self.app_handle.emit("job-status", format!("Job {} success: {}", job.id, msg));
                    info!("Job {} success: {}", job.id, msg);
                }
                Err(e) => {
                    let _ = self.app_handle.emit("job-status", format!("Job {} failed: {}", job.id, e));
                    error!("Job {} failed: {}", job.id, e);
                }
            }
            return;
        }

        if job.job_type == "string_search" {
            let file_path = match job.args.get(0) {
                Some(p) => p.clone(),
                None => {
                    let _ = self.app_handle.emit("job-status", format!("Job {} failed: Missing file path arg", job.id));
                    return;
                }
            };
            
            let search_terms: Vec<String> = job.args.iter().skip(1).cloned().collect();
            if search_terms.is_empty() {
                 let _ = self.app_handle.emit("job-status", format!("Job {} failed: No search terms provided", job.id));
                 return;
            }

            let _ = self.app_handle.emit("job-status", format!("Job {}: Searching in {} for {:?}", job.id, file_path, search_terms));

            let result = tokio::task::spawn_blocking(move || {
                use std::fs::File;
                use std::io::{BufRead, BufReader};

                let file = File::open(file_path).map_err(|e| e.to_string())?;
                let mut reader = BufReader::new(file);
                let mut matches = 0;
                let mut buffer = Vec::new();

                loop {
                    buffer.clear();
                    match reader.read_until(b'\n', &mut buffer) {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            let line = String::from_utf8_lossy(&buffer);
                            for term in &search_terms {
                                if line.contains(term) {
                                    matches += 1;
                                }
                            }
                        }
                        Err(e) => return Err(e.to_string()),
                    }
                }
                Ok::<usize, String>(matches)
            }).await.unwrap_or_else(|e| Err(format!("Join error: {}", e)));

            match result {
                Ok(count) => {
                    let _ = self.app_handle.emit("job-status", format!("Job {} finished: Found {} matches", job.id, count));
                    info!("Job {} finished: Found {} matches", job.id, count);
                }
                Err(e) => {
                    let _ = self.app_handle.emit("job-status", format!("Job {} failed: {}", job.id, e));
                    error!("Job {} failed: {}", job.id, e);
                }
            }
            return;
        }

        // Security: In a real app, validate command path and args against allowlist
        // For now, we assume the coordinator is trusted or we are running in a sandbox
        
        let mut cmd = Command::new(&job.command);
        cmd.args(&job.args);
        cmd.envs(&job.env);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Windows Job Object or similar sandboxing would go here
        
        match cmd.spawn() {
            Ok(mut child) => {
                let _ = self.app_handle.emit("job-status", format!("Job {} running", job.id));
                
                // Stream stdout/stderr
                // In a real implementation, we would read lines and emit events
                
                match child.wait().await {
                    Ok(status) => {
                        info!("Job {} finished with status: {}", job.id, status);
                        let _ = self.app_handle.emit("job-status", format!("Job {} finished: {}", job.id, status));
                    }
                    Err(e) => {
                        error!("Job {} failed to wait: {}", job.id, e);
                        let _ = self.app_handle.emit("job-status", format!("Job {} error: {}", job.id, e));
                    }
                }
            }
            Err(e) => {
                error!("Failed to spawn job {}: {}", job.id, e);
                let _ = self.app_handle.emit("job-status", format!("Job {} spawn failed: {}", job.id, e));
            }
        }
    }
}

fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    for i in 2..=(n as f64).sqrt() as u64 {
        if n % i == 0 { return false; }
    }
    true
}
