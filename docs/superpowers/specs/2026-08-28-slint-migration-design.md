# DevTray: Slint + Pure Rust Migration Design Specification

- **Date:** 2026-08-28
- **Status:** Approved
- **Target Stack:** Rust 1.96+, Slint 1.17+, `ksni` 0.3.6 (Freedesktop/KDE StatusNotifierItem D-Bus)

---

## 1. Problem & Goals

DevTray 2.0 was migrated to Qt 6.11 using Qt Quick (QML). While functional, the QML runtime brings significant memory overhead on Linux:
- **~177 MB Resident Set Size (RSS)** due to Mesa GPU drivers, LLVM shader JIT compilers (`libLLVM.so` & `libgallium.so`), and the V4 JavaScript engine.
- **~46 MB Private Dirty Memory (USS)**.

The goals of this migration are:
1. **Ultra-Low Memory Footprint:** Reduce total RSS from ~177 MB to **~25 MB**, and private dirty RAM to **< 5 MB** using Slint's native 2D software renderer (`tiny-skia` / `softbuffer`).
2. **Pure Rust Stack:** Remove all C++ bindings and Qt dependencies (`cxx`, `cxx-qt`, `cxx-qt-build`, `libQt6*`), achieving fast builds and zero C++ runtime dependencies.
3. **Full Feature Parity:**
   - Grouped task cards with active status glow badges.
   - Task reordering with Move Up (`▲`) and Move Down (`▼`) buttons.
   - Dark monospace live log viewer with real-time streaming and ANSI stripping.
   - Modal dialogs for Add/Edit tasks with validation, and Delete/Quit confirmations.
   - Feature-rich Linux system tray (`ksni`) with dynamic group submenus, checkable task status items, active task count badge in tooltip, and minimize-to-tray lifecycle.
4. **Preserve Robust Backend:** Retain 100% of `src/core/` (`ProcessManager` with PGID termination, `ConfigManager`, `LogBroadcaster`).

---

## 2. System Architecture

```
+-------------------------------------------------------------+
|                      Slint UI (ui/*.slint)                  |
|  - MainWindow (Grouped Task Cards, Add Task, Status Badges) |
|  - TaskCard (Status dot, Move Up/Down, Logs, Edit, Delete)  |
|  - LogViewer (Real-time live log stream, Clear, Auto-scroll)|
|  - TaskDialog (Add / Edit Task Modal Form)                  |
|  - ConfirmDialog (Delete / Quit Confirmation Modals)        |
+------------------------------+------------------------------+
                               | (Slint Callbacks & slint::ModelRc)
+------------------------------v------------------------------+
|                Slint App Controller / Bridge                |
|  - Maps Rust TaskConfig structs to Slint VecModel           |
|  - Handles UI actions (start/stop/reorder/save/delete)      |
|  - Streams LogBroadcaster lines into Slint LogViewer        |
+------------------------------+------------------------------+
                               |
+------------------------------+------------------------------+
|             Linux Tray Daemon (`ksni` SNI D-Bus)             |
|  - Tooltip: "DevTray (X active)"                            |
|  - Dynamic Group Submenus (Start/Stop All, Task Items)      |
|  - Left-Click to toggle window, Right-Click menu            |
+------------------------------+------------------------------+
                               |
+------------------------------v------------------------------+
|             devtray-core (100% Reused Pure Rust)            |
|  - ProcessManager (Child lifecycle, PGID SIGKILL tree kill) |
|  - ConfigManager  (~/.config/devtray/config.json)           |
|  - LogBroadcaster (File persistence + Ring buffer channels) |
+-------------------------------------------------------------+
```

---

## 3. Dependencies & Build Configuration

### `Cargo.toml`
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

### `build.rs`
```rust
fn main() {
    slint_build::compile("ui/main_window.slint").expect("Failed to compile Slint UI");
}
```

---

## 4. UI Design & Components (`ui/`)

