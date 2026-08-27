use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QList, QMap, QMapPair_QString_QVariant, QString, QStringList, QVariant};

use crate::core::config::ConfigManager;
use crate::core::logs::LogBroadcaster;
use crate::core::model::{ModelError, TaskConfig};
use crate::core::process::ProcessManager;

const ICON_BYTES: &[u8] = include_bytes!("../../assets/icon.png");

pub fn extract_app_icon() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cache_dir = PathBuf::from(&home).join(".cache").join("devtray");
    let _ = std::fs::create_dir_all(&cache_dir);
    let icon_path = cache_dir.join("devtray-icon.png");
    let _ = std::fs::write(&icon_path, ICON_BYTES);

    let temp_icon = std::env::temp_dir().join("devtray-icon.png");
    let _ = std::fs::write(&temp_icon, ICON_BYTES);

    format!("file://{}", icon_path.display())
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ModelError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub fn task_to_qvariant(task: &TaskConfig, is_running: bool) -> QVariant {
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
    map.insert(QString::from("is_running"), QVariant::from(&is_running));
    QVariant::from(&map)
}

pub fn tasks_to_qvariant(tasks: &[TaskConfig], pm: Option<&ProcessManager>) -> QVariant {
    let mut list = QList::<QVariant>::default();
    for task in tasks {
        let is_running = pm.map(|p| p.is_running(&task.id)).unwrap_or(false);
        list.append(task_to_qvariant(task, is_running));
    }
    QVariant::from(&list)
}

