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

type RunningMap = Arc<Mutex<HashMap<String, u32>>>;
type ExitCallback = Arc<Mutex<Option<Box<dyn Fn(String) + Send + Sync>>>>;

pub struct ProcessManager {
    broadcaster: LogBroadcaster,
    running: RunningMap,
    on_exit: ExitCallback,
}

impl ProcessManager {
    pub fn new(broadcaster: LogBroadcaster) -> Self {
        Self {
            broadcaster,
            running: Arc::new(Mutex::new(HashMap::new())),
            on_exit: Arc::new(Mutex::new(None)),
        }
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
                let cb_guard = on_exit_cb.lock().unwrap();
                if let Some(cb) = cb_guard.as_ref() {
                    cb(task_id);
                }
            }
        });

        Ok(())
    }

    pub fn stop_with_config(&self, task: &TaskConfig) -> std::io::Result<()> {
        let pid = {
            let mut running = self.running.lock().unwrap();
            match running.remove(&task.id) {
                Some(pid) => pid,
                None => return Ok(()),
            }
        };

        if let Some(ref cmd) = task.stop_command {
            let cmd_trimmed = cmd.trim();
            if !cmd_trimmed.is_empty() {
                let cmd_str = cmd_trimmed.to_string();
                let cwd = ConfigManager::expand_path(&task.working_directory);
                thread::spawn(move || {
                    let _ = Command::new("bash")
                        .arg("-c")
                        .arg(&cmd_str)
                        .current_dir(cwd)
                        .status();
                });
            }
        }

        Self::escalate_stop_signals(pid);
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

        Self::escalate_stop_signals(pid);
        Ok(())
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
        let pgid = Pid::from_raw(-(pid as i32));
        let p_pid = Pid::from_raw(pid as i32);

        // Step A: kill(-pid, Signal::SIGINT)
        let _ = kill(pgid, Signal::SIGINT);

        // Step B: Loop for sigint_timeout checking if PID has terminated
        let start = std::time::Instant::now();
        while start.elapsed() < sigint_timeout {
            if !Self::is_process_alive(p_pid, pgid) {
                return;
            }
            let remaining = sigint_timeout.saturating_sub(start.elapsed());
            thread::sleep(poll_interval.min(remaining));
        }
        if !Self::is_process_alive(p_pid, pgid) {
            return;
        }

        // Step C: If still alive, kill(-pid, Signal::SIGTERM)
        let _ = kill(pgid, Signal::SIGTERM);

        // Step D: Loop for sigterm_timeout checking if PID has terminated
        let start = std::time::Instant::now();
        while start.elapsed() < sigterm_timeout {
            if !Self::is_process_alive(p_pid, pgid) {
                return;
            }
            let remaining = sigterm_timeout.saturating_sub(start.elapsed());
            thread::sleep(poll_interval.min(remaining));
        }
        if !Self::is_process_alive(p_pid, pgid) {
            return;
        }

        // Step E: If still alive, kill(-pid, Signal::SIGKILL)
        let _ = kill(pgid, Signal::SIGKILL);
    }

    fn is_process_alive(pid: Pid, pgid: Pid) -> bool {
        kill(pid, None).is_ok() || kill(pgid, None).is_ok()
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
