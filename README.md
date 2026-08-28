# DevTray

A lightweight, ultra-low memory Linux system tray application for developers to manage background services (e.g., Node dev servers, Docker containers, database instances).

![DevTray](assets/icon.png)

Run long-running background tasks without keeping terminal windows open. Group tasks by project, reorder them with intuitive move controls, view live streaming logs, and start/stop services instantly from the system tray. Processes are terminated cleanly via process group PGID management, preventing detached child processes or lingering ports.

- **Ultra-Low Memory:** Built with pure Rust and Slint (uses ~22MB RSS, ~3.9MB private RAM).
- **Zero C++/Qt Runtime Overhead:** Native statically compiled binary.
- **D-Bus System Tray:** Full StatusNotifierItem support for GNOME, KDE, and XFCE.

## Installation

### Pre-compiled Binary

Download `devtray-*-x86_64-unknown-linux-gnu` or `devtray-*-linux-x86_64.tar.gz` from [Releases](https://github.com/wdsfgd/devtray/releases):

```bash
chmod +x devtray-*
./devtray-*
```

---

## Building from Source

### Prerequisites

- **Rust toolchain** (1.80+)
- **Standard Linux build libraries** (`pkg-config`, `libfontconfig1-dev`, `libfreetype6-dev`, `libdbus-1-dev`)

#### Ubuntu / Debian (22.04 / 24.04)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config \
  libx11-dev libxcursor-dev libxkbcommon-dev \
  libfontconfig1-dev libfreetype6-dev libdbus-1-dev
```

#### Fedora

```bash
sudo dnf install -y gcc fontconfig-devel freetype-devel \
  libxkbcommon-devel dbus-devel
```

### Build

```bash
git clone https://github.com/wdsfgd/devtray.git
cd devtray
cargo build --release
```

The optimized binary is located at `./target/release/devtray`.

---

## Usage

Start the app:
```bash
./target/release/devtray
```

- **Add Task**: Click "+ Add Task" to configure command, working directory, and group.
- **Reorder**: Use the Move Up (`▲`) and Move Down (`▼`) buttons to reorder tasks.
- **Live Logs**: Click "Logs" to stream live process output with color rendering and auto-scroll.
- **System Tray**: Left-click to toggle the main window; right-click for quick access to group submenus, Start/Stop All, or Quit.
- **Data & Logs**:
  - Config: `~/.config/devtray/config.json`
  - Logs: `~/.cache/devtray/logs/<task-name>.log`