// ========== Fragment shader ==========
@group(1) @binding(0)
var texture_sampler: sampler;

@group(1) @binding(1) var texture_array_16: texture_2d_array<f32>;
@group(1) @binding(2) var texture_array_32: texture_2d_array<f32>;
@group(1) @binding(3) var texture_array_64: texture_2d_array<f32>;
@group(1) @binding(4) var texture_array_128: texture_2d_array<f32>;
@group(1) @binding(5) var texture_array_256: texture_2d_array<f32>;
@group(1) @binding(6) var texture_array_512: texture_2d_array<f32>;
@group(1) @binding(7) var texture_array_1024: texture_2d_array<f32>;
@group(1) @binding(8) var texture_array_2048: texture_2d_array<f32>;
@group(1) @binding(9) var texture_array_4096: texture_2d_array<f32>;
@group(1) @binding(10) var texture_array_8192: texture_2d_array<f32>;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    switch vertex.texture_array_index {
        case 0u: { return textureSample(texture_array_16, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 1u: { return textureSample(texture_array_32, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 2u: { return textureSample(texture_array_64, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 3u: { return textureSample(texture_array_128, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 4u: { return textureSample(texture_array_256, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 5u: { return textureSample(texture_array_512, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 6u: { return textureSample(texture_array_1024, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 7u: { return textureSample(texture_array_2048, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 8u: { return textureSample(texture_array_4096, texture_sampler, vertex.uv, vertex.texture_layer); }
        case 9u: { return textureSample(texture_array_8192, texture_sampler, vertex.uv, vertex.texture_layer); }
        default: { return vec4<f32>(1.0, 1.0, 1.0, 1.0); }
    }
}
