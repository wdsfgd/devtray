use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Zh,
}

impl Language {
    pub fn toggle(&self) -> Self {
        match self {
            Language::En => Language::Zh,
            Language::Zh => Language::En,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Zh => "zh",
        }
    }

    pub fn strings(&self) -> I18nStrings {
        I18nStrings::for_language(*self)
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "en" | "english" => Ok(Language::En),
            "zh" | "zh-cn" | "chinese" | "mandarin" => Ok(Language::Zh),
            other => Err(format!("Unknown language: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct I18nStrings {
    // Header & Language switcher
    pub lang_code: &'static str,
    pub lang_switch_label: &'static str,
    pub active_suffix: &'static str,
    pub new_task: &'static str,

    // Empty state
    pub no_tasks: &'static str,
    pub no_tasks_hint: &'static str,

    // Task grouping
    pub uncategorized: &'static str,

    // Task card
    pub running: &'static str,
    pub stopped: &'static str,
    pub logs: &'static str,
    pub edit: &'static str,
    pub del: &'static str,

    // Bottom action bar & Tray
    pub start_all: &'static str,
    pub stop_all: &'static str,
    pub quit: &'static str,
    pub open_window: &'static str,

    // Task modal dialog
    pub add_task_title: &'static str,
    pub edit_task_title: &'static str,
    pub field_name: &'static str,
    pub placeholder_name: &'static str,
    pub field_command: &'static str,
    pub placeholder_command: &'static str,
    pub field_working_dir: &'static str,
    pub field_group: &'static str,
    pub placeholder_group: &'static str,
    pub err_name_empty: &'static str,
    pub err_command_empty: &'static str,
    pub cancel: &'static str,
    pub save: &'static str,

    // Confirm dialog (Delete & Quit)
    pub delete_task_title: &'static str,
    pub delete_confirm_prefix: &'static str,
    pub delete_confirm_suffix: &'static str,
    pub delete_task_submessage: &'static str,
    pub delete_confirm_button: &'static str,
    pub quit_title: &'static str,
    pub quit_message: &'static str,
    pub quit_submessage: &'static str,

    // Log viewer modal
    pub logs_title_prefix: &'static str,
    pub no_logs_recorded: &'static str,
    pub copy_logs: &'static str,
    pub clear_view: &'static str,
    pub close: &'static str,
}

pub const EN_STRINGS: I18nStrings = I18nStrings {
    lang_code: "en",
    lang_switch_label: "ZH",
    active_suffix: " active",
    new_task: "+ New Task",

    no_tasks: "No tasks configured",
    no_tasks_hint: "Click '+ New Task' above to start managing background services.",

    uncategorized: "UNCATEGORIZED",

    running: "Running",
    stopped: "Stopped",
    logs: "Logs",
    edit: "Edit",
    del: "Del",

    start_all: "▶ Start All",
    stop_all: "⏹ Stop All",
    quit: "Quit",
    open_window: "Open Window",

    add_task_title: "Add Task",
    edit_task_title: "Edit Task",
    field_name: "Name",
    placeholder_name: "e.g. Frontend Server",
    field_command: "Command",
    placeholder_command: "e.g. npm run dev",
    field_working_dir: "Working Directory",
    field_group: "Group (Optional)",
    placeholder_group: "e.g. Web",
    err_name_empty: "Task name cannot be empty",
    err_command_empty: "Task command cannot be empty",
    cancel: "Cancel",
    save: "Save",

    delete_task_title: "Delete Task",
    delete_confirm_prefix: "Delete task '",
    delete_confirm_suffix: "'?",
    delete_task_submessage: "This will stop the task if running and remove it permanently.",
    delete_confirm_button: "Delete",
    quit_title: "Quit DevTray",
    quit_message: "Are you sure you want to quit?",
    quit_submessage: "This will stop all running tasks.",

    logs_title_prefix: "Logs: ",
    no_logs_recorded: "(No logs recorded yet...)",
    copy_logs: "Copy Logs",
    clear_view: "Clear View",
    close: "Close",
};

pub const ZH_STRINGS: I18nStrings = I18nStrings {
    lang_code: "zh",
    lang_switch_label: "EN",
    active_suffix: " 运行中",
    new_task: "+ 新建任务",

    no_tasks: "未配置任务",
    no_tasks_hint: "点击上方的“+ 新建任务”开始管理后台服务。",

    uncategorized: "未分类",

    running: "运行中",
    stopped: "已停止",
    logs: "日志",
    edit: "编辑",
    del: "删除",

    start_all: "▶ 全部启动",
    stop_all: "⏹ 全部停止",
    quit: "退出",
    open_window: "打开窗口",

    add_task_title: "新建任务",
    edit_task_title: "编辑任务",
    field_name: "名称",
    placeholder_name: "例如：前端开发服务",
    field_command: "启动命令",
    placeholder_command: "例如：npm run dev",
    field_working_dir: "工作目录",
    field_group: "分组（可选）",
    placeholder_group: "例如：Web",
    err_name_empty: "任务名称不能为空",
    err_command_empty: "启动命令不能为空",
    cancel: "取消",
    save: "保存",

    delete_task_title: "删除任务",
    delete_confirm_prefix: "确认删除任务“",
    delete_confirm_suffix: "”？",
    delete_task_submessage: "如果任务正在运行将被终止，并从配置中永久删除。",
    delete_confirm_button: "删除",
    quit_title: "退出 DevTray",
    quit_message: "确定要退出程序吗？",
    quit_submessage: "这将停止所有正在运行的任务。",

    logs_title_prefix: "日志：",
    no_logs_recorded: "（暂无日志记录...）",
    copy_logs: "复制日志",
    clear_view: "清空视图",
    close: "关闭",
};

impl I18nStrings {
    pub const fn en() -> Self {
        EN_STRINGS
    }

    pub const fn zh() -> Self {
        ZH_STRINGS
    }

    pub fn for_language(lang: Language) -> Self {
        match lang {
            Language::En => Self::en(),
            Language::Zh => Self::zh(),
        }
    }

    pub fn delete_confirm_message(&self, task_name: &str) -> String {
        format!("{}{}{}", self.delete_confirm_prefix, task_name, self.delete_confirm_suffix)
    }
}

impl Default for I18nStrings {
    fn default() -> Self {
        Self::en()
    }
}

impl From<Language> for I18nStrings {
    fn from(lang: Language) -> Self {
        Self::for_language(lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_defaults_and_toggle() {
        let lang = Language::default();
        assert_eq!(lang, Language::En);
        let toggled = lang.toggle();
        assert_eq!(toggled, Language::Zh);
        assert_eq!(toggled.toggle(), Language::En);
    }

    #[test]
    fn test_language_serialization() {
        let en_json = serde_json::to_string(&Language::En).unwrap();
        assert_eq!(en_json, "\"en\"");
        let zh_json = serde_json::to_string(&Language::Zh).unwrap();
        assert_eq!(zh_json, "\"zh\"");

        let deserialized_en: Language = serde_json::from_str("\"en\"").unwrap();
        assert_eq!(deserialized_en, Language::En);
        let deserialized_zh: Language = serde_json::from_str("\"zh\"").unwrap();
        assert_eq!(deserialized_zh, Language::Zh);
    }

    #[test]
    fn test_i18n_strings_switch_label() {
        let en_strings = I18nStrings::for_language(Language::En);
        assert_eq!(en_strings.lang_switch_label, "ZH");

        let zh_strings = I18nStrings::for_language(Language::Zh);
        assert_eq!(zh_strings.lang_switch_label, "EN");
    }

    #[test]
    fn test_delete_confirm_message_formatting() {
        let en_strings = I18nStrings::en();
        assert_eq!(en_strings.delete_confirm_message("Web"), "Delete task 'Web'?");

        let zh_strings = I18nStrings::zh();
        assert_eq!(zh_strings.delete_confirm_message("Web"), "确认删除任务“Web”？");
    }

    #[test]
    fn test_open_window_localization() {
        assert_eq!(I18nStrings::en().open_window, "Open Window");
        assert_eq!(I18nStrings::zh().open_window, "打开窗口");
    }
}
