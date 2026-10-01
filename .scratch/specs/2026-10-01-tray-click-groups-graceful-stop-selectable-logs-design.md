# DevTray Tray Menu on Click, Group Quick Chips, Graceful Stop & Selectable Log Search Specification

**Date:** 2026-10-01  
**Status:** Approved  
**Topic:** Tray Left-Click Menu, Group Quick Chips, Graceful Stop with Custom Stop Command, Selectable Logs with Find/Highlight Search  

---

## 1. Context & Motivation

Following daily usage and real developer workflow demands in `devtray`, four major UX and process management pain points were identified:
1. **Tray Menu Activation:** On Linux desktop environments (e.g. KDE Plasma, GNOME Wayland), left-clicking the tray icon does not show the context menu; users are forced to right-click. Setting `MENU_ON_ACTIVATE = true` in `ksni::Tray` allows left-click to open the menu directly.
2. **Group Assignment Usability:** Entering group names in `TaskDialog` currently requires typing the exact string from memory. Typos create accidental duplicate groups. Providing clickable quick chips/pills for existing groups under the input eliminates manual typing while preserving the ability to type a new group.
3. **Graceful Stop Lifecycle & Stop Commands:** Currently, stopping a task sends `SIGKILL` directly to the process group, abruptly killing tasks without allowing graceful shutdown or cleanup handlers (critical for `pnpm`, `npm`, `python`, `node`). Furthermore, orchestrators like `podman compose` or containerized services require custom teardown commands (`podman compose down`). Users currently resort to creating dummy tasks like "Stop WARP Proxies".
4. **Log Selection & Find (Ctrl+F):** Logs are currently rendered in a non-selectable Slint `Text` element. Users have to copy all logs into an external text editor just to select a snippet or search for an error. Replacing `Text` with `TextEdit { read-only: true }` and adding an in-dialog Find/Highlight bar provides native cursor text selection and search navigation.

---

## 2. Architecture & Components

### 2.1 Tray Menu Activation (`src/gui/tray.rs`)
- In `impl ksni::Tray for DevTraySysTray`:
  ```rust
  const MENU_ON_ACTIVATE: bool = true;
  ```
- This triggers the standard StatusNotifierItem context menu on left-click as well as right-click across all supported desktop shells.

### 2.2 Group Quick Chips in Task Dialog (`ui/task_dialog.slint` & `src/gui/bridge.rs`)
- **UI Component:**
  - `TaskDialog` receives `in property <[string]> available_groups;`.
  - Under the Group `LineEdit`, render horizontal chips for all distinct non-empty groups currently defined in tasks.
  - Clicking a chip populates `group_text`. If the clicked chip matches the active `group_text`, clicking it again toggles it off (clears `group_text`).
  - The `LineEdit` remains editable so users can type completely new group names.
- **Config & Model:**
  - Backwards-compatible: `group` remains `Option<String>` in `TaskConfig`.

### 2.3 Graceful Stop Lifecycle & Custom Stop Command (`src/core/process.rs`, `src/core/model.rs`, `ui/task_dialog.slint`)
- **Data Model:**
  - `TaskConfig` is extended with:
    ```rust
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_command: Option<String>,
    ```
- **Stop Workflow:**
  - When `ProcessManager::stop(task_id)` is invoked:
    - If task has a non-empty `stop_command`:
      DevTray executes `bash -c "<stop_command>"` in the task's `working_directory` in a background thread to gracefully trigger application shutdown (e.g. `podman compose down`).
    - Standard Process Termination Escalation (for tasks without `stop_command` or when terminating spawned process groups):
      1. Send `SIGINT` (Ctrl+C equivalent) to `-pid` process group.
      2. Spawn a background monitor thread: wait up to 3.0 seconds checking if the process group has terminated.
      3. If still running after 3.0s, send `SIGTERM` to `-pid`.
      4. Wait up to 2.0 seconds checking if the process group has terminated.
      5. If still running after 2.0s, send `SIGKILL` to `-pid` as a fail-safe fallback.
- **Task Dialog:**
  - Add optional field `Stop Command (Optional)` / `停止命令 (可选)` to `TaskDialog`.

### 2.4 Selectable Logs & Find/Highlight Search (`ui/log_viewer.slint`, `src/gui/bridge.rs`, `src/core/i18n.rs`)
- **Selectable Text Box:**
  - Replace static `Text` with Slint's `TextEdit`:
    ```slint
    TextEdit {
        read-only: true;
        text: root.log_text;
        font-family: "DejaVu Sans Mono, monospace";
        font-size: 11px;
        wrap: word-wrap;
    }
    ```
  - Enables drag selection, double-click word selection, Ctrl+A select all, and Ctrl+C copy.
- **Find/Highlight Search Bar:**
  - Positioned above the terminal log view.
  - Inputs:
    - `LineEdit` for search keyword.
    - Match count text indicator (e.g. `2/5` or `0/0`).
    - Previous (`▲`) and Next (`▼`) navigation buttons.
    - Keyboard triggers: Enter navigates to next match; Shift+Enter navigates to previous match.
  - Matching Logic:
    - Case-insensitive search across `log_text`.
    - Computes character offsets for matches.
    - Calls `textEdit.set-selection-offsets(start, end)` to highlight and automatically scroll viewport to the active match.
- **Localization (Strict Zero Emojis Rule):**
  - Add translations in English and Mandarin for all new strings in `i18n.rs` and `i18n.slint`:
    - `field_stop_command`, `placeholder_stop_command`, `search_logs`, `no_matches`, `match_counter`.

---

## 3. Verification & Testing Strategy

1. **Unit Tests:**
   - Test `TaskConfig` serialization/deserialization with and without `stop_command`.
   - Test signal escalation logic in `ProcessManager`.
   - Test substring match offset calculation for log search.
2. **Manual & Interactive Verification:**
   - Left-click on system tray icon opens the context menu on desktop.
   - Quick chips in `TaskDialog` update the group input field on click and deselect when clicked again.
   - Stop action triggers graceful termination (`SIGINT` -> `SIGTERM` -> `SIGKILL`) or runs custom `stop_command`.
   - Text inside LogViewer can be highlighted and copied with mouse and keyboard shortcuts.
   - Search bar in LogViewer correctly finds occurrences, displays match counts, and jumps between matches via Next/Prev buttons.
3. **Emoji & Style Verification:**
   - Verify zero emojis in UI strings and templates.
