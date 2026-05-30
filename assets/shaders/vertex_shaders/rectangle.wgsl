// ========== Rectangle vertex shader ========== 
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>
};

struct InstanceInput {
    @location(2) position: vec3<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) texture_array_index: u32,
    @location(5) texture_layer: u32
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) texture_array_index: u32,
    @location(2) @interpolate(flat) texture_layer: u32
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var output: VertexOutput;

    let offset_x = vertex.position.x * instance.scale.x;
    let offset_y = vertex.position.y * instance.scale.y;
 
    let world_position = vec3<f32>(
        instance.position.x + offset_x,
        instance.position.y + offset_y,
        instance.position.z
    );

    output.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0); 
    output.uv = vertex.uv;
    output.texture_array_index = instance.texture_array_index;
    output.texture_layer = instance.texture_layer;
    
    return output;
}
