# DevTray: Rust + Qt Migration Design Specification

- **Date:** 2026-08-21
- **Status:** Approved
- **Target Stack:** Rust 1.96+, Qt 6.11+ (Qt Quick / QML + QSystemTrayIcon), Qt Bridge for Rust

---

## 1. Problem & Goals

DevTray is currently written in Go using `gotk3` and `libayatana-appindicator`.
The goal of this migration is to:
1. Modernize the codebase to **Rust + Qt6**.
2. Decouple from legacy `libayatana` GTK C bindings while leveraging Qt's native cross-desktop capabilities (`QSystemTrayIcon` via StatusNotifierItem/DBus).
3. Provide a robust, thread-safe process manager with process group termination (`setpgid`, `SIGKILL` to negative PGID) to ensure zero orphan background processes.
4. Enhance the user experience by adding an **in-app live log viewer** with real-time streaming alongside file-based logging.

---

## 2. System Architecture

The project is structured into two main layers:

```
+-----------------------------------------------------------+
|                     QML Frontend                          |
|  - MainWindow.qml (Task cards, Group controls, Dialogs)   |
|  - LogViewer.qml  (Live stream, clear, search)            |
|  - System Tray    (QSystemTrayIcon, context submenus)     |
+-----------------------------+-----------------------------+
                              | (Qt Bridge / Signals & Slots)
+-----------------------------v-----------------------------+
|                Rust Backend (devtray-core)                |
|  - TaskManagerBridge (QObject / QML Interface)            |
|  - ProcessManager    (Child process lifecycle & PGIDs)    |
|  - LogBroadcaster    (File writer + Ring buffer channels) |
|  - ConfigManager     (~/.config/devtray/config.json)      |
+-----------------------------------------------------------+
```

---

## 3. Data Models & Configuration

### Config Storage
- Path: `~/.config/devtray/config.json`
- Log Directory: `~/.cache/devtray/logs/`

### Task Model
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    pub working_directory: String,
    #[serde(default)]
    pub group: Option<String>,
}
```

### Runtime Task State
```rust
#[derive(Debug, Clone)]
pub enum TaskStatus {
    Stopped { exit_code: Option<i32> },
    Running { pid: u32, started_at: std::time::Instant },
}
```

---

## 4. Process Manager & Log Streaming

1. **Process Execution**:
   - Spawns `bash -c "<command>"` in the resolved working directory (with `~` and env variable expansion).
   - In `pre_exec`, calls `libc::setpgid(0, 0)` so the child process and all its descendants belong to a distinct process group.
2. **Termination**:
   - To stop a process, sends `SIGKILL` to `-pgid` (the negative PID of the spawned process), ensuring all child workers, dev servers (e.g. Node/Vite/Docker/Postgres), and spawned subshells are killed immediately.
3. **Log Handling**:
   - Standard output and standard error are piped and captured asynchronously.
   - Each line is:
     1. Written to `~/.cache/devtray/logs/{task_name}.log`.
     2. Appended to an in-memory ring buffer (default 1,000 lines) for instant retrieval upon opening the UI.
     3. Emitted over Qt signals to active QML live log viewers.
4. **Process Reaping**:
   - A background thread / worker waits on process exit, updates internal state to `TaskStatus::Stopped`, and signals the UI and tray menu.

---

## 5. Qt UI & System Tray Specification

### Main Window (`MainWindow.qml`)
- **Header**: Title, running/stopped summary badge, and "+ Add Task" button.
- **Grouped Task Cards**:
  - Organized by Group headers with "▶️ Start All" and "🛑 Stop All" action buttons.
  - Task Card:
    - Status Indicator (🟢 Active / ⚪ Idle).
    - Task Name & Working Directory label.
    - Start/Stop toggle button.
    - Live Log button (opens log drawer).
    - Reorder (Move Up / Down) within group.
    - Edit and Delete dialog triggers.

### Live Log Viewer (`LogViewer.qml`)
- Drawer or dedicated window with dark monospace terminal styling.
- Auto-scroll toggle, "Clear View", "Open in Text Editor", and "Copy All" actions.

### System Tray (`QSystemTrayIcon`)
- Tooltip reflecting active tasks count.
- Left-click toggles / presents Main Window.
- Context Menu:
  - Open Main Window
  - Separator
  - Dynamic Group Submenus:
    - `▶️ Start All`
    - `🛑 Stop All`
    - Separator
    - Checkable Task Items (checked = running)
  - Uncategorized Tasks list
  - Separator
  - `Quit DevTray` (safely stops all running processes before exit)

---

## 6. Rust-QML Bridge Contract

```rust
// Exposes task management methods to QML
pub struct TaskManagerBridge {
    // Signals
    // tasks_updated()
    // task_status_changed(id: String, is_running: bool, exit_code: i32)
    // log_line_received(task_id: String, line: String)
}

impl TaskManagerBridge {
    pub fn start_task(&self, id: &str);
    pub fn stop_task(&self, id: &str);
    pub fn start_group(&self, group: &str);
    pub fn stop_group(&self, group: &str);
    pub fn save_task(&self, id: &str, name: &str, command: &str, dir: &str, group: &str);
    pub fn delete_task(&self, id: &str);
    pub fn move_task(&self, id: &str, direction: i32);
    pub fn get_recent_logs(&self, id: &str) -> Vec<String>;
    pub fn quit_app(&self);
}
```

---

## 7. Verification & Testing Strategy

1. **Unit Tests (Core)**:
   - Config loading, saving, corruption recovery.
   - Path expansion (`~`, relative paths, env vars).
   - Task reordering within and across groups.
2. **Process Management Tests**:
   - Process group spawning and verification of clean tree termination (`SIGKILL -pgid`).
   - Log writing and ring-buffer boundary limits.
3. **Integration & Manual UI Tests**:
   - Launching tasks and verifying live log streaming in QML.
   - System tray menu interaction, group bulk actions, and graceful quit.
