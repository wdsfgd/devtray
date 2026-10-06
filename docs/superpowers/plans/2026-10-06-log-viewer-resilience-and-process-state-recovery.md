# Log Viewer Resilience & Process State Recovery Implementation Plan

## Overview & Context

This plan addresses two critical defects in DevTray:
1. **Log Viewer Crash on High Volume:** When tasks output large amounts of logs or stream logs rapidly, opening the `LogViewer` crashes or freezes the application due to unvirtualized Slint `TextEdit` layout overhead, per-line event loop flood, unbounded string concatenations, and leaked background subscriber threads.
2. **Missing Process State Recovery on Restart:** DevTray's `ProcessManager` tracks running PIDs purely in memory. Because tasks are launched in separate process groups (`libc::setpgid(0, 0)`), child processes survive application crashes or restarts. When DevTray is reopened, all tasks appear stopped (`is_running = false`) despite still running in the background.

Per user instruction, runtime state and logs are persisted under `.cache` (default: `~/.cache/devtray/state.json` or respecting `$XDG_CACHE_HOME`), NOT under `.config`.

---

## Architecture & Design Decisions

### 1. Log Viewer Resilience
- **UI Line Capping (Sliding Window):** When loading or streaming logs into Slint `ui.set_log_viewer_text`, retain only the most recent $N$ lines (e.g., 500 lines). Older lines are trimmed from the head of the string.
- **Throttled / Batched Updates:** In the log subscriber thread, collect incoming lines into a batch and flush to Slint via `slint::invoke_from_event_loop` at a throttled interval (100ms) rather than queuing one closure per individual line.
- **Explicit Modal Close Lifecycle:** Wire `ui.on_close_logs` (triggered by `LogViewer.close` and `root.is_open = false`) to signal the active subscriber thread to exit immediately and clean up the subscriber handle.
- **Open Log File Option:** Add a helper / button to open the log file (`~/.cache/devtray/logs/<task_name>.log`) with the system's default text viewer (`xdg-open`) so users can inspect massive historical logs without UI performance penalties.

### 2. Process State Recovery (PID Persistence & Adoption)
- **Cache Storage Location:** Store active process mapping in `state_file_path`:
  `$XDG_CACHE_HOME/devtray/state.json` (fallback: `~/.cache/devtray/state.json`).
  Schema:
  ```json
  {
    "tasks": {
      "<task_id>": {
        "pid": 12345,
        "name": "api-server",
        "started_at": 1728189600
      }
    }
  }
  ```
- **Persistence Triggers:**
  - `start(&task)`: Write `task_id` -> PID to `state.json`.
  - `stop(&task)` / `stop_with_config(&task)`: Remove `task_id` from `state.json`.
  - Process exit (wait thread or monitoring thread): Remove `task_id` from `state.json`.
- **Startup Adoption (`recover_running_processes`):**
  - On `ProcessManager::new` or startup initialization, load `state.json`.
  - For each saved task entry:
    - Check if `is_process_alive(pid)` is true.
    - Check `/proc/<pid>/cmdline` or ensure the process is valid (prevent PID recycling mismatch).
    - If alive: insert into in-memory `running` map, update `running_count`.
    - Spawn a background polling monitor thread (e.g., 500ms check interval) to detect process exit, since POSIX `waitpid()` cannot wait on processes that were not spawned by the current process instance.
    - If dead: prune the stale record from `state.json`.

---

## File Structure & Modifications

