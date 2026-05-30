// modules, that define a mesh type, and are used as an instance of that mesh in the graphics core
pub mod rectangle;
pub mod triangle;
pub mod circle;
// if you're a developer, don't forget to add the new mesh type to the graphics core :)


// a common structure for all geometry types, used to define vertices and in the vertex shader  
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    uv: [f32; 2]
}

impl Vertex {
    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2
                }
            ]
        }
    }
}

pub trait InstanceData: bytemuck::Pod + bytemuck::Zeroable {
    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static>;
    fn vertices() -> Vec<Vertex>;
    fn indices() -> Vec<u16>;
    fn vertex_shader() -> &'static str;
    fn fragment_shader() -> &'static str;
}
