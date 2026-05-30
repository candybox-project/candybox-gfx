use std::collections::HashMap;
use crate::KERNEL_PANIC;
use crate::core::materials::textures::TextureArraySize;
use thiserror::Error;


// мемори граф хранит ноды и мапиинг по токену. методы алокации есть как в самой ноде, так и в
// графе(что бы был маппинг, и два токены не были алоцированы в двух разных нодах). граф так же
// дает методы управления нодами(удалить, добавить), потому что resources тоже имеет эти методы


// ========== Memory Node ==========
struct MemoryNode {
    pub capacity: usize,
    pub len: usize,
    pub free_layers: Vec<usize>
}

// ========== Memory Graph ========== 
pub struct MemoryGraph {
    // memory segment token → (TextureArraySize, layer)
    allocations: HashMap<usize, (TextureArraySize, usize)>,

    memory_nodes: HashMap<TextureArraySize, MemoryNode>
}

impl MemoryGraph {
    const MAX_LAYERS: usize = 256;

    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            memory_nodes: HashMap::new()
        }
    }

    pub fn allocate_memory(
        &mut self,
        memory_segment_token: usize,
        texture_array_size: &TextureArraySize
    ) -> Result<GraphEvent, GraphError> {
        if self.allocations.contains_key(&memory_segment_token) {
            return Err(GraphError::MemoryOccupied { memory_segment_token })
        }

        let memory_node = match self.memory_nodes.get_mut(texture_array_size) {
            Some(node) => node,
            None => return Err(GraphError::TextureArrayNotFound)
        };

        if memory_node.len >= memory_node.capacity && memory_node.free_layers.is_empty() {
            return Err(GraphError::OutOfMemmory)
        };

        let texture_layer = if let Some(layer) = memory_node.free_layers.pop() {
            layer
        } else {
            let layer = memory_node.len;
            memory_node.len += 1;
            layer
        };

        self.allocations.insert(memory_segment_token, (*texture_array_size, texture_layer));

        Ok(GraphEvent::MemoryAllocated { texture_layer })
    }

    pub fn deallocate_memory(&mut self, memory_segment_token: usize) -> Result<GraphEvent, GraphError> {
        let (texture_array_size, texture_layer) = match self.allocations.remove(&memory_segment_token) {
            Some((array_size, layer)) => (array_size, layer),
            None => return Err(GraphError::MemorySegmentNotFound { memory_segment_token })
        };
        
        let memory_node = self.memory_nodes.get_mut(&texture_array_size).unwrap();

        memory_node.free_layers.push(texture_layer);

        Ok(GraphEvent::MemoryDeallocated { texture_array_size, texture_layer })
    }

    pub fn lookup_memory_segment(&self, memory_segment_token: usize) -> Result<GraphEvent, GraphError> {
        match self.allocations.get(&memory_segment_token) {
            Some((texture_array_size, texture_layer)) => {
                Ok(GraphEvent::MemorySegmentFound {
                    texture_array_size: *texture_array_size,
                    texture_layer: *texture_layer
                })
            },
            None => Err(GraphError::MemorySegmentNotFound { memory_segment_token })
        }
    }

    pub fn expand_memory(&mut self, texture_array_size: &TextureArraySize) -> Result<(), GraphError> {
        if self.memory_nodes.contains_key(texture_array_size) {
            return Err(GraphError::TextureArrayAlreadyExists)
        }

        let memory_node = MemoryNode {
            capacity: 8,
            len: 0,
            free_layers: Vec::new()
        };
    
        self.memory_nodes.insert(*texture_array_size, memory_node);
        
        Ok(())
    }

    pub fn drop_memory(&mut self, texture_array_size: &TextureArraySize) -> Result<(), GraphError> {
        match self.memory_nodes.remove(texture_array_size) {
            Some(_) => Ok(()),
            None => Err(GraphError::TextureArrayNotFound)
        }
    }

    pub fn scale_memory(&mut self, texture_array_size: &TextureArraySize) -> Result<(), GraphError> {
        match self.memory_nodes.get_mut(texture_array_size) {
            Some(memory_node) => {
                if memory_node.capacity >= Self::MAX_LAYERS {
                    return Err(GraphError::OutOfGPUMemory)
                }

                memory_node.capacity *= 2;
                Ok(())
            },
            None => Err(GraphError::TextureArrayNotFound)
        }
    }

    pub fn can_drop_node(&self, texture_array_size: &TextureArraySize) -> Result<bool, GraphError> {
        let memory_node = match self.memory_nodes.get(texture_array_size) {
            Some(node) => node,
            None => return Err(GraphError::TextureArrayNotFound)
        };

        let active_layers = memory_node.len - memory_node.free_layers.len();
        
        if active_layers == 0 { Ok(true) } else { Ok(false) }
    }
}

// ========== Event types ==========
pub enum GraphEvent {
    MemoryAllocated {
        texture_layer: usize 
    },

    MemoryDeallocated {
        texture_array_size: TextureArraySize,
        texture_layer: usize
    },

    MemorySegmentFound {
        texture_array_size: TextureArraySize,
        texture_layer: usize
    }
}

// ========== Error types ==========
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("out of memory! memory for texture array is occupied and needs to be scaling!")]
    OutOfMemmory,

    #[error("out of memory! the GPU's maximum image count limit has been reached!")]
    OutOfGPUMemory,

    #[error("segmentation fault! memory with segment token: {} is occupied!", .memory_segment_token)]
    MemoryOccupied { memory_segment_token: usize },

    #[error("segmentation fault! memory segment with token: {} not found!", .memory_segment_token)]
    MemorySegmentNotFound { memory_segment_token: usize },
    
    #[error("segmentation fault! texture array not found!")]
    TextureArrayNotFound,

    #[error("memory occupied! texture array is alredy exists!")]
    TextureArrayAlreadyExists
}
