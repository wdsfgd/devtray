use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QMap, QMapPair_QString_QVariant, QString, QStringList, QVariant};

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

pub fn task_to_qvariant(task: &TaskConfig) -> QVariant {
    let mut map = QMap::<QMapPair_QString_QVariant>::default();
    map.insert(QString::from("id"), QVariant::from(&QString::from(&task.id)));
    map.insert(QString::from("name"), QVariant::from(&QString::from(&task.name)));
    map.insert(QString::from("command"), QVariant::from(&QString::from(&task.command)));
    map.insert(
        QString::from("working_directory"),
        QVariant::from(&QString::from(&task.working_directory)),
    );
    let group = task.group.as_deref().unwrap_or("");
    map.insert(QString::from("group"), QVariant::from(&QString::from(group)));
    QVariant::from(&map)
}

pub fn tasks_to_qvariant(tasks: &[TaskConfig]) -> QVariant {
    let mut list = QList::<QVariant>::default();
    for task in tasks {
        list.append(task_to_qvariant(task));
    }
    QVariant::from(&list)
}

pub struct TaskManagerBridgeRust {
    pub tasks: QVariant,
    pub(crate) config_manager: Arc<ConfigManager>,
    pub(crate) process_manager: Arc<ProcessManager>,
    pub(crate) broadcaster: LogBroadcaster,
    pub(crate) task_list: Arc<Mutex<Vec<TaskConfig>>>,
}

impl Default for TaskManagerBridgeRust {
    fn default() -> Self {
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
}

impl TaskManagerBridgeRust {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_managers(
        config_manager: ConfigManager,
        process_manager: ProcessManager,
        broadcaster: LogBroadcaster,
    ) -> Self {
        let tasks = config_manager.load().unwrap_or_default();
        let tasks_variant = tasks_to_qvariant(&tasks);
        Self {
            tasks: tasks_variant,
            config_manager: Arc::new(config_manager),
            process_manager: Arc::new(process_manager),
            broadcaster,
            task_list: Arc::new(Mutex::new(tasks)),
        }
    }

    pub fn tasks(&self) -> Vec<TaskConfig> {
        let tasks = self.task_list.lock().unwrap();
        tasks.clone()
    }

    pub fn get_task(&self, task_id: &str) -> Option<TaskConfig> {
        let tasks = self.task_list.lock().unwrap();
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
        let mut tasks = self.task_list.lock().unwrap();
        tasks.push(task.clone());
        self.config_manager.save(&tasks)?;
        Ok(task)
    }

    pub fn add_task_config(&self, task: TaskConfig) -> Result<(), BridgeError> {
        task.validate()?;
        let mut tasks = self.task_list.lock().unwrap();
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

        let mut tasks = self.task_list.lock().unwrap();
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
        let mut tasks = self.task_list.lock().unwrap();
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
        let _ = self.process_manager.stop(task_id);

        let mut tasks = self.task_list.lock().unwrap();
        let initial_len = tasks.len();
        tasks.retain(|t| t.id != task_id);

        if tasks.len() == initial_len {
            return Err(BridgeError::TaskNotFound(task_id.to_string()));
        }

        self.config_manager.save(&tasks)?;
        Ok(())
    }

