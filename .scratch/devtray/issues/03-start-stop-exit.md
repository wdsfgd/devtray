# 03 — Start/Stop & Graceful Exit (GUI)

**What to build:** The tray menu items become interactive. Clicking a task toggles its running state. Clicking "Quit" shows a GTK confirmation dialog, and if confirmed, tells the `TaskManager` to terminate all active tasks before the application exits.

**Blocked by:** 02 — System Tray & Basic State Display (GUI)

**Status:** ready-for-agent

- [ ] Connect task menu items to `TaskManager.start()` and `TaskManager.stop()`
- [ ] Update tray menu text immediately when state changes
- [ ] Implement "Quit" menu item to open a `Gtk.MessageDialog` for confirmation
- [ ] On confirmation, invoke `TaskManager.stop_all()` and `Gtk.main_quit()`
