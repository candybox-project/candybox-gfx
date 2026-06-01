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


use std::collections::HashMap;
use thiserror::Error;
use crate::core::meshes::buffers::BufferUUID;


// ========== Memory heap ========== 
pub struct MemoryHeap {
    // these are buffers counters, they are needed to track scaling
    max_buffers_count: usize,
    buffers_count: usize,

    // the buffers capacity is fixed for all nodes
    buffers_capacity: usize,

    // buffers identifers
    buffers_uuid: Vec<BufferUUID>,
    
    // buffers occupancy
    buffers_len: Vec<usize>,

    // token of the memory segment -> index of memory segment in the buffer 
    segment_indices: Vec<HashMap<usize, usize>>,
    
    // free indices for memory segments in the buffer
    free_indices: Vec<Vec<usize>>
}

impl MemoryHeap {
     pub fn new() -> Self {
        Self {
            max_buffers_count: 8,
            buffers_count: 0,
            buffers_capacity: 8,
            buffers_uuid: Vec::new(),
            buffers_len: Vec::new(),
            segment_indices: Vec::new(),
            free_indices: Vec::new()
        }
    }

    // find and allocate memory for an segment
    pub fn allocate_memory(&mut self, memory_segment_token: usize) -> Result<HeapEvent, HeapError> {
        let soa_index = self.get_index_of_free_buffer()?;

        if self.segment_indices[soa_index].contains_key(&memory_segment_token) {
            return Err(HeapError::MemoryOccupied { memory_segment_token });
        }

        let buffer_index = if let Some(index) = self.free_indices[soa_index].pop() {
            index
        } else {
            self.segment_indices[soa_index].len()
        };
        
        self.segment_indices[soa_index].insert(memory_segment_token, buffer_index);
        self.buffers_len[soa_index] += 1;       

        Ok(HeapEvent::MemoryAllocated {
            buffer_uuid: self.buffers_uuid[soa_index],
            buffer_index
        })
    }

    // find and deallocate memory for an segment
    pub fn deallocate_memory(&mut self, memory_segment_token: usize) -> Result<HeapEvent, HeapError> {
        let soa_index = self.get_buffer_index_by_memory_segment_token(memory_segment_token)?; 

        // SAFETY unwrap(), because function "get_buffer_index_by_memory_segment_token" will return
        // an error if the buffer_index is not found (segmentation fault)
        let buffer_index = self.segment_indices[soa_index].remove(&memory_segment_token).unwrap();
        
        self.free_indices[soa_index].push(buffer_index);

        // WARNING! the value of the self.buffers_len[soa_index] sohuld not be decreased, since
        // this garbage collector keeps indexes stable for reuse!

        Ok(HeapEvent::MemoryDeallocated {
            buffer_uuid: self.buffers_uuid[soa_index],
            buffer_index
        })
    }

    // get memory segment by token
    pub fn lookup_memory_segment(&self, memory_segment_token: usize) -> Result<HeapEvent, HeapError> {
        let soa_index = self.get_buffer_index_by_memory_segment_token(memory_segment_token)?;

        // SAFETY unwrap(), because function "get_buffer_index_by_memory_segment_token" will return
        // an error if the buffer_index is not found (segmentation fault)
        let buffer_index = self.segment_indices[soa_index].get(&memory_segment_token).unwrap();

        Ok(HeapEvent::MemorySegmentFound{
            buffer_uuid: self.buffers_uuid[soa_index],
            buffer_index: *buffer_index
        })
    }

    // expand memory
    pub fn expand_memory(&mut self, buffer_uuid: &BufferUUID) -> Result<(), HeapError> {
        if self.need_memory_scaling() { return Err(HeapError::ScalingRequired); }

        self.buffers_count += 1;
        self.buffers_uuid.push(*buffer_uuid);
        self.buffers_len.push(0);
        self.segment_indices.push(HashMap::new());
        self.free_indices.push(Vec::new());
        
        Ok(())
    }

    // incrase the limit on the number of buffers and their capacity
    pub fn scale_memory(&mut self) {
        self.max_buffers_count *= 2;
        self.buffers_capacity *= 2;
    }
    
    // update outdated buffer uuid
    pub fn update_buffer_uuid(&mut self, old_uuid: &BufferUUID, new_uuid: &BufferUUID) -> Result<(), HeapError> {
        let soa_index = self.buffers_uuid.iter()
            .position(|uuid| uuid == old_uuid)
            .ok_or(HeapError::BufferNotFound { uuid: *old_uuid })?;
        
        if self.buffers_uuid.contains(new_uuid) { return Err(HeapError::BufferAlredyExists { uuid: *new_uuid }); }
        
        self.buffers_uuid[soa_index] = *new_uuid;
        
        Ok(())
    }
    
    // get index of the free buffer 
    fn get_index_of_free_buffer(&self) -> Result<usize, HeapError> {
        let mut found_index = None;
        for (index, buffer_len) in self.buffers_len.iter().enumerate() {
            if &self.buffers_capacity > buffer_len {
                found_index = Some(index);
                break;
            }
        }

        found_index.ok_or(HeapError::OutOfMemmory)
    }

    // get buffer index by memory segment token
    fn get_buffer_index_by_memory_segment_token(&self, memory_segment_token: usize) -> Result<usize, HeapError> {
        let mut found_index = None;
        for (index, segment_indices) in self.segment_indices.iter().enumerate() {
            if segment_indices.contains_key(&memory_segment_token) {
                found_index = Some(index);
                break;
            }
        }

        found_index.ok_or(HeapError::MemorySegmentNotFound { memory_segment_token })
    }
    
    // get the current buffers capacity 
    pub fn get_capacity(&self) -> usize { self.buffers_capacity.clone() }

     // check the need for memory scaling 
    pub fn need_memory_scaling(&self) -> bool {
        if self.buffers_count >= self.max_buffers_count { true } else { false }
    }

    // get the number of insances of all buffers 
    pub fn get_nums_instances(&self) -> Vec<u32> {
        let mut num_instances = Vec::with_capacity(self.buffers_len.len());

        for len in &self.buffers_len {
            num_instances.push(len.clone() as u32);
        }

        num_instances
    }   
}

// ========== Event types ==========
pub enum HeapEvent {
    MemoryAllocated {
        buffer_uuid: BufferUUID,
        buffer_index: usize 
    },

    MemoryDeallocated {
        buffer_uuid: BufferUUID,
        buffer_index: usize
    },

    MemorySegmentFound {
        buffer_uuid: BufferUUID,
        buffer_index: usize
    }
}

// ========== Error types ==========
#[derive(Error, Debug)]
pub enum HeapError {
    #[error("out of memory! all memory is occupied and needs to be expanded!")]
    OutOfMemmory,

    #[error("memory expansion failed! scaling is required, please, use the \"scale_memory\" function!")]
    ScalingRequired,

    #[error("segmentation fault! memory with segment token: {} is occupied!", .memory_segment_token)]
    MemoryOccupied { memory_segment_token: usize },

    #[error("segmentation fault! memory segment with token: {} not found!", .memory_segment_token)]
    MemorySegmentNotFound { memory_segment_token: usize },

    #[error("segmentation fault! buffer with uuid: {} not found!", .uuid)]
    BufferNotFound { uuid: BufferUUID },

    #[error("memory occupied! buffer with uuid: {} is alredy exists!", .uuid)]
    BufferAlredyExists { uuid: BufferUUID }
}
