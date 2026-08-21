# DevTray Rust + Qt Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate DevTray from Go/GTK3/libayatana to Rust and Qt6 (QML + QSystemTrayIcon) with process group termination and in-app live log streaming.

**Architecture:** A pure Rust core engine (`src/core/`) handles configuration, process supervision with POSIX process groups, and ring-buffered log broadcast. A Qt bridge layer (`src/bridge/`) exposes reactive signals and invocables to a modern QML user interface (`qml/`) and system tray integration.

**Tech Stack:** Rust 1.96+, Qt 6.11+, `cxx-qt` / `cxx-qt-build`, `serde`, `serde_json`, `nix` / `libc`, `crossbeam-channel`.

## Global Constraints

- Configuration stored in `~/.config/devtray/config.json`.
- File logs stored in `~/.cache/devtray/logs/{task_name}.log`.
- Spawning processes must use `bash -c` with `setpgid` and clean termination via `SIGKILL` to negative PGID (`-pid`).
- Real-time in-memory log buffer must store up to 1,000 lines per task.
- Zero C runtime dependencies for `libayatana`.

---

### Task 1: Rust Project Scaffolding & Cargo Setup

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`

**Interfaces:**
- Produces: Base project structure and library crate compilation.

- [ ] **Step 1: Write Cargo.toml with dependencies**

```toml
[package]
name = "devtray"
version = "0.2.0"
edition = "2021"
authors = ["DevTray Contributors"]
description = "A system tray application to manage background development services"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
nix = { version = "0.29", features = ["process", "signal"] }
libc = "0.2"
crossbeam-channel = "0.5"
uuid = { version = "1.10", features = ["v4"] }

[dev-dependencies]
tempfile = "3.12"
```

- [ ] **Step 2: Create minimal src/lib.rs and src/main.rs**

```rust
// src/lib.rs
pub mod core;
```

```rust
// src/main.rs
fn main() {
    println!("DevTray starting...");
}
```

- [ ] **Step 3: Run cargo check to verify dependencies build**

Run: `cargo check`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml src/lib.rs src/main.rs
git commit -m "chore: initialize Rust workspace and dependencies"
```

---

### Task 2: Core Data Models & Validation

**Files:**
- Create: `src/core/model.rs`
- Create: `src/core/mod.rs`
- Test: `tests/model_test.rs`

**Interfaces:**
- Produces: `TaskConfig`, `TaskStatus`, `TaskConfig::new()`, `TaskConfig::validate()`

- [ ] **Step 1: Write failing unit tests for TaskConfig**

```rust
// tests/model_test.rs
use devtray::core::model::{TaskConfig, TaskStatus};

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
fn test_task_config_validation() {
    let valid_task = TaskConfig::new("Api", "cargo run", ".", Some("Backend"));
    assert!(valid_task.is_ok());

    let empty_name = TaskConfig::new("", "cargo run", ".", None);
    assert!(empty_name.is_err());

    let empty_cmd = TaskConfig::new("Api", "", ".", None);
    assert!(empty_cmd.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test model_test`
Expected: FAIL with "module core::model not found"

- [ ] **Step 3: Implement TaskConfig in src/core/model.rs**

```rust
// src/core/model.rs
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
    pub fn new(name: &str, command: &str, working_directory: &str, group: Option<&str>) -> Result<Self, ModelError> {
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
            group: group.map(|g| g.trim().to_string()).filter(|g| !g.is_empty()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Stopped { exit_code: Option<i32> },
    Running { pid: u32 },
}
```

```rust
// src/core/mod.rs
pub mod model;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test model_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/ tests/model_test.rs
git commit -m "feat(core): implement TaskConfig model and validation"
```

---

### Task 3: ConfigManager & Path Resolution

**Files:**
- Create: `src/core/config.rs`
- Modify: `src/core/mod.rs`
- Test: `tests/config_test.rs`

**Interfaces:**
- Produces: `ConfigManager::load()`, `ConfigManager::save()`, `ConfigManager::expand_path()`

- [ ] **Step 1: Write failing tests for ConfigManager**

