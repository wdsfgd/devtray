use devtray::bridge::{BridgeError, TaskManagerBridge};
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::model::TaskConfig;
use devtray::core::process::ProcessManager;
use std::time::Duration;
use tempfile::tempdir;

fn setup_test_bridge() -> (TaskManagerBridge, tempfile::TempDir) {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let logs_dir = dir.path().join("logs");

    let cm = ConfigManager::with_path(config_file);
    let broadcaster = LogBroadcaster::new(logs_dir, 100);
    let pm = ProcessManager::new(broadcaster.clone());

    let bridge = TaskManagerBridge::with_managers(cm, pm, broadcaster);
    (bridge, dir)
}

#[test]
fn test_bridge_initialization_loads_existing_tasks() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let logs_dir = dir.path().join("logs");

    let initial_tasks = vec![
        TaskConfig::new("T1", "echo 1", ".", Some("GroupA")).unwrap(),
        TaskConfig::new("T2", "echo 2", "/tmp", None).unwrap(),
    ];
    let cm_setup = ConfigManager::with_path(config_file.clone());
    cm_setup.save(&initial_tasks).unwrap();

    let cm = ConfigManager::with_path(config_file);
    let broadcaster = LogBroadcaster::new(logs_dir, 100);
    let pm = ProcessManager::new(broadcaster.clone());

    let bridge = TaskManagerBridge::with_managers(cm, pm, broadcaster);
    let tasks = bridge.tasks();

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].name, "T1");
    assert_eq!(tasks[1].name, "T2");
}

#[test]
fn test_bridge_add_task_and_persistence() {
    let (bridge, dir) = setup_test_bridge();
    let config_file = dir.path().join("config.json");

    assert_eq!(bridge.tasks().len(), 0);

    let created = bridge
        .add_task("Server", "echo running", ".", Some("Backend"))
        .expect("add_task should succeed");

    assert_eq!(created.name, "Server");
    assert_eq!(created.command, "echo running");
    assert_eq!(created.working_directory, ".");
    assert_eq!(created.group.as_deref(), Some("Backend"));
    assert!(!created.id.is_empty());

    assert_eq!(bridge.tasks().len(), 1);
    assert_eq!(bridge.tasks()[0].id, created.id);

    // Verify persistence to file
    let cm_verify = ConfigManager::with_path(config_file);
    let loaded = cm_verify.load().unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].id, created.id);
}

#[test]
fn test_bridge_add_task_validation_error() {
    let (bridge, _dir) = setup_test_bridge();

    let empty_name = bridge.add_task("", "echo 1", ".", None);
    assert!(matches!(empty_name, Err(BridgeError::ValidationError(_))));

    let empty_cmd = bridge.add_task("Task", "", ".", None);
    assert!(matches!(empty_cmd, Err(BridgeError::ValidationError(_))));
}

#[test]
fn test_bridge_update_task() {
    let (bridge, dir) = setup_test_bridge();
    let config_file = dir.path().join("config.json");

    let created = bridge.add_task("Old Name", "echo old", ".", None).unwrap();

    let updated = bridge
        .update_task(&created.id, "New Name", "echo new", "/tmp", Some("Web"))
        .expect("update_task should succeed");

    assert_eq!(updated.id, created.id);
    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.command, "echo new");
    assert_eq!(updated.working_directory, "/tmp");
    assert_eq!(updated.group.as_deref(), Some("Web"));

    let tasks = bridge.tasks();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].name, "New Name");

    // Verify persistence
    let cm_verify = ConfigManager::with_path(config_file);
    let loaded = cm_verify.load().unwrap();
    assert_eq!(loaded[0].name, "New Name");
}

