use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use devtray::gui::bridge::SlintAppController;
use devtray::gui::tray::DevTraySysTray;
use devtray::MainWindow;
use slint::ComponentHandle;
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Default to software renderer for ultra-low memory usage (~25MB RSS)
    if std::env::var("SLINT_BACKEND").is_err() {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }
    if std::env::var("TOKIO_WORKER_THREADS").is_err() {
        std::env::set_var("TOKIO_WORKER_THREADS", "1");
    }

    // 2. Initialize LogBroadcaster, ProcessManager, ConfigManager, SlintAppController
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_dir = std::path::PathBuf::from(&home)
        .join(".cache")
        .join("devtray")
        .join("logs");
    let broadcaster = LogBroadcaster::new(log_dir, 1000);
    let process_mgr = ProcessManager::new(broadcaster.clone());
    let config_mgr = ConfigManager::new();

    let controller = Arc::new(SlintAppController::new(config_mgr, process_mgr, broadcaster));

    // 3. Initialize MainWindow and call controller.bind_to_ui(&main_window)
    let main_window = MainWindow::new()?;
    controller.bind_to_ui(&main_window);

    // 4. Call controller.refresh_tasks(&main_window)
    controller.refresh_tasks(&main_window);

    // 5. Spawn DevTraySysTray::spawn(...) for system tray integration
    let _tray_handle = DevTraySysTray::spawn(controller.clone(), main_window.as_weak());

    // 6. Setup a slint::Timer to periodically sync running process status every 1 second
    let timer = slint::Timer::default();
    {
        let c = controller.clone();
        let ui_weak = main_window.as_weak();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            },
        );
    }

    // 7. Intercept window close to hide window so clicking X keeps the app running in system tray
    main_window
        .window()
        .on_close_requested(|| slint::CloseRequestResponse::HideWindow);

    // 8. Call main_window.run()?
    let run_res = main_window.run();

    // 9. Ensure controller.stop_all() is executed on application termination
    controller.stop_all();

    run_res?;
    Ok(())
}
