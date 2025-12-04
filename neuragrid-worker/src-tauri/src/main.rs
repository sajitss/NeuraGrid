mod connection;
mod hardware;
mod runner;
use connection::ConnectionManager;
use hardware::HardwareMonitor;


fn main() {
  tracing_subscriber::fmt::init();
  tauri::Builder::default()
    .setup(|app| {
        let handle = app.handle().clone();
        tauri::async_runtime::spawn(async move {
            let manager = ConnectionManager::new(handle.clone(), "ws://127.0.0.1:3000/ws".to_string());
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
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
