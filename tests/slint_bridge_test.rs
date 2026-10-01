use devtray::core::config::ConfigManager;
use devtray::core::i18n::Language;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use devtray::gui::bridge::{BridgeError, SlintAppController};
use devtray::MainWindow;
use slint::Model;
use std::time::Duration;
use tempfile::tempdir;

fn setup_test_controller() -> (SlintAppController, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let log_dir = dir.path().join("logs");

    let config = ConfigManager::with_path(config_file);
    let logs = LogBroadcaster::new(log_dir, 100);
    let process = ProcessManager::new(logs.clone());

    let controller = SlintAppController::new(config, process, logs);
    (controller, dir)
}

#[test]
fn test_bridge_task_conversion_and_ordering() {
    let (controller, _dir) = setup_test_controller();

    controller
        .add_task("Backend", "echo 1", ".", Some("Web"))
        .unwrap();
    controller
        .add_task("Frontend", "echo 2", ".", Some("Web"))
        .unwrap();
    controller
        .add_task("DB", "echo 3", ".", Some("Database"))
        .unwrap();
    controller
        .add_task("OneOff", "echo 4", ".", None)
        .unwrap();

    let items = controller.get_slint_task_items();
    assert_eq!(items.len(), 4);

    // Group "Database" (1 item: neither up nor down)
    assert_eq!(items[0].group.as_str(), "Database");
    assert_eq!(items[0].name.as_str(), "DB");
    assert!(!items[0].can_move_up);
    assert!(!items[0].can_move_down);

    // Group "Web" (2 items: Backend can move down, Frontend can move up)
    assert_eq!(items[1].group.as_str(), "Web");
    assert_eq!(items[1].name.as_str(), "Backend");
    assert!(!items[1].can_move_up);
    assert!(items[1].can_move_down);

    assert_eq!(items[2].group.as_str(), "Web");
    assert_eq!(items[2].name.as_str(), "Frontend");
    assert!(items[2].can_move_up);
    assert!(!items[2].can_move_down);

    // Uncategorized (1 item: neither up nor down)
    assert_eq!(items[3].group.as_str(), "");
    assert_eq!(items[3].name.as_str(), "OneOff");
    assert!(!items[3].can_move_up);
    assert!(!items[3].can_move_down);
}

