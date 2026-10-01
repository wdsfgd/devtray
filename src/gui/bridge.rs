use crate::core::config::ConfigManager;
use crate::core::i18n::{I18nStrings, Language};
use crate::core::logs::LogBroadcaster;
use crate::core::model::{ModelError, TaskConfig};
use crate::core::process::ProcessManager;
use crate::{I18nData, MainWindow, TaskItem};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskSnapshot {
    pub id: String,
    pub name: String,
    pub command: String,
    pub working_directory: String,
    pub group: String,
    pub is_running: bool,
    pub can_move_up: bool,
    pub can_move_down: bool,
}

impl From<&TaskItem> for TaskSnapshot {
    fn from(item: &TaskItem) -> Self {
        Self {
            id: item.id.to_string(),
            name: item.name.to_string(),
            command: item.command.to_string(),
            working_directory: item.working_directory.to_string(),
            group: item.group.to_string(),
            is_running: item.is_running,
            can_move_up: item.can_move_up,
            can_move_down: item.can_move_down,
        }
    }
}

pub type TaskItemSnapshot = TaskSnapshot;

impl From<I18nStrings> for I18nData {
    fn from(s: I18nStrings) -> Self {
        Self {
            lang_code: SharedString::from(s.lang_code),
            lang_switch_label: SharedString::from(s.lang_switch_label),
            active_suffix: SharedString::from(s.active_suffix),
            new_task: SharedString::from(s.new_task),
            no_tasks: SharedString::from(s.no_tasks),
            no_tasks_hint: SharedString::from(s.no_tasks_hint),
            uncategorized: SharedString::from(s.uncategorized),
            running: SharedString::from(s.running),
            stopped: SharedString::from(s.stopped),
            logs: SharedString::from(s.logs),
            edit: SharedString::from(s.edit),
            del: SharedString::from(s.del),
            start_all: SharedString::from(s.start_all),
            stop_all: SharedString::from(s.stop_all),
            quit: SharedString::from(s.quit),
            add_task_title: SharedString::from(s.add_task_title),
            edit_task_title: SharedString::from(s.edit_task_title),
            field_name: SharedString::from(s.field_name),
            placeholder_name: SharedString::from(s.placeholder_name),
            field_command: SharedString::from(s.field_command),
            placeholder_command: SharedString::from(s.placeholder_command),
            field_working_dir: SharedString::from(s.field_working_dir),
            field_group: SharedString::from(s.field_group),
            placeholder_group: SharedString::from(s.placeholder_group),
            field_stop_command: SharedString::from(s.field_stop_command),
            placeholder_stop_command: SharedString::from(s.placeholder_stop_command),
            err_name_empty: SharedString::from(s.err_name_empty),
            err_command_empty: SharedString::from(s.err_command_empty),
            cancel: SharedString::from(s.cancel),
            save: SharedString::from(s.save),
            delete_task_title: SharedString::from(s.delete_task_title),
            delete_confirm_prefix: SharedString::from(s.delete_confirm_prefix),
            delete_confirm_suffix: SharedString::from(s.delete_confirm_suffix),
            delete_task_submessage: SharedString::from(s.delete_task_submessage),
            delete_confirm_button: SharedString::from(s.delete_confirm_button),
            quit_title: SharedString::from(s.quit_title),
            quit_message: SharedString::from(s.quit_message),
            quit_submessage: SharedString::from(s.quit_submessage),
            logs_title_prefix: SharedString::from(s.logs_title_prefix),
            no_logs_recorded: SharedString::from(s.no_logs_recorded),
            copy_logs: SharedString::from(s.copy_logs),
            clear_view: SharedString::from(s.clear_view),
            close: SharedString::from(s.close),
            search_logs: SharedString::from(s.search_logs),
            no_matches: SharedString::from(s.no_matches),
        }
    }
}

