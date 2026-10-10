use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::core::config::ConfigManager;
use crate::core::logs::LogBroadcaster;
use crate::core::model::TaskConfig;
use crate::core::state::{StateManager, TaskRuntimeRecord};

type RunningMap = Arc<Mutex<HashMap<String, u32>>>;
type ExitCallback = Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;

pub struct ProcessManager {
    broadcaster: LogBroadcaster,
    running: RunningMap,
    on_exit: ExitCallback,
    state_manager: StateManager,
}

impl ProcessManager {
    pub fn new(broadcaster: LogBroadcaster) -> Self {
        Self::with_state_manager(broadcaster, StateManager::new())
    }

    pub fn with_state_manager(broadcaster: LogBroadcaster, state_manager: StateManager) -> Self {
        Self {
            broadcaster,
            running: Arc::new(Mutex::new(HashMap::new())),
            on_exit: Arc::new(Mutex::new(None)),
            state_manager,
        }
    }

    pub fn state_manager(&self) -> &StateManager {
        &self.state_manager
    }

    pub fn set_on_exit(&self, callback: impl Fn(String) + Send + Sync + 'static) {
        let mut on_exit = self.on_exit.lock().unwrap();
        *on_exit = Some(Box::new(callback));
    }

    pub fn is_running(&self, task_id: &str) -> bool {
        let running = self.running.lock().unwrap();
        running.contains_key(task_id)
    }

    pub fn get_pid(&self, task_id: &str) -> Option<u32> {
        let running = self.running.lock().unwrap();
        running.get(task_id).copied()
    }

    pub fn running_count(&self) -> usize {
        let running = self.running.lock().unwrap();
        running.len()
    }

