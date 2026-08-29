use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use devtray::core::single_instance::{SingleInstance, SingleInstanceStatus};
use devtray::gui::bridge::SlintAppController;
use devtray::gui::tray::DevTraySysTray;
use devtray::MainWindow;
use slint::ComponentHandle;
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Single Instance Check: ensure only 1 DevTray runs and prevent duplicate tray icons
    let socket_path = SingleInstance::default_socket_path();
    let mut single_instance_guard = match SingleInstance::acquire(&socket_path) {
        Ok(SingleInstanceStatus::Primary(guard)) => Some(guard),
        Ok(SingleInstanceStatus::Secondary) => {
            println!("DevTray is already running. Signaled existing instance to show window.");
            return Ok(());
        }
        Err(e) => {
            eprintln!("[SingleInstance] Warning: Failed to acquire single instance lock: {e}");
            None
        }
    };

    // 2. Default to software renderer for ultra-low memory usage (~25MB RSS)
    if std::env::var("SLINT_BACKEND").is_err() {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }
    if std::env::var("TOKIO_WORKER_THREADS").is_err() {
        std::env::set_var("TOKIO_WORKER_THREADS", "1");
    }

    // 3. Initialize LogBroadcaster, ProcessManager, ConfigManager, SlintAppController
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_dir = std::path::PathBuf::from(&home)
        .join(".cache")
        .join("devtray")
        .join("logs");
    let broadcaster = LogBroadcaster::new(log_dir, 1000);
    let process_mgr = ProcessManager::new(broadcaster.clone());
    let config_mgr = ConfigManager::new();

    let controller = Arc::new(SlintAppController::new(config_mgr, process_mgr, broadcaster));

    // 4. Initialize MainWindow and call controller.bind_to_ui(&main_window)
    let main_window = MainWindow::new()?;
    controller.bind_to_ui(&main_window);

    // 5. Call controller.refresh_tasks(&main_window)
    controller.refresh_tasks(&main_window);

    // 6. Listen for secondary instances requesting to show the window
    if let Some(guard) = &mut single_instance_guard {
        let ui_weak = main_window.as_weak();
        guard.start_listener(move || {
            let ui_weak = ui_weak.clone();
            slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.show().ok();
                }
            })
            .ok();
        });
    }

    // 7. Spawn DevTraySysTray::spawn(...) for system tray integration
    let _tray_handle = DevTraySysTray::spawn(controller.clone(), main_window.as_weak());

    // Release any transient startup initialization heap allocations
    #[cfg(target_os = "linux")]
    unsafe {
        libc::malloc_trim(0);
    }

    // 8. Setup a slint::Timer to periodically sync running process status every 1 second
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

    // 9. Intercept window close to hide window so clicking X keeps the app running in system tray
    let ui_weak = main_window.as_weak();
    main_window.window().on_close_requested(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.hide().ok();
        }
        slint::CloseRequestResponse::HideWindow
    });

    // 10. Show main window and run event loop until explicit quit (from tray menu or quit dialog)
    main_window.show()?;
    let run_res = slint::run_event_loop_until_quit();

    // 11. Ensure controller.stop_all() is executed on application termination
    controller.stop_all();

    run_res?;
    Ok(())
}
