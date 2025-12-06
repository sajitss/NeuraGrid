use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WorkerConfig {
    pub name: Option<String>,
    pub coordinator_url: Option<String>,
}

impl WorkerConfig {
    fn get_config_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
        // Use app_local_data_dir or app_config_dir. 
        // In Tauri v2, we access paths via the path manager.
        // Assuming app_handle.path() is available or a similar mechanism.
        // For 2.0.0-rc, it's typically app_handle.path().app_config_dir()
        
        let path_resolver = app_handle.path();
        match path_resolver.app_config_dir() {
            Ok(mut path) => {
                path.push("worker_config.json");
                Some(path)
            }
            Err(_) => None,
        }
    }

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        if let Some(path) = Self::get_config_path(app_handle) {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        if let Some(path) = Self::get_config_path(app_handle) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
            fs::write(path, content).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Could not resolve config path".to_string())
        }
    }
}
