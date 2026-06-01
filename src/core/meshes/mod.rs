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
use std::sync::{Arc, mpsc};
use thiserror::Error;
use tokio::sync::oneshot;
use crate::meshes::InstanceData;
use crate::{INFO, KERNEL_PANIC, OK};
use crate::core::meshes::{
    buffers::{BufferError, BufferUUID},
    memory_heap::{HeapError, HeapEvent}
};


mod resources;
mod buffers;
mod memory_heap;


// ========== Meshes ==========
pub struct Meshes {
    pub rectangle: Mesh<crate::meshes::rectangle::Rectangle>,
    pub triangle: Mesh<crate::meshes::triangle::Triangle>
}

impl Meshes {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_config: wgpu::SurfaceConfiguration) -> Self {
        let rectangle = Mesh::new(device.clone(), queue.clone(), surface_config.clone());
        let triangle = Mesh::new(device.clone(), queue.clone(), surface_config.clone());

        Self {
            rectangle,
            triangle
        }
    }
}

// ========== Mesh Builder ==========
pub struct Mesh<T: InstanceData> {
    tx: mpsc::Sender<Commands<T>>,
    _marker: std::marker::PhantomData<T>
}

impl<T: InstanceData + std::marker::Send> Mesh<T> {
    fn new(device: wgpu::Device, queue: wgpu::Queue, surface_config: wgpu::SurfaceConfiguration) -> Self {
        let (tx, rx) = mpsc::channel();

        let resources = resources::Resources::new(&device);

        let mut mesh_system = MeshSystem {
            rx,
            memory_heap: memory_heap::MemoryHeap::new(),
            cow_data: HashMap::new(),
            device,
            queue,
            surface_config,
            resources
        };
        
        std::thread::spawn(move || {
            mesh_system.run();
        });

        Self {
            tx,
            _marker: std::marker::PhantomData
        }
    }

