# DevTray

DevTray is a lightweight, system-tray-based task manager built with Go and GTK3. It is designed specifically for developers who need quick access to start, stop, and manage background scripts (like dev servers, build watchers, and docker containers) directly from their system tray.

![DevTray](assets/icon.png)

## Features

- System Tray Integration: Manage your background processes without keeping terminal windows open.
- Task Grouping: Group related tasks together (e.g., Frontend, Backend, Infrastructure) with nested submenus.
- Start/Stop All: Quickly start or stop entire groups of tasks with a single click.
- Main Window GUI: A clean interface to Add, Edit, Delete, and manually reorder your tasks.
- Robust Process Management: Uses SIGKILL to ensure that even stubborn dev servers (like Node, Vite, Webpack) stop completely.
- Persistent State: Configuration and task definitions are safely stored in your user configuration directory.

## Installation

### Prerequisites

DevTray requires GTK3 and AppIndicator libraries to be installed on your system.

**Ubuntu / Debian:**
```bash
sudo apt-get install libgtk-3-dev libayatana-appindicator3-dev
```
*(Note: libayatana-appindicator3-dev is the modern replacement for libappindicator3-dev on newer Ubuntu versions, but both work).*

**Fedora:**
```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/wdsfgd/devtray.git
   cd devtray
   ```

2. Build the binary (it is standard practice in Go to output to a `bin/` directory):
   ```bash
   mkdir -p bin
   go build -o bin/devtray
   ```

3. Run DevTray:
   ```bash
   ./bin/devtray
   ```

## Usage

When you launch DevTray, it will appear in your system tray and automatically open the Main Window.

### Managing Tasks
- Click Add Task to define a new script.
- Provide a Name, Command, Working Directory, and an optional Group.
- Use the up and down buttons to manually reorder tasks within their groups.
- Start or stop tasks using the toggle buttons in the Main Window, or by checking/unchecking them in the Tray Menu.

### Files and Directories
- Config: `~/.config/devtray/config.json`
- Logs: `~/.cache/devtray/logs/` (Standard output and errors for each task are logged here).