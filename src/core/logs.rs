use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::{HashMap, VecDeque};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Strips ANSI escape sequences (colors, styles, cursor commands) from text.
pub fn strip_ansi_codes(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.peek() == Some(&'[') {
                chars.next();
                while let Some(&next) = chars.peek() {
                    chars.next();
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// Finds all case-insensitive occurrences of `needle` in `haystack`, returning
/// their byte offset pairs `(start, end)` suitable for text selection / slicing.
pub fn find_matches(haystack: &str, needle: &str) -> Vec<(usize, usize)> {
    if needle.is_empty() || haystack.is_empty() {
        return Vec::new();
    }
    let needle_lower = needle.to_lowercase();
    let haystack_lower = haystack.to_lowercase();

    // Fast path: if lowercasing preserved byte length (ASCII, CJK, etc.)
    if haystack_lower.len() == haystack.len() {
        let mut matches = Vec::new();
        let mut start = 0;
        while let Some(pos) = haystack_lower[start..].find(&needle_lower) {
            let actual_start = start + pos;
            let actual_end = actual_start + needle_lower.len();
            matches.push((actual_start, actual_end));
            start = actual_end;
        }
        return matches;
    }

    // Unicode edge case: lowercasing changed byte length
    let mut lower_to_orig = Vec::with_capacity(haystack_lower.len() + 1);
    for (orig_byte, ch) in haystack.char_indices() {
        let ch_lower_len = ch.to_lowercase().to_string().len();
        for _ in 0..ch_lower_len {
            lower_to_orig.push(orig_byte);
        }
    }
    lower_to_orig.push(haystack.len());

    let mut matches = Vec::new();
    let mut start = 0;
    while let Some(pos) = haystack_lower[start..].find(&needle_lower) {
        let lower_start = start + pos;
        let lower_end = lower_start + needle_lower.len();
        let orig_start = lower_to_orig[lower_start];
        let orig_end = if lower_end < lower_to_orig.len() {
            lower_to_orig[lower_end]
        } else {
            haystack.len()
        };
        matches.push((orig_start, orig_end));
        start = lower_end;
    }
    matches
}


#[derive(Clone, Debug)]
pub struct LogBroadcaster {
    log_dir: PathBuf,
    max_history: usize,
    buffers: Arc<Mutex<HashMap<String, VecDeque<String>>>>,
    senders: Arc<Mutex<HashMap<String, Vec<Sender<String>>>>>,
    files: Arc<Mutex<HashMap<String, File>>>,
}

impl LogBroadcaster {
    pub fn new(log_dir: PathBuf, max_history: usize) -> Self {
        Self {
            log_dir,
            max_history,
            buffers: Arc::new(Mutex::new(HashMap::new())),
            senders: Arc::new(Mutex::new(HashMap::new())),
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn append(&self, task_name: &str, raw_line: &str) -> std::io::Result<()> {
        let line = strip_ansi_codes(raw_line);

        // Write to cached file handle
        {
            let mut files = self.files.lock().unwrap();
            if !files.contains_key(task_name) {
                fs::create_dir_all(&self.log_dir)?;
                let log_file = self.log_dir.join(format!("{}.log", task_name));
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)?;
                files.insert(task_name.to_string(), file);
            }
            if let Some(file) = files.get_mut(task_name) {
                writeln!(file, "{}", line)?;
                file.flush()?;
            }
        }

        // Update in-memory ring buffer
        if self.max_history > 0 {
            let mut buffers = self.buffers.lock().unwrap();
            let buf = buffers.entry(task_name.to_string()).or_default();
            while buf.len() >= self.max_history {
                buf.pop_front();
            }
            buf.push_back(line.clone());
        }

        // Notify subscribers
        {
            let mut senders_map = self.senders.lock().unwrap();
            if let Some(senders) = senders_map.get_mut(task_name) {
                senders.retain(|s| s.send(line.clone()).is_ok());
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
