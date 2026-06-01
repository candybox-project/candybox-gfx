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


use crate::meshes::InstanceData;


// ========== Render pipelines ==========
pub struct Pipelines {
    pub rectangle: Pipeline<crate::meshes::rectangle::Rectangle>,
    // triangle: Pipeline<crate::meshes::triangle::Triangle>
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        materials_bind_group_layout: &wgpu::BindGroupLayout
    ) -> Self {
        let rectangle = Pipeline::new(
            device,
            surface_config,
            camera_bind_group_layout,
            materials_bind_group_layout
        );

        // let triangle = Pipeline::new(
        //     device,
        //     surface_config,
        //     camera_bind_group_layout,
        //     materials_bind_group_layout
        // );

        Self {
            rectangle,
            // triangle
        }
    }
}

// ========== Render pipeline ==========
pub struct Pipeline<T: InstanceData> {
    pub render_pipeline: wgpu::RenderPipeline,
    pub shader_module: wgpu::ShaderModule,

    _marker: std::marker::PhantomData<T>
}

impl<T: InstanceData> Pipeline<T> {
    fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &wgpu::BindGroupLayout,
        materials_bind_group_layout: &wgpu::BindGroupLayout
    ) -> Self {
        let shader_source = format!("{}\n{}", T::vertex_shader(), T::fragment_shader());
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(shader_source.into())
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                Some(camera_bind_group_layout),
                Some(materials_bind_group_layout)
            ],
            immediate_size: 0
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                buffers: &[
                    crate::meshes::Vertex::vertex_buffer_layout(),
                    T::vertex_buffer_layout()
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default()
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview_mask: None,
            cache: None
        });

        Self {
            render_pipeline,
            shader_module,
            _marker: std::marker::PhantomData
        }
    }
}
