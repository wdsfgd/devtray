# Tray Menu Click, Group Quick Chips, Graceful Stop & Selectable Log Search Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement left-click tray menu activation, group quick chips in task dialog, graceful stop lifecycle (SIGINT -> SIGTERM -> SIGKILL) with custom optional stop_command, and selectable logs with interactive find/highlight search in the log viewer.

**Architecture:** 
1. `ksni::Tray::MENU_ON_ACTIVATE = true` handles left-click menu invocation natively.
2. `TaskConfig` is extended with optional `stop_command`, which `ProcessManager` executes upon task stop. For standard terminations, `ProcessManager` sends `SIGINT`, waits with timeout, escalates to `SIGTERM`, and finally `SIGKILL` as fallback.
3. `TaskDialog` is updated with existing group chips for 1-click selection and a stop command input.
4. `LogViewer` replaces static `Text` with `TextEdit { read-only: true }` and adds a Find/Highlight search bar with Next/Prev and selection offsets.

**Tech Stack:** Rust, Slint UI 1.17, ksni 0.3.6, nix 0.29 (signals/process), crossbeam-channel, serde_json.

## Global Constraints
- Zero emojis across all UI templates and translations (strict project rule).
- Retain full backward compatibility with existing `config.json` files.
- All newly introduced UI strings must support both English (`en`) and Mandarin (`zh`).
- Do not block the Slint UI main thread during graceful stop timeouts.

---

### Task 1: System Tray Left-Click Menu Activation

**Files:**
- Modify: `src/gui/tray.rs:59-65`

**Interfaces:**
- Consumes: `ksni::Tray` trait
- Produces: `DevTraySysTray` with `MENU_ON_ACTIVATE = true`

- [ ] **Step 1: Write test for Tray configuration**

Add a unit test in `src/gui/tray.rs` (or `tests/tray_test.rs`):
```rust
#[test]
fn test_menu_on_activate_constant() {
    assert!(<crate::gui::tray::DevTraySysTray as ksni::Tray>::MENU_ON_ACTIVATE);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_menu_on_activate_constant`
Expected: FAIL (constant defaults to `false`)

- [ ] **Step 3: Update `src/gui/tray.rs`**

In `impl ksni::Tray for DevTraySysTray`:
```rust
impl ksni::Tray for DevTraySysTray {
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        "devtray".to_string()
    }
    // ...
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_menu_on_activate_constant`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/gui/tray.rs
git commit -m "feat(tray): enable MENU_ON_ACTIVATE for left-click tray menu"
```

---

### Task 2: Data Model & Process Graceful Stop with Custom Stop Command

**Files:**
- Modify: `src/core/model.rs:5-20`
- Modify: `src/core/process.rs:130-155`
- Test: `tests/process_test.rs`

**Interfaces:**
- Consumes: `TaskConfig.stop_command: Option<String>`
- Produces: `ProcessManager::stop_with_config(&self, task: &TaskConfig) -> std::io::Result<()>` and updated `ProcessManager::stop(&self, task_id: &str)`

- [ ] **Step 1: Extend `TaskConfig` in `src/core/model.rs`**

Add `stop_command` field:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskConfig {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default = "default_working_directory")]
    pub working_directory: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stop_command: Option<String>,
}
```

- [ ] **Step 2: Write tests for `TaskConfig` stop_command serialization & process stop escalation**

In `src/core/model.rs`:
```rust
#[test]
fn test_task_config_stop_command_serde() {
    let json = r#"{
        "id": "t1",
        "name": "Warp",
        "command": "podman compose up",
        "working_directory": ".",
        "stop_command": "podman compose down"
    }"#;
    let task: TaskConfig = serde_json::from_str(json).unwrap();
    assert_eq!(task.stop_command.as_deref(), Some("podman compose down"));
}
```

- [ ] **Step 3: Implement Graceful Stop Escalation in `src/core/process.rs`**

Refactor `ProcessManager`:
1. Provide `stop_with_config(&self, task: &TaskConfig) -> std::io::Result<()>`:
   - If `task.stop_command` is `Some(ref cmd)` and `!cmd.trim().is_empty()`:
     Spawn a thread running `bash -c <cmd>` in `task.working_directory`.
   - Also stop process group via graceful signals escalation in a worker thread:
     - Step A: `kill(-pid, Signal::SIGINT)`
     - Step B: Loop for 3000ms checking if PID has terminated (via `kill(Pid::from_raw(pid), None)`). If terminated, break.
     - Step C: If still alive, `kill(-pid, Signal::SIGTERM)`.
     - Step D: Loop for 2000ms checking if PID has terminated. If terminated, break.
     - Step E: If still alive, `kill(-pid, Signal::SIGKILL)`.
2. Make `stop(&self, task_id: &str)` call graceful signal escalation.
3. Update `SlintAppController::stop_task` to pass the corresponding `TaskConfig` to `ProcessManager`.

