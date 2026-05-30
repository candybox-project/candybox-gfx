use std::sync::Arc;

use wgpu::util::DeviceExt;


// ========== Graphics Resources ==========
pub struct Resources {
    pub uniform_buffer: Arc<wgpu::Buffer>,
    pub bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub bind_group: Arc<wgpu::BindGroup>
}

impl Resources {
    pub fn new(device: &wgpu::Device, uniform_data: super::view_projection::CameraViewProjection) -> Self {
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[uniform_data]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None
                }
            ]
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding()
                }
            ]
        });

        Self {
            uniform_buffer: Arc::new(uniform_buffer),
            bind_group_layout: Arc::new(bind_group_layout),
            bind_group: Arc::new(bind_group)
        }
    }
}
