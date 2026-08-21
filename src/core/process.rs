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

pub struct ProcessManager {
    broadcaster: LogBroadcaster,
    running: RunningMap,
}

impl ProcessManager {
    pub fn new(broadcaster: LogBroadcaster) -> Self {
        Self {
            broadcaster,
            running: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn is_running(&self, task_id: &str) -> bool {
        let running = self.running.lock().unwrap();
        running.contains_key(task_id)
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
        let task_id = task.id.clone();
        thread::spawn(move || {
            let _ = child.wait();
            let mut running = running_map.lock().unwrap();
            if let Some(&current_pid) = running.get(&task_id) {
                if current_pid == pid {
                    running.remove(&task_id);
                }
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

        // Send SIGKILL to the entire process group (-pid)
        let _ = kill(Pid::from_raw(-(pid as i32)), Signal::SIGKILL);

        Ok(())
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
