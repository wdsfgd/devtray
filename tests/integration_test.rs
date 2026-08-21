use std::fs;
use std::time::Duration;
use tempfile::tempdir;

use devtray::bridge::TaskManagerBridge;
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::model::TaskConfig;
use devtray::core::process::ProcessManager;

#[test]
fn test_end_to_end_task_management() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let logs_dir = dir.path().join("logs");

    let cm = ConfigManager::with_path(config_file);
    let broadcaster = LogBroadcaster::new(logs_dir.clone(), 100);
    let pm = ProcessManager::new(broadcaster.clone());

    let tasks = vec![TaskConfig::new(
        "Echoer",
        "echo 'hello from task'; sleep 1",
        ".",
        Some("GroupA"),
    )
    .unwrap()];
    cm.save(&tasks).unwrap();

    let _rx = broadcaster.subscribe("Echoer");
    pm.start(&tasks[0]).unwrap();

    // Allow time for process execution and log streaming
    let mut saw_log = false;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(50));
        let logs = broadcaster.get_recent_lines("Echoer");
        if logs.iter().any(|line| line.contains("hello from task")) {
            saw_log = true;
            break;
        }
    }
    assert!(saw_log, "Expected log message 'hello from task' not found");

    pm.stop_all();
}

#[test]
fn test_end_to_end_live_log_streaming() {
    let dir = tempdir().unwrap();
    let logs_dir = dir.path().join("logs");

    let broadcaster = LogBroadcaster::new(logs_dir.clone(), 100);
    let pm = ProcessManager::new(broadcaster.clone());

    let task = TaskConfig::new(
        "Streamer",
        "echo 'stream_line_1'; sleep 0.05; echo 'stream_line_2'",
        ".",
        None,
    )
    .unwrap();

    let rx = broadcaster.subscribe("Streamer");
    pm.start(&task).unwrap();

    let mut received = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while std::time::Instant::now() < deadline && received.len() < 2 {
        if let Ok(line) = rx.recv_timeout(Duration::from_millis(200)) {
            received.push(line);
        }
    }

    assert!(
        received.iter().any(|l| l.contains("stream_line_1")),
        "Streamed logs should contain stream_line_1, got: {:?}",
        received
    );
    assert!(
        received.iter().any(|l| l.contains("stream_line_2")),
        "Streamed logs should contain stream_line_2, got: {:?}",
        received
    );

    // Verify persisted log file on disk
    let log_file_path = logs_dir.join("Streamer.log");
    let content = fs::read_to_string(&log_file_path).unwrap_or_default();
    assert!(
        content.contains("stream_line_1"),
        "Disk log file should contain stream_line_1"
    );
    assert!(
        content.contains("stream_line_2"),
        "Disk log file should contain stream_line_2"
    );

    pm.stop(&task.id).unwrap();
}

#[test]
fn test_end_to_end_group_lifecycle_and_process_tree() {
    let dir = tempdir().unwrap();
    let logs_dir = dir.path().join("logs");

    let broadcaster = LogBroadcaster::new(logs_dir, 100);
    let pm = ProcessManager::new(broadcaster);

    let task1 = TaskConfig::new("Service1", "sleep 30", ".", Some("Backend")).unwrap();
    let task2 = TaskConfig::new("Service2", "sleep 30", ".", Some("Backend")).unwrap();
    let task3 = TaskConfig::new("Frontend", "sleep 30", ".", Some("Frontend")).unwrap();

    // Start all Backend tasks
    pm.start(&task1).unwrap();
    pm.start(&task2).unwrap();
    std::thread::sleep(Duration::from_millis(100));

    assert!(pm.is_running(&task1.id));
    assert!(pm.is_running(&task2.id));
    assert!(!pm.is_running(&task3.id));

    // Stop Backend tasks individually
    pm.stop(&task1.id).unwrap();
    pm.stop(&task2.id).unwrap();
    std::thread::sleep(Duration::from_millis(100));

    assert!(!pm.is_running(&task1.id));
    assert!(!pm.is_running(&task2.id));
}

#[test]
fn test_end_to_end_bridge_full_lifecycle() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let logs_dir = dir.path().join("logs");

    let cm = ConfigManager::with_path(config_file.clone());
    let broadcaster = LogBroadcaster::new(logs_dir.clone(), 500);
    let pm = ProcessManager::new(broadcaster.clone());

    let bridge = TaskManagerBridge::with_managers(cm, pm, broadcaster);

    // 1. Add tasks through bridge
    let api_task = bridge
        .add_task(
            "API-Server",
            "echo 'api ready'; sleep 30",
            ".",
            Some("Services"),
        )
        .expect("add API-Server should succeed");
    let worker_task = bridge
        .add_task(
            "Worker",
            "echo 'worker ready'; sleep 30",
            ".",
            Some("Services"),
        )
        .expect("add Worker should succeed");
    let standalone_task = bridge
        .add_task("OneOff", "echo 'oneoff running'", ".", None)
        .expect("add OneOff should succeed");

    assert_eq!(bridge.tasks().len(), 3);

    // 2. Start group "Services"
    let rx_api = bridge.subscribe_logs("API-Server");
    bridge
        .start_group("Services")
        .expect("start_group should succeed");
    std::thread::sleep(Duration::from_millis(150));

    assert!(bridge.is_task_running(&api_task.id));
    assert!(bridge.is_task_running(&worker_task.id));
    assert!(!bridge.is_task_running(&standalone_task.id));

    // 3. Verify live logs received through bridge subscriber
    let received_api_line = rx_api
        .recv_timeout(Duration::from_secs(2))
        .expect("should receive log line from API-Server");
    assert!(received_api_line.contains("api ready"));

    // 4. Verify recent logs query through bridge
    let recent = bridge.get_recent_logs("API-Server");
    assert!(recent.iter().any(|l| l.contains("api ready")));

    // 5. Stop group "Services"
    bridge
        .stop_group("Services")
        .expect("stop_group should succeed");
    std::thread::sleep(Duration::from_millis(150));

    assert!(!bridge.is_task_running(&api_task.id));
    assert!(!bridge.is_task_running(&worker_task.id));

    // 6. Move tasks and verify reordering
    assert_eq!(bridge.tasks()[0].id, api_task.id);
    bridge.move_task(&worker_task.id, -1).unwrap();
    assert_eq!(bridge.tasks()[0].id, worker_task.id);

    // 7. Delete task and verify persistence
    bridge.delete_task(&standalone_task.id).unwrap();
    assert_eq!(bridge.tasks().len(), 2);

    let cm_verify = ConfigManager::with_path(config_file);
    let loaded = cm_verify.load().unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].name, "Worker");
    assert_eq!(loaded[1].name, "API-Server");

    // Clean up
    bridge.stop_all();
}