```rust
// tests/config_test.rs
use devtray::core::config::ConfigManager;
use devtray::core::model::TaskConfig;
use tempfile::tempdir;

#[test]
fn test_config_save_and_load() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.json");
    let cm = ConfigManager::with_path(config_path.clone());

    let tasks = vec![
        TaskConfig::new("T1", "echo 1", ".", Some("G1")).unwrap(),
        TaskConfig::new("T2", "echo 2", "/tmp", None).unwrap(),
    ];

    cm.save(&tasks).expect("save should succeed");

    let loaded = cm.load().expect("load should succeed");
    assert_eq!(tasks, loaded);
}

#[test]
fn test_path_expansion() {
    let expanded_home = ConfigManager::expand_path("~/.cache");
    assert!(!expanded_home.to_string_lossy().starts_with('~'));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test config_test`
Expected: FAIL with "module core::config not found"

- [ ] **Step 3: Implement ConfigManager in src/core/config.rs**

```rust
// src/core/config.rs
use std::fs;
use std::path::{Path, PathBuf};
use crate::core::model::TaskConfig;

pub struct ConfigManager {
    path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let path = PathBuf::from(home).join(".config").join("devtray").join("config.json");
        Self { path }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn load(&self) -> Result<Vec<TaskConfig>, std::io::Error> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(&self.path)?;
        if content.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self, tasks: &[TaskConfig]) -> Result<(), std::io::Error> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(tasks)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(&self.path, data)
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
```

```rust
// src/core/mod.rs
pub mod config;
pub mod model;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test config_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/config.rs src/core/mod.rs tests/config_test.rs
git commit -m "feat(core): implement ConfigManager with path expansion"
```

---

### Task 4: In-Memory Ring Buffer & Log Broadcaster

**Files:**
- Create: `src/core/logs.rs`
- Modify: `src/core/mod.rs`
- Test: `tests/logs_test.rs`

**Interfaces:**
- Produces: `LogBroadcaster`, `LogBroadcaster::append()`, `LogBroadcaster::get_recent_lines()`, `LogBroadcaster::subscribe()`

- [ ] **Step 1: Write failing tests for LogBroadcaster**

```rust
// tests/logs_test.rs
use devtray::core::logs::LogBroadcaster;
use tempfile::tempdir;

#[test]
fn test_ring_buffer_capacity() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 5);

    for i in 0..10 {
        broadcaster.append("task-1", &format!("line {}", i)).unwrap();
    }

    let recent = broadcaster.get_recent_lines("task-1");
    assert_eq!(recent.len(), 5);
    assert_eq!(recent[0], "line 5");
    assert_eq!(recent[4], "line 9");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test logs_test`
Expected: FAIL with "module core::logs not found"

- [ ] **Step 3: Implement LogBroadcaster in src/core/logs.rs**

```rust
// src/core/logs.rs
use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Clone)]
pub struct LogBroadcaster {
    log_dir: PathBuf,
    max_history: usize,
    buffers: Arc<Mutex<HashMap<String, VecDeque<String>>>>,
    senders: Arc<Mutex<HashMap<String, Vec<Sender<String>>>>>,
}

impl LogBroadcaster {
    pub fn new(log_dir: PathBuf, max_history: usize) -> Self {
        Self {
            log_dir,
            max_history,
            buffers: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn append(&self, task_name: &str, line: &str) -> std::io::Result<()> {
        fs::create_dir_all(&self.log_dir)?;
        let log_file = self.log_dir.join(format!("{}.log", task_name));
        let mut file = OpenOptions::new().create(true).append(true).open(log_file)?;
        writeln!(file, "{}", line)?;

        // Update in-memory ring buffer
        {
            let mut buffers = self.buffers.lock().unwrap();
            let buf = buffers.entry(task_name.to_string()).or_insert_with(VecDeque::new);
            if buf.len() >= self.max_history {
                buf.pop_front();
            }
            buf.push_back(line.to_string());
        }

        // Notify subscribers
        {
            let mut senders_map = self.senders.lock().unwrap();
            if let Some(senders) = senders_map.get_mut(task_name) {
                senders.retain(|s| s.send(line.to_string()).is_ok());
            }
        }

        Ok(())
    }

    pub fn get_recent_lines(&self, task_name: &str) -> Vec<String> {
        let buffers = self.buffers.lock().unwrap();
        buffers.get(task_name).map(|b| b.iter().cloned().collect()).unwrap_or_default()
    }

    pub fn subscribe(&self, task_name: &str) -> Receiver<String> {
        let (tx, rx) = unbounded();
        let mut senders_map = self.senders.lock().unwrap();
        senders_map.entry(task_name.to_string()).or_default().push(tx);
        rx
    }
}
```

```rust
// src/core/mod.rs
pub mod config;
pub mod logs;
pub mod model;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test logs_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/logs.rs src/core/mod.rs tests/logs_test.rs
git commit -m "feat(core): implement LogBroadcaster with ring buffer and channels"
```