    pub fn start(&self, task: &TaskConfig) -> std::io::Result<()> {
        if self.is_running(&task.id) {
            return Ok(());
        }

        let cwd = ConfigManager::expand_path(&task.working_directory);
        let mut cmd = Command::new("bash");
        cmd.arg("-c").arg(&task.command);
        cmd.current_dir(cwd);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Set process group ID so child and sub-processes can be killed together
        unsafe {
            cmd.pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            });
        }

        let mut child = cmd.spawn()?;
        let pid = child.id();
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        {
            let mut running = self.running.lock().unwrap();
            running.insert(task.id.clone(), pid);
        }

        let _ = self.state_manager.set_task(
            &task.id,
            TaskRuntimeRecord {
                pid,
                name: task.name.clone(),
                started_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            },
        );

        // Stream stdout
        if let Some(stdout) = stdout {
            let broadcaster = self.broadcaster.clone();
            let task_name = task.name.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = broadcaster.append(&task_name, &line);
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = stderr {
            let broadcaster = self.broadcaster.clone();
            let task_name = task.name.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    let _ = broadcaster.append(&task_name, &line);
                }
            });
        }

        // Wait thread for reaping
        let running_map = Arc::clone(&self.running);
        let on_exit_cb = Arc::clone(&self.on_exit);
        let state_mgr = self.state_manager.clone();
        let task_id = task.id.clone();
        thread::spawn(move || {
            let _ = child.wait();
            let removed = {
                let mut running = running_map.lock().unwrap();
                if let Some(&current_pid) = running.get(&task_id) {
                    if current_pid == pid {
                        running.remove(&task_id);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            };
            if removed {
                let _ = state_mgr.remove_task(&task_id);
                let cb_guard = on_exit_cb.lock().unwrap();
                if let Some(cb) = cb_guard.as_ref() {
                    cb(task_id);
                }
            }
        });

        Ok(())
    }

    fn system_boot_time_secs() -> Option<u64> {
        let uptime_str = std::fs::read_to_string("/proc/uptime").ok()?;
        let uptime_secs: f64 = uptime_str.split_whitespace().next()?.parse().ok()?;
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()?
            .as_secs();
        now_secs.checked_sub(uptime_secs as u64)
    }

    pub fn stop_with_config(&self, task: &TaskConfig) -> std::io::Result<()> {
        let pid = {
            let mut running = self.running.lock().unwrap();
            match running.remove(&task.id) {
                Some(pid) => pid,
                None => return Ok(()),
            }
        };
        let _ = self.state_manager.remove_task(&task.id);

        let stop_command = task.stop_command.clone();
        let working_dir = task.working_directory.clone();

        thread::spawn(move || {
            if let Some(cmd) = stop_command {
                let cmd_trimmed = cmd.trim();
                if !cmd_trimmed.is_empty() {
                    let cwd = ConfigManager::expand_path(&working_dir);
                    let _ = Command::new("bash")
                        .arg("-c")
                        .arg(cmd_trimmed)
                        .current_dir(cwd)
                        .status();

                    // Allow process up to 3 seconds to exit naturally following stop_command
                    let start = std::time::Instant::now();
                    let wait_limit = std::time::Duration::from_millis(3000);
                    while start.elapsed() < wait_limit {
                        if !Self::is_process_alive(pid as i32) && !Self::is_process_alive(-(pid as i32)) {
                            return;
                        }
                        thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }

            if Self::is_process_alive(pid as i32) || Self::is_process_alive(-(pid as i32)) {
                Self::run_stop_escalation(
                    pid,
                    std::time::Duration::from_millis(3000),
                    std::time::Duration::from_millis(2000),
                    std::time::Duration::from_millis(50),
                );
            }
        });

        Ok(())
    }

    pub fn stop(&self, task_id: &str) -> std::io::Result<()> {
        let pid = {
            let mut running = self.running.lock().unwrap();
            match running.remove(task_id) {
                Some(pid) => pid,
                None => return Ok(()),
            }
        };
        let _ = self.state_manager.remove_task(task_id);

        Self::escalate_stop_signals(pid);
        Ok(())
    }

    pub fn recover_running_processes(&self, tasks: &[TaskConfig]) -> usize {
        let saved_records = self.state_manager.load();
        if saved_records.is_empty() {
            return 0;
        }

        let mut recovered_count = 0;
        let mut to_remove = Vec::new();
        let boot_time = Self::system_boot_time_secs();

        for (task_id, record) in saved_records {
            let is_known = tasks.iter().any(|t| t.id == task_id);
            if !is_known {
                to_remove.push(task_id);
                continue;
            }

            // Stale PID check: if the record started before current system boot, PID is recycled
            if let Some(boot_secs) = boot_time {
                if record.started_at > 0 && record.started_at + 5 < boot_secs {
                    to_remove.push(task_id);
                    continue;
                }
            }

            if Self::is_process_alive(record.pid as i32) {
                {
                    let mut running = self.running.lock().unwrap();
                    running.insert(task_id.clone(), record.pid);
                }
                recovered_count += 1;

                // Spawn orphan monitor thread to detect exit of adopted process
                let running_map = Arc::clone(&self.running);
                let on_exit_cb = Arc::clone(&self.on_exit);
                let state_mgr = self.state_manager.clone();
                let task_id_clone = task_id.clone();
                let pid = record.pid;
                thread::spawn(move || {
                    while Self::is_process_alive(pid as i32) {
                        thread::sleep(std::time::Duration::from_millis(500));
                    }
                    let removed = {
                        let mut running = running_map.lock().unwrap();
                        if let Some(&current_pid) = running.get(&task_id_clone) {
                            if current_pid == pid {
                                running.remove(&task_id_clone);
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };
                    if removed {
                        let _ = state_mgr.remove_task(&task_id_clone);
                        let cb_guard = on_exit_cb.lock().unwrap();
                        if let Some(cb) = cb_guard.as_ref() {
                            cb(task_id_clone);
                        }
                    }
                });
            } else {
                to_remove.push(task_id);
            }
        }

        for id in to_remove {
            let _ = self.state_manager.remove_task(&id);
        }

        recovered_count
    }

    fn escalate_stop_signals(pid: u32) {
        thread::spawn(move || {
            Self::run_stop_escalation(
                pid,
                std::time::Duration::from_millis(3000),
                std::time::Duration::from_millis(2000),
                std::time::Duration::from_millis(50),
            );
        });
    }

    pub fn run_stop_escalation(
        pid: u32,
        sigint_timeout: std::time::Duration,
        sigterm_timeout: std::time::Duration,
        poll_interval: std::time::Duration,
    ) {
        if pid == 0 {
            return;
        }
        let pgid = Pid::from_raw(-(pid as i32));
        let pid_i32 = pid as i32;
        let pgid_i32 = -(pid as i32);
        let is_alive = || Self::is_process_alive(pid_i32) || Self::is_process_alive(pgid_i32);

        // Step A: kill(-pid, Signal::SIGINT)
        let _ = kill(pgid, Signal::SIGINT);

        // Step B: Loop for sigint_timeout checking if PID has terminated
        let start = std::time::Instant::now();
        while start.elapsed() < sigint_timeout {
            if !is_alive() {
                return;
            }
            let remaining = sigint_timeout.saturating_sub(start.elapsed());
            thread::sleep(poll_interval.min(remaining));
        }
        if !is_alive() {
            return;
        }

        // Step C: If still alive, kill(-pid, Signal::SIGTERM)
        let _ = kill(pgid, Signal::SIGTERM);

        // Step D: Loop for sigterm_timeout checking if PID has terminated
        let start = std::time::Instant::now();
        while start.elapsed() < sigterm_timeout {
            if !is_alive() {
                return;
            }
            let remaining = sigterm_timeout.saturating_sub(start.elapsed());
            thread::sleep(poll_interval.min(remaining));
        }
        if !is_alive() {
            return;
        }

        // Step E: If still alive, kill(-pid, Signal::SIGKILL)
        let _ = kill(pgid, Signal::SIGKILL);
    }

    pub fn is_alive_result(res: Result<(), nix::errno::Errno>) -> bool {
        match res {
            Ok(_) => true,
            Err(nix::errno::Errno::EPERM) => true,
            Err(_) => false,
        }
    }

    pub fn is_process_alive(pid: i32) -> bool {
        if pid <= 0 {
            return false;
        }
        let nix_pid = Pid::from_raw(pid);
        match kill(nix_pid, None) {
            Ok(_) => true,
            Err(nix::errno::Errno::EPERM) => true,
            Err(_) => false,
        }
    }

    pub fn stop_all(&self) {
        let task_ids: Vec<String> = {
            let running = self.running.lock().unwrap();
            running.keys().cloned().collect()
        };
        for id in task_ids {
            let _ = self.stop(&id);
        }
    }
}

pub fn is_process_alive(pid: i32) -> bool {
    ProcessManager::is_process_alive(pid)
}
