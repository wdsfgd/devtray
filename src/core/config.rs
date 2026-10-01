use crate::core::i18n::Language;
use crate::core::model::TaskConfig;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub tasks: Vec<TaskConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: Language::default(),
            tasks: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn new(tasks: Vec<TaskConfig>, language: Language) -> Self {
        Self { language, tasks }
    }
}

impl From<(Vec<TaskConfig>, Language)> for AppConfig {
    fn from((tasks, language): (Vec<TaskConfig>, Language)) -> Self {
        Self { language, tasks }
    }
}

impl From<AppConfig> for (Vec<TaskConfig>, Language) {
    fn from(config: AppConfig) -> Self {
        (config.tasks, config.language)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawConfig {
    Full(AppConfig),
    Legacy(Vec<TaskConfig>),
}

pub struct ConfigManager {
    path: PathBuf,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = PathBuf::from(home)
            .join(".config")
            .join("devtray")
            .join("config.json");
        Self { path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load_config(&self) -> Result<AppConfig, std::io::Error> {
        if !self.path.exists() {
            return Ok(AppConfig::default());
        }
        let content = fs::read_to_string(&self.path)?;
        if content.trim().is_empty() {
            return Ok(AppConfig::default());
        }
        let raw: RawConfig = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        match raw {
            RawConfig::Full(cfg) => Ok(cfg),
            RawConfig::Legacy(tasks) => Ok(AppConfig {
                language: Language::default(),
                tasks,
            }),
        }
    }

    pub fn load(&self) -> Result<Vec<TaskConfig>, std::io::Error> {
        self.load_config().map(|c| c.tasks)
    }

    pub fn load_language(&self) -> Result<Language, std::io::Error> {
        self.load_config().map(|c| c.language)
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(config)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&self.path, data)
    }

    pub fn save_full(&self, tasks: &[TaskConfig], language: Language) -> Result<(), std::io::Error> {
        self.save_config(&AppConfig {
            language,
            tasks: tasks.to_vec(),
        })
    }

    pub fn save(&self, tasks: &[TaskConfig]) -> Result<(), std::io::Error> {
        let current_language = self.load_language().unwrap_or_default();
        self.save_full(tasks, current_language)
    }

    pub fn save_language(&self, language: Language) -> Result<(), std::io::Error> {
        let current_tasks = self.load().unwrap_or_default();
        self.save_full(&current_tasks, language)
    }

    pub fn expand_path<P: AsRef<Path>>(path: P) -> PathBuf {
        let path_str = path.as_ref().to_string_lossy();
        if path_str == "~" {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home);
            }
        } else if let Some(stripped) = path_str.strip_prefix("~/") {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(stripped);
            }
        }
        PathBuf::from(path.as_ref())
    }
}
