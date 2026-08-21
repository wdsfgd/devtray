use devtray::core::model::{ModelError, TaskConfig, TaskStatus};

#[test]
fn test_task_config_serialization() {
    let task = TaskConfig {
        id: "task-1".to_string(),
        name: "Backend Server".to_string(),
        command: "npm run dev".to_string(),
        working_directory: "~/project".to_string(),
        group: Some("Web".to_string()),
    };

    let json = serde_json::to_string(&task).expect("failed to serialize");
    let deserialized: TaskConfig = serde_json::from_str(&json).expect("failed to deserialize");
    assert_eq!(task, deserialized);
}

#[test]
fn test_task_config_deserialization_defaults() {
    let json = r#"{
        "name": "Backend",
        "command": "cargo run"
    }"#;
    let task: TaskConfig = serde_json::from_str(json).expect("failed to deserialize");
    assert!(!task.id.is_empty());
    assert_eq!(task.name, "Backend");
    assert_eq!(task.command, "cargo run");
    assert_eq!(task.working_directory, ".");
    assert_eq!(task.group, None);
}

#[test]
fn test_task_config_validation() {
    let valid_task = TaskConfig::new("Api", "cargo run", ".", Some("Backend"));
    assert!(valid_task.is_ok());
    let task = valid_task.unwrap();
    assert_eq!(task.name, "Api");
    assert_eq!(task.command, "cargo run");
    assert_eq!(task.working_directory, ".");
    assert_eq!(task.group, Some("Backend".to_string()));
    assert!(task.validate().is_ok());

    let empty_name = TaskConfig::new("", "cargo run", ".", None);
    assert_eq!(empty_name, Err(ModelError::EmptyName));

    let whitespace_name = TaskConfig::new("   ", "cargo run", ".", None);
    assert_eq!(whitespace_name, Err(ModelError::EmptyName));

    let empty_cmd = TaskConfig::new("Api", "", ".", None);
    assert_eq!(empty_cmd, Err(ModelError::EmptyCommand));

    let whitespace_cmd = TaskConfig::new("Api", "   ", ".", None);
    assert_eq!(whitespace_cmd, Err(ModelError::EmptyCommand));

    let default_workdir = TaskConfig::new("Api", "cargo run", "", None).unwrap();
    assert_eq!(default_workdir.working_directory, ".");
    assert_eq!(default_workdir.group, None);
}

#[test]
fn test_task_status() {
    let stopped_none = TaskStatus::Stopped { exit_code: None };
    let stopped_some = TaskStatus::Stopped { exit_code: Some(0) };
    let running = TaskStatus::Running { pid: 12345 };

    assert_ne!(stopped_none, stopped_some);
    assert_ne!(stopped_none, running);
    if let TaskStatus::Running { pid } = running {
        assert_eq!(pid, 12345);
    } else {
        panic!("Expected running status");
    }
}
