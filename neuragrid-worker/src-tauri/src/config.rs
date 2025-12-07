use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkerConfig {
    pub name: Option<String>,
    pub coordinator_url: Option<String>,
    #[serde(default)]
    pub silent_mode: bool,
    #[serde(default = "default_schedule")]
    pub schedule: Vec<Vec<bool>>, // 7 days (index 0=Today) x 24 hours
}

fn default_schedule() -> Vec<Vec<bool>> {
    // Default: All days, All hours enabled
    vec![vec![true; 24]; 7]
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            name: None,
            coordinator_url: None,
            silent_mode: false,
            schedule: default_schedule(),
        }
    }
}

impl WorkerConfig {
    fn get_config_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
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
