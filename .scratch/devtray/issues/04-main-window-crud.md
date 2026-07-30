# 04 — Main Window (Management & CRUD) (GUI)

**What to build:** A main `Gtk.Window` accessible from the tray menu. It displays the full task list and includes forms/buttons to add new tasks or delete existing ones. Changes reflect instantly in the `config.json` and the tray menu.

**Blocked by:** 03 — Start/Stop & Graceful Exit (GUI)

**Status:** ready-for-agent

- [ ] Create `MainWindow` class with a `Gtk.ListBox` showing all tasks
- [ ] Add an "Add Task" button that opens a dialog with inputs for Name, Command, and Working Directory
- [ ] Add a "Delete" button for each task in the list
- [ ] Persist added/deleted tasks via `TaskManager.save_config()`
- [ ] Ensure tray menu re-renders when the task list is mutated in the main window