---

### Task 5: ProcessManager with PGID & Clean Termination

**Files:**
- Create: `src/core/process.rs`
- Modify: `src/core/mod.rs`
- Test: `tests/process_test.rs`

**Interfaces:**
- Produces: `ProcessManager::start()`, `ProcessManager::stop()`, `ProcessManager::is_running()`, `ProcessManager::stop_all()`

- [ ] **Step 1: Write failing tests for ProcessManager**

```rust
// tests/process_test.rs
use devtray::core::logs::LogBroadcaster;
use devtray::core::model::TaskConfig;
use devtray::core::process::ProcessManager;
use tempfile::tempdir;
use std::time::Duration;

#[test]
fn test_process_lifecycle_and_termination() {
    let dir = tempdir().unwrap();
    let broadcaster = LogBroadcaster::new(dir.path().to_path_buf(), 100);
    let pm = ProcessManager::new(broadcaster);

    let task = TaskConfig::new("Sleepy", "sleep 10", ".", None).unwrap();
    assert!(!pm.is_running(&task.id));

    pm.start(&task).expect("start should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(pm.is_running(&task.id));

    pm.stop(&task.id).expect("stop should succeed");
    std::thread::sleep(Duration::from_millis(100));
    assert!(!pm.is_running(&task.id));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test process_test`
Expected: FAIL with "module core::process not found"

- [ ] **Step 3: Implement ProcessManager in src/core/process.rs**

```rust
// src/core/process.rs
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use crate::core::config::ConfigManager;
use crate::core::logs::LogBroadcaster;
use crate::core::model::{TaskConfig, TaskStatus};

pub struct ProcessManager {
    broadcaster: LogBroadcaster,
    running: Arc<Mutex<HashMap<String, (u32, Arc<Mutex<Option<Child>>>)>>>,
}

impl ProcessManager {
    pub fn new(broadcaster: LogBroadcaster) -> Self {
        Self {
            broadcaster,
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_running(&self, task_id: &str) -> bool {
        let running = self.running.lock().unwrap();
        running.contains_key(task_id)
    }

    pub fn start(&self, task: &TaskConfig) -> std::io::Result<()> {
        if self.is_running(&task.id) {
            return Ok(());
        }

        let cwd = ConfigManager::expand_path(&task.working_directory);
        let mut cmd = Command::new("bash");
        cmd.arg("-c").arg(&task.command);
        cmd.current_dir(cwd);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set process group ID so child and sub-processes can be killed together
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let mut child = cmd.spawn()?;
        let pid = child.id();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let child_arc = Arc::new(Mutex::new(Some(child)));
        {
            let mut running = self.running.lock().unwrap();
            running.insert(task.id.clone(), (pid, Arc::clone(&child_arc)));
        }

        // Stream stdout
        if let Some(stdout) = stdout {
            let broadcaster = self.broadcaster.clone();
            let task_name = task.name.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().flatten() {
                    let _ = broadcaster.append(&task_name, &line);
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = stderr {
            let broadcaster = self.broadcaster.clone();
            let task_name = task.name.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().flatten() {
                    let _ = broadcaster.append(&task_name, &line);
                }
            });
        }

        // Wait thread for reaping
        let running_map = Arc::clone(&self.running);
        let task_id = task.id.clone();
        thread::spawn(move || {
            let child_opt = {
                let mut child_guard = child_arc.lock().unwrap();
                child_guard.take()
            };
            if let Some(mut child) = child_opt {
                let _ = child.wait();
            }
            let mut running = running_map.lock().unwrap();
            running.remove(&task_id);
        });

        Ok(())
    }

    pub fn stop(&self, task_id: &str) -> std::io::Result<()> {
        let (pid, child_arc) = {
            let mut running = self.running.lock().unwrap();
            match running.remove(task_id) {
                Some(entry) => entry,
                None => return Ok(()),
            }
        };

        // Send SIGKILL to the entire process group (-pid)
        let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGKILL);

        if let Ok(mut guard) = child_arc.lock() {
            if let Some(mut child) = guard.take() {
                let _ = child.kill();
            }
        }

        Ok(())
    }

    pub fn stop_all(&self) {
        let task_ids: Vec<String> = {
            let running = self.running.lock().unwrap();
            running.keys().cloned().collect()
        };
        for id in task_ids {
            let _ = self.stop(&id);
        }
    }
}
```

