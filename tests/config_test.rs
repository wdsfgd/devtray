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

#[test]
fn test_language_config_persistence() {
    use devtray::core::i18n::Language;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.json");
    let cm = ConfigManager::with_path(config_path);

    // Default language is English when nonexistent
    assert_eq!(cm.load_language().unwrap(), Language::En);

    let tasks = vec![TaskConfig::new("T1", "echo 1", ".", None).unwrap()];

    // Save full config with Zh
    cm.save_full(&tasks, Language::Zh)
        .expect("save_full should succeed");

    // Persisted language is Mandarin
    assert_eq!(cm.load_language().unwrap(), Language::Zh);
    let loaded = cm.load_config().expect("load_config should succeed");
    assert_eq!(loaded.language, Language::Zh);
    assert_eq!(loaded.tasks, tasks);

    // Toggle language
    let toggled = loaded.language.toggle();
    assert_eq!(toggled, Language::En);
    cm.save_language(toggled).expect("save_language should succeed");
    assert_eq!(cm.load_language().unwrap(), Language::En);
    assert_eq!(cm.load().unwrap(), tasks);
}

#[test]
fn test_legacy_array_backward_compatibility() {
    use devtray::core::i18n::Language;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("legacy.json");
    let tasks = vec![TaskConfig::new("T1", "echo 1", ".", None).unwrap()];
    let legacy_json = serde_json::to_string_pretty(&tasks).unwrap();
    fs::write(&config_path, legacy_json).unwrap();

    let cm = ConfigManager::with_path(config_path);
    // Legacy array should load tasks successfully and default language to English
    let loaded_tasks = cm.load().expect("loading legacy array should succeed");
    assert_eq!(loaded_tasks, tasks);
    assert_eq!(cm.load_language().unwrap(), Language::En);
}

