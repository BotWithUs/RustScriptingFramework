use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tracing;

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid frame size: {0}")]
    InvalidFrameSize(u32),
    #[error("Stream closed")]
    Closed,
}

const MAX_FRAME_SIZE: u32 = 8 * 1024 * 1024; // 8 MB

/// Stream info returned by the start_stream RPC call.
#[derive(Debug, Clone)]
pub struct StreamInfo {
    pub pipe_name: String,
    pub width: u32,
    pub height: u32,
    pub quality: u32,
    pub frame_skip: u32,
}

/// Reads JPEG frames from a dedicated streaming pipe.
/// Equivalent to Java's StreamPipeReader.
///
/// Frame format: [4-byte LE uint32: frame size][N bytes: JPEG data]
pub struct StreamPipeReader {
    running: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl StreamPipeReader {
    /// Start reading frames from the given pipe, invoking the callback for each frame.
    pub fn start<F, E>(
        pipe_name: String,
        frame_callback: F,
        error_callback: Option<E>,
    ) -> Self
    where
        F: Fn(Vec<u8>) + Send + 'static,
        E: Fn(StreamError) + Send + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = std::thread::Builder::new()
            .name("stream-reader".into())
            .spawn(move || {
                tracing::info!("Stream reader connecting to: {}", pipe_name);

                let mut file = match std::fs::File::open(&pipe_name) {
                    Ok(f) => f,
                    Err(e) => {
                        tracing::error!("Failed to open stream pipe: {}", e);
                        if let Some(ref cb) = error_callback {
                            cb(StreamError::Io(e));
                        }
                        return;
                    }
                };

                tracing::info!("Stream reader connected");

                while running_clone.load(Ordering::SeqCst) {
                    // Read frame size
                    let mut size_buf = [0u8; 4];
                    match file.read_exact(&mut size_buf) {
                        Ok(()) => {}
                        Err(e) => {
                            if running_clone.load(Ordering::SeqCst) {
                                tracing::error!("Stream read error: {}", e);
                                if let Some(ref cb) = error_callback {
                                    cb(StreamError::Io(e));
                                }
                            }
                            break;
                        }
                    }

                    let frame_size = u32::from_le_bytes(size_buf);
                    if frame_size == 0 || frame_size > MAX_FRAME_SIZE {
                        tracing::error!("Invalid stream frame size: {}", frame_size);
                        if let Some(ref cb) = error_callback {
                            cb(StreamError::InvalidFrameSize(frame_size));
                        }
                        break;
                    }

                    // Read frame data
                    let mut frame = vec![0u8; frame_size as usize];
                    match file.read_exact(&mut frame) {
                        Ok(()) => {
                            frame_callback(frame);
                        }
                        Err(e) => {
                            if running_clone.load(Ordering::SeqCst) {
                                tracing::error!("Stream frame read error: {}", e);
                                if let Some(ref cb) = error_callback {
                                    cb(StreamError::Io(e));
                                }
                            }
                            break;
                        }
                    }
                }

                tracing::info!("Stream reader stopped");
            })
            .expect("Failed to start stream reader thread");

        Self {
            running,
            handle: Some(handle),
        }
    }

    /// Stop the stream reader.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for StreamPipeReader {
    fn drop(&mut self) {
        self.stop();
    }
}