#[test]
fn test_bridge_crud_and_persistence() {
    let (controller, dir) = setup_test_controller();
    let config_file = dir.path().join("config.json");

    assert_eq!(controller.tasks().len(), 0);

    // 1. Add task
    let created = controller
        .add_task("Server", "echo running", ".", Some("Backend"))
        .expect("add_task should succeed");
    assert_eq!(created.name, "Server");
    assert_eq!(created.command, "echo running");
    assert_eq!(created.working_directory, ".");
    assert_eq!(created.group.as_deref(), Some("Backend"));
    assert_eq!(controller.tasks().len(), 1);

    // Check persistence
    let cm_verify = ConfigManager::with_path(config_file.clone());
    let loaded = cm_verify.load().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, created.id);

    // 2. Validation error
    let empty_name = controller.add_task("", "echo 1", ".", None);
    assert!(matches!(empty_name, Err(BridgeError::ValidationError(_))));
    let empty_cmd = controller.add_task("Task", "", ".", None);
    assert!(matches!(empty_cmd, Err(BridgeError::ValidationError(_))));

    // 3. Update task
    let updated = controller
        .update_task(&created.id, "New Server", "echo updated", "/tmp", Some("API"))
        .expect("update_task should succeed");
    assert_eq!(updated.name, "New Server");
    assert_eq!(updated.command, "echo updated");
    assert_eq!(updated.working_directory, "/tmp");
    assert_eq!(updated.group.as_deref(), Some("API"));

    let loaded2 = cm_verify.load().unwrap();
    assert_eq!(loaded2[0].name, "New Server");

    // 4. Save task convenience
    // New task (empty id)
    let t2 = controller
        .save_task("", "Worker", "echo work", ".", None)
        .expect("save_task with empty id should add");
    assert_eq!(controller.tasks().len(), 2);

    // Update existing task
    let t2_updated = controller
        .save_task(&t2.id, "Worker Renamed", "echo work 2", ".", Some("BG"))
        .expect("save_task with id should update");
    assert_eq!(t2_updated.name, "Worker Renamed");

    // 5. Delete task
    controller.delete_task(&created.id).expect("delete should succeed");
    assert_eq!(controller.tasks().len(), 1);

    let non_existent = controller.delete_task("nonexistent-id");
    assert!(matches!(non_existent, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_move_task() {
    let (controller, dir) = setup_test_controller();
    let config_file = dir.path().join("config.json");

    let _t1 = controller.add_task("T1", "echo 1", ".", Some("G")).unwrap();
    let t2 = controller.add_task("T2", "echo 2", ".", Some("G")).unwrap();
    let t3 = controller.add_task("T3", "echo 3", ".", Some("G")).unwrap();

    assert_eq!(
        controller.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Move T2 up (-1) -> [T2, T1, T3]
    let moved = controller.move_task(&t2.id, -1).unwrap();
    assert!(moved);
    assert_eq!(
        controller.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T2", "T1", "T3"]
    );

    // Move T2 up again at boundary -> returns false
    let moved_again = controller.move_task(&t2.id, -1).unwrap();
    assert!(!moved_again);

    // Move T2 down (+1) -> [T1, T2, T3]
    let moved_down = controller.move_task(&t2.id, 1).unwrap();
    assert!(moved_down);
    assert_eq!(
        controller.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Move T3 down (+1) at boundary -> returns false
    let moved_t3_down = controller.move_task(&t3.id, 1).unwrap();
    assert!(!moved_t3_down);

    // Verify persistence
    let cm = ConfigManager::with_path(config_file);
    let loaded = cm.load().unwrap();
    assert_eq!(
        loaded.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Nonexistent task
    let err = controller.move_task("invalid", 1);
    assert!(matches!(err, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_reorder_task() {
    let (controller, _dir) = setup_test_controller();

    let _t1 = controller.add_task("T1", "echo 1", ".", Some("Backend")).unwrap();
    let _t2 = controller.add_task("T2", "echo 2", ".", Some("Backend")).unwrap();
    let t3 = controller.add_task("T3", "echo 3", ".", Some("Frontend")).unwrap();

    // Reorder T3 to index 0 -> [T3, T1, T2], adopting Backend group
    let reordered = controller.reorder_task(&t3.id, 0).unwrap();
    assert!(reordered);

    let tasks = controller.tasks();
    assert_eq!(
        tasks.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T3", "T1", "T2"]
    );
    assert_eq!(tasks[0].group.as_deref(), Some("Backend"));

    // Reorder nonexistent
    let err = controller.reorder_task("invalid", 0);
    assert!(matches!(err, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_process_management_and_groups() {
    let (controller, _dir) = setup_test_controller();

    let g1_t1 = controller
        .add_task("G1-1", "sleep 10", ".", Some("Alpha"))
        .unwrap();
    let g1_t2 = controller
        .add_task("G1-2", "sleep 10", ".", Some("Alpha"))
        .unwrap();
    let g2_t1 = controller
        .add_task("G2-1", "sleep 10", ".", Some("Beta"))
        .unwrap();

    assert_eq!(controller.running_count(), 0);

    // Start single task
    controller.start_task(&g2_t1.id).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(controller.is_task_running(&g2_t1.id));
    assert_eq!(controller.running_count(), 1);

    // Stop single task
    controller.stop_task(&g2_t1.id).unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(!controller.is_task_running(&g2_t1.id));
    assert_eq!(controller.running_count(), 0);

    // Start group "Alpha"
    controller.start_group("Alpha").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(controller.is_task_running(&g1_t1.id));
    assert!(controller.is_task_running(&g1_t2.id));
    assert!(!controller.is_task_running(&g2_t1.id));
    assert_eq!(controller.running_count(), 2);

    // Stop group "Alpha"
    controller.stop_group("Alpha").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    assert!(!controller.is_task_running(&g1_t1.id));
    assert!(!controller.is_task_running(&g1_t2.id));
    assert_eq!(controller.running_count(), 0);

    // Start All
    controller.start_all();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(controller.running_count(), 3);

    // Stop All
    controller.stop_all();
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(controller.running_count(), 0);
}

#[test]
fn test_bridge_logs_and_streaming() {
    let (controller, _dir) = setup_test_controller();

    let task = controller
        .add_task("Logger", "echo 'log_line_alpha'; sleep 0.05; echo 'log_line_beta'", ".", None)
        .unwrap();

    let rx = controller.subscribe_logs("Logger");
    controller.start_task(&task.id).unwrap();

    let mut received = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline && received.len() < 2 {
        if let Ok(line) = rx.recv_timeout(Duration::from_millis(200)) {
            received.push(line);
        }
    }

    assert!(received.iter().any(|l| l.contains("log_line_alpha")));
    assert!(received.iter().any(|l| l.contains("log_line_beta")));

    let recent = controller.get_recent_logs("Logger");
    assert!(recent.iter().any(|l| l.contains("log_line_alpha")));
    assert!(recent.iter().any(|l| l.contains("log_line_beta")));

    controller.stop_all();
}

#[test]
fn test_bridge_ui_binding_and_refresh() {
    let (controller, _dir) = setup_test_controller();

    controller
        .add_task("Service 1", "echo 1", ".", Some("Backend"))
        .unwrap();
    controller
        .add_task("Service 2", "echo 2", ".", Some("Backend"))
        .unwrap();

    if let Ok(window) = MainWindow::new() {
        controller.bind_to_ui(&window);
        controller.refresh_tasks(&window);

        assert_eq!(window.get_tasks().row_count(), 2);
        assert_eq!(window.get_running_count(), 0);
    }

    let groups = controller.get_groups();
    assert_eq!(groups, vec!["Backend"]);
}

#[test]
fn test_bridge_refresh_tasks_change_detection() {
    let (controller, _dir) = setup_test_controller();

    let t1 = controller
        .add_task("Service 1", "echo 1", ".", Some("Backend"))
        .unwrap();
    let t2 = controller
        .add_task("Service 2", "echo 2", ".", Some("Backend"))
        .unwrap();

    let Ok(window) = MainWindow::new() else {
        eprintln!("Skipping test_bridge_refresh_tasks_change_detection: no display available");
        return;
    };

    controller.bind_to_ui(&window);

    // First call: snapshot was None -> must perform initial render and return true
    let updated = controller.refresh_tasks(&window);
    assert!(updated, "First refresh_tasks call should update model");
    assert_eq!(window.get_tasks().row_count(), 2);
    assert_eq!(window.get_running_count(), 0);

    // Second call with NO changes: should detect unchanged state, skip ui.set_tasks, and return false
    let updated_again = controller.refresh_tasks(&window);
    assert!(
        !updated_again,
        "Subsequent refresh with no changes must skip model update to prevent hover reset"
    );

    // Third call with NO changes: still false
    assert!(!controller.refresh_tasks(&window));

    // Start a task: process state changes -> refresh_tasks should detect running change and return true
    let t3 = controller
        .add_task("Service 3", "sleep 5", ".", None)
        .unwrap();

    // Adding task invalidates/modifies snapshot -> next refresh must return true
    let updated_after_add = controller.refresh_tasks(&window);
    assert!(updated_after_add, "Refresh after add_task should update model");
    assert_eq!(window.get_tasks().row_count(), 3);

    // Consecutive refresh with no changes -> false
    assert!(!controller.refresh_tasks(&window));

    // Now start the task
    controller.start_task(&t3.id).unwrap();
    std::thread::sleep(Duration::from_millis(50));

    // Refresh should detect process is now running -> returns true
    let updated_after_start = controller.refresh_tasks(&window);
    assert!(
        updated_after_start,
        "Refresh after task start should update model"
    );
    assert_eq!(window.get_running_count(), 1);

    // Consecutive refresh while running -> false (hover is preserved!)
    assert!(!controller.refresh_tasks(&window));

    // Stop task
    controller.stop_task(&t3.id).unwrap();
    std::thread::sleep(Duration::from_millis(50));

    // Refresh should detect process stopped -> returns true
    let updated_after_stop = controller.refresh_tasks(&window);
    assert!(
        updated_after_stop,
        "Refresh after task stop should update model"
    );
    assert_eq!(window.get_running_count(), 0);

    // Consecutive refresh -> false
    assert!(!controller.refresh_tasks(&window));

    // Move task -> reorders tasks -> next refresh must return true
    let moved = controller.move_task(&t2.id, -1).unwrap();
    assert!(moved);
    let updated_after_move = controller.refresh_tasks(&window);
    assert!(
        updated_after_move,
        "Refresh after move_task should update model"
    );

    // Consecutive refresh -> false
    assert!(!controller.refresh_tasks(&window));

    // Update task
    controller
        .update_task(&t1.id, "Renamed 1", "echo 11", ".", Some("Backend"))
        .unwrap();
    let updated_after_update = controller.refresh_tasks(&window);
    assert!(
        updated_after_update,
        "Refresh after update_task should update model"
    );

    // Consecutive refresh -> false
    assert!(!controller.refresh_tasks(&window));

    // Delete task
    controller.delete_task(&t3.id).unwrap();
    let updated_after_delete = controller.refresh_tasks(&window);
    assert!(
        updated_after_delete,
        "Refresh after delete_task should update model"
    );
    assert_eq!(window.get_tasks().row_count(), 2);

    // Consecutive refresh -> false
    assert!(!controller.refresh_tasks(&window));
}

#[test]
fn test_bridge_i18n_binding_and_language_toggle() {
    let (controller, dir) = setup_test_controller();
    let config_file = dir.path().join("config.json");

    // 1. Check default controller language and i18n data
    assert_eq!(controller.language(), Language::En);
    let i18n = controller.get_i18n_data();
    assert_eq!(i18n.lang_code.as_str(), "en");
    assert_eq!(i18n.lang_switch_label.as_str(), "ZH");
    assert_eq!(i18n.new_task.as_str(), "+ New Task");
    assert_eq!(i18n.active_suffix.as_str(), " active");

    let Ok(window) = MainWindow::new() else {
        eprintln!("Skipping UI portion of test_bridge_i18n_binding_and_language_toggle: no display");
        return;
    };

    // 2. Bind to UI -> tr should be populated with English strings
    controller.bind_to_ui(&window);

    let tr = window.get_tr();
    assert_eq!(tr.lang_code.as_str(), "en");
    assert_eq!(tr.lang_switch_label.as_str(), "ZH");
    assert_eq!(tr.new_task.as_str(), "+ New Task");
    assert_eq!(tr.active_suffix.as_str(), " active");

    // 3. Invoke toggle_language callback from UI -> should switch to Chinese
    window.invoke_toggle_language();

    assert_eq!(controller.language(), Language::Zh);
    let tr_zh = window.get_tr();
    assert_eq!(tr_zh.lang_code.as_str(), "zh");
    assert_eq!(tr_zh.lang_switch_label.as_str(), "EN");
    assert_eq!(tr_zh.new_task.as_str(), "+ 新建任务");
    assert_eq!(tr_zh.active_suffix.as_str(), " 运行中");

    // Check persistence to config file
    let cm_verify = ConfigManager::with_path(config_file.clone());
    assert_eq!(cm_verify.load_language().unwrap(), Language::Zh);

    // 4. Invoke toggle_language again -> should switch back to English
    window.invoke_toggle_language();

    assert_eq!(controller.language(), Language::En);
    let tr_en = window.get_tr();
    assert_eq!(tr_en.lang_code.as_str(), "en");
    assert_eq!(tr_en.lang_switch_label.as_str(), "ZH");
    assert_eq!(tr_en.new_task.as_str(), "+ New Task");
    assert_eq!(tr_en.active_suffix.as_str(), " active");

    assert_eq!(cm_verify.load_language().unwrap(), Language::En);

    // 5. Test initialization with pre-configured Chinese language
    let dir2 = tempdir().unwrap();
    let config_file2 = dir2.path().join("config.json");
    let cm2 = ConfigManager::with_path(config_file2.clone());
    cm2.save_language(Language::Zh).unwrap();

    let logs2 = LogBroadcaster::new(dir2.path().join("logs"), 100);
    let process2 = ProcessManager::new(logs2.clone());
    let controller2 = SlintAppController::new(cm2, process2, logs2);

    assert_eq!(controller2.language(), Language::Zh);
    let i18n_zh = controller2.get_i18n_data();
    assert_eq!(i18n_zh.lang_code.as_str(), "zh");
    assert_eq!(i18n_zh.lang_switch_label.as_str(), "EN");
    assert_eq!(i18n_zh.new_task.as_str(), "+ 新建任务");

    if let Ok(window2) = MainWindow::new() {
        controller2.bind_to_ui(&window2);
        let tr2 = window2.get_tr();
        assert_eq!(tr2.lang_code.as_str(), "zh");
        assert_eq!(tr2.lang_switch_label.as_str(), "EN");
        assert_eq!(tr2.new_task.as_str(), "+ 新建任务");
    }
}

