use devtray::core::config::ConfigManager;
use devtray::core::model::TaskConfig;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_config_save_and_load() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.json");
    let cm = ConfigManager::with_path(config_path.clone());

    let tasks = vec![
        TaskConfig::new("T1", "echo 1", ".", Some("G1")).unwrap(),
        TaskConfig::new("T2", "echo 2", "/tmp", None).unwrap(),
    ];

    cm.save(&tasks).expect("save should succeed");

    let loaded = cm.load().expect("load should succeed");
    assert_eq!(tasks, loaded);
}

#[test]
fn test_config_load_nonexistent() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("nonexistent.json");
    let cm = ConfigManager::with_path(config_path);

    let loaded = cm
        .load()
        .expect("loading nonexistent file should succeed with empty vec");
    assert!(loaded.is_empty());
}

#[test]
fn test_config_load_empty_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("empty.json");
    fs::write(&config_path, "   \n").unwrap();
    let cm = ConfigManager::with_path(config_path);

    let loaded = cm
        .load()
        .expect("loading empty file should succeed with empty vec");
    assert!(loaded.is_empty());
}

#[test]
fn test_config_save_creates_nested_directories() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("sub").join("nested").join("config.json");
    let cm = ConfigManager::with_path(config_path);

    let tasks = vec![TaskConfig::new("T1", "echo 1", ".", None).unwrap()];
    cm.save(&tasks)
        .expect("save should create parent directories");

    let loaded = cm.load().expect("load should succeed");
    assert_eq!(tasks, loaded);
}

#[test]
fn test_path_expansion() {
    let expanded_home = ConfigManager::expand_path("~/.cache");
    assert!(!expanded_home.to_string_lossy().starts_with('~'));

    let expanded_tilde = ConfigManager::expand_path("~");
    assert!(!expanded_tilde.to_string_lossy().starts_with('~'));

    let regular_path = ConfigManager::expand_path("/tmp/test");
    assert_eq!(regular_path.to_string_lossy(), "/tmp/test");

    let rel_path = ConfigManager::expand_path("relative/path");
    assert_eq!(rel_path.to_string_lossy(), "relative/path");
}