    pub fn move_task(&self, task_id: &str, direction: i32) -> Result<bool, BridgeError> {
        let mut tasks = self.task_list.lock().unwrap();
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
            let tasks = self.task_list.lock().unwrap();
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
            let tasks = self.task_list.lock().unwrap();
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
            let tasks = self.task_list.lock().unwrap();
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

pub type TaskManagerBridge = TaskManagerBridgeRust;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qstringlist.h");
        type QStringList = cxx_qt_lib::QStringList;
        include!("cxx-qt-lib/qvariant.h");
        type QVariant = cxx_qt_lib::QVariant;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QVariant, tasks)]
        type TaskManagerBridge = super::TaskManagerBridgeRust;
    }

    #[auto_cxx_name]
    unsafe extern "RustQt" {
        #[qinvokable]
        fn start_task(self: Pin<&mut TaskManagerBridge>, task_id: &QString);

        #[qinvokable]
        fn stop_task(self: Pin<&mut TaskManagerBridge>, task_id: &QString);

        #[qinvokable]
        fn start_group(self: Pin<&mut TaskManagerBridge>, group: &QString);

        #[qinvokable]
        fn stop_group(self: Pin<&mut TaskManagerBridge>, group: &QString);

        #[qinvokable]
        fn save_task(
            self: Pin<&mut TaskManagerBridge>,
            id: &QString,
            name: &QString,
            command: &QString,
            working_dir: &QString,
            group: &QString,
        );

        #[qinvokable]
        fn delete_task(self: Pin<&mut TaskManagerBridge>, task_id: &QString);

        #[qinvokable]
        fn move_task(self: Pin<&mut TaskManagerBridge>, task_id: &QString, direction: i32) -> bool;

        #[qinvokable]
        fn is_task_running(self: &TaskManagerBridge, task_id: &QString) -> bool;

        #[qinvokable]
        fn get_recent_logs(self: &TaskManagerBridge, task_name: &QString) -> QStringList;

        #[qinvokable]
        fn refresh_tasks(self: Pin<&mut TaskManagerBridge>);
    }
}

impl qobject::TaskManagerBridge {
    pub fn refresh_tasks(mut self: Pin<&mut Self>) {
        let tasks = self.as_ref().rust().tasks();
        let variant = tasks_to_qvariant(&tasks);
        self.as_mut().set_tasks(variant);
    }

    pub fn start_task(self: Pin<&mut Self>, task_id: &QString) {
        let id_str = task_id.to_string();
        let _ = self.as_ref().rust().start_task(&id_str);
    }

    pub fn stop_task(self: Pin<&mut Self>, task_id: &QString) {
        let id_str = task_id.to_string();
        let _ = self.as_ref().rust().stop_task(&id_str);
    }

    pub fn start_group(self: Pin<&mut Self>, group: &QString) {
        let group_str = group.to_string();
        let _ = self.as_ref().rust().start_group(&group_str);
    }

    pub fn stop_group(self: Pin<&mut Self>, group: &QString) {
        let group_str = group.to_string();
        let _ = self.as_ref().rust().stop_group(&group_str);
    }

    pub fn save_task(
        mut self: Pin<&mut Self>,
        id: &QString,
        name: &QString,
        command: &QString,
        working_dir: &QString,
        group: &QString,
    ) {
        let id_str = id.to_string();
        let name_str = name.to_string();
        let cmd_str = command.to_string();
        let dir_str = working_dir.to_string();
        let group_str = group.to_string();
        let group_opt = if group_str.is_empty() {
            None
        } else {
            Some(group_str.as_str())
        };

        let _ = self
            .as_ref()
            .rust()
            .save_task(&id_str, &name_str, &cmd_str, &dir_str, group_opt);
        self.as_mut().refresh_tasks();
    }

    pub fn delete_task(mut self: Pin<&mut Self>, task_id: &QString) {
        let id_str = task_id.to_string();
        let _ = self.as_ref().rust().delete_task(&id_str);
        self.as_mut().refresh_tasks();
    }

    pub fn move_task(mut self: Pin<&mut Self>, task_id: &QString, direction: i32) -> bool {
        let id_str = task_id.to_string();
        let res = self
            .as_ref()
            .rust()
            .move_task(&id_str, direction)
            .unwrap_or(false);
        if res {
            self.as_mut().refresh_tasks();
        }
        res
    }

    pub fn is_task_running(&self, task_id: &QString) -> bool {
        let id_str = task_id.to_string();
        self.rust().is_task_running(&id_str)
    }

    pub fn get_recent_logs(&self, task_name: &QString) -> QStringList {
        let name_str = task_name.to_string();
        let lines = self.rust().get_recent_logs(&name_str);
        let mut list = QStringList::default();
        for line in lines {
            list.append(QString::from(&line));
        }
        list
    }
}