pub fn order_tasks(tasks: &mut Vec<TaskConfig>) {
    let mut groups: Vec<String> = tasks
        .iter()
        .filter_map(|t| t.group.as_deref())
        .map(|g| g.trim().to_string())
        .filter(|g| !g.is_empty())
        .collect();
    groups.sort();
    groups.dedup();

    let mut ordered: Vec<TaskConfig> = Vec::with_capacity(tasks.len());
    for group in &groups {
        for t in tasks.iter() {
            if t.group.as_deref().map(|g| g.trim()) == Some(group.as_str()) {
                ordered.push(t.clone());
            }
        }
    }
    for t in tasks.iter() {
        let is_uncat = match &t.group {
            None => true,
            Some(g) => g.trim().is_empty(),
        };
        if is_uncat {
            ordered.push(t.clone());
        }
    }
    *tasks = ordered;
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
        let mut tasks = config_manager.load().unwrap_or_default();
        order_tasks(&mut tasks);
        let tasks_variant = tasks_to_qvariant(&tasks, Some(&process_manager));
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

    pub fn get_groups(&self) -> Vec<String> {
        let tasks = self.task_list.lock().unwrap();
        let mut groups: Vec<String> = tasks
            .iter()
            .filter_map(|t| t.group.as_deref())
            .map(|g| g.trim().to_string())
            .filter(|g| !g.is_empty())
            .collect();
        groups.sort();
        groups.dedup();
        groups
    }

    pub fn running_count(&self) -> i32 {
        self.process_manager.running_count() as i32
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
        order_tasks(&mut tasks);
        self.config_manager.save(&tasks)?;
        Ok(task)
    }

    pub fn add_task_config(&self, task: TaskConfig) -> Result<(), BridgeError> {
        task.validate()?;
        let mut tasks = self.task_list.lock().unwrap();
        tasks.push(task);
        order_tasks(&mut tasks);
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
        order_tasks(&mut tasks);
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
        order_tasks(&mut tasks);
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

        let task_group = tasks[pos].group.clone();

        let target_pos = if direction < 0 {
            // Move up: find previous task in the SAME group
            (0..pos).rev().find(|&i| tasks[i].group == task_group)
        } else {
            // Move down: find next task in the SAME group
            (pos + 1..tasks.len()).find(|&i| tasks[i].group == task_group)
        };

        if let Some(target) = target_pos {
            tasks.swap(pos, target);
            self.config_manager.save(&tasks)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn reorder_task(&self, task_id: &str, target_index: usize) -> Result<bool, BridgeError> {
        let mut tasks = self.task_list.lock().unwrap();
        let current_pos = tasks
            .iter()
            .position(|t| t.id == task_id)
            .ok_or_else(|| BridgeError::TaskNotFound(task_id.to_string()))?;

        if tasks.is_empty() {
            return Ok(false);
        }

        let clamped_target = target_index.min(tasks.len() - 1);
        if current_pos == clamped_target {
            return Ok(false);
        }

        // Determine target group from destination position BEFORE removing
        let target_group = tasks[clamped_target].group.clone();

        let mut task = tasks.remove(current_pos);
        task.group = target_group;

        let insert_idx = clamped_target.min(tasks.len());
        tasks.insert(insert_idx, task);

        order_tasks(&mut tasks);
        self.config_manager.save(&tasks)?;
        Ok(true)
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

        if let Err(e) = self.process_manager.start(&task) {
            let _ = self.broadcaster.append(
                &task.name,
                &format!("[DevTray Error] Failed to start task: {}", e),
            );
            return Err(BridgeError::IoError(e));
        }
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
        fn reorder_task(
            self: Pin<&mut TaskManagerBridge>,
            task_id: &QString,
            target_index: i32,
        ) -> bool;

        #[qinvokable]
        fn is_task_running(self: &TaskManagerBridge, task_id: &QString) -> bool;

        #[qinvokable]
        fn running_count(self: &TaskManagerBridge) -> i32;

        #[qinvokable]
        fn get_groups(self: &TaskManagerBridge) -> QStringList;

        #[qinvokable]
        fn get_recent_logs(self: &TaskManagerBridge, task_name: &QString) -> QStringList;

        #[qinvokable]
        fn refresh_tasks(self: Pin<&mut TaskManagerBridge>);

        #[qinvokable]
        fn stop_all(self: Pin<&mut TaskManagerBridge>);

        #[qinvokable]
        fn icon_path(self: &TaskManagerBridge) -> QString;
    }
}

impl qobject::TaskManagerBridge {
    pub fn refresh_tasks(mut self: Pin<&mut Self>) {
        let tasks = self.as_ref().rust().tasks();
        let variant = tasks_to_qvariant(&tasks, Some(&self.as_ref().rust().process_manager));
        self.as_mut().set_tasks(variant);
    }

    pub fn start_task(mut self: Pin<&mut Self>, task_id: &QString) {
        let id_str = task_id.to_string();
        let _ = self.as_ref().rust().start_task(&id_str);
        self.as_mut().refresh_tasks();
    }

    pub fn stop_task(mut self: Pin<&mut Self>, task_id: &QString) {
        let id_str = task_id.to_string();
        let _ = self.as_ref().rust().stop_task(&id_str);
        self.as_mut().refresh_tasks();
    }

    pub fn start_group(mut self: Pin<&mut Self>, group: &QString) {
        let group_str = group.to_string();
        let _ = self.as_ref().rust().start_group(&group_str);
        self.as_mut().refresh_tasks();
    }

    pub fn stop_group(mut self: Pin<&mut Self>, group: &QString) {
        let group_str = group.to_string();
        let _ = self.as_ref().rust().stop_group(&group_str);
        self.as_mut().refresh_tasks();
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

    pub fn reorder_task(
        mut self: Pin<&mut Self>,
        task_id: &QString,
        target_index: i32,
    ) -> bool {
        let id_str = task_id.to_string();
        if target_index < 0 {
            return false;
        }
        let res = self
            .as_ref()
            .rust()
            .reorder_task(&id_str, target_index as usize)
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

    pub fn running_count(&self) -> i32 {
        self.rust().running_count()
    }

    pub fn get_groups(&self) -> QStringList {
        let groups = self.rust().get_groups();
        let mut list = QStringList::default();
        for g in groups {
            list.append(QString::from(&g));
        }
        list
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

    pub fn stop_all(mut self: Pin<&mut Self>) {
        self.as_ref().rust().stop_all();
        self.as_mut().refresh_tasks();
    }

    pub fn icon_path(&self) -> QString {
        QString::from(&extract_app_icon())
    }
}
