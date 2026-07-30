# 02 — System Tray & Basic State Display (GUI)

**What to build:** A GTK3 `AppIndicator` that integrates with `TaskManager` to display the loaded tasks as a system tray menu. The menu should accurately display "Running" or "Stopped" for each task. The UI is read-only at this stage.

**Blocked by:** 01 — Core Task Manager (No GUI)

**Status:** ready-for-agent

- [ ] Initialize GTK3 and AppIndicator3 safely (addressing the deprecation warning by using correct bindings)
- [ ] Build tray menu dynamically based on `TaskManager.get_tasks()`
- [ ] Render visual status (Running/Stopped) next to task names
