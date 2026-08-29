use devtray::core::single_instance::{SingleInstance, SingleInstanceStatus};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn test_single_instance_primary_and_secondary() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("test_devtray.sock");

    // 1. First instance acquires primary
    let status1 = SingleInstance::acquire(&socket_path).expect("First acquire should succeed");
    let mut guard = match status1 {
        SingleInstanceStatus::Primary(guard) => guard,
        SingleInstanceStatus::Secondary => panic!("Expected first instance to be Primary"),
    };

    let show_called = Arc::new(AtomicBool::new(false));
    let show_called_clone = show_called.clone();

    guard.start_listener(move || {
        show_called_clone.store(true, Ordering::SeqCst);
    });

    // 2. Second instance attempts to acquire on same path -> should return Secondary and trigger callback
    let status2 = SingleInstance::acquire(&socket_path).expect("Second acquire should succeed");
    match status2 {
        SingleInstanceStatus::Primary(_) => panic!("Expected second instance to be Secondary"),
        SingleInstanceStatus::Secondary => {}
    }

    // Wait briefly for callback to execute
    std::thread::sleep(Duration::from_millis(250));
    assert!(show_called.load(Ordering::SeqCst), "show callback should have been triggered by second instance");

    // 3. Drop guard -> socket file is cleaned up
    drop(guard);
    std::thread::sleep(Duration::from_millis(50));
    assert!(!socket_path.exists(), "Socket file should be removed on guard drop");

    // 4. New instance can now acquire primary again
    let status3 = SingleInstance::acquire(&socket_path).expect("Third acquire should succeed");
    assert!(matches!(status3, SingleInstanceStatus::Primary(_)));
}

#[test]
fn test_single_instance_stale_socket_recovery() {
    let dir = tempdir().unwrap();
    let socket_path = dir.path().join("stale_devtray.sock");

    // Create a dummy stale socket/file
    std::fs::write(&socket_path, b"stale").unwrap();
    assert!(socket_path.exists());

    // Acquire should detect stale file, clean it, and become Primary
    let status = SingleInstance::acquire(&socket_path).expect("Acquire on stale socket should succeed");
    assert!(matches!(status, SingleInstanceStatus::Primary(_)));
}
