# DevTray

**DevTray** is a lightweight, system-tray-based task manager built with Go and GTK3. It is designed specifically for developers who need quick access to start, stop, and manage background scripts (like dev servers, build watchers, and docker containers) directly from their system tray.

![DevTray](assets/icon.png)

## Features

- **🚀 System Tray Integration**: Manage your background processes without keeping terminal windows open.
- **🗂️ Task Grouping**: Group related tasks together (e.g., Frontend, Backend, Infrastructure) with nested submenus.
- **⚡ Start/Stop All**: Quickly start or stop entire groups of tasks with a single click.
- **🛠️ Main Window GUI**: A clean interface to Add, Edit, Delete, and manually reorder your tasks.
- **🔴 Robust Process Management**: Uses `SIGKILL` to ensure that even stubborn dev servers (like Node, Vite, Webpack) stop completely.
- **📂 Persistent State**: Configuration and task definitions are safely stored in your user configuration directory.

## Installation

### Prerequisites

DevTray requires GTK3 and AppIndicator libraries to be installed on your system.

**Ubuntu / Debian:**
```bash
sudo apt-get install libgtk-3-dev libappindicator3-dev
```

**Fedora:**
```bash
sudo dnf install gtk3-devel libappindicator-gtk3-devel
```

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/devtray.git
   cd devtray
   ```

2. Build the binary:
   ```bash
   go build -o devtray_go_app
   ```

3. Run DevTray:
   ```bash
   ./devtray_go_app
   ```

## Usage

When you launch DevTray, it will appear in your system tray and automatically open the **Main Window**.

### Managing Tasks
- Click **Add Task** to define a new script.
- Provide a **Name**, **Command**, **Working Directory**, and an optional **Group**.
- Use the **⬆️** and **⬇️** buttons to manually reorder tasks within their groups.
- Start or stop tasks using the **▶️ / 🛑** buttons in the Main Window, or by checking/unchecking them in the Tray Menu.

### Files & Directories
- **Config**: `~/.config/devtray/config.json`
- **Logs**: `~/.cache/devtray/logs/` (Standard output and errors for each task are logged here).

## Architecture & Stack

- **Language**: [Go](https://golang.org/)
- **UI Framework**: [gotk3](https://github.com/gotk3/gotk3) (GTK3 bindings for Go)
- **Tray Provider**: [go-appindicator](https://github.com/dawidd6/go-appindicator)

## License

MIT License. See [LICENSE](LICENSE) for more information.