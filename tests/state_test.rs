use devtray::core::state::{StateManager, TaskRuntimeRecord};
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_state_manager_default_path_under_cache() {
    let sm = StateManager::new();
    let path = sm.path();
    let path_str = path.to_string_lossy();
    assert!(
        path_str.contains(".cache/devtray") || path_str.contains("cache/devtray"),
        "StateManager path should be under .cache/devtray, but got: {}",
        path_str
    );
    assert!(
        !path_str.contains(".config/devtray"),
        "StateManager path should NOT be under .config/devtray"
    );
}

#[test]
fn test_state_manager_load_save_roundtrip() {
    let dir = tempdir().unwrap();
    let state_file = dir.path().join("state.json");
    let sm = StateManager::with_path(state_file.clone());

    // 1. Initial load on non-existent file returns empty map
    let initial = sm.load();
    assert!(initial.is_empty());

    // 2. Save records
    let mut tasks = HashMap::new();
    tasks.insert(
        "task-1".to_string(),
        TaskRuntimeRecord {
            pid: 1234,
            name: "Server".to_string(),
            started_at: 1000,
        },
    );
    tasks.insert(
        "task-2".to_string(),
        TaskRuntimeRecord {
            pid: 5678,
            name: "Worker".to_string(),
            started_at: 1050,
        },
    );

    sm.save(&tasks).expect("save state");
    assert!(state_file.exists());

    // 3. Load records
    let loaded = sm.load();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.get("task-1").unwrap().pid, 1234);
    assert_eq!(loaded.get("task-1").unwrap().name, "Server");
    assert_eq!(loaded.get("task-2").unwrap().pid, 5678);

    // 4. Update individual task
    sm.set_task(
        "task-3",
        TaskRuntimeRecord {
            pid: 9999,
            name: "Cron".to_string(),
            started_at: 2000,
        },
    )
    .expect("set task");

    let loaded2 = sm.load();
    assert_eq!(loaded2.len(), 3);
    assert_eq!(loaded2.get("task-3").unwrap().pid, 9999);

    // 5. Remove individual task
    sm.remove_task("task-1").expect("remove task");
    let loaded3 = sm.load();
    assert_eq!(loaded3.len(), 2);
    assert!(!loaded3.contains_key("task-1"));
    assert!(loaded3.contains_key("task-2"));
    assert!(loaded3.contains_key("task-3"));

    // 6. Clear state
    sm.clear().expect("clear state");
    assert!(sm.load().is_empty());
}

#[test]
fn test_state_manager_handles_corrupt_file_gracefully() {
    let dir = tempdir().unwrap();
    let state_file = dir.path().join("state.json");
    std::fs::write(&state_file, "{ invalid json content").unwrap();

    let sm = StateManager::with_path(state_file);
    let loaded = sm.load();
    assert!(loaded.is_empty(), "Corrupt file should fallback to empty map");
}
