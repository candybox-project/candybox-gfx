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


use super::{InstanceData, Vertex};
use crate::{include_fragment_shader, include_vertex_shader, materials::MaterialHandle}; 

// ========== Rectangle ========== 
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rectangle {
    pub position: [f32; 3],
    pub scale: [f32; 2],
    pub texture_array_index: u32,
    pub texture_layer: u32
}

impl Rectangle {
    pub fn new(x: f32, y: f32, z: f32, width: f32, height: f32) -> Self {
        Self {
            position: [x, y, z],
            scale: [width, height],
            texture_array_index: 0,
            texture_layer: 0
        }
    }

    pub fn set_material(&mut self, material_handle: MaterialHandle) {
        self.texture_array_index = material_handle.texture_array_index;
        self.texture_layer = material_handle.texture_layer;
    }
}

impl InstanceData for Rectangle {
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
        include_vertex_shader!("rectangle.wgsl")
    }

    fn fragment_shader() -> &'static str {
        include_fragment_shader!("shader.wgsl")
    }
}
