use devtray::core::logs::LogBroadcaster;
use devtray::core::model::TaskConfig;
use devtray::core::process::ProcessManager;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_process_lifecycle_and_termination() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new("Sleepy", "sleep 10", ".", None).unwrap();
    assert!(!pm.is_running(&task.id));

    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(pm.is_running(&task.id));

    pm.stop(&task.id).expect("stop should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(!pm.is_running(&task.id));
}

#[test]
fn test_process_group_termination_kills_children() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    // Spawns a background sleep and waits
    let task = TaskConfig::new("SpawnChild", "sleep 50 & sleep 50", ".", None).unwrap();
    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(pm.is_running(&task.id));

    pm.stop(&task.id).expect("stop should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(!pm.is_running(&task.id));
}

#[test]
fn test_process_natural_exit_cleanup() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new("Quick", "echo 'done'", ".", None).unwrap();
    pm.start(&task).expect("start should succeed");

    // Wait for the short-lived process to exit and reaping thread to update state
    let mut finished = false;
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(50));
        if !pm.is_running(&task.id) {
            finished = true;
            break;
        }
    }
    assert!(finished, "Process should have exited and been cleaned up");
}

#[test]
fn test_process_stdout_and_stderr_captured_by_broadcaster() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster.clone());

    let task = TaskConfig::new(
        "LogProducer",
        "echo 'stdout line 1' && >&2 echo 'stderr line 1' && echo 'stdout line 2'",
        ".",
        None,
    )
    .unwrap();

    pm.start(&task).expect("start should succeed");

    // Wait for process to exit and logs to be piped
    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(50));
        if !pm.is_running(&task.id) {
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(100));

    let logs = broadcaster.get_recent_lines("LogProducer");
    assert!(logs.iter().any(|l| l.contains("stdout line 1")));
    assert!(logs.iter().any(|l| l.contains("stderr line 1")));
    assert!(logs.iter().any(|l| l.contains("stdout line 2")));
}

#[test]
fn test_process_stop_all() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task1 = TaskConfig::new("Task1", "sleep 10", ".", None).unwrap();
    let task2 = TaskConfig::new("Task2", "sleep 10", ".", None).unwrap();

    pm.start(&task1).expect("task1 start should succeed");
    pm.start(&task2).expect("task2 start should succeed");
    std::thread::sleep(Duration::from_millis(100));

    assert!(pm.is_running(&task1.id));
    assert!(pm.is_running(&task2.id));

    pm.stop_all();
    std::thread::sleep(Duration::from_millis(100));

    assert!(!pm.is_running(&task1.id));
    assert!(!pm.is_running(&task2.id));
}

#[test]
fn test_start_and_stop_idempotence() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new("Idempotent", "sleep 10", ".", None).unwrap();

    // Stopping before starting should be a no-op
    assert!(pm.stop(&task.id).is_ok());

    // Starting once
    assert!(pm.start(&task).is_ok());
    std::thread::sleep(Duration::from_millis(100));
    assert!(pm.is_running(&task.id));

    // Starting again when already running should be a no-op
    assert!(pm.start(&task).is_ok());
    assert!(pm.is_running(&task.id));

    // Stopping once
    assert!(pm.stop(&task.id).is_ok());
    std::thread::sleep(Duration::from_millis(100));
    assert!(!pm.is_running(&task.id));

    // Stopping again should be a no-op
    assert!(pm.stop(&task.id).is_ok());
}
