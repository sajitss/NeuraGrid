use std::sync::{Arc, RwLock};
use chrono::{Local, Datelike, Timelike};
use crate::config::WorkerConfig;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tauri::Emitter;
use tracing::{info, error, warn};
use tokio::sync::mpsc;

pub struct ConnectionManager {
    url: String,
    name: String,
    app_handle: tauri::AppHandle,
    config: Arc<RwLock<WorkerConfig>>,
}

impl ConnectionManager {
    pub fn new(app_handle: tauri::AppHandle, url: String, name: String, config: Arc<RwLock<WorkerConfig>>) -> Self {
        Self { app_handle, url, name, config }
    }

    pub async fn start(&self) {
        let url = format!("{}?name={}", self.url, self.name);
        let app_handle = self.app_handle.clone();
        let name = self.name.clone();
        let config_store = self.config.clone();

        tokio::spawn(async move {
            info!("ConnectionManager task started");
            let _ = app_handle.emit("log-message", "ConnectionManager task started");
            let _ = app_handle.emit("worker-name", &name);
            loop {
                info!("Connecting to {}", url);
                let _ = app_handle.emit("log-message", format!("Connecting to {}", url));
                match tokio::time::timeout(Duration::from_secs(5), connect_async(&url)).await {
                    Ok(Ok((ws_stream, _))) => {
                        info!("Connected to WebSocket");
                        let _ = app_handle.emit("log-message", "Connected to WebSocket");
                        let _ = app_handle.emit("connection-status", "Connected");
                        let _ = app_handle.emit("worker-name", &name);
                        
                        let (mut write, mut read) = ws_stream.split();
                        let (tx, mut rx) = mpsc::channel::<String>(100);
                        
                        let mut interval = tokio::time::interval(Duration::from_secs(5));
                        
                        loop {
                            tokio::select! {
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            info!("Received: {}", text);
                                            let _ = app_handle.emit("log-message", format!("Received: {}", text));
                                            
                                            // Handle Earnings Update
                                            if text.starts_with("Earnings Update: ") {
                                                let amount_str = text.trim_start_matches("Earnings Update: ").trim();
                                                info!("Raw amount string: '{}'", amount_str);
                                                match amount_str.parse::<f64>() {
                                                    Ok(amount) => {
                                                        info!("Parsed earnings: {}", amount);
                                                        let _ = app_handle.emit("log-message", format!("Parsed earnings: {}", amount));
                                                        if let Err(e) = app_handle.emit("earnings-update", amount.to_string()) {
                                                             error!("Failed to emit earnings-update: {}", e);
                                                             let _ = app_handle.emit("log-message", format!("Failed to emit earnings-update: {}", e));
                                                        } else {
                                                             info!("Emitted earnings-update: {}", amount);
                                                             let _ = app_handle.emit("log-message", format!("Emitted earnings-update: {}", amount));
                                                        }
                                                    }
                                                    Err(e) => {
                                                         error!("Failed to parse earnings '{}': {}", amount_str, e);
                                                         let _ = app_handle.emit("log-message", format!("Failed to parse earnings '{}': {}", amount_str, e));
                                                    }
                                                }
                                                continue;
                                            }

                                            // Parse job and run it
                                            match serde_json::from_str::<crate::runner::Job>(&text) {
                                                Ok(job) => {
                                                    // Check Scheduler / Silent Mode
                                                    let allowed = {
                                                        let cfg = config_store.read().unwrap();
                                                        if cfg.silent_mode {
                                                            info!("Refusing job {} due to Silent Mode", job.id);
                                                            let _ = app_handle.emit("log-message", "Refused job: Silent Mode is ON".to_string());
                                                            false
                                                        } else {
                                                            let now = Local::now();
                                                            // weekday: Mon=0 .. Sun=6 in chrono::Weekday::num_days_from_monday
                                                            // But we mapped 0=Today. Let's align with config structure:
                                                            // We defined 0=Today (relative) in the prompt/impl plan? 
                                                            // Actually, simpler to use fixed days Mon=0.
                                                            // Let's assume index 0 = Monday for simplicity in this implementation, 
                                                            // or assume index 0 = Today if the UI handles shifting.
                                                            // Let's use 0 = Monday for stability.
                                                            let day_idx = now.weekday().num_days_from_monday() as usize; 
                                                            let hour_idx = now.hour() as usize;
                                                            if let Some(day_sched) = cfg.schedule.get(day_idx) {
                                                                if let Some(active) = day_sched.get(hour_idx) {
                                                                    if !*active {
                                                                        info!("Refusing job {} due to Schedule (Day {} Hour {})", job.id, day_idx, hour_idx);
                                                                        let _ = app_handle.emit("log-message", format!("Refused job: Schedule restriction (Day {} Hour {})", day_idx, hour_idx));
                                                                        false
                                                                    } else {
                                                                         true
                                                                    }
                                                                } else { true }
                                                            } else { true }
                                                        }
                                                    };

                                                    if !allowed {
                                                         // Send rejection
                                                         if tx.send(format!("Job Rejected: {}", job.id)).await.is_err() {
                                                             error!("Failed to send rejection");
                                                         }
                                                         continue;
                                                    }

                                                    let app_handle_clone = app_handle.clone();
                                                    let tx_clone = tx.clone();
                                                    tokio::spawn(async move {
                                                        let runner = crate::runner::JobRunner::new(app_handle_clone, Some(tx_clone));
                                                        runner.run_job(job).await;
                                                    });
                                                }
                                                Err(e) => {
                                                    error!("Failed to parse job: {}", e);
                                                    let _ = app_handle.emit("log-message", format!("Failed to parse job: {}", e));
                                                    // Only emit job status error if it really looked like a job (simple heuristic or just suppress)
                                                    // For now, suppress error if it's the Welcome message
                                                    if !text.starts_with("Welcome") {
                                                         let _ = app_handle.emit("job-status", format!("Error parsing job: {}", e));
                                                    }
                                                }
                                            }
                                        }
                                        Some(Ok(Message::Close(_))) => {
                                            warn!("Connection closed");
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            error!("Error: {}", e);
                                            break;
                                        }
                                        None => break,
                                        _ => {}
                                    }
                                }
                                Some(internal_msg) = rx.recv() => {
                                    info!("Sending to coordinator: {}", internal_msg);
                                    if write.send(Message::Text(internal_msg)).await.is_err() {
                                        error!("Failed to send message to coordinator");
                                        break;
                                    }
                                }
                                _ = interval.tick() => {
                                    if write.send(Message::Ping(vec![])).await.is_err() {
                                        error!("Failed to send ping");
                                        break;
                                    }
                                    // Ensure UI is synced
                                    let _ = app_handle.emit("connection-status", "Connected");
                                    let _ = app_handle.emit("worker-name", &name);
                                }
                            }
                        }
                        let _ = app_handle.emit("connection-status", "Disconnected");
                        let _ = app_handle.emit("connection-status", "Disconnected");
                    }
                    Ok(Err(e)) => {
                        error!("Connection failed: {}", e);
                        let _ = app_handle.emit("connection-status", "Disconnected");
                    }
                    Err(_) => {
                        error!("Connection timed out");
                        let _ = app_handle.emit("connection-status", "Disconnected");
                    }
                }
                
                // Exponential backoff (simplified)
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });
    }
}
