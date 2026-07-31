# DevTray

A simple Linux system tray application for developers to manage background services (e.g., Node dev servers, Docker containers, database instances).

![DevTray](assets/icon.png)

Run long-running background tasks without keeping terminal windows open. Group tasks by project, reorder them, and start/stop them instantly from the system tray. Processes are killed cleanly via process group termination, preventing detached child processes or lingering ports.

## Development & Build

*(Note: End-users do not need to install anything. They can simply download the pre-compiled `devtray` binary and run it. The following instructions are strictly for developers who want to compile the application from source).*

Install GTK3 and AppIndicator development headers:

```bash
# Ubuntu / Debian
sudo apt-get install libgtk-3-dev libayatana-appindicator3-dev

# Fedora
sudo dnf install gtk3-devel libayatana-appindicator-gtk3-devel
```

Build the binary:

```bash
git clone https://github.com/wdsfgd/devtray.git
cd devtray
mkdir -p bin
go build -o bin/devtray
```

## Usage

Start the app:
```bash
./bin/devtray
```

- Add a new script (e.g., `npm run dev`) and specify its working directory.
- Click the tray icon to quickly toggle your running scripts.
- Logs are automatically saved in `~/.cache/devtray/logs/`.
- Config is stored in `~/.config/devtray/config.json`.