#[test]
fn test_bridge_update_nonexistent_task() {
    let (bridge, _dir) = setup_test_bridge();

    let res = bridge.update_task("nonexistent-id", "Name", "cmd", ".", None);
    assert!(matches!(res, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_save_task_convenience() {
    let (bridge, _dir) = setup_test_bridge();

    // Empty ID -> adds new task
    let t1 = bridge
        .save_task("", "Task 1", "echo 1", ".", None)
        .expect("save_task with empty ID should add");
    assert_eq!(bridge.tasks().len(), 1);

    // Existing ID -> updates task
    let t1_updated = bridge
        .save_task(
            &t1.id,
            "Task 1 Renamed",
            "echo 1 updated",
            "/tmp",
            Some("G"),
        )
        .expect("save_task with existing ID should update");
    assert_eq!(t1_updated.name, "Task 1 Renamed");
    assert_eq!(bridge.tasks().len(), 1);
    assert_eq!(bridge.tasks()[0].name, "Task 1 Renamed");
}

#[test]
fn test_bridge_delete_task_stops_and_removes() {
    let (bridge, dir) = setup_test_bridge();
    let config_file = dir.path().join("config.json");

    let t = bridge
        .add_task("Long Sleeper", "sleep 30", ".", None)
        .unwrap();

    bridge.start_task(&t.id).expect("start_task should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(bridge.is_task_running(&t.id));

    bridge
        .delete_task(&t.id)
        .expect("delete_task should succeed");
    std::thread::sleep(Duration::from_millis(100));

    // Process should be stopped and task removed from list
    assert!(!bridge.is_task_running(&t.id));
    assert_eq!(bridge.tasks().len(), 0);

    // Verify config file is updated
    let cm_verify = ConfigManager::with_path(config_file);
    let loaded = cm_verify.load().unwrap();
    assert_eq!(loaded.len(), 0);
}

#[test]
fn test_bridge_delete_nonexistent_task() {
    let (bridge, _dir) = setup_test_bridge();
    let res = bridge.delete_task("nonexistent");
    assert!(matches!(res, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_move_task() {
    let (bridge, dir) = setup_test_bridge();
    let config_file = dir.path().join("config.json");

    let _t1 = bridge.add_task("T1", "echo 1", ".", None).unwrap();
    let t2 = bridge.add_task("T2", "echo 2", ".", None).unwrap();
    let t3 = bridge.add_task("T3", "echo 3", ".", None).unwrap();

    // Initial order: [T1, T2, T3]
    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Move T2 up (-1) -> [T2, T1, T3]
    let moved = bridge.move_task(&t2.id, -1).unwrap();
    assert!(moved);
    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T2", "T1", "T3"]
    );

    // Move T2 up again (-1) at index 0 -> should return false (boundary)
    let moved_again = bridge.move_task(&t2.id, -1).unwrap();
    assert!(!moved_again);
    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T2", "T1", "T3"]
    );

    // Move T2 down (+1) -> [T1, T2, T3]
    let moved_down = bridge.move_task(&t2.id, 1).unwrap();
    assert!(moved_down);
    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Move T3 down (+1) at index 2 (last index) -> should return false
    let moved_t3_down = bridge.move_task(&t3.id, 1).unwrap();
    assert!(!moved_t3_down);

    // Verify persistence
    let cm_verify = ConfigManager::with_path(config_file);
    let loaded = cm_verify.load().unwrap();
    assert_eq!(
        loaded.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );
}

#[test]
fn test_bridge_move_nonexistent_task() {
    let (bridge, _dir) = setup_test_bridge();
    let res = bridge.move_task("invalid", 1);
    assert!(matches!(res, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_reorder_task_within_group() {
    let (bridge, dir) = setup_test_bridge();
    let config_file = dir.path().join("config.json");

    let t1 = bridge.add_task("T1", "echo 1", ".", Some("GroupA")).unwrap();
    let _t2 = bridge.add_task("T2", "echo 2", ".", Some("GroupA")).unwrap();
    let t3 = bridge.add_task("T3", "echo 3", ".", Some("GroupA")).unwrap();

    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Reorder T3 (index 2) to index 0 -> [T3, T1, T2]
    let reordered = bridge.reorder_task(&t3.id, 0).unwrap();
    assert!(reordered);
    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T3", "T1", "T2"]
    );

    // Reorder T1 (currently index 1) to index 2 -> [T3, T2, T1]
    let reordered2 = bridge.reorder_task(&t1.id, 2).unwrap();
    assert!(reordered2);
    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T3", "T2", "T1"]
    );

    // Verify persistence
    let cm_verify = ConfigManager::with_path(config_file);
    let loaded = cm_verify.load().unwrap();
    assert_eq!(
        loaded.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T3", "T2", "T1"]
    );
}

#[test]
fn test_bridge_reorder_task_across_groups() {
    let (bridge, _dir) = setup_test_bridge();

    let _t1 = bridge.add_task("T1", "echo 1", ".", Some("Backend")).unwrap();
    let _t2 = bridge.add_task("T2", "echo 2", ".", Some("Backend")).unwrap();
    let t3 = bridge.add_task("T3", "echo 3", ".", None).unwrap(); // Uncategorized

    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Drag T3 (index 2, uncategorized) up to index 0 (Backend)
    let reordered = bridge.reorder_task(&t3.id, 0).unwrap();
    assert!(reordered);

    let tasks = bridge.tasks();
    assert_eq!(
        tasks.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T3", "T1", "T2"]
    );
    assert_eq!(tasks[0].group.as_deref(), Some("Backend"));
}

#[test]
fn test_bridge_reorder_task_downward_across_groups() {
    let (bridge, _dir) = setup_test_bridge();

    let t1 = bridge.add_task("T1", "echo 1", ".", Some("Backend")).unwrap();
    let _t2 = bridge.add_task("T2", "echo 2", ".", Some("Backend")).unwrap();
    let _t3 = bridge.add_task("T3", "echo 3", ".", Some("Frontend")).unwrap();

    assert_eq!(
        bridge.tasks().iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T1", "T2", "T3"]
    );

    // Drag T1 (index 0, Backend) down to index 2 (Frontend)
    let reordered = bridge.reorder_task(&t1.id, 2).unwrap();
    assert!(reordered);

    let tasks = bridge.tasks();
    assert_eq!(
        tasks.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T2", "T3", "T1"]
    );
    assert_eq!(tasks[2].group.as_deref(), Some("Frontend"));
}

#[test]
fn test_bridge_reorder_task_downward_within_group() {
    let (bridge, _dir) = setup_test_bridge();

    let t1 = bridge.add_task("T1", "echo 1", ".", Some("Backend")).unwrap();
    let _t2 = bridge.add_task("T2", "echo 2", ".", Some("Backend")).unwrap();
    let _t3 = bridge.add_task("T3", "echo 3", ".", Some("Frontend")).unwrap();

    // Drag T1 (index 0) down to index 1 (T2 in Backend)
    let reordered = bridge.reorder_task(&t1.id, 1).unwrap();
    assert!(reordered);

    let tasks = bridge.tasks();
    assert_eq!(
        tasks.iter().map(|t| &t.name).collect::<Vec<_>>(),
        vec!["T2", "T1", "T3"]
    );
    assert_eq!(tasks[0].group.as_deref(), Some("Backend"));
    assert_eq!(tasks[1].group.as_deref(), Some("Backend"));
    assert_eq!(tasks[2].group.as_deref(), Some("Frontend"));
}

#[test]
fn test_bridge_reorder_nonexistent_task() {
    let (bridge, _dir) = setup_test_bridge();
    let res = bridge.reorder_task("invalid", 0);
    assert!(matches!(res, Err(BridgeError::TaskNotFound(_))));
}

#[test]
fn test_bridge_start_and_stop_task() {
    let (bridge, _dir) = setup_test_bridge();

    let t = bridge.add_task("Worker", "sleep 10", ".", None).unwrap();

    assert!(!bridge.is_task_running(&t.id));

    bridge.start_task(&t.id).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(bridge.is_task_running(&t.id));

    bridge.stop_task(&t.id).expect("stop should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(!bridge.is_task_running(&t.id));
}

#[test]
fn test_bridge_start_and_stop_group() {
    let (bridge, _dir) = setup_test_bridge();

    let g1_t1 = bridge
        .add_task("G1-1", "sleep 10", ".", Some("Alpha"))
        .unwrap();
    let g1_t2 = bridge
        .add_task("G1-2", "sleep 10", ".", Some("Alpha"))
        .unwrap();
    let g2_t1 = bridge
        .add_task("G2-1", "sleep 10", ".", Some("Beta"))
        .unwrap();

    bridge
        .start_group("Alpha")
        .expect("start_group should succeed");
    std::thread::sleep(Duration::from_millis(100));

    assert!(bridge.is_task_running(&g1_t1.id));
    assert!(bridge.is_task_running(&g1_t2.id));
    assert!(!bridge.is_task_running(&g2_t1.id));

    bridge
        .stop_group("Alpha")
        .expect("stop_group should succeed");
    std::thread::sleep(Duration::from_millis(100));

    assert!(!bridge.is_task_running(&g1_t1.id));
    assert!(!bridge.is_task_running(&g1_t2.id));
    assert!(!bridge.is_task_running(&g2_t1.id));
}

#[test]
fn test_bridge_get_recent_logs() {
    let (bridge, _dir) = setup_test_bridge();

    let task = bridge
        .add_task("Logger", "echo 'log line 1'; echo 'log line 2'", ".", None)
        .unwrap();

    bridge.start_task(&task.id).unwrap();
    std::thread::sleep(Duration::from_millis(200));

    let logs = bridge.get_recent_logs("Logger");
    assert!(logs.iter().any(|l| l.contains("log line 1")));
    assert!(logs.iter().any(|l| l.contains("log line 2")));
}

#[test]
fn test_bridge_stop_all() {
    let (bridge, _dir) = setup_test_bridge();

    let t1 = bridge.add_task("T1", "sleep 10", ".", None).unwrap();
    let t2 = bridge.add_task("T2", "sleep 10", ".", None).unwrap();

    bridge.start_task(&t1.id).unwrap();
    bridge.start_task(&t2.id).unwrap();
    std::thread::sleep(Duration::from_millis(100));

    assert!(bridge.is_task_running(&t1.id));
    assert!(bridge.is_task_running(&t2.id));

    bridge.stop_all();
    std::thread::sleep(Duration::from_millis(100));

    assert!(!bridge.is_task_running(&t1.id));
    assert!(!bridge.is_task_running(&t2.id));
}