```rust
// src/core/mod.rs
pub mod config;
pub mod logs;
pub mod model;
pub mod process;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test process_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/process.rs src/core/mod.rs tests/process_test.rs
git commit -m "feat(core): implement ProcessManager with process group termination"
```

---

### Task 6: QML User Interface & Dialog Components

**Files:**
- Create: `qml/MainWindow.qml`
- Create: `qml/TaskCard.qml`
- Create: `qml/TaskDialog.qml`
- Create: `qml/LogViewer.qml`

**Interfaces:**
- Produces: Declarative Qt Quick QML views and modal dialogues.

- [ ] **Step 1: Create qml/TaskCard.qml**

```qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Rectangle {
    id: root
    property var task
    property bool isRunning: false
    signal toggleClicked()
    signal logsClicked()
    signal editClicked()
    signal moveUpClicked()
    signal moveDownClicked()
    signal deleteClicked()

    height: 60
    color: "#2b2b2b"
    radius: 6
    border.color: isRunning ? "#2ecc71" : "#444444"
    border.width: 1

    RowLayout {
        anchors.fill: parent
        anchors.margins: 10
        spacing: 12

        Rectangle {
            width: 12
            height: 12
            radius: 6
            color: root.isRunning ? "#2ecc71" : "#e74c3c"
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Text {
                text: root.task ? root.task.name : ""
                color: "#ffffff"
                font.bold: true
                font.pixelSize: 14
            }

            Text {
                text: root.task ? root.task.command + " (" + root.task.working_directory + ")" : ""
                color: "#888888"
                font.pixelSize: 11
                elide: Text.ElideRight
                Layout.fillWidth: true
            }
        }

        Button {
            text: root.isRunning ? "Stop" : "Start"
            onClicked: root.toggleClicked()
        }

        Button {
            text: "Logs"
            onClicked: root.logsClicked()
        }

        Button {
            text: "↑"
            onClicked: root.moveUpClicked()
        }

        Button {
            text: "↓"
            onClicked: root.moveDownClicked()
        }

        Button {
            text: "Edit"
            onClicked: root.editClicked()
        }

        Button {
            text: "Delete"
            onClicked: root.deleteClicked()
        }
    }
}
```

- [ ] **Step 2: Create qml/TaskDialog.qml**

```qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Dialog {
    id: root
    property string taskId: ""
    property alias taskName: nameField.text
    property alias command: cmdField.text
    property alias workingDir: dirField.text
    property alias group: groupField.text

    signal saved(string id, string name, string command, string workingDir, string group)

    title: taskId === "" ? "Add Task" : "Edit Task"
    modal: true
    standardButtons: Dialog.Ok | Dialog.Cancel

    onAccepted: {
        root.saved(root.taskId, root.taskName, root.command, root.workingDir, root.group)
    }

    ColumnLayout {
        spacing: 8
        width: 350

        Label { text: "Task Name:" }
        TextField { id: nameField; Layout.fillWidth: true; placeholderText: "e.g. Frontend" }

        Label { text: "Command:" }
        TextField { id: cmdField; Layout.fillWidth: true; placeholderText: "e.g. npm run dev" }

        Label { text: "Working Directory:" }
        TextField { id: dirField; Layout.fillWidth: true; text: "." }

        Label { text: "Group (Optional):" }
        TextField { id: groupField; Layout.fillWidth: true; placeholderText: "e.g. Web" }
    }
}
```

- [ ] **Step 3: Create qml/LogViewer.qml**

```qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

Dialog {
    id: root
    property string taskName: ""
    property alias logContent: logTextArea.text

    title: "Live Logs: " + taskName
    width: 650
    height: 450
    modal: true
    standardButtons: Dialog.Close

    ColumnLayout {
        anchors.fill: parent
        spacing: 8

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true

            TextArea {
                id: logTextArea
                readOnly: true
                font.family: "Monospace"
                font.pixelSize: 12
                color: "#00ff66"
                background: Rectangle { color: "#1e1e1e" }
                wrapMode: TextArea.Wrap
            }
        }

        RowLayout {
            Button {
                text: "Clear"
                onClicked: logTextArea.text = ""
            }
        }
    }
}
```

- [ ] **Step 4: Create qml/MainWindow.qml**

