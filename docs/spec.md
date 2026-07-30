## Problem Statement
The user frequently runs multiple background development tasks (e.g., starting dev servers, running docker containers) during their daily workflow. Managing these processes directly from the terminal is cognitively exhausting, as it requires keeping track of which terminal tabs are running which tasks, and periodically checking if a task has crashed or stopped. The user needs a simple, persistent way to quickly start, stop, and monitor these background tasks without leaving their current context.

## Solution
DevTray is a lightweight, low-footprint Linux system tray application. It allows the user to define a list of development tasks and quickly toggle them on or off from the system tray. A main window provides a simple management interface to add, edit, or remove these tasks. Under the hood, DevTray tracks the processes, records their terminal output into log files, and safely terminates them when the application closes. 

## User Stories

1. As a developer, I want to click a system tray icon to see a menu of all my configured tasks, so that I can quickly check their running statuses without opening a terminal.
2. As a developer, I want to click "Start" on a task from the tray menu, so that I can launch a background process effortlessly.
3. As a developer, I want to click "Stop" on a running task from the tray menu, so that I can terminate the background process immediately.
4. As a developer, I want to open a Main Window from the tray menu, so that I can manage my list of tasks.
5. As a developer, I want to add a new task (Name, Command, Working Directory) via the Main Window, so that I can start tracking a new devtool.
6. As a developer, I want to delete an existing task via the Main Window, so that I can remove devtools I no longer use.
7. As a developer, I want my task configuration to be saved persistently, so that I don't have to re-enter my tasks when I restart DevTray.
8. As a developer, I want the output (stdout/stderr) of my running tasks to be redirected to a log file, so that I can inspect failures if a task crashes.
9. As a developer, I want DevTray to gracefully kill all active tasks when I quit the application, so that I don't leave zombie/orphan processes running in the background.
10. As a developer, I want a confirmation dialog before DevTray quits, so that I don't accidentally kill all my running tasks by misclicking.

## Implementation Decisions

- **UI Framework**: Python with GTK3 (`PyGObject`). Note: GTK4 is explicitly avoided as it removed system tray support (Gtk.StatusIcon). 
- **System Tray Library**: `libayatana-appindicator3` via `gi.repository`. *Technical Clarification*: We will ensure the most up-to-date Python bindings and library linking are used to prevent deprecation warnings (e.g. `libayatana-appindicator is deprecated. Please use libayatana-appindicator-glib`).
- **Configuration Storage**: Tasks are stored as a JSON array in `~/.config/devtray/config.json`.
- **Log Storage**: Task outputs are redirected to `~/.cache/devtray/logs/<task-name>.log`.
- **Process Management**: Tasks are executed using Python's `subprocess.Popen(..., shell=True)`. DevTray tracks their liveness via `process.poll()` or by checking the PID.
- **Application Lifecycle**: Clicking "Quit DevTray" triggers a GTK confirmation dialog. If confirmed, SIGTERM is sent to all active PIDs tracked by the application before it exits.

## Testing Decisions

To ensure a robust test suite, we will avoid testing the GTK UI directly (as GUI testing is brittle and slow). Instead, we will test at the **Task Manager** seam.
- **The Seam**: A `TaskManager` class that exposes methods to `start_task()`, `stop_task()`, `get_active_tasks()`, and `load_config()`.
- **Test Strategy**: We will write integration-level tests against the `TaskManager`. Rather than mocking `subprocess`, we will run actual lightweight shell commands (e.g., `sleep 0.1` or `echo "test"`) to verify that PIDs are correctly tracked, that logs are correctly written to the cache directory, and that processes are successfully terminated.
- **Why this makes a good test**: It tests the external behavior (processes actually spawning and logging) without coupling to the implementation details of GTK UI widgets.

## Out of Scope
- Cross-platform support (Windows/macOS are unsupported; this relies on Linux XDG standards and AppIndicator).
- Parsing or displaying the logs directly inside the DevTray UI (users can open the log files in their preferred text editor).
- Auto-restarting tasks that crash (DevTray only monitors and displays the stopped status).

## Further Notes
- We must ensure that the JSON configuration gracefully handles missing fields or corrupted data by falling back to an empty task list.
