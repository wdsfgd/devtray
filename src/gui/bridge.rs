use crate::core::config::ConfigManager;
use crate::core::logs::LogBroadcaster;
use crate::core::model::{ModelError, TaskConfig};
use crate::core::process::ProcessManager;
use crate::{MainWindow, TaskItem};
use slint::{ComponentHandle, SharedString, VecModel};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ModelError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
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

#[derive(Clone)]
pub struct SlintAppController {
    pub(crate) config_manager: Arc<ConfigManager>,
    pub(crate) process_manager: Arc<ProcessManager>,
    pub(crate) broadcaster: LogBroadcaster,
    pub(crate) task_list: Arc<Mutex<Vec<TaskConfig>>>,
    active_log_subscription: Arc<Mutex<Option<crossbeam_channel::Sender<()>>>>,
}

impl Default for SlintAppController {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let log_dir = PathBuf::from(&home)
            .join(".cache")
            .join("devtray")
            .join("logs");
        let broadcaster = LogBroadcaster::new(log_dir, 1000);
        let process_manager = ProcessManager::new(broadcaster.clone());
        let config_manager = ConfigManager::new();
        Self::new(config_manager, process_manager, broadcaster)
    }
}

impl SlintAppController {
    pub fn new(
        config_manager: ConfigManager,
        process_manager: ProcessManager,
        broadcaster: LogBroadcaster,
    ) -> Self {
        let mut tasks = config_manager.load().unwrap_or_default();
        order_tasks(&mut tasks);
        Self {
            config_manager: Arc::new(config_manager),
            process_manager: Arc::new(process_manager),
            broadcaster,
            task_list: Arc::new(Mutex::new(tasks)),
            active_log_subscription: Arc::new(Mutex::new(None)),
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

    pub fn running_count(&self) -> usize {
        self.process_manager.running_count()
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

    pub fn get_slint_task_items(&self) -> Vec<TaskItem> {
        let tasks = self.task_list.lock().unwrap();
        let len = tasks.len();
        tasks
            .iter()
            .enumerate()
            .map(|(i, task)| {
                let is_running = self.process_manager.is_running(&task.id);
                let can_move_up = tasks[0..i].iter().any(|t| t.group == task.group);
                let can_move_down = tasks[i + 1..len].iter().any(|t| t.group == task.group);
                TaskItem {
                    id: SharedString::from(&task.id),
                    name: SharedString::from(&task.name),
                    command: SharedString::from(&task.command),
                    working_directory: SharedString::from(&task.working_directory),
                    group: SharedString::from(task.group.as_deref().unwrap_or("")),
                    is_running,
                    can_move_up,
                    can_move_down,
                }
            })
            .collect()
    }

    pub fn refresh_tasks(&self, ui: &MainWindow) {
        let items = self.get_slint_task_items();
        let model = Rc::new(VecModel::from(items));
        ui.set_tasks(model.into());
        ui.set_running_count(self.running_count() as i32);
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

        let target_group = tasks[clamped_target].group.clone();
        let mut task = tasks.remove(current_pos);
        task.group = target_group;
        tasks.insert(clamped_target, task);

        self.config_manager.save(&tasks)?;
        Ok(true)
    }

    pub fn start_task(&self, task_id: &str) -> Result<(), BridgeError> {
        let task = self
            .get_task(task_id)
            .ok_or_else(|| BridgeError::TaskNotFound(task_id.to_string()))?;
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

    pub fn start_all(&self) {
        let tasks = self.tasks();
        for task in tasks {
            let _ = self.process_manager.start(&task);
        }
    }

    pub fn stop_all(&self) {
        self.process_manager.stop_all();
    }

    pub fn show_logs_for_task(&self, task_name: &str, ui_weak: &slint::Weak<MainWindow>) {
        let recent = self.get_recent_logs(task_name);
        let initial_text = recent.join("\n");

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_log_viewer_text(initial_text.into());
            ui.set_log_viewer_task_name(task_name.into());
            ui.set_log_viewer_open(true);
        }

        let (stop_tx, stop_rx) = crossbeam_channel::bounded::<()>(1);
        {
            let mut sub_guard = self.active_log_subscription.lock().unwrap();
            *sub_guard = Some(stop_tx);
        }

        let rx = self.subscribe_logs(task_name);
        let ui_weak_clone = ui_weak.clone();
        let target_name = task_name.to_string();

        std::thread::spawn(move || {
            loop {
                crossbeam_channel::select! {
                    recv(stop_rx) -> _ => break,
                    recv(rx) -> msg => {
                        match msg {
                            Ok(line) => {
                                let ui_weak = ui_weak_clone.clone();
                                let line_clone = line;
                                let expected_name = target_name.clone();
                                slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = ui_weak.upgrade() {
                                        if ui.get_log_viewer_open()
                                            && ui.get_log_viewer_task_name().as_str() == expected_name
                                        {
                                            let current = ui.get_log_viewer_text();
                                            let new_text = if current.is_empty() {
                                                line_clone
                                            } else {
                                                format!("{}\n{}", current, line_clone)
                                            };
                                            ui.set_log_viewer_text(new_text.into());
                                        }
                                    }
                                })
                                .ok();
                            }
                            Err(_) => break,
                        }
                    }
                }
            }
        });
    }

    pub fn bind_to_ui(&self, ui: &MainWindow) {
        let controller = self.clone();

        // 1. Toggle Task (start / stop)
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_toggle_task(move |task_id| {
                let id = task_id.as_str();
                if c.is_task_running(id) {
                    let _ = c.stop_task(id);
                } else {
                    let _ = c.start_task(id);
                }
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }

        // 2. Move Task
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_move_task(move |task_id, direction| {
                let _ = c.move_task(task_id.as_str(), direction);
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }

        // 3. Save Task (Add / Edit)
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_save_task(move |id, name, cmd, dir, group| {
                let group_opt = if group.trim().is_empty() {
                    None
                } else {
                    Some(group.as_str())
                };
                let _ = c.save_task(id.as_str(), name.as_str(), cmd.as_str(), dir.as_str(), group_opt);
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }

        // 4. Delete Task
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_confirm_delete_task(move |task_id| {
                let _ = c.delete_task(task_id.as_str());
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_delete_task(move |task_id| {
                let _ = c.delete_task(task_id.as_str());
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }

        // 5. Edit task callback
        {
            ui.on_edit_task(|_| {});
        }

        // 6. Start All / Stop All
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_start_all(move || {
                c.start_all();
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_stop_all(move || {
                c.stop_all();
                if let Some(ui) = ui_weak.upgrade() {
                    c.refresh_tasks(&ui);
                }
            });
        }

        // 7. Show Logs
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_show_logs(move |_task_id, task_name| {
                c.show_logs_for_task(task_name.as_str(), &ui_weak);
            });
        }

        // 8. Clear Logs
        {
            let ui_weak = ui.as_weak();
            ui.on_clear_logs(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_log_viewer_text("".into());
                }
            });
        }

        // 9. Copy Logs
        {
            let ui_weak = ui.as_weak();
            ui.on_copy_logs(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let text = ui.get_log_viewer_text();
                    copy_to_clipboard(text.as_str());
                }
            });
        }

        // 10. Quit App
        {
            let c = controller.clone();
            ui.on_quit_app(move || {
                c.stop_all();
                slint::quit_event_loop().ok();
            });
        }

        // 11. Process exit handler
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            self.process_manager.set_on_exit(move |_task_id| {
                let c = c.clone();
                let ui_weak = ui_weak.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_weak.upgrade() {
                        c.refresh_tasks(&ui);
                    }
                })
                .ok();
            });
        }
    }
}

/// Copies the given text to the system clipboard across Wayland and X11 environments.
pub fn copy_to_clipboard(text: &str) {
    use std::io::Write;

    // 1. Try wl-copy (Wayland)
    if let Ok(mut child) = std::process::Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }

    // 2. Try xclip (X11)
    if let Ok(mut child) = std::process::Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }

    // 3. Try xsel (X11 fallback)
    if let Ok(mut child) = std::process::Command::new("xsel")
        .arg("--clipboard")
        .arg("--input")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}
