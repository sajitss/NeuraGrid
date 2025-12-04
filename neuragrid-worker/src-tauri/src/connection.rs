use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tauri::Emitter;
use tracing::{info, error, warn};

pub struct ConnectionManager {
    url: String,
    app_handle: tauri::AppHandle,
}

impl ConnectionManager {
    pub fn new(app_handle: tauri::AppHandle, url: String) -> Self {
        Self { app_handle, url }
    }

    pub async fn start(&self) {
        let url = self.url.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            info!("ConnectionManager task started");
            loop {
                info!("Connecting to {}", url);
                match tokio::time::timeout(Duration::from_secs(5), connect_async(&url)).await {
                    Ok(Ok((ws_stream, _))) => {
                        info!("Connected to WebSocket");
                        let _ = app_handle.emit("connection-status", "Connected");
                        
                        let (mut write, mut read) = ws_stream.split();
                        
                        let mut interval = tokio::time::interval(Duration::from_secs(5));
                        
                        loop {
                            tokio::select! {
                                msg = read.next() => {
                                    match msg {
                                        Some(Ok(Message::Text(text))) => {
                                            info!("Received: {}", text);
                                            // Parse job and run it
                                            match serde_json::from_str::<crate::runner::Job>(&text) {
                                                Ok(job) => {
                                                    let app_handle_clone = app_handle.clone();
                                                    tokio::spawn(async move {
                                                        let runner = crate::runner::JobRunner::new(app_handle_clone);
                                                        runner.run_job(job).await;
                                                    });
                                                }
                                                Err(e) => {
                                                    error!("Failed to parse job: {}", e);
                                                    let _ = app_handle.emit("job-status", format!("Error parsing job: {}", e));
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
                                _ = interval.tick() => {
                                    if write.send(Message::Ping(vec![])).await.is_err() {
                                        error!("Failed to send ping");
                                        break;
                                    }
                                    // Ensure UI is synced
                                    let _ = app_handle.emit("connection-status", "Connected");
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
