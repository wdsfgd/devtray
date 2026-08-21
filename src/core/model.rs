use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskConfig {
    #[serde(default = "default_id")]
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default = "default_working_directory")]
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

fn default_id() -> String {
    Uuid::new_v4().to_string()
}

fn default_working_directory() -> String {
    ".".to_string()
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ModelError {
    #[error("Task name cannot be empty")]
    EmptyName,
    #[error("Task command cannot be empty")]
    EmptyCommand,
}

impl TaskConfig {
    pub fn new(
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
    ) -> Result<Self, ModelError> {
        let name = name.trim();
        let command = command.trim();
        if name.is_empty() {
            return Err(ModelError::EmptyName);
        }
        if command.is_empty() {
            return Err(ModelError::EmptyCommand);
        }
        let working_directory = if working_directory.trim().is_empty() {
            ".".to_string()
        } else {
            working_directory.trim().to_string()
        };

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            command: command.to_string(),
            working_directory,
            group: group
                .map(|g| g.trim().to_string())
                .filter(|g| !g.is_empty()),
        })
    }

    pub fn validate(&self) -> Result<(), ModelError> {
        if self.name.trim().is_empty() {
            return Err(ModelError::EmptyName);
        }
        if self.command.trim().is_empty() {
            return Err(ModelError::EmptyCommand);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Stopped { exit_code: Option<i32> },
    Running { pid: u32 },
}
