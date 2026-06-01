// Candybox-GFX - High-performance graphics engine
//     Copyright (C) 2026  Candybox Project
//
//     This program is free software: you can redistribute it and/or modify
//     it under the terms of the GNU General Public License as published by
//     the Free Software Foundation, either version 3 of the License, or
//     (at your option) any later version.
//
//     This program is distributed in the hope that it will be useful,
//     but WITHOUT ANY WARRANTY; without even the implied warranty of
//     MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//     GNU General Public License for more details.
//
//     You should have received a copy of the GNU General Public License
//     along with this program.  If not, see <https://www.gnu.org/licenses/>.


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
