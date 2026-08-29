use crossbeam_channel::{bounded, Sender};
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Result of attempting to acquire single-instance lock.
pub enum SingleInstanceStatus {
    /// This process is the primary instance and owns the socket.
    Primary(SingleInstanceGuard),
    /// Another instance is already running; we notified it and should exit.
    Secondary,
}

/// Guard managing the Unix domain socket for the primary instance.
pub struct SingleInstanceGuard {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
    stop_tx: Option<Sender<()>>,
    listener_thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
        if let Some(handle) = self.listener_thread.take() {
            let _ = handle.join();
        }
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

impl SingleInstanceGuard {
    /// Spawns a background thread listening for incoming connection requests to show the window.
    pub fn start_listener<F>(&mut self, on_show: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Some(listener) = self.listener.take() {
            let (tx, rx) = bounded::<()>(1);
            self.stop_tx = Some(tx);

            if let Err(e) = listener.set_nonblocking(true) {
                eprintln!("[SingleInstance] Failed to set listener non-blocking: {e}");
            }

            let handle = std::thread::spawn(move || {
                while rx.try_recv().is_err() {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let mut buf = [0u8; 32];
                            if let Ok(n) = stream.read(&mut buf) {
                                if n > 0 {
                                    on_show();
                                }
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(50));
                        }
                        Err(_) => break,
                    }
                }
            });

            self.listener_thread = Some(handle);
        }
    }
}

pub struct SingleInstance;

impl SingleInstance {
    /// Computes the default socket path for DevTray.
    pub fn default_socket_path() -> PathBuf {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let path = PathBuf::from(runtime_dir).join("devtray.sock");
            return path;
        }
        if let Ok(home) = std::env::var("HOME") {
            let cache_dir = PathBuf::from(home).join(".cache").join("devtray");
            let _ = std::fs::create_dir_all(&cache_dir);
            return cache_dir.join("devtray.sock");
        }
        std::env::temp_dir().join("devtray.sock")
    }

    /// Attempts to acquire the single-instance lock.
    pub fn acquire(socket_path: &Path) -> io::Result<SingleInstanceStatus> {
        // 1. Try to connect to existing socket
        match UnixStream::connect(socket_path) {
            Ok(mut stream) => {
                // Another instance is actively listening
                let _ = stream.write_all(b"show\n");
                let _ = stream.flush();
                return Ok(SingleInstanceStatus::Secondary);
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                // Socket does not exist yet
            }
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
                // Stale socket file from previous crashed instance
                let _ = std::fs::remove_file(socket_path);
            }
            Err(_) => {
                // Any other error (e.g. invalid file format or permission on stale file)
                let _ = std::fs::remove_file(socket_path);
            }
        }

        // 2. Ensure parent directory exists
        if let Some(parent) = socket_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // 3. Bind the Unix listener
        let listener = UnixListener::bind(socket_path)?;

        Ok(SingleInstanceStatus::Primary(SingleInstanceGuard {
            socket_path: socket_path.to_path_buf(),
            listener: Some(listener),
            stop_tx: None,
            listener_thread: None,
        }))
    }
}
