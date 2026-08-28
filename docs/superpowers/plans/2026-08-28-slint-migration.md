# Slint + Pure Rust Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate DevTray's UI from Qt6/QML to Slint and `ksni` in pure Rust, reducing idle memory footprint from ~177 MB to ~25 MB RSS (< 5 MB private RAM) while keeping 100% feature parity.

**Architecture:** The core backend (`ProcessManager`, `ConfigManager`, `LogBroadcaster`, `TaskConfig`) in `src/core/` is retained intact. A new `src/gui/` module bridges core state to declarative Slint UI components in `ui/*.slint`, and coordinates a native Linux system tray using `ksni` (D-Bus StatusNotifierItem).

**Tech Stack:** Rust 1.96+, `slint` 1.17.1, `slint-build` 1.17.1, `ksni` 0.3.6, `image` 0.25.10, `nix` 0.29, `crossbeam-channel` 0.5.

## Global Constraints

- Must retain all existing functionality in `src/core/` and pass all core unit tests.
- UI must support: Task Cards with active glow badges, Move Up/Down reordering, Live Monospace Log Viewer with ANSI stripping and clear view, Add/Edit modal dialogs with validation, Delete & Quit confirmation modals.
- System Tray must support: embedded icon, dynamic tooltip `DevTray (X active)`, Left-Click toggle window, Right-Click menu with group submenus (Start/Stop All, checkable task status items), and graceful Quit.
- Default renderer must use software rendering (`SLINT_BACKEND=winit-software`) unless overridden by the user's environment.
- Commits must be frequent and self-contained.

---

### Task 1: Update `Cargo.toml`, `build.rs`, and remove Qt C++ Bridge

**Files:**
- Modify: `Cargo.toml`
- Modify: `build.rs`
- Modify: `src/lib.rs`
- Delete: `src/bridge/` (replaced by new pure Rust `src/gui/`)
- Delete: `qml/` (replaced by new `ui/`)
- Test: `Cargo.toml`

**Interfaces:**
- Produces: Clean pure Rust build pipeline compiling Slint `.slint` templates via `slint_build::compile`.

- [ ] **Step 1: Update Cargo.toml dependencies**

Update `Cargo.toml`:
```toml
[package]
name = "devtray"
version = "2.1.0"
edition = "2021"
authors = ["DevTray Contributors"]
description = "A lightweight system tray application to manage background development services"

[dependencies]
slint = { version = "1.17.1", default-features = false, features = ["std", "compat-1-2", "backend-winit", "renderer-software", "renderer-femtovg", "accessibility"] }
ksni = "0.3.6"
image = { version = "0.25.10", default-features = false, features = ["png"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
nix = { version = "0.29", features = ["process", "signal"] }
libc = "0.2"
crossbeam-channel = "0.5"
uuid = { version = "1.10", features = ["v4"] }
thiserror = "2.0"

[build-dependencies]
slint-build = "1.17.1"

[dev-dependencies]
tempfile = "3.12"
```

- [ ] **Step 2: Update build.rs**

Update `build.rs`:
```rust
fn main() {
    slint_build::compile("ui/main_window.slint").expect("Failed to compile Slint UI");
}
```

- [ ] **Step 3: Update src/lib.rs and create minimal placeholder ui/main_window.slint**

Create `ui/main_window.slint`:
```slint
export component MainWindow inherits Window {
    title: "DevTray";
    width: 400px;
    height: 500px;
}
```

Update `src/lib.rs`:
```rust
pub mod core;
pub mod gui;

slint::include_modules!();
```

Create placeholder `src/gui/mod.rs`:
```rust
pub mod bridge;
pub mod tray;
```

- [ ] **Step 4: Verify initial compilation**

