mod connection;
mod hardware;
mod runner;
mod capabilities;
mod profiles;
use connection::ConnectionManager;
use hardware::HardwareMonitor;


use clap::Parser;
mod config;
use config::WorkerConfig;

use std::sync::{Arc, RwLock};
use tauri::State;
use tauri::Manager;

struct AppState {
    config: Arc<RwLock<WorkerConfig>>,
}

#[tauri::command]
fn get_config(state: State<AppState>) -> WorkerConfig {
    state.config.read().unwrap().clone()
}

#[tauri::command]
fn save_config(app_handle: tauri::AppHandle, state: State<AppState>, new_config: WorkerConfig) -> Result<(), String> {
    {
        let mut config = state.config.write().unwrap();
        *config = new_config.clone();
    } // Drop lock
    new_config.save(&app_handle).map_err(|e| e.to_string())
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
   #[arg(long)]
   name: Option<String>,
   #[arg(long)]
   url: Option<String>,
}

fn main() {
  let cli_args = Args::parse();
  tracing_subscriber::fmt::init();
  tauri::Builder::default()
    .setup(move |app| {
        let handle = app.handle().clone();
        
        // Load config
        let mut config = WorkerConfig::load(&handle);
        let mut config_changed = false;
        
        if let Some(name) = cli_args.name.clone() {
             config.name = Some(name);
             config_changed = true;
        }
        
        if let Some(url) = cli_args.url.clone() {
             config.coordinator_url = Some(url);
             config_changed = true;
        }
        
        if config_changed {
             if let Err(e) = config.save(&handle) {
                 eprintln!("Failed to save config: {}", e);
             }
        }

        // Determine final values before moving config
        let final_name = config.name.clone().unwrap_or_else(|| {
            let id = uuid::Uuid::new_v4().to_string();
            format!("Worker-{}", &id[0..4])
        });
        
        let final_url = config.coordinator_url.clone().unwrap_or_else(|| "ws://127.0.0.1:3000/ws".to_string());

        let app_config_state = Arc::new(RwLock::new(config));
        app.manage(AppState { config: app_config_state.clone() });

        tauri::async_runtime::spawn(async move {
            let manager = ConnectionManager::new(handle.clone(), final_url, final_name, app_config_state);
            manager.start().await;
        });

        let handle2 = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let monitor = HardwareMonitor::new(handle2);
            let _info = monitor.scan().await;
        });

        /*
        let handle3 = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            let runner = JobRunner::new(handle3);
            // Example job
            runner.run_job(Job {
                id: "test-job-1".to_string(),
                job_type: "custom".to_string(),
                command: "cmd".to_string(), // Use cmd /c echo on Windows
                args: vec!["/C".to_string(), "echo Hello from NeuraGrid Runner".to_string()],
                env: HashMap::new(),
            }).await;
        });
        */

        Ok(())
    })
    .invoke_handler(tauri::generate_handler![get_config, save_config])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
