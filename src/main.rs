use devtray::bridge::TaskManagerBridge;
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_dir = PathBuf::from(&home)
        .join(".cache")
        .join("devtray")
        .join("logs");
    let broadcaster = LogBroadcaster::new(log_dir, 1000);
    let process_manager = ProcessManager::new(broadcaster.clone());
    let config_manager = ConfigManager::new();
    let _bridge = TaskManagerBridge::with_managers(config_manager, process_manager, broadcaster);

    println!("DevTray initialized successfully.");
}
