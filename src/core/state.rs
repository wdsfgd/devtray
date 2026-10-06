use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskRuntimeRecord {
    pub pid: u32,
    pub name: String,
    #[serde(default)]
    pub started_at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppRuntimeState {
    #[serde(default)]
    pub tasks: HashMap<String, TaskRuntimeRecord>,
}

#[derive(Debug, Clone)]
pub struct StateManager {
    path: PathBuf,
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager {
    pub fn new() -> Self {
        let base_dir = if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
            let trimmed = xdg_cache.trim();
            if !trimmed.is_empty() {
                PathBuf::from(trimmed)
            } else {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".cache")
            }
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".cache")
        };

        let path = base_dir.join("devtray").join("state.json");
        Self { path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> HashMap<String, TaskRuntimeRecord> {
        if !self.path.exists() {
            return HashMap::new();
        }

        let content = match fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => return HashMap::new(),
        };

        if content.trim().is_empty() {
            return HashMap::new();
        }

        match serde_json::from_str::<AppRuntimeState>(&content) {
            Ok(state) => state.tasks,
            Err(e) => {
                eprintln!(
                    "[StateManager] Warning: failed to parse {}: {e}",
                    self.path.display()
                );
                HashMap::new()
            }
        }
    }

    pub fn save(&self, tasks: &HashMap<String, TaskRuntimeRecord>) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let state = AppRuntimeState {
            tasks: tasks.clone(),
        };

        let json = serde_json::to_string_pretty(&state)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let tmp_path = self.path.with_extension("tmp");
        fs::write(&tmp_path, json)?;
        fs::rename(&tmp_path, &self.path)?;

        Ok(())
    }

    pub fn set_task(&self, task_id: &str, record: TaskRuntimeRecord) -> std::io::Result<()> {
        let mut tasks = self.load();
        tasks.insert(task_id.to_string(), record);
        self.save(&tasks)
    }

    pub fn remove_task(&self, task_id: &str) -> std::io::Result<()> {
        let mut tasks = self.load();
        if tasks.remove(task_id).is_some() {
            self.save(&tasks)?;
        }
        Ok(())
    }

    pub fn clear(&self) -> std::io::Result<()> {
        self.save(&HashMap::new())
    }
}
