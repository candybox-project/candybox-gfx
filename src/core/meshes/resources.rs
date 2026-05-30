use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::DeviceExt;
use crate::KERNEL_PANIC;
use crate::core::meshes::{InstanceData, buffers::{BufferError, BufferUUID}};


// ========== Graphic resources ==========
pub struct Resources<T: InstanceData> {
    // mesh defining buffers
    pub vertex_buffer: Arc<wgpu::Buffer>,
    pub index_buffer: Arc<wgpu::Buffer>,
    pub num_vertices: u32,
    pub num_indices: u32,

    // mesh instance buffers
    pub instance_buffers: Vec<Arc<wgpu::Buffer>>,
    pub instance_buffers_indices: HashMap<BufferUUID, usize>,

    _marker: std::marker::PhantomData<T>
}

impl<T: InstanceData> Resources<T> {
    pub fn new(device: &wgpu::Device) -> Self {
        let vertices = T::vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX
        });

        let indices = T::indices();
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX
        });

        Self {
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            num_vertices: vertices.len() as u32,
            num_indices: indices.len() as u32,
            instance_buffers_indices: HashMap::new(),
            instance_buffers: Vec::new(),
            _marker: std::marker::PhantomData
        }
    }

    pub fn add_instance_buffer(&mut self, device: &wgpu::Device, size: u64) -> Result<BufferUUID, BufferError> {
        let uuid = BufferUUID::new();
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let index = self.instance_buffers.len();
        self.instance_buffers.push(Arc::new(instance_buffer));
        self.instance_buffers_indices.insert(uuid, index);
        
        Ok(uuid)
    }

    pub fn update_instance_buffer(
        &mut self,
        queue: &wgpu::Queue,
        uuid: &BufferUUID,
        offset: u64,
        instance_data: T
    ) -> Result<(), BufferError> {
        if let Some(index) = self.instance_buffers_indices.get(uuid) {
            let instance_buffer = self.instance_buffers.get(*index).expect(
                format!("{} instance buffer: {} not found by index: {}!", KERNEL_PANIC, uuid, index).as_str()
            );

            let data_size = std::mem::size_of::<T>() as u64;

            if offset + data_size > instance_buffer.size() {
                return Err(BufferError::OutOfMemory {
                    data_size,
                    buffer_size: instance_buffer.size(),
                });
            }

            queue.write_buffer(
                instance_buffer,
                offset,
                bytemuck::cast_slice(&[instance_data])
            );

            Ok(())
        } else { Err(BufferError::BufferNotFound { uuid: *uuid }) }
    }

    pub fn delete_instance_buffer(&mut self, uuid: &BufferUUID) -> Result<(), BufferError> {
        if let Some(index) = self.instance_buffers_indices.remove(uuid) {
            self.instance_buffers.remove(index);
            
            // updating shifted indices
            for (_, idx) in self.instance_buffers_indices.iter_mut() {
                if *idx > index {
                    *idx -= 1;
                } 
            } 

            Ok(())
        } else { Err(BufferError::BufferNotFound { uuid: *uuid }) }
    }
}
