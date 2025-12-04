use wgpu::{Instance, Backends};
use sysinfo::System;
use serde::{Serialize, Deserialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub backend: String,
    pub driver: String,
    pub driver_info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub cpu_brand: String,
    pub cpu_cores: usize,
    pub memory_total: u64,
    pub memory_used: u64,
    pub gpus: Vec<GpuInfo>,
}

pub struct HardwareMonitor {
    app_handle: tauri::AppHandle,
}

impl HardwareMonitor {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    pub async fn scan(&self) -> SystemInfo {
        let mut sys = System::new_all();
        sys.refresh_all();

        let os_name = System::name().unwrap_or("Unknown".to_string());
        let os_version = System::os_version().unwrap_or("Unknown".to_string());
        let cpu_brand = sys.cpus().first().map(|cpu| cpu.brand().to_string()).unwrap_or_default();
        let cpu_cores = sys.cpus().len();
        let memory_total = sys.total_memory();
        let memory_used = sys.used_memory();

        let gpus = self.detect_gpus().await;

        let info = SystemInfo {
            os_name,
            os_version,
            cpu_brand,
            cpu_cores,
            memory_total,
            memory_used,
            gpus,
        };

        let _ = self.app_handle.emit("hardware-info", &info);
        info
    }

    async fn detect_gpus(&self) -> Vec<GpuInfo> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let mut gpus = Vec::new();

        println!("Scanning for GPUs...");
        for adapter in instance.enumerate_adapters(Backends::all()) {
            let info = adapter.get_info();
            println!("Found adapter: {:?} (Backend: {:?})", info.name, info.backend);
            
            gpus.push(GpuInfo {
                name: info.name,
                vendor: format!("{:?}", info.vendor),
                backend: format!("{:?}", info.backend),
                driver: info.driver,
                driver_info: info.driver_info,
            });
        }
        
        if gpus.is_empty() {
            println!("No GPUs found via wgpu enumeration. Trying request_adapter fallback...");
            // Fallback: try requesting a default adapter
            if let Some(adapter) = instance.request_adapter(&wgpu::RequestAdapterOptions::default()).await {
                 let info = adapter.get_info();
                 println!("Fallback adapter found: {:?}", info.name);
                 gpus.push(GpuInfo {
                    name: info.name,
                    vendor: format!("{:?}", info.vendor),
                    backend: format!("{:?}", info.backend),
                    driver: info.driver,
                    driver_info: info.driver_info,
                });
            }
        }

        println!("Total GPUs detected: {}", gpus.len());
        gpus
    }
}
