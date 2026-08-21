use devtray::core::logs::LogBroadcaster;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ring_buffer_capacity() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    for i in 0..10 {
        broadcaster
            .append("task-1", &format!("line {}", i))
            .unwrap();
    }

    let recent = broadcaster.get_recent_lines("task-1");
    assert_eq!(recent.len(), 5);
    assert_eq!(recent[0], "line 5");
    assert_eq!(recent[4], "line 9");
}

#[test]
fn test_log_file_writing() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    broadcaster.append("task-1", "first line").unwrap();
    broadcaster.append("task-1", "second line").unwrap();

    let log_file = dir.path().join("task-1.log");
    assert!(log_file.exists());

    let content = fs::read_to_string(log_file).unwrap();
    assert_eq!(content, "first line\nsecond line\n");
}

#[test]
fn test_log_subscribe_and_broadcast() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    let rx = broadcaster.subscribe("task-1");

    broadcaster.append("task-1", "message 1").unwrap();
    broadcaster.append("task-1", "message 2").unwrap();

    assert_eq!(rx.recv().unwrap(), "message 1");
    assert_eq!(rx.recv().unwrap(), "message 2");
}

#[test]
fn test_multiple_subscribers() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    let rx1 = broadcaster.subscribe("task-1");
    let rx2 = broadcaster.subscribe("task-1");

    broadcaster.append("task-1", "broadcast msg").unwrap();

    assert_eq!(rx1.recv().unwrap(), "broadcast msg");
    assert_eq!(rx2.recv().unwrap(), "broadcast msg");
}

#[test]
fn test_subscriber_isolation_between_tasks() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    let rx1 = broadcaster.subscribe("task-1");
    let rx2 = broadcaster.subscribe("task-2");

    broadcaster.append("task-1", "msg for task 1").unwrap();
    broadcaster.append("task-2", "msg for task 2").unwrap();

    assert_eq!(rx1.recv().unwrap(), "msg for task 1");
    assert_eq!(rx2.recv().unwrap(), "msg for task 2");
    assert!(rx1.try_recv().is_err());
    assert!(rx2.try_recv().is_err());
}

#[test]
fn test_dead_subscriber_cleanup() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    let rx1 = broadcaster.subscribe("task-1");
    let rx2 = broadcaster.subscribe("task-1");

    drop(rx1); // subscriber 1 disconnects

    // Appending should still succeed without error
    broadcaster.append("task-1", "msg after drop").unwrap();

    assert_eq!(rx2.recv().unwrap(), "msg after drop");
}

#[test]
fn test_get_recent_lines_nonexistent_task() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    let recent = broadcaster.get_recent_lines("nonexistent");
    assert!(recent.is_empty());
}

#[test]
fn test_zero_capacity_ring_buffer() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 0);

    broadcaster.append("task-1", "line 0").unwrap();
    let recent = broadcaster.get_recent_lines("task-1");
    assert!(recent.is_empty());

    let log_file = dir.path().join("task-1.log");
    let content = fs::read_to_string(log_file).unwrap();
    assert_eq!(content, "line 0\n");
}

#[test]
fn test_cached_file_handle_verbose_logging() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 500);

    for i in 0..500 {
        broadcaster
            .append("verbose-task", &format!("verbose log line {}", i))
            .unwrap();
    }

    let log_file = dir.path().join("verbose-task.log");
    assert!(log_file.exists());
    let content = fs::read_to_string(log_file).unwrap();
    let line_count = content.lines().count();
    assert_eq!(line_count, 500);
}