Run: `cargo check`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git rm -rf qml/ src/bridge/
git add Cargo.toml build.rs src/lib.rs src/gui/ ui/main_window.slint
git commit -m "refactor: replace Qt/cxx-qt build with Slint build configuration"
```

---

### Task 2: Implement Complete Slint Declarative UI Components

**Files:**
- Create: `ui/theme.slint`
- Create: `ui/task_card.slint`
- Create: `ui/log_viewer.slint`
- Create: `ui/task_dialog.slint`
- Create: `ui/confirm_dialog.slint`
- Modify: `ui/main_window.slint`

**Interfaces:**
- Produces:
  - `MainWindow` component with full UI: Task list grouped by section, status glow, Move Up/Down buttons, log viewer modal, task add/edit dialog, confirmation dialogs, and callbacks for Rust integration.

- [ ] **Step 1: Create `ui/theme.slint` with color palette and widget styles**

```slint
export global Theme {
    out property <color> background: #1e1e1e;
    out property <color> card-bg: #242424;
    out property <color> card-hover: #2a2a2a;
    out property <color> border: #333333;
    out property <color> text-primary: #ffffff;
    out property <color> text-secondary: #dcdcdc;
    out property <color> text-muted: #888888;
    out property <color> accent: #3584e4;
    out property <color> running: #2ecc71;
    out property <color> stopped: #e74c3c;
    out property <color> terminal-bg: #141414;
    out property <color> terminal-text: #33d17a;
}
```

- [ ] **Step 2: Create `ui/task_card.slint` with Move Up/Down, Status Glow, and Actions**

```slint
import { Theme } from "theme.slint";
import { Button, HorizontalBox, VerticalBox } from "std-widgets.slint";

export struct TaskItem {
    id: string,
    name: string,
    command: string,
    working_directory: string,
    group: string,
    is_running: bool,
    can_move_up: bool,
    can_move_down: bool,
}

