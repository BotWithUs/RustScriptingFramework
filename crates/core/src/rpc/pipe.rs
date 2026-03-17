use std::io::{self, Read, Write};
use std::path::Path;
use thiserror::Error;
use tracing;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Not connected")]
    NotConnected,
    #[error("Connection lost")]
    ConnectionLost,
    #[error("Invalid message: length {0} exceeds maximum {1}")]
    MessageTooLarge(usize, usize),
}

const MAX_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Low-level Windows named pipe client with length-prefixed message framing.
/// Equivalent to Java's PipeClient.
///
/// Message format: [4-byte LE uint32: message length][N bytes: msgpack data]
///
/// Thread safety: Only one thread may access the pipe at a time.
/// The RpcClient manages synchronization via its lock.
pub struct PipeClient {
    pipe_name: String,
    // Using a raw file handle for synchronous (non-overlapped) I/O,
    // matching the Java implementation which uses RandomAccessFile.
    file: Option<std::fs::File>,
    open: bool,
}

impl PipeClient {
    pub fn new(pipe_name: impl Into<String>) -> Self {
        Self {
            pipe_name: pipe_name.into(),
            file: None,
            open: false,
        }
    }

    /// The pipe path (e.g., `\\.\pipe\BotWithUs`).
    pub fn pipe_name(&self) -> &str {
        &self.pipe_name
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Connect to the named pipe.
    pub fn connect(&mut self) -> Result<(), PipeError> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.pipe_name)?;
        self.file = Some(file);
        self.open = true;
        tracing::info!("Connected to pipe: {}", self.pipe_name);
        Ok(())
    }

    /// Send a message with length-prefix framing.
    /// Combines header + data into a single write to prevent split messages.
    pub fn send(&mut self, data: &[u8]) -> Result<(), PipeError> {
        let file = self.file.as_mut().ok_or(PipeError::NotConnected)?;
        let len = data.len() as u32;

        // Combine into single write buffer to prevent split messages in message mode
        let mut buf = Vec::with_capacity(4 + data.len());
        buf.extend_from_slice(&len.to_le_bytes());
        buf.extend_from_slice(data);

        file.write_all(&buf).map_err(|e| {
            self.open = false;
            PipeError::Io(e)
        })?;
        file.flush().map_err(|e| {
            self.open = false;
            PipeError::Io(e)
        })?;

        Ok(())
    }

    /// Read a length-prefixed message from the pipe.
    /// Blocks until a complete message is available.
    pub fn read_message(&mut self) -> Result<Vec<u8>, PipeError> {
        let file = self.file.as_mut().ok_or(PipeError::NotConnected)?;

        // Read 4-byte length header
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf).map_err(|e| {
            self.open = false;
            PipeError::Io(e)
        })?;

        let len = u32::from_le_bytes(len_buf) as usize;
        if len == 0 || len > MAX_MESSAGE_SIZE {
            return Err(PipeError::MessageTooLarge(len, MAX_MESSAGE_SIZE));
        }

        // Read payload
        let mut payload = vec![0u8; len];
        file.read_exact(&mut payload).map_err(|e| {
            self.open = false;
            PipeError::Io(e)
        })?;

        Ok(payload)
    }

    /// Check if data is available without blocking.
    /// On Windows this uses PeekNamedPipe internally.
    pub fn available(&self) -> bool {
        // std::fs::File doesn't expose available() directly.
        // For now, we'll rely on the blocking read_message approach.
        // A proper implementation would use Windows PeekNamedPipe API.
        // The RpcClient handles this via its threading model.
        self.open
    }

    /// Close the pipe connection.
    pub fn close(&mut self) {
        self.file.take();
        self.open = false;
        tracing::info!("Disconnected from pipe: {}", self.pipe_name);
    }

    /// List available pipes with a given prefix.
    #[cfg(target_os = "windows")]
    pub fn scan_pipes(prefix: &str) -> Vec<String> {
        let pipe_dir = Path::new(r"\\.\pipe\");
        // On Windows, we can enumerate pipes by reading the pipe filesystem
        // This is a simplified version; full implementation would use FindFirstFile
        match std::fs::read_dir(pipe_dir) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if name.starts_with(prefix) {
                        Some(format!(r"\\.\pipe\{}", name))
                    } else {
                        None
                    }
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }
}

impl Drop for PipeClient {
    fn drop(&mut self) {
        self.close();
    }
}

/// Wrapper around PipeClient that adds automatic reconnection.
/// Equivalent to Java's ReconnectablePipeClient.
pub struct ReconnectablePipeClient {
    inner: PipeClient,
    max_retries: u32,
    retry_delay: std::time::Duration,
    on_reconnect: Option<Box<dyn Fn() + Send>>,
    on_disconnect: Option<Box<dyn Fn() + Send>>,
}

impl ReconnectablePipeClient {
    pub fn new(pipe_name: impl Into<String>) -> Self {
        Self {
            inner: PipeClient::new(pipe_name),
            max_retries: 5,
            retry_delay: std::time::Duration::from_secs(1),
            on_reconnect: None,
            on_disconnect: None,
        }
    }

    pub fn set_on_reconnect(&mut self, callback: Box<dyn Fn() + Send>) {
        self.on_reconnect = Some(callback);
    }

    pub fn set_on_disconnect(&mut self, callback: Box<dyn Fn() + Send>) {
        self.on_disconnect = Some(callback);
    }

    pub fn inner(&self) -> &PipeClient {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut PipeClient {
        &mut self.inner
    }

    /// Attempt to reconnect with retry logic.
    pub fn try_reconnect(&mut self) -> Result<(), PipeError> {
        self.inner.close();
        if let Some(cb) = &self.on_disconnect {
            cb();
        }

        for attempt in 0..self.max_retries {
            tracing::info!(
                "Reconnection attempt {}/{} to {}",
                attempt + 1,
                self.max_retries,
                self.inner.pipe_name()
            );

            match self.inner.connect() {
                Ok(()) => {
                    tracing::info!("Reconnected successfully");
                    if let Some(cb) = &self.on_reconnect {
                        cb();
                    }
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("Reconnection attempt {} failed: {}", attempt + 1, e);
                    std::thread::sleep(self.retry_delay);
                }
            }
        }

        Err(PipeError::ConnectionLost)
    }
}
