use std::io::{self, Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

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
    shutdown_stream: Option<UnixStream>,
    listener_thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if let Some(mut stream) = self.shutdown_stream.take() {
            let _ = stream.write_all(&[1]);
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
            let (shutdown_tx, shutdown_rx) = match UnixStream::pair() {
                Ok(pair) => pair,
                Err(e) => {
                    eprintln!("[SingleInstance] Failed to create shutdown socket pair: {e}");
                    return;
                }
            };

            self.shutdown_stream = Some(shutdown_tx);

            let handle = std::thread::spawn(move || {
                let listener_fd = listener.as_raw_fd();
                let shutdown_fd = shutdown_rx.as_raw_fd();

                loop {
                    let mut fds = [
                        libc::pollfd {
                            fd: listener_fd,
                            events: libc::POLLIN,
                            revents: 0,
                        },
                        libc::pollfd {
                            fd: shutdown_fd,
                            events: libc::POLLIN | libc::POLLHUP,
                            revents: 0,
                        },
                    ];

                    let ret = unsafe { libc::poll(fds.as_mut_ptr(), 2, -1) };
                    if ret <= 0 {
                        break;
                    }

                    // Check if shutdown was requested
                    if fds[1].revents != 0 {
                        break;
                    }

                    // Check if new connection arrived on listener
                    if (fds[0].revents & libc::POLLIN) != 0 {
                        match listener.accept() {
                            Ok((mut stream, _)) => {
                                let mut buf = [0u8; 32];
                                if let Ok(n) = stream.read(&mut buf) {
                                    if n > 0 {
                                        on_show();
                                    }
                                }
                            }
                            Err(_) => break,
                        }
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
            shutdown_stream: None,
            listener_thread: None,
        }))
    }
}