export component TaskCard inherits Rectangle {
    in property <TaskItem> task;
    callback toggle_task();
    callback show_logs();
    callback edit_task();
    callback delete_task();
    callback move_up();
    callback move_down();

    height: 44px;
    background: touchArea.has-hover ? Theme.card-hover : Theme.card-bg;
    border-color: Theme.border;
    border-width: 1px;
    border-radius: 6px;

    touchArea := TouchArea {}

    HorizontalLayout {
        padding-left: 10px;
        padding-right: 10px;
        spacing: 8px;
        alignment: center;

        // Move Up / Down Buttons
        VerticalLayout {
            alignment: center;
            spacing: 2px;
            width: 16px;

            Rectangle {
                width: 16px;
                height: 14px;
                background: btnUp.has-hover ? #3a3a3a : transparent;
                border-radius: 2px;
                btnUp := TouchArea {
                    clicked => { root.move_up(); }
                }
                Text {
                    text: "▲";
                    font-size: 8px;
                    color: root.task.can_move_up ? (btnUp.has-hover ? #ffffff : #888888) : #444444;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            Rectangle {
                width: 16px;
                height: 14px;
                background: btnDown.has-hover ? #3a3a3a : transparent;
                border-radius: 2px;
                btnDown := TouchArea {
                    clicked => { root.move_down(); }
                }
                Text {
                    text: "▼";
                    font-size: 8px;
                    color: root.task.can_move_down ? (btnDown.has-hover ? #ffffff : #888888) : #444444;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
        }

        // Status Indicator Dot with subtle glow
        Rectangle {
            width: 12px;
            height: 12px;
            border-radius: 6px;
            background: root.task.is_running ? Theme.running : Theme.stopped;

            if root.task.is_running : Rectangle {
                width: 16px;
                height: 16px;
                x: -2px;
                y: -2px;
                border-radius: 8px;
                background: Theme.running;
                opacity: 0.35;
            }
        }

        // Task Name & Subtitle Info
        VerticalLayout {
            alignment: center;
            spacing: 2px;
            horizontal-stretch: 1;

            Text {
                text: root.task.name;
                font-size: 13px;
                font-weight: 700;
                color: Theme.text-primary;
                overflow: elide;
            }

            Text {
                text: root.task.working_directory != "" ? root.task.working_directory : root.task.command;
                font-size: 10px;
                color: Theme.text-muted;
                overflow: elide;
            }
        }

        // Action Buttons
        HorizontalLayout {
            spacing: 4px;
            alignment: center;

            // Start/Stop Toggle
            Rectangle {
                width: 28px;
                height: 26px;
                background: btnToggle.has-hover ? #3c3c3c : #2c2c2c;
                border-color: #424242;
                border-width: 1px;
                border-radius: 4px;
                btnToggle := TouchArea {
                    clicked => { root.toggle_task(); }
                }
                Text {
                    text: root.task.is_running ? "⏹" : "▶";
                    font-size: 11px;
                    color: root.task.is_running ? #ff5555 : Theme.running;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            // Logs
            Rectangle {
                width: 38px;
                height: 26px;
                background: btnLogs.has-hover ? #3c3c3c : #2c2c2c;
                border-color: #424242;
                border-width: 1px;
                border-radius: 4px;
                btnLogs := TouchArea {
                    clicked => { root.show_logs(); }
                }
                Text {
                    text: "Logs";
                    font-size: 11px;
                    color: Theme.text-secondary;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            // Edit
            Rectangle {
                width: 36px;
                height: 26px;
                background: btnEdit.has-hover ? #3c3c3c : #2c2c2c;
                border-color: #424242;
                border-width: 1px;
                border-radius: 4px;
                btnEdit := TouchArea {
                    clicked => { root.edit_task(); }
                }
                Text {
                    text: "Edit";
                    font-size: 11px;
                    color: Theme.text-secondary;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }

            // Delete
            Rectangle {
                width: 44px;
                height: 26px;
                background: btnDelete.has-hover ? #4d2222 : #2c2c2c;
                border-color: btnDelete.has-hover ? #884444 : #424242;
                border-width: 1px;
                border-radius: 4px;
                btnDelete := TouchArea {
                    clicked => { root.delete_task(); }
                }
                Text {
                    text: "Delete";
                    font-size: 11px;
                    color: btnDelete.has-hover ? #ff7777 : Theme.text-secondary;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
        }
    }
}
```

- [ ] **Step 3: Create `ui/log_viewer.slint`, `ui/task_dialog.slint`, and `ui/confirm_dialog.slint`**

Create `ui/log_viewer.slint`:
- Overlay with dark terminal background, live text area, auto-scroll, clear logs button, and close button.

Create `ui/task_dialog.slint`:
- Modal popup with fields for Name, Command, Working Directory, and Group with save/cancel callbacks.

Create `ui/confirm_dialog.slint`:
- Modal confirmation dialog with custom title, message, sub-message, destructive styling for delete/quit.

- [ ] **Step 4: Assemble `ui/main_window.slint`**

Integrate all components, group headers, empty state, and callbacks into `ui/main_window.slint`.

- [ ] **Step 5: Verify Slint UI compilation**

Run: `cargo check`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add ui/
git commit -m "feat(ui): implement Slint declarative UI components and theme"
```

---

### Task 3: Implement Slint Bridge & App Controller (`src/gui/bridge.rs`)

**Files:**
- Create: `src/gui/bridge.rs`
- Modify: `src/gui/mod.rs`
- Create: `tests/slint_bridge_test.rs`

**Interfaces:**
- Consumes: `src/core/config.rs`, `src/core/process.rs`, `src/core/logs.rs`, `src/core/model.rs`
- Produces: `SlintAppController` struct:
  - `pub fn new(config: ConfigManager, process: ProcessManager, logs: LogBroadcaster) -> Self`
  - `pub fn bind_to_ui(&self, ui: &MainWindow)`
  - `pub fn refresh_tasks(&self, ui: &MainWindow)`
  - `pub fn move_task(&self, id: &str, direction: i32) -> bool`

- [ ] **Step 1: Write failing test in `tests/slint_bridge_test.rs`**

```rust
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::model::TaskConfig;
use devtray::core::process::ProcessManager;
use devtray::gui::bridge::SlintAppController;
use tempfile::tempdir;

#[test]
fn test_bridge_task_conversion_and_ordering() {
    let dir = tempdir().unwrap();
    let config_file = dir.path().join("config.json");
    let log_dir = dir.path().join("logs");

    let config = ConfigManager::with_path(config_file);
    let logs = LogBroadcaster::new(log_dir, 100);
    let process = ProcessManager::new(logs.clone());

    let controller = SlintAppController::new(config, process, logs);
    controller.add_task("Backend", "echo 1", ".", Some("Web")).unwrap();
    controller.add_task("Frontend", "echo 2", ".", Some("Web")).unwrap();
    controller.add_task("DB", "echo 3", ".", Some("Database")).unwrap();

    let items = controller.get_slint_task_items();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].group, "Database");
    assert_eq!(items[1].group, "Web");
    assert_eq!(items[2].group, "Web");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test slint_bridge_test`
Expected: FAIL with "cannot find SlintAppController"

- [ ] **Step 3: Implement `src/gui/bridge.rs`**

Implement `SlintAppController` handling:
- `get_slint_task_items(&self) -> Vec<TaskItem>`
- Task actions: `start_task`, `stop_task`, `save_task`, `delete_task`, `move_task`, `start_group`, `stop_group`, `start_all`, `stop_all`
- Live log streaming subscription via `LogBroadcaster::subscribe`
- Reordering with bounds checking.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test slint_bridge_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/gui/ tests/slint_bridge_test.rs
git commit -m "feat(gui): implement Slint bridge and controller with unit tests"
```

---

### Task 4: Implement System Tray Integration with `ksni` (`src/gui/tray.rs`)

**Files:**
- Create: `src/gui/tray.rs`
- Modify: `src/gui/mod.rs`
- Test: `tests/tray_test.rs`

**Interfaces:**
- Consumes: `SlintAppController`, `assets/icon.png`, `ksni`
- Produces: `DevTraySysTray` struct implementing `ksni::Tray`:
  - `pub fn spawn(controller: Arc<SlintAppController>, ui_handle: slint::Weak<MainWindow>) -> ksni::Handle<DevTraySysTray>`

- [ ] **Step 1: Write tray menu and tooltip unit test in `tests/tray_test.rs`**

```rust
use devtray::gui::tray::format_tray_tooltip;

#[test]
fn test_tray_tooltip_formatting() {
    assert_eq!(format_tray_tooltip(0), "DevTray");
    assert_eq!(format_tray_tooltip(1), "DevTray (1 active)");
    assert_eq!(format_tray_tooltip(4), "DevTray (4 active)");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tray_test`
Expected: FAIL with "cannot find function `format_tray_tooltip`"

- [ ] **Step 3: Implement `src/gui/tray.rs`**

- Implement `format_tray_tooltip` and `DevTraySysTray` struct.
- Implement `ksni::Tray` methods:
  - `id`: `"devtray"`
  - `icon_pixmap`: Converts `assets/icon.png` via `image::load_from_memory` to `ksni::Icon`
  - `tooltip`: Dynamic tooltip string
  - `activate`: Invokes `ui_handle.upgrade().unwrap().show()`
  - `menu`: Dynamic group submenus with `Start All`, `Stop All`, and checkable task items.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test tray_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/gui/tray.rs tests/tray_test.rs
git commit -m "feat(tray): implement ksni StatusNotifierItem system tray integration"
```

---

### Task 5: Implement `src/main.rs` & Application Lifecycle

**Files:**
- Modify: `src/main.rs`

**Interfaces:**
- Consumes: `src/gui/bridge.rs`, `src/gui/tray.rs`, `MainWindow`
- Produces: Full application entry point with software rendering defaults, minimize-to-tray handling, and graceful process cleanup on exit.

- [ ] **Step 1: Implement `src/main.rs`**

```rust
use std::sync::Arc;
use devtray::core::config::ConfigManager;
use devtray::core::logs::LogBroadcaster;
use devtray::core::process::ProcessManager;
use devtray::gui::bridge::SlintAppController;
use devtray::gui::tray::DevTraySysTray;
use devtray::MainWindow;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default to software renderer for ultra-low memory usage (~25MB RSS)
    if std::env::var("SLINT_BACKEND").is_err() {
        std::env::set_var("SLINT_BACKEND", "winit-software");
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let log_dir = std::path::PathBuf::from(&home).join(".cache").join("devtray").join("logs");
    let broadcaster = LogBroadcaster::new(log_dir, 1000);
    let process_mgr = ProcessManager::new(broadcaster.clone());
    let config_mgr = ConfigManager::new();

    let controller = Arc::new(SlintAppController::new(config_mgr, process_mgr, broadcaster));
    let main_window = MainWindow::new()?;

    controller.bind_to_ui(&main_window);
    controller.refresh_tasks(&main_window);

    // Spawn ksni system tray daemon in background
    let _tray_handle = DevTraySysTray::spawn(controller.clone(), main_window.as_weak());

    main_window.run()?;

    // Gracefully stop all child processes on quit
    controller.stop_all();

    Ok(())
}
```

- [ ] **Step 2: Verify binary compilation**

Run: `cargo build --release`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire main application entry point with low-RAM default and tray"
```

---

### Task 6: Full Verification, Integration Testing & Live Memory Profiling

**Files:**
- Modify: `tests/integration_test.rs`
- Delete old obsolete integration tests if referencing Qt

- [ ] **Step 1: Run complete test suite**

Run: `cargo test`
Expected: All unit and integration tests pass (100% PASS).

- [ ] **Step 2: Build release binary**

Run: `cargo build --release`
Expected: Binary `/home/azizz/project/devtray/target/release/devtray` builds with 0 errors.

- [ ] **Step 3: Run live memory profiling on the release binary**

Run the release binary, capture PID, and inspect `/proc/$PID/status` and `/proc/$PID/smaps_rollup`:
- Verify **RSS ≤ 30 MB** (target ~25 MB).
- Verify **Private dirty RAM (USS) ≤ 5 MB** (target ~3 MB).
- Verify active threads ≤ 8 (target ~6).

- [ ] **Step 4: Final commit and clean up**

```bash
git add .
git commit -m "chore: complete Slint migration and verify memory performance"
```