### 4.1 Theme & Styling
- **Color Palette (Dark Modern / Adwaita Dark)**:
  - Background: `#1e1e1e` (Window), `#242424` (Cards/Modals), `#141414` (Terminal)
  - Borders: `#333333` / `#3d3d3d`
  - Text: `#ffffff` (Primary), `#dcdcdc` (Secondary), `#888888` (Muted)
  - Accents: `#3584e4` (Blue Primary), `#2ecc71` / `#33d17a` (Running / Success), `#e74c3c` (Stopped / Danger)
  - Fonts: System Sans for UI, `DejaVu Sans Mono, Monospace` for terminal logs.

### 4.2 Component Breakdown
1. **`MainWindow.slint`**:
   - Header with title, active count summary badge, and `+ Add Task` button.
   - Vertical list of tasks grouped by group name with section headers.
   - Empty state placeholder when no tasks exist.
   - Bottom status toolbar.
2. **`TaskCard.slint`**:
   - Status indicator dot (`🟢 Active` / `⚪ Idle`) with glow effect when running.
   - Task Name (bold) and command/directory subtitle.
   - Move Up (`▲`) and Move Down (`▼`) buttons to reorder within the same group.
   - Start/Stop toggle button (`▶` / `⏹`).
   - Action buttons: `Logs`, `Edit`, `Delete`.
3. **`LogViewer.slint`**:
   - Monospace terminal log box with auto-scroll to bottom.
   - `Clear View` and `Close` buttons.
4. **`TaskDialog.slint`**:
   - Modal form for creating/editing tasks (`Name`, `Command`, `Working Directory`, `Group`).
   - Field validation and error messages.
5. **`ConfirmDialog.slint`**:
   - Reusable modal dialog for Delete and Quit confirmations.

---

## 5. Slint Data Model & Bridge Contract

### Slint Data Model (`TaskItem`)
```slint
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
```

### Rust Slint App Bridge (`src/gui/mod.rs` & `src/gui/bridge.rs`)
- Wraps `ConfigManager`, `ProcessManager`, and `LogBroadcaster`.
- Initializes `slint::VecModel<TaskItem>` and sets it on the `MainWindow` instance.
- Exposes Rust callbacks:
  - `on_toggle_task(id: SharedString)`
  - `on_move_task(id: SharedString, direction: int)`
  - `on_save_task(id: SharedString, name: SharedString, cmd: SharedString, dir: SharedString, group: SharedString)`
  - `on_delete_task(id: SharedString)`
  - `on_start_group(group: SharedString)`
  - `on_stop_group(group: SharedString)`
  - `on_start_all()`
  - `on_stop_all()`
  - `on_show_logs(task_name: SharedString)`
  - `on_quit_app()`

---

## 6. System Tray Implementation (`src/gui/tray.rs`)

Implements `ksni::Tray`:
- **Icon:** Loads `assets/icon.png` as raw ARGB pixmap.
- **Tooltip:** `DevTray (X active)` when tasks are running, or `DevTray` when idle.
- **Left-Click Activation:** Calls `MainWindow::show()` and `MainWindow::window().request_focus()`.
- **Context Menu:**
  - `Open DevTray`
  - Separator
  - Group submenus (`▶ Start All`, `🛑 Stop All`, Separator, Checkable task items)
  - Uncategorized tasks list
  - Separator
  - `Quit DevTray` (stops all processes, exits event loop)

---

## 7. Memory & Process Lifecycle Guarantees

1. **Default Software Renderer:**
   - In `main.rs`, set `std::env::var("SLINT_BACKEND").is_err() -> set_var("SLINT_BACKEND", "winit-software")` before window initialization.
   - Guarantees **~25 MB RSS** and **< 5 MB private RAM**.
2. **Window Hide on Close:**
   - Intercept window close event to hide window rather than exit process.
3. **Safe Shutdown:**
   - On quit, invoke `ProcessManager::stop_all()` before exiting to ensure zero orphan background processes.

---

## 8. Verification Strategy

1. **Unit & Integration Tests:**
   - `cargo test`: Verify all core tests (model, config, logs ring buffer, process groups).
   - Bridge unit tests: Verify task sorting, group categorization, and reordering.
2. **Live Benchmarks:**
   - Measure memory usage via `ps`, `top`, and `/proc/<pid>/smaps_rollup`.
   - Verify RSS ≤ 30 MB and private memory ≤ 5 MB.