| File | Changes |
| :--- | :--- |
| `src/core/state.rs` *(New)* | `StateManager`: Loads/saves `{ task_id: ProcessState }` to `$XDG_CACHE_HOME/devtray/state.json`. Handles safe atomic writes and directory creation. |
| `src/core/mod.rs` | Expose `pub mod state;`. |
| `src/core/process.rs` | Integrate `StateManager`. Add `recover_running_processes(&self, tasks: &[TaskConfig])`. Spawn monitoring thread for adopted processes. Persist on start, remove on exit/stop. |
| `src/gui/bridge.rs` | 1. Implement batched throttled log streaming (100ms flush, sliding window max 500 lines).<br>2. Wire `on_close_logs` / `close_log_viewer` to terminate subscription.<br>3. Add `open_log_file` using `xdg-open` or `$EDITOR`.<br>4. Call process recovery on startup before `refresh_tasks`. |
| `ui/log_viewer.slint` | Add `open_file` callback and button; ensure `close` triggers bridge cleanup. |
| `ui/main_window.slint` | Forward `open_file` and `close` callbacks from `LogViewer` to bridge. |
| `src/core/i18n.rs` | Add bilingual strings for "Open in Editor" / "打开日志文件" (`open_log_file`). Zero emojis. |
| `tests/state_test.rs` *(New)* | Unit tests for `StateManager` persistence, serialization, and pruning. |
| `tests/process_test.rs` | Unit tests for process adoption and orphan liveness monitoring. |
| `tests/slint_bridge_test.rs` | Tests for log batching, sliding window trimming, and log viewer close cleanup. |

---

## Implementation Tasks

### Task 1: Process State Persistence Module (`src/core/state.rs`)
- Create `src/core/state.rs` with `TaskRuntimeRecord` and `StateManager`.
- Default path to `$XDG_CACHE_HOME/devtray/state.json` with fallback to `~/.cache/devtray/state.json`.
- Implement `load() -> HashMap<String, TaskRuntimeRecord>` and `save(&HashMap<String, TaskRuntimeRecord>)`.
- Implement atomic write (`.tmp` rename) to avoid corrupt state during crashes.
- Write unit tests in `tests/state_test.rs`.

### Task 2: Process Adoption & Recovery in `ProcessManager` (`src/core/process.rs`)
- Add `state_manager: StateManager` to `ProcessManager`.
- Update `start()` to record runtime state in `state_manager`.
- Update `stop_with_config()` and `stop()` to remove from `state_manager`.
- Implement `recover_running_processes(&self, tasks: &[TaskConfig])`:
  - Verify PID liveness via `is_process_alive(pid)`.
  - Re-register living processes into `running` map.
  - Spawn non-child polling monitor threads that sleep and poll `is_process_alive(pid)` every 500ms, triggering `on_exit` callback when the process dies.
  - Remove dead processes from `state_manager`.
- Update `tests/process_test.rs` to verify adoption and exit detection.

### Task 3: Log Viewer UI Line Capping & Batched Streaming (`src/gui/bridge.rs`)
- Implement a helper `truncate_to_max_lines(text: &str, max_lines: usize) -> String` to maintain a sliding window of max 500 lines.
- In `show_logs_for_task`:
  - Cap `initial_text` to the last 500 lines.
  - In the subscriber thread: collect incoming lines into a `Vec<String>` buffer.
  - Use a ticker / timeout (100ms) to flush batched lines to the UI via `slint::invoke_from_event_loop`.
  - Cap UI text to max 500 lines on each flush.
  - Only run search matching when query is non-empty and batched text changed.

### Task 4: Log Viewer Close Lifecycle & Open Log File Button
- Bind `ui.on_close_logs` in `bridge.rs` and connect it in `main_window.slint` and `log_viewer.slint`.
- In `close_logs`: drop `active_log_subscription` (which signals `stop_rx`), clear search state, and clear `log_viewer_text` if desired to free memory.
- Add `open_log_file(task_name)`: launch `xdg-open <log_file>` in background.
- Add "Open in Editor" button in `ui/log_viewer.slint` with localized text from `i18n.rs`.
- Verify zero emoji rule across all new UI elements and labels.

### Task 5: End-to-End Verification & Integration Tests
- Run `cargo test --all` across all modules.
- Verify crash recovery scenario: launch process, simulate controller reload, verify status is active.
- Verify high-volume log emission (10,000 lines generated in a loop) does not freeze or crash the UI.