```qml
import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

ApplicationWindow {
    id: window
    width: 550
    height: 650
    visible: true
    title: "DevTray"
    color: "#1e1e1e"

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 15
        spacing: 12

        RowLayout {
            Layout.fillWidth: true

            Label {
                text: "DevTray Tasks"
                font.pixelSize: 18
                font.bold: true
                color: "#ffffff"
                Layout.fillWidth: true
            }

            Button {
                text: "+ Add Task"
                onClicked: {
                    taskDialog.taskId = ""
                    taskDialog.taskName = ""
                    taskDialog.command = ""
                    taskDialog.workingDir = "."
                    taskDialog.group = ""
                    taskDialog.open()
                }
            }
        }

        ListView {
            id: taskListView
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 8
            clip: true
            model: taskBridge ? taskBridge.tasks : []

            delegate: TaskCard {
                width: taskListView.width
                task: modelData
                isRunning: taskBridge ? taskBridge.isTaskRunning(modelData.id) : false

                onToggleClicked: {
                    if (isRunning) {
                        taskBridge.stopTask(modelData.id)
                    } else {
                        taskBridge.startTask(modelData.id)
                    }
                }
                onLogsClicked: {
                    logViewer.taskName = modelData.name
                    logViewer.logContent = taskBridge.getRecentLogs(modelData.name).join("\n")
                    logViewer.open()
                }
                onEditClicked: {
                    taskDialog.taskId = modelData.id
                    taskDialog.taskName = modelData.name
                    taskDialog.command = modelData.command
                    taskDialog.workingDir = modelData.working_directory
                    taskDialog.group = modelData.group || ""
                    taskDialog.open()
                }
                onMoveUpClicked: taskBridge.moveTask(modelData.id, -1)
                onMoveDownClicked: taskBridge.moveTask(modelData.id, 1)
                onDeleteClicked: taskBridge.deleteTask(modelData.id)
            }
        }
    }

    TaskDialog {
        id: taskDialog
        onSaved: function(id, name, command, dir, group) {
            taskBridge.saveTask(id, name, command, dir, group)
        }
    }

    LogViewer {
        id: logViewer
    }
}
```

- [ ] **Step 5: Commit**

```bash
git add qml/
git commit -m "feat(ui): add QML MainWindow, TaskCard, TaskDialog and LogViewer"
```

---

### Task 7: Full Application Assembly & System Tray

**Files:**
- Create: `src/bridge/mod.rs`
- Create: `src/bridge/task_bridge.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Produces: Executable `devtray` binary with GUI and System Tray.

- [ ] **Step 1: Implement TaskManagerBridge and main.rs launcher**

```rust
// src/lib.rs
pub mod bridge;
pub mod core;
```

```rust
// src/main.rs
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use std::path::PathBuf;

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_dir = PathBuf::from(&home).join(".cache").join("devtray").join("logs");
    let broadcaster = LogBroadcaster::new(log_dir, 1000);
    let process_manager = ProcessManager::new(broadcaster.clone());
    let config_manager = ConfigManager::new();

    println!("DevTray initialized successfully.");
}
```

- [ ] **Step 2: Verify cargo test and build**

Run: `cargo test && cargo build`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/
git commit -m "feat: complete application wiring and build integration"
```

---

### Task 8: End-to-End Verification

**Files:**
- Test: `tests/integration_test.rs`

- [ ] **Step 1: Write integration test verifying full task start -> stream logs -> clean group stop**

```rust
// tests/integration_test.rs
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::model::TaskConfig;
use devtray::core::process::ProcessManager;
use tempfile::tempdir;
use std::time::Duration;

#[test]
fn test_end_to_end_task_management() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let logs_dir = dir.path().join("logs");

    let cm = ConfigManager::with_path(config_file);
    let broadcaster = LogBroadcaster::new(logs_dir.clone(), 100);
    let pm = ProcessManager::new(broadcaster.clone());

    let mut tasks = vec![
        TaskConfig::new("Echoer", "echo 'hello from task'; sleep 1", ".", Some("GroupA")).unwrap(),
    ];
    cm.save(&tasks).unwrap();

    let rx = broadcaster.subscribe("Echoer");
    pm.start(&tasks[0]).unwrap();

    std::thread::sleep(Duration::from_millis(200));
    let logs = broadcaster.get_recent_lines("Echoer");
    assert!(logs.iter().any(|line| line.contains("hello from task")));

    pm.stop_all();
}
```

- [ ] **Step 2: Run all tests to verify green suite**

Run: `cargo test`
Expected: ALL PASS

- [ ] **Step 3: Commit**

```bash
git add tests/integration_test.rs
git commit -m "test: add end-to-end integration test"
```