impl From<Language> for I18nData {
    fn from(lang: Language) -> Self {
        I18nData::from(lang.strings())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LogSearchState {
    pub query: String,
    pub matches: Vec<(usize, usize)>,
    pub current_index: usize,
}

#[derive(Clone)]
pub struct SlintAppController {
    pub(crate) config_manager: Arc<ConfigManager>,
    pub(crate) process_manager: Arc<ProcessManager>,
    pub(crate) broadcaster: LogBroadcaster,
    pub(crate) task_list: Arc<Mutex<Vec<TaskConfig>>>,
    pub(crate) last_task_snapshot: Arc<Mutex<Option<Vec<TaskSnapshot>>>>,
    active_log_subscription: Arc<Mutex<Option<crossbeam_channel::Sender<()>>>>,
    pub(crate) language: Arc<Mutex<Language>>,
    pub(crate) log_search_state: Arc<Mutex<LogSearchState>>,
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
        let language = config_manager.load_language().unwrap_or_default();
        Self {
            config_manager: Arc::new(config_manager),
            process_manager: Arc::new(process_manager),
            broadcaster,
            task_list: Arc::new(Mutex::new(tasks)),
            last_task_snapshot: Arc::new(Mutex::new(None)),
            active_log_subscription: Arc::new(Mutex::new(None)),
            language: Arc::new(Mutex::new(language)),
            log_search_state: Arc::new(Mutex::new(LogSearchState::default())),
        }
    }

    pub fn language(&self) -> Language {
        *self.language.lock().unwrap()
    }

    pub fn get_language(&self) -> Language {
        self.language()
    }

    pub fn get_i18n_data(&self) -> I18nData {
        let lang = self.language();
        I18nData::from(lang)
    }

    pub fn set_language(&self, new_lang: Language) {
        let mut lang = self.language.lock().unwrap();
        *lang = new_lang;
        drop(lang);
        let _ = self.config_manager.save_language(new_lang);
    }

    pub fn toggle_language(&self) -> Language {
        let mut lang = self.language.lock().unwrap();
        let new_lang = lang.toggle();
        *lang = new_lang;
        drop(lang);
        let _ = self.config_manager.save_language(new_lang);
        new_lang
    }

    pub fn invalidate_snapshot(&self) {
        let mut snapshot = self.last_task_snapshot.lock().unwrap();
        *snapshot = None;
    }

    pub fn last_snapshot(&self) -> Option<Vec<TaskSnapshot>> {
        self.last_task_snapshot.lock().unwrap().clone()
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

    pub fn refresh_tasks(&self, ui: &MainWindow) -> bool {
        let items = self.get_slint_task_items();
        let current_snapshots: Vec<TaskSnapshot> = items.iter().map(TaskSnapshot::from).collect();
        let running = self.running_count() as i32;

        let mut snapshot_lock = self.last_task_snapshot.lock().unwrap();
        let needs_model_update = match &*snapshot_lock {
            Some(cached) => cached != &current_snapshots,
            None => true,
        };

        if needs_model_update {
            *snapshot_lock = Some(current_snapshots);
            let model = Rc::new(VecModel::from(items));
            ui.set_tasks(model.into());
            ui.set_running_count(running);
            let groups = self.get_groups();
            let groups_model = Rc::new(VecModel::from(
                groups.into_iter().map(SharedString::from).collect::<Vec<_>>(),
            ));
            ui.set_available_groups(groups_model.into());
            true
        } else {
            if ui.get_running_count() != running {
                ui.set_running_count(running);
            }
            false
        }
    }

    pub fn add_task(
        &self,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
    ) -> Result<TaskConfig, BridgeError> {
        self.add_task_with_stop_command(name, command, working_directory, group, None)
    }

    pub fn add_task_with_stop_command(
        &self,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
        stop_command: Option<&str>,
    ) -> Result<TaskConfig, BridgeError> {
        let task = TaskConfig::with_stop_command(name, command, working_directory, group, stop_command)?;
        let mut tasks = self.task_list.lock().unwrap();
        tasks.push(task.clone());
        order_tasks(&mut tasks);
        self.config_manager.save(&tasks)?;
        drop(tasks);
        self.invalidate_snapshot();
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
        let existing_stop = self.get_task(id).and_then(|t| t.stop_command);
        self.update_task_with_stop_command(
            id,
            name,
            command,
            working_directory,
            group,
            existing_stop.as_deref(),
        )
    }

    pub fn update_task_with_stop_command(
        &self,
        id: &str,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
        stop_command: Option<&str>,
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
        let stop_cmd_opt = stop_command
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let mut tasks = self.task_list.lock().unwrap();
        let task = tasks
            .iter_mut()
            .find(|t| t.id == id)
            .ok_or_else(|| BridgeError::TaskNotFound(id.to_string()))?;

        task.name = name_trimmed.to_string();
        task.command = command_trimmed.to_string();
        task.working_directory = working_dir;
        task.group = group_opt;
        task.stop_command = stop_cmd_opt;

        let updated_task = task.clone();
        order_tasks(&mut tasks);
        self.config_manager.save(&tasks)?;
        drop(tasks);
        self.invalidate_snapshot();
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
        self.save_task_with_stop_command(id, name, command, working_directory, group, None)
    }

    pub fn save_task_with_stop_command(
        &self,
        id: &str,
        name: &str,
        command: &str,
        working_directory: &str,
        group: Option<&str>,
        stop_command: Option<&str>,
    ) -> Result<TaskConfig, BridgeError> {
        if id.trim().is_empty() {
            self.add_task_with_stop_command(name, command, working_directory, group, stop_command)
        } else {
            self.update_task_with_stop_command(
                id,
                name,
                command,
                working_directory,
                group,
                stop_command,
            )
        }
    }

    pub fn open_add_dialog(&self, ui: &MainWindow) {
        let groups = self.get_groups();
        let groups_model = Rc::new(VecModel::from(
            groups.into_iter().map(SharedString::from).collect::<Vec<_>>(),
        ));
        ui.set_available_groups(groups_model.into());
        ui.set_task_dialog_id("".into());
        ui.set_task_dialog_name("".into());
        ui.set_task_dialog_command("".into());
        ui.set_task_dialog_working_dir(".".into());
        ui.set_task_dialog_group("".into());
        ui.set_task_dialog_stop_command("".into());
        ui.set_task_dialog_error("".into());
        ui.set_task_dialog_is_edit(false);
        ui.set_task_dialog_open(true);
    }

    pub fn open_edit_dialog(&self, task_id: &str, ui: &MainWindow) -> bool {
        let groups = self.get_groups();
        let groups_model = Rc::new(VecModel::from(
            groups.into_iter().map(SharedString::from).collect::<Vec<_>>(),
        ));
        ui.set_available_groups(groups_model.into());

        if let Some(task) = self.get_task(task_id) {
            ui.set_task_dialog_id(task.id.clone().into());
            ui.set_task_dialog_name(task.name.into());
            ui.set_task_dialog_command(task.command.into());
            ui.set_task_dialog_working_dir(task.working_directory.into());
            ui.set_task_dialog_group(task.group.unwrap_or_default().into());
            ui.set_task_dialog_stop_command(task.stop_command.unwrap_or_default().into());
            ui.set_task_dialog_error("".into());
            ui.set_task_dialog_is_edit(true);
            ui.set_task_dialog_open(true);
            true
        } else {
            false
        }
    }

    pub fn delete_task(&self, task_id: &str) -> Result<(), BridgeError> {
        if let Some(task) = self.get_task(task_id) {
            let _ = self.process_manager.stop_with_config(&task);
        } else {
            let _ = self.process_manager.stop(task_id);
        }

        let mut tasks = self.task_list.lock().unwrap();
        let initial_len = tasks.len();
        tasks.retain(|t| t.id != task_id);

        if tasks.len() == initial_len {
            return Err(BridgeError::TaskNotFound(task_id.to_string()));
        }

        self.config_manager.save(&tasks)?;
        drop(tasks);
        self.invalidate_snapshot();
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
            drop(tasks);
            self.invalidate_snapshot();
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
        drop(tasks);
        self.invalidate_snapshot();
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
        if let Some(task) = self.get_task(task_id) {
            self.process_manager.stop_with_config(&task)?;
        } else {
            self.process_manager.stop(task_id)?;
        }
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
        let tasks_to_stop = {
            let tasks = self.task_list.lock().unwrap();
            tasks
                .iter()
                .filter(|t| t.group.as_deref() == Some(group))
                .cloned()
                .collect::<Vec<_>>()
        };

        for task in tasks_to_stop {
            self.process_manager.stop_with_config(&task)?;
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
        let tasks = self.tasks();
        for task in &tasks {
            let _ = self.process_manager.stop_with_config(task);
        }
        self.process_manager.stop_all();
    }

    pub fn show_logs_for_task(&self, task_name: &str, ui_weak: &slint::Weak<MainWindow>) {
        let recent = self.get_recent_logs(task_name);
        let initial_text = recent.join("\n");

        if let Some(ui) = ui_weak.upgrade() {
            ui.set_log_viewer_text(initial_text.into());
            ui.set_log_viewer_task_name(task_name.into());
            ui.set_log_viewer_open(true);
            ui.set_log_viewer_search_query("".into());
            ui.set_log_viewer_match_count(0);
            ui.set_log_viewer_match_index(0);
            ui.invoke_set_log_selection(0, 0);
            let mut search_guard = self.log_search_state.lock().unwrap();
            *search_guard = LogSearchState::default();
        }

        let (stop_tx, stop_rx) = crossbeam_channel::bounded::<()>(1);
        {
            let mut sub_guard = self.active_log_subscription.lock().unwrap();
            *sub_guard = Some(stop_tx);
        }

        let rx = self.subscribe_logs(task_name);
        let ui_weak_clone = ui_weak.clone();
        let target_name = task_name.to_string();
        let search_state_clone = self.log_search_state.clone();

        std::thread::spawn(move || loop {
            crossbeam_channel::select! {
                recv(stop_rx) -> _ => break,
                recv(rx) -> msg => {
                    match msg {
                        Ok(line) => {
                            let ui_weak = ui_weak_clone.clone();
                            let line_clone = line;
                            let expected_name = target_name.clone();
                            let search_state = search_state_clone.clone();
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
                                        ui.set_log_viewer_text(new_text.as_str().into());

                                        let mut search_guard = search_state.lock().unwrap();
                                        if !search_guard.query.is_empty() {
                                            search_guard.matches = crate::core::logs::find_matches(
                                                new_text.as_str(),
                                                &search_guard.query,
                                            );
                                            ui.set_log_viewer_match_count(search_guard.matches.len() as i32);
                                        }
                                    }
                                }
                            })
                            .ok();
                        }
                        Err(_) => break,
                    }
                }
            }
        });
    }

    pub fn search_logs(&self, query: &str, ui: &MainWindow) {
        let mut state = self.log_search_state.lock().unwrap();
        state.query = query.to_string();
        if query.is_empty() {
            state.matches.clear();
            state.current_index = 0;
            ui.set_log_viewer_match_count(0);
            ui.set_log_viewer_match_index(0);
            ui.invoke_set_log_selection(0, 0);
            return;
        }

        let text = ui.get_log_viewer_text();
        let matches = crate::core::logs::find_matches(text.as_str(), query);
        let count = matches.len();
        state.matches = matches;
        state.current_index = 0;

        ui.set_log_viewer_match_count(count as i32);
        ui.set_log_viewer_match_index(0);

        if let Some(&(start, end)) = state.matches.first() {
            ui.invoke_set_log_selection(start as i32, end as i32);
        } else {
            ui.invoke_set_log_selection(0, 0);
        }
    }

    pub fn find_next_log_match(&self, ui: &MainWindow) {
        let mut state = self.log_search_state.lock().unwrap();
        if state.matches.is_empty() {
            return;
        }
        state.current_index = (state.current_index + 1) % state.matches.len();
        let idx = state.current_index;
        let (start, end) = state.matches[idx];
        ui.set_log_viewer_match_index(idx as i32);
        ui.invoke_set_log_selection(start as i32, end as i32);
    }

    pub fn find_prev_log_match(&self, ui: &MainWindow) {
        let mut state = self.log_search_state.lock().unwrap();
        if state.matches.is_empty() {
            return;
        }
        state.current_index = if state.current_index == 0 {
            state.matches.len() - 1
        } else {
            state.current_index - 1
        };
        let idx = state.current_index;
        let (start, end) = state.matches[idx];
        ui.set_log_viewer_match_index(idx as i32);
        ui.invoke_set_log_selection(start as i32, end as i32);
    }

    pub fn log_search_state(&self) -> LogSearchState {
        self.log_search_state.lock().unwrap().clone()
    }

    pub fn bind_to_ui(&self, ui: &MainWindow) {
        ui.set_tr(self.get_i18n_data());

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
            ui.on_save_task(move |id, name, cmd, dir, group, stop_cmd| {
                let group_opt = if group.trim().is_empty() {
                    None
                } else {
                    Some(group.as_str())
                };
                let stop_cmd_opt = if stop_cmd.trim().is_empty() {
                    None
                } else {
                    Some(stop_cmd.as_str())
                };
                let _ = c.save_task_with_stop_command(
                    id.as_str(),
                    name.as_str(),
                    cmd.as_str(),
                    dir.as_str(),
                    group_opt,
                    stop_cmd_opt,
                );
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
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_edit_task(move |task_id| {
                if let Some(ui) = ui_weak.upgrade() {
                    c.open_edit_dialog(task_id.as_str(), &ui);
                }
            });
        }

        // 5b. Open Add Task callback
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_open_add_task(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    c.open_add_dialog(&ui);
                }
            });
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
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_clear_logs(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_log_viewer_text("".into());
                    ui.set_log_viewer_search_query("".into());
                    ui.set_log_viewer_match_count(0);
                    ui.set_log_viewer_match_index(0);
                    ui.invoke_set_log_selection(0, 0);
                    let mut search_guard = c.log_search_state.lock().unwrap();
                    *search_guard = LogSearchState::default();
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

        // 12. Toggle language
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_toggle_language(move || {
                c.toggle_language();
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_tr(c.get_i18n_data());
                }
            });
        }

        // 13. Log Search
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_log_search(move |query| {
                if let Some(ui) = ui_weak.upgrade() {
                    c.search_logs(query.as_str(), &ui);
                }
            });
        }
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_log_find_next(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    c.find_next_log_match(&ui);
                }
            });
        }
        {
            let c = controller.clone();
            let ui_weak = ui.as_weak();
            ui.on_log_find_prev(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    c.find_prev_log_match(&ui);
                }
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
