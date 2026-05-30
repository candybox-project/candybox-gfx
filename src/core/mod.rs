use std::{collections::HashMap, ops::{Deref, Index}, sync::Arc, u32};

use itertools::izip;
use thiserror::Error;
use wgpu::CurrentSurfaceTexture;
use winit::{event_loop::ActiveEventLoop, keyboard::KeyCode, platform::pump_events};
use crate::{ERR, KERNEL_PANIC, OK, meshes::InstanceData};


mod context;
mod camera;
mod materials;
mod meshes;
mod pipelines;


// ========== Graphics core ==========
pub struct Core {
    context: context::Context,
    
    pub camera: camera::Camera,
    pub meshes: meshes::Meshes,
    pub materials: materials::Materials,
    
    pipelines: pipelines::Pipelines
}

impl Core {
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        let context = pollster::block_on(context::Context::new(window)).unwrap_or_else(|error| {
            panic!("{} failed to create graphic context: {}", KERNEL_PANIC, error);
        });

        let camera = camera::Camera::new(
            context.device.clone(),
            context.queue.clone(),
            context.surface_config.clone()
        );

        let meshes = meshes::Meshes::new(
            context.device.clone(),
            context.queue.clone(),
            context.surface_config.clone()
        );

        let materials = materials::Materials::new(
            context.device.clone(),
            context.queue.clone(),
            context.surface_config.clone()
        );

        let pipelines = pipelines::Pipelines::new(
            &context.device,
            &context.surface_config,
            camera.bind_group_layout().as_ref(),
            materials.bind_group_layout().as_ref()
        );

        Self {
            context,
            camera,
            meshes,
            materials,
            pipelines
        }
    }
   
    pub fn render_frame(&self) -> Result<(), SurfaceError> {
        if !self.context.is_surface_configurated { return Err(SurfaceError::NotConfigured); }

        self.context.window.request_redraw();
        
        let mut needs_resize = false;

        let surface_texture = match self.context.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture) => texture,
            wgpu::CurrentSurfaceTexture::Timeout => return Err(SurfaceError::Timeout),
            wgpu::CurrentSurfaceTexture::Outdated => return Err(SurfaceError::Outdated),
            wgpu::CurrentSurfaceTexture::Lost => return Err(SurfaceError::Lost),
            wgpu::CurrentSurfaceTexture::Occluded => return Err(SurfaceError::Occluded),
            wgpu::CurrentSurfaceTexture::Validation => panic!("{} surface texture validation error", KERNEL_PANIC),
            wgpu::CurrentSurfaceTexture::Suboptimal(texture) => {
                needs_resize = true;
                texture
            }
        };

        let view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.02121901,
                            g: 0.023153365,
                            b: 0.036889445,
                            a: 1.0
                        }),
                        store: wgpu::StoreOp::Store
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None
            });

            let camera_bind_group = self.camera.bind_group();
            let materials_bind_group = self.materials.bind_group();

            render_pass.set_bind_group(0, camera_bind_group.as_ref(), &[]);
            
            if let Some(bind_group) = materials_bind_group {
                render_pass.set_bind_group(1, bind_group.as_ref(), &[]);
            }

            self.render_mesh(&mut render_pass, &self.pipelines.rectangle.render_pipeline, &self.meshes.rectangle);
        }

        self.context.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        if needs_resize { Err(SurfaceError::Suboptimal) } else { Ok(()) }
    }

    fn render_mesh<T: InstanceData + std::marker::Send>(
        &self,
        render_pass: &mut wgpu::RenderPass,
        pipeline: &wgpu::RenderPipeline,
        mesh: &meshes::Mesh<T>,
    ) {
        let vertex_buffer = mesh.vertex_buffer();
        let num_vertices = mesh.num_vertices();
        
        let index_buffer = mesh.index_buffer();
        let num_indices = mesh.num_indices();
        
        let instance_buffers = mesh.instance_buffers();
        let nums_instances = mesh.nums_instances();
        
        if nums_instances.is_empty() { return; }

        render_pass.set_pipeline(pipeline);

        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        for (instance_buffer, num_instances) in izip!(instance_buffers, nums_instances) {
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
            render_pass.draw_indexed(0..num_indices, 0, 0..num_instances);
        }
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }

        self.context.surface_config.width = width;
        self.context.surface_config.height = height;
        self.context.surface.configure(&self.context.device, &self.context.surface_config);
        self.context.is_surface_configurated = true;
    }

    pub fn reconfigure_surface(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }

        let surface_caps = self.context.surface.get_capabilities(&self.context.adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: width,
            height: height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[1],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };
        
        self.context.surface_config = surface_config;
        self.context.surface.configure(&self.context.device, &self.context.surface_config);
    }

    pub fn handle_key(&mut self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        if code == KeyCode::Escape && is_pressed {
            event_loop.exit();
        } else {
            self.camera.handle_key(code, is_pressed);
        }
    }

    pub fn update(&mut self) {
        self.camera.update();
    }
}

// ========== Error types ==========
#[derive(Error, Debug)]
pub enum SurfaceError {
    #[error("surface not configured!")]
    NotConfigured,

    #[error("surface timeout!")]
    Timeout,

    #[error("surface outdated!")]
    Outdated,

    #[error("surface lost!")]
    Lost,

    #[error("surface occluded!")]
    Occluded,

    #[error("surface suboptimal!")]
    Suboptimal
}

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("missing graphic context")]
    MissingContext,

    #[error("graphic context error: {0}")]
    ContextError(#[from] context::ContextError),

    #[error("failed to get frame buffer: {0}")]
    GetFrameBufferError(#[from] wgpu::CreateSurfaceError)
}