- [ ] **Step 4: Run tests to verify model and process logic**

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/core/model.rs src/core/process.rs src/gui/bridge.rs
git commit -m "feat(process): implement graceful stop escalation and optional stop_command execution"
```

---

### Task 3: i18n Localization & Group Quick Chips in Task Dialog

**Files:**
- Modify: `src/core/i18n.rs`
- Modify: `ui/i18n.slint`
- Modify: `ui/task_dialog.slint`
- Modify: `src/gui/bridge.rs`

**Interfaces:**
- Consumes: `SlintAppController::get_groups()`
- Produces: `TaskDialog.available_groups: [string]`, `TaskDialog.stop_command_text: string`, chip selection and toggle logic

- [ ] **Step 1: Add localization keys in `src/core/i18n.rs` & `ui/i18n.slint`**

Add strings for:
- `field_stop_command`: "Stop Command (Optional)" / "停止命令 (可选)"
- `placeholder_stop_command`: "e.g. podman compose down" / "例如: podman compose down"
- `search_logs`: "Search logs..." / "搜索日志..."
- `no_matches`: "No matches" / "无匹配项"

- [ ] **Step 2: Update `ui/task_dialog.slint`**

1. Add properties:
   ```slint
   in-out property <string> stop_command_text: "";
   in property <[string]> available_groups: [];
   ```
2. Below Group `LineEdit`, render quick chips:
   ```slint
   if root.available_groups.length > 0 : HorizontalLayout {
       spacing: 6px;
       for grp in root.available_groups : Rectangle {
           height: 22px;
           background: root.group_text == grp ? Theme.accent : Theme.input-bg;
           border-color: root.group_text == grp ? Theme.accent-border : Theme.border;
           border-width: 1px;
           border-radius: 4px;
           // TouchArea clicked => toggles root.group_text
       }
   }
   ```
3. Add Field 5 for `Stop Command (Optional)`.
4. Update `save` callback signature to include `stop_command`.

- [ ] **Step 3: Update `src/gui/bridge.rs`**

1. Supply `available_groups` to UI when opening TaskDialog.
2. Handle `save` callback with `stop_command` saving to `config.json`.
3. Populate `stop_command_text` when editing existing tasks.

- [ ] **Step 4: Run cargo check and verify dialog compiles**

Run: `cargo check`
Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add src/core/i18n.rs ui/i18n.slint ui/task_dialog.slint src/gui/bridge.rs
git commit -m "feat(ui): add group quick chips and stop command input in task dialog"
```

---

### Task 4: Selectable Logs & Find/Highlight Search in Log Viewer

**Files:**
- Modify: `ui/log_viewer.slint`
- Modify: `src/gui/bridge.rs`
- Modify: `src/core/logs.rs`

**Interfaces:**
- Consumes: `log_text: string`, `search_query: string`
- Produces: `TextEdit { read-only: true }`, Find bar with `match_index`, `match_count`, Next/Prev navigation, and auto-scroll selection

- [ ] **Step 1: Write helper for substring character offsets in `src/core/logs.rs`**

```rust
pub fn find_matches(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.is_empty() {
        return Vec::new();
    }
    let needle_lower = needle.to_lowercase();
    let haystack_lower = haystack.to_lowercase();
    let mut matches = Vec::new();
    let mut start = 0;
    while let Some(pos) = haystack_lower[start..].find(&needle_lower) {
        let actual_start = start + pos;
        let actual_end = actual_start + needle_lower.len();
        // Convert byte offsets to UTF-8 char offsets if needed or return byte indices
        matches.push((actual_start, actual_end));
        start = actual_end;
    }
    matches
}
```
Add unit tests for `find_matches` (case insensitivity, multiple occurrences, empty query).

- [ ] **Step 2: Run test for find_matches**

Run: `cargo test test_find_matches`
Expected: PASS

- [ ] **Step 3: Update `ui/log_viewer.slint`**

1. Replace `ScrollView` + `Text` with `TextEdit`:
   ```slint
   logTextEdit := TextEdit {
       read-only: true;
       text: root.log_text;
       font-family: "DejaVu Sans Mono, monospace";
       font-size: 11px;
       wrap: word-wrap;
   }
   ```
2. Add Find toolbar between header and log box:
   - Search `LineEdit` (`placeholder-text: root.tr.search_logs`)
   - Match count indicator `Text` (`0/0` or `1/4`)
   - `▲` Prev button and `▼` Next button
   - Callbacks `find_next()` and `find_prev()` or direct Slint state handling.
3. Call `logTextEdit.set-selection-offsets(start, end)` to highlight the active match.

- [ ] **Step 4: Connect LogViewer Search in `src/gui/bridge.rs`**

Bind callbacks between Slint LogViewer and match finder, updating current match indicator and selection offsets.

- [ ] **Step 5: Run tests and cargo check**

Run: `cargo check` and `cargo test`
Expected: SUCCESS

- [ ] **Step 6: Commit**

```bash
git add src/core/logs.rs ui/log_viewer.slint src/gui/bridge.rs
git commit -m "feat(logs): add selectable TextEdit and interactive find search in log viewer"
```

---

### Task 5: End-to-End Verification & Verification Suite

**Files:**
- Test: `tests/integration_test.rs`
- Modify: `tests/` if needed

- [ ] **Step 1: Run full automated test suite**

Run: `cargo test --all`
Expected: All tests PASS.

- [ ] **Step 2: Run release build check**

Run: `cargo build --release`
Expected: Zero warnings, zero errors.

- [ ] **Step 3: Verification of zero emojis**

Run: `python3 -c "import re, glob; emojis = [f for f in glob.glob('ui/**/*.slint', recursive=True) + glob.glob('src/**/*.rs', recursive=True) if re.search(r'[\U00010000-\U0010ffff]', open(f).read())]; assert not emojis, f'Found emojis in {emojis}'"`
Expected: Clean exit (no emojis found).

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "chore: verify test suite and release build for tray, groups, graceful stop, and log search"
```
