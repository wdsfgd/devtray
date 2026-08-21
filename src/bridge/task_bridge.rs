use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::config::ConfigManager;
use crate::core::logs::LogBroadcaster;
use crate::core::model::{ModelError, TaskConfig};
use crate::core::process::ProcessManager;

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ModelError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct TaskManagerBridge {
    config_manager: ConfigManager,
    process_manager: ProcessManager,
    broadcaster: LogBroadcaster,
    tasks: Arc<Mutex<Vec<TaskConfig>>>,
}

impl Default for TaskManagerBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManagerBridge {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let log_dir = PathBuf::from(&home)
            .join(".cache")
            .join("devtray")
            .join("logs");
        let broadcaster = LogBroadcaster::new(log_dir, 1000);
        let process_manager = ProcessManager::new(broadcaster.clone());
        let config_manager = ConfigManager::new();
        Self::with_managers(config_manager, process_manager, broadcaster)
    }

    pub fn with_managers(
        config_manager: ConfigManager,
        process_manager: ProcessManager,
        broadcaster: LogBroadcaster,
    ) -> Self {
        let tasks = config_manager.load().unwrap_or_default();
        Self {
            config_manager,
            process_manager,
            broadcaster,
            tasks: Arc::new(Mutex::new(tasks)),
        }
    }

    pub fn tasks(&self) -> Vec<TaskConfig> {
        let tasks = self.tasks.lock().unwrap();
        tasks.clone()
    }

    pub fn get_task(&self, task_id: &str) -> Option<TaskConfig> {
        let tasks = self.tasks.lock().unwrap();
        tasks.iter().find(|t| t.id == task_id).cloned()
    }

    pub fn add_task(
        &self,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
    ) -> Result<TaskConfig, BridgeError> {
        let task = TaskConfig::new(name, command, working_directory, group)?;
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task.clone());
        self.config_manager.save(&tasks)?;
        Ok(task)
    }

    pub fn add_task_config(&self, task: TaskConfig) -> Result<(), BridgeError> {
        task.validate()?;
        let mut tasks = self.tasks.lock().unwrap();
        tasks.push(task);
        self.config_manager.save(&tasks)?;
        Ok(())
    }

    pub fn update_task(
        &self,
        id: &str,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
    ) -> Result<TaskConfig, BridgeError> {
        let name_trimmed = name.trim();
        if name_trimmed.is_empty() {
            return Err(BridgeError::ValidationError(ModelError::EmptyName));
        }
        let command_trimmed = command.trim();
        if command_trimmed.is_empty() {
            return Err(BridgeError::ValidationError(ModelError::EmptyCommand));
        }
        let working_dir = if working_directory.trim().is_empty() {
            ".".to_string()
        } else {
            working_directory.trim().to_string()
        };
        let group_opt = group
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty());

        let mut tasks = self.tasks.lock().unwrap();
        let task = tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| BridgeError::TaskNotFound(id.to_string()))?;

        task.name = name_trimmed.to_string();
        task.command = command_trimmed.to_string();
        task.working_directory = working_dir;
        task.group = group_opt;

        let updated_task = task.clone();
        self.config_manager.save(&tasks)?;
        Ok(updated_task)
    }

    pub fn update_task_config(&self, task: TaskConfig) -> Result<(), BridgeError> {
        task.validate()?;
        let mut tasks = self.tasks.lock().unwrap();
        let existing = tasks
            .iter_mut()
            .find(|t| t.id == task.id)
            .ok_or_else(|| BridgeError::TaskNotFound(task.id.clone()))?;

        *existing = task;
        self.config_manager.save(&tasks)?;
        Ok(())
    }

    pub fn save_task(
        &self,
        id: &str,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
    ) -> Result<TaskConfig, BridgeError> {
        if id.trim().is_empty() {
            self.add_task(name, command, working_directory, group)
        } else {
            self.update_task(id, name, command, working_directory, group)
        }
    }

    pub fn delete_task(&self, task_id: &str) -> Result<(), BridgeError> {
        // Stop the task if running
        let _ = self.process_manager.stop(task_id);

        let mut tasks = self.tasks.lock().unwrap();
        let initial_len = tasks.len();
        tasks.retain(|t| t.id != task_id);

        if tasks.len() == initial_len {
            return Err(BridgeError::TaskNotFound(task_id.to_string()));
        }

        self.config_manager.save(&tasks)?;
        Ok(())
    }

    pub fn move_task(&self, task_id: &str, direction: i32) -> Result<bool, BridgeError> {
        let mut tasks = self.tasks.lock().unwrap();
        let pos = tasks
            .iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| BridgeError::TaskNotFound(task_id.to_string()))?;

        if direction < 0 && pos > 0 {
            tasks.swap(pos, pos - 1);
            self.config_manager.save(&tasks)?;
            Ok(true)
        } else if direction > 0 && pos + 1 < tasks.len() {
            tasks.swap(pos, pos + 1);
            self.config_manager.save(&tasks)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn start_task(&self, task_id: &str) -> Result<(), BridgeError> {
        let task = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .iter()
                .find(|t| t.id == task_id)
                .cloned()
                .ok_or_else(|| BridgeError::TaskNotFound(task_id.to_string()))?
        };

        self.process_manager.start(&task)?;
        Ok(())
    }

    pub fn stop_task(&self, task_id: &str) -> Result<(), BridgeError> {
        self.process_manager.stop(task_id)?;
        Ok(())
    }

    pub fn start_group(&self, group: &str) -> Result<(), BridgeError> {
        let tasks_to_start = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .iter()
                .filter(|t| t.group.as_deref() == Some(group))
                .cloned()
                .collect::<Vec<_>>()
        };

        for task in tasks_to_start {
            self.process_manager.start(&task)?;
        }
        Ok(())
    }

    pub fn stop_group(&self, group: &str) -> Result<(), BridgeError> {
        let task_ids_to_stop = {
            let tasks = self.tasks.lock().unwrap();
            tasks
                .iter()
                .filter(|t| t.group.as_deref() == Some(group))
                .map(|t| t.id.clone())
                .collect::<Vec<_>>()
        };

        for id in task_ids_to_stop {
            self.process_manager.stop(&id)?;
        }
        Ok(())
    }

    pub fn is_task_running(&self, task_id: &str) -> bool {
        self.process_manager.is_running(task_id)
    }

    pub fn get_recent_logs(&self, task_name: &str) -> Vec<String> {
        self.broadcaster.get_recent_lines(task_name)
    }

    pub fn subscribe_logs(&self, task_name: &str) -> crossbeam_channel::Receiver<String> {
        self.broadcaster.subscribe(task_name)
    }

    pub fn stop_all(&self) {
        self.process_manager.stop_all();
    }
}
