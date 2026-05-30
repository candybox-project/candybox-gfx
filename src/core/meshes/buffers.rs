use std::sync::atomic::{AtomicUsize, Ordering};
use thiserror::Error;


// ========== Buffers's UUID generator ==========
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BufferUUID { id: usize }

impl BufferUUID {
    pub fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self { id: COUNTER.fetch_add(1, Ordering::SeqCst) }
    }
}

impl std::fmt::Display for BufferUUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UUID-{}", self.id)
    }
}

// ========== Error types ==========
#[derive(Error, Debug)]
pub enum BufferError {
    #[error("out of memory! data size: {}, buffer size: {}",.data_size, .buffer_size)]
    OutOfMemory {
        data_size: u64,
        buffer_size: u64,
    },

    #[error("segmentation fault! buffer: {} is alredy exists!", .uuid)]
    BufferAlredyExists { uuid: BufferUUID },
    
    #[error("segmentation fault! buffer: {} not found!", .uuid)]
    BufferNotFound { uuid: BufferUUID }
}