    pub fn add_mesh(&self, mesh_uuid: usize, data: T) -> Result<(), MeshError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::AddMesh { mesh_uuid, data, response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn update_mesh(&self, mesh_uuid: usize, data: T) -> Result<(), MeshError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::UpdateMesh { mesh_uuid, data, response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn delete_mesh(&self, mesh_uuid: usize) -> Result<(), MeshError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::DeleteMesh { mesh_uuid, response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn vertex_buffer(&self) -> Arc<wgpu::Buffer> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetVertexBuffer { response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn index_buffer(&self) -> Arc<wgpu::Buffer> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetIndexBuffer { response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn instance_buffers(&self) -> Vec<Arc<wgpu::Buffer>> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetInstanceBuffers { response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn num_vertices(&self) -> u32 {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::GetNumVertices { response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn num_indices(&self) -> u32 {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::GetNumIndices { response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn nums_instances(&self) -> Vec<u32> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::GetNumsInstancs { response: tx });
        rx.blocking_recv()
            .expect(format!("{} mesh system is crashed!", KERNEL_PANIC).as_str())
    }
}


// ========== Mesh System ==========
struct MeshSystem<T: InstanceData> {
    rx: mpsc::Receiver<Commands<T>>,

    // memory heap
    memory_heap: memory_heap::MemoryHeap,

    // Copy-on-Write data: node uuid -> instance uuid -> (data, index in the buffer) 
    cow_data: HashMap<BufferUUID, HashMap<usize, (T, usize)>>,

    // graphic context
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,

    // graphic resources
    resources: resources::Resources<T>
}

impl<T: InstanceData> MeshSystem<T> {
    fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Commands::AddMesh { mesh_uuid, data, response }) => {
                    response.send(self.add_mesh(mesh_uuid, data)).ok();
                },
                Ok(Commands::UpdateMesh { mesh_uuid, data, response }) => {
                    response.send(self.update_mesh(mesh_uuid, data)).ok();
                },
                Ok(Commands::DeleteMesh { mesh_uuid, response }) => {
                    response.send(self.delete_mesh(mesh_uuid)).ok();
                },
                Ok(Commands::GetVertexBuffer { response }) => {
                    response.send(self.resources.vertex_buffer.clone()).ok();
                },
                Ok(Commands::GetIndexBuffer { response }) => {
                    response.send(self.resources.index_buffer.clone()).ok();
                },
                Ok(Commands::GetInstanceBuffers { response }) => {
                    response.send(self.resources.instance_buffers.clone()).ok();
                },
                Ok(Commands::GetNumVertices { response }) => {
                    response.send(self.resources.num_vertices).ok();
                },
                Ok(Commands::GetNumIndices { response }) => {
                    response.send(self.resources.num_indices).ok();
                },
                Ok(Commands::GetNumsInstancs { response }) => {
                    response.send(self.memory_heap.get_nums_instances()).ok();
                }
                Err(_) => break
            }
        }
    }

    fn add_mesh(&mut self, mesh_uuid: usize, data: T) -> Result<(), MeshError> {
        loop {
            match self.memory_heap.allocate_memory(mesh_uuid) {
                Ok(event) => {
                    match event {
                        HeapEvent::MemoryAllocated { buffer_uuid, buffer_index } => {
                            let offset = (buffer_index * std::mem::size_of::<T>()) as u64;
                            match self.resources.update_instance_buffer(&self.queue, &buffer_uuid, offset, data) {
                                Ok(()) => println!("{} mesh added successfulyy!", OK),
                                Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                            };

                            self.cow_data
                                .entry(buffer_uuid)
                                .or_insert_with(HashMap::new)
                                .insert(mesh_uuid, (data, buffer_index));

                            return Ok(());
                        },
                        _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                    }
                },
                Err(error) => {
                    match error {
                        HeapError::OutOfMemmory => {
                            if self.memory_heap.need_memory_scaling() {
                                self.scale_memory().unwrap_or_else(|error| {
                                    panic!("{} {}", KERNEL_PANIC, error);
                                });
                            } else {
                                self.expand_memory().unwrap_or_else(|error| {
                                    panic!("{} {}", KERNEL_PANIC, error);
                                });
                            }
                        },
                        HeapError::MemoryOccupied { memory_segment_token } => {
                            return Err(MeshError::MemoryOccupied { mesh_uuid: memory_segment_token });
                        },
                        _ => panic!("{} {}", KERNEL_PANIC, error)
                    }
                }
            }
        }
    }

    fn delete_mesh(&mut self, mesh_uuid: usize) -> Result<(), MeshError> {
        match self.memory_heap.deallocate_memory(mesh_uuid) {
            Ok(event) => {
                match event {
                    HeapEvent::MemoryDeallocated { buffer_uuid, buffer_index } => {
                        let data = T::zeroed();
                        let offset = (buffer_index * std::mem::size_of::<T>()) as u64;
                        match self.resources.update_instance_buffer(&self.queue, &buffer_uuid, offset, data) {
                            Ok(()) => println!("{} mesh deleted successfulyy!", OK),
                            Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                        };

                        self.cow_data
                            .get_mut(&buffer_uuid)
                            .expect(format!("{} Copy-on-Write data not found!", KERNEL_PANIC).as_str())
                            .remove(&mesh_uuid);

                        Ok(())
                    },
                    _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                }
            },
            Err(error) => {
                match error {
                    HeapError::MemorySegmentNotFound { memory_segment_token } => {
                        Err(MeshError::SegmentationFault { mesh_uuid: memory_segment_token })
                    },
                    _ => panic!("{} {}", KERNEL_PANIC, error)
                }
            }
        }
    }

    fn update_mesh(&mut self, mesh_uuid: usize, data: T) -> Result<(), MeshError> {
        match self.memory_heap.lookup_memory_segment(mesh_uuid) {
            Ok(event) => {
                match event {
                    HeapEvent::MemorySegmentFound { buffer_uuid, buffer_index } => {
                        let offset = (buffer_index * std::mem::size_of::<T>()) as u64;
                        match self.resources.update_instance_buffer(&self.queue, &buffer_uuid, offset, data) {
                            Ok(()) => println!("{} mesh updated successfulyy!", OK),
                            Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                        };

                        self.cow_data
                            .get_mut(&buffer_uuid)
                            .expect(format!("{} Copy-on-Write data not found!", KERNEL_PANIC).as_str())
                            .insert(mesh_uuid, (data, buffer_index));

                        Ok(())
                    },
                    _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                }
            },
            Err(error) => {
                match error {
                    HeapError::MemorySegmentNotFound { memory_segment_token } => {
                        Err(MeshError::SegmentationFault { mesh_uuid: memory_segment_token })
                    },
                    _ => panic!("{} {}", KERNEL_PANIC, error)
                }
            }
        }
    }

    fn expand_memory(&mut self) -> Result<(), BufferError> {
        println!("{} meshes: expanding memory ...", INFO);

        // calculating the size for a new buffer ... 
        let capacity = self.memory_heap.get_capacity();
        let buffer_size = (capacity * std::mem::size_of::<T>()) as u64;

        // adding a new buffer ...
        let buffer_uuid = self.resources.add_instance_buffer(&self.device, buffer_size)?;

        // expanding the memory heap ...
        self.memory_heap.expand_memory(&buffer_uuid).unwrap_or_else(|error| {
            panic!("{} {}", KERNEL_PANIC, error) // out of memory! scaling is required
        });

        println!("{} meshes: expanding completed successfully!", OK);
        Ok(())
    }

    fn scale_memory(&mut self) -> Result<(), BufferError> {
        println!("{} meshes: scaling memory ...", INFO);

        // scaling the memory heap ...
        self.memory_heap.scale_memory();

        // calculating the size for a new buffers ...
        let capacity = self.memory_heap.get_capacity();
        let size = (capacity * std::mem::size_of::<T>()) as u64;

        let mut new_cow_data = HashMap::new();

        for (old_uuid, node_data) in &self.cow_data {
            // memory allocation: creating a buffer with new size and writing old data ...
            let new_uuid = self.resources.add_instance_buffer(&self.device, size)?;
            for (instance_uuid, (data, index)) in node_data {
                let offset = (*index * std::mem::size_of::<T>()) as u64;
                self.resources.update_instance_buffer(&self.queue, &new_uuid, offset, *data);
            }

            // updating buffer uuid in the memory heap ...
            self.memory_heap.update_buffer_uuid(old_uuid, &new_uuid).unwrap_or_else(|error| {
                panic!("{} {}", KERNEL_PANIC, error); // memory occupied! node alredy exists 
            });

            // memory deallocation: deleting the old buffer ...
            self.resources.delete_instance_buffer(old_uuid)?;

            new_cow_data.insert(new_uuid, node_data.clone());
        }

        self.cow_data = new_cow_data;

        println!("{} meshes: scaling completed successfully!", OK);
        Ok(())
    }
}

// ========== System Commands ==========
enum Commands<T: InstanceData> {
    AddMesh { mesh_uuid: usize, data: T, response: oneshot::Sender<Result<(), MeshError>> },
    UpdateMesh { mesh_uuid: usize, data: T, response: oneshot::Sender<Result<(), MeshError>> },
    DeleteMesh { mesh_uuid: usize, response: oneshot::Sender<Result<(), MeshError>> },
    GetVertexBuffer { response: oneshot::Sender<Arc<wgpu::Buffer>> },
    GetIndexBuffer { response: oneshot::Sender<Arc<wgpu::Buffer>> },
    GetInstanceBuffers { response: oneshot::Sender<Vec<Arc<wgpu::Buffer>>> },
    GetNumVertices { response: oneshot::Sender<u32> },
    GetNumIndices { response: oneshot::Sender<u32> },
    GetNumsInstancs { response: oneshot::Sender<Vec<u32>> }
}

// ========== Error Types ========== 
#[derive(Error, Debug)]
pub enum MeshError {
    #[error("memory occupied! mesh: {} is alredy exists!", .mesh_uuid)]
    MemoryOccupied { mesh_uuid: usize },

    #[error("segmentation fault! mesh: {} not found!", .mesh_uuid)]
    SegmentationFault { mesh_uuid: usize }
}
