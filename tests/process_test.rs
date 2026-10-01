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

#[test]
fn test_rapid_restart_reaper_synchronization() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new("Rapid", "sleep 10", ".", None).unwrap();
    pm.start(&task).expect("initial start should succeed");
    std::thread::sleep(Duration::from_millis(50));
    assert!(pm.is_running(&task.id));

    // Rapid stop and restart
    pm.stop(&task.id).expect("stop should succeed");
    pm.start(&task).expect("rapid restart should succeed");

    // Wait long enough for the previous reaper thread to finish
    std::thread::sleep(Duration::from_millis(200));

    // The restarted task should STILL be running (not reaped by the first process exit)
    assert!(pm.is_running(&task.id));

    pm.stop(&task.id).expect("final stop should succeed");
}

#[test]
fn test_stop_with_config_custom_stop_command() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let marker_file = dir.path().join("stop_marker.txt");
    let marker_path = marker_file.to_str().unwrap();

    let mut task = TaskConfig::new("CustomStop", "sleep 10", ".", None).unwrap();
    task.stop_command = Some(format!("touch {}", marker_path));

    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(pm.is_running(&task.id));

    pm.stop_with_config(&task)
        .expect("stop_with_config should succeed");
    std::thread::sleep(Duration::from_millis(300));

    assert!(!pm.is_running(&task.id));
    assert!(
        marker_file.exists(),
        "stop_command should have executed and created marker file"
    );
}

#[test]
fn test_process_graceful_stop_sigint_handled() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let cleanup_file = dir.path().join("sigint_clean.txt");
    let cleanup_path = cleanup_file.to_str().unwrap();

    let cmd = format!(
        "trap 'touch \"{}\"; exit 0' INT; while true; do sleep 0.05; done",
        cleanup_path
    );
    let task = TaskConfig::new("IntHandler", &cmd, ".", None).unwrap();

    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(150));
    assert!(pm.is_running(&task.id));

    pm.stop(&task.id).expect("stop should succeed");

    // Wait up to 1 second for trap to finish writing file
    let mut cleaned_up = false;
    for _ in 0..20 {
        if cleanup_file.exists() {
            cleaned_up = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(cleaned_up, "SIGINT trap handler should have executed");
    assert!(!pm.is_running(&task.id));
}

#[test]
fn test_escalation_advances_to_sigterm() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let term_file = dir.path().join("sigterm_clean.txt");
    let term_path = term_file.to_str().unwrap();

    let cmd = format!(
        "trap '' INT; trap 'touch \"{}\"; exit 0' TERM; while true; do sleep 0.05; done",
        term_path
    );
    let task = TaskConfig::new("TermHandler", &cmd, ".", None).unwrap();

    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(150));
    assert!(pm.is_running(&task.id));

    let pid = pm.get_pid(&task.id).expect("pid should be present");

    // Run escalation with 200ms SIGINT timeout and 500ms SIGTERM timeout
    ProcessManager::run_stop_escalation(
        pid,
        Duration::from_millis(200),
        Duration::from_millis(500),
        Duration::from_millis(20),
    );

    assert!(
        term_file.exists(),
        "Process ignoring INT should have been terminated by SIGTERM"
    );
}

#[test]
fn test_escalation_advances_to_sigkill() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new(
        "Immortal",
        "trap '' INT TERM; while true; do sleep 0.05; done",
        ".",
        None,
    )
    .unwrap();

    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(150));
    assert!(pm.is_running(&task.id));

    let pid = pm.get_pid(&task.id).expect("pid should be present");

    // Run escalation with short timeouts to verify SIGKILL kills it
    ProcessManager::run_stop_escalation(
        pid,
        Duration::from_millis(100),
        Duration::from_millis(100),
        Duration::from_millis(20),
    );

    // After escalation completes, process should be terminated by SIGKILL
    std::thread::sleep(Duration::from_millis(50));
    assert!(
        nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid as i32), None).is_err(),
        "Process should be killed by SIGKILL"
    );
}

#[test]
fn test_stop_does_not_block() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new(
        "Stubborn",
        "trap '' INT; while true; do sleep 0.05; done",
        ".",
        None,
    )
    .unwrap();
    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(150));
    assert!(pm.is_running(&task.id));

    let start = std::time::Instant::now();
    pm.stop(&task.id).expect("stop should succeed");
    let elapsed = start.elapsed();

    // stop() must return almost immediately (< 50ms) without waiting for escalation timeouts
    assert!(
        elapsed < Duration::from_millis(50),
        "stop() took {:?}, expected < 50ms (non-blocking)",
        elapsed
    );
    assert!(!pm.is_running(&task.id));
}

#[test]
fn test_is_process_alive_handles_eperm_and_liveness() {
    // Error classification: EPERM indicates the process exists but cannot be signaled, so it is alive.
    assert!(ProcessManager::is_alive_result(Ok(())));
    assert!(ProcessManager::is_alive_result(Err(nix::errno::Errno::EPERM)));
    assert!(!ProcessManager::is_alive_result(Err(nix::errno::Errno::ESRCH)));

    // Process liveness verification on real processes
    let current_pid = std::process::id() as i32;
    assert!(ProcessManager::is_process_alive(current_pid));
    assert!(devtray::core::process::is_process_alive(current_pid));

    let mut child = std::process::Command::new("sleep")
        .arg("5")
        .spawn()
        .expect("child should spawn");
    let child_pid = child.id() as i32;
    assert!(ProcessManager::is_process_alive(child_pid));

    child.kill().expect("child should be killed");
    let _ = child.wait();

    // Reaped process should not be alive
    assert!(!ProcessManager::is_process_alive(child_pid));

    // Non-existent PID should not be alive
    assert!(!ProcessManager::is_process_alive(999_999));
}
