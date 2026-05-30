use super::{InstanceData, Vertex};
use crate::{include_vertex_shader, include_fragment_shader}; 


// ========== Triangle ==========
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    pub position: [f32; 3],
    pub scale: [f32; 2],
    pub texture_array_index: u32,
    pub texture_layer: u32
}

impl Triangle {
    pub fn new(
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        texture_array_index: u32,
        texture_layer: u32
    ) -> Self {
        Self {
            position: [x, y, z],
            scale: [width, height],
            texture_array_index,
            texture_layer
        }
    }
}

impl InstanceData for Triangle {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // position: vec3<f32>
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3
                },
                // scale: vec2<f32>
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2
                },
                // texture_array_index: u32
                wgpu::VertexAttribute {
                    offset: 20,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Uint32
                },
                // texture_layer: u32
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32
                }
            ]
        }
    }

    fn vertices() -> Vec<Vertex> {
        vec![
            Vertex { position: [-1.0, -1.0], uv: [0.0, 1.0] },
            Vertex { position: [1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [1.0, 1.0] , uv: [1.0, 0.0] },
            Vertex { position: [-1.0, 1.0], uv: [0.0, 0.0] }
        ]
    }

    fn indices() -> Vec<u16> {
        vec![
            0, 1, 2,
            0, 2, 3
        ]
    }

    fn vertex_shader() -> &'static str {
        include_vertex_shader!("triangle.wgsl")
    }

    fn fragment_shader() -> &'static str {
        include_fragment_shader!("shader.wgsl")
    }
}
