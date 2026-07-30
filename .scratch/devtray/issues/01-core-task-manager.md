# 01 — Core Task Manager (No GUI)

**What to build:** The core logic (TaskManager) that handles loading/saving the task list from `~/.config/devtray/config.json`, spawning subprocesses with stdout/stderr redirected to `~/.cache/devtray/logs/`, checking PID liveness, and killing processes. It includes test scripts to verify process management.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] Implement Task model (name, command, working_directory)
- [ ] Implement TaskManager to load/save JSON config
- [ ] Implement process spawning and log redirection to `~/.cache/devtray/logs/`
- [ ] Implement PID liveness check (`is_running()`) and graceful termination (`stop()`)
- [ ] Write integration test (using dummy shell commands) to verify state tracking without GUI
