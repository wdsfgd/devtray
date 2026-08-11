# DevTray

A simple Linux system tray application for developers to manage background services (e.g., Node dev servers, Docker containers, database instances).

![DevTray](assets/icon.png)

Run long-running background tasks without keeping terminal windows open. Group tasks by project, reorder them, and start/stop them instantly from the system tray. Processes are killed cleanly via process group termination, preventing detached child processes or lingering ports.

## Installation (End Users)

Although DevTray is distributed as a pre-compiled binary, it relies on your Linux desktop's native GUI libraries to render the system tray correctly. You must have the runtime dependencies installed on your system.

Install the runtime dependencies:

```bash
# Ubuntu / Debian
sudo apt-get install libayatana-appindicator3-1

# Fedora
sudo dnf install libayatana-appindicator-gtk3
```

## Development & Build

If you want to compile the application from source yourself, you will also need the development headers in addition to the runtime libraries:

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