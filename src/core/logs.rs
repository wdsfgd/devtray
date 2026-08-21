use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct LogBroadcaster {
    log_dir: PathBuf,
    max_history: usize,
    buffers: Arc<Mutex<HashMap<String, VecDeque<String>>>>,
    senders: Arc<Mutex<HashMap<String, Vec<Sender<String>>>>>,
}

impl LogBroadcaster {
    pub fn new(log_dir: PathBuf, max_history: usize) -> Self {
        Self {
            log_dir,
            max_history,
            buffers: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn append(&self, task_name: &str, line: &str) -> std::io::Result<()> {
        fs::create_dir_all(&self.log_dir)?;
        let log_file = self.log_dir.join(format!("{}.log", task_name));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
        writeln!(file, "{}", line)?;

        // Update in-memory ring buffer
        if self.max_history > 0 {
            let mut buffers = self.buffers.lock().unwrap();
            let buf = buffers.entry(task_name.to_string()).or_default();
            while buf.len() >= self.max_history {
                buf.pop_front();
            }
            buf.push_back(line.to_string());
        }

        // Notify subscribers
        {
            let mut senders_map = self.senders.lock().unwrap();
            if let Some(senders) = senders_map.get_mut(task_name) {
                senders.retain(|s| s.send(line.to_string()).is_ok());
            }
        }

        Ok(())
    }

    pub fn get_recent_lines(&self, task_name: &str) -> Vec<String> {
        let buffers = self.buffers.lock().unwrap();
        buffers
            .get(task_name)
            .map(|b| b.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn subscribe(&self, task_name: &str) -> Receiver<String> {
        let (tx, rx) = unbounded();
        let mut senders_map = self.senders.lock().unwrap();
        senders_map
            .entry(task_name.to_string())
            .or_default()
            .push(tx);
        rx
    }
}
