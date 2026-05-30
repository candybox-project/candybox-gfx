use tokio::sync::oneshot;
use winit::keyboard::KeyCode;
use std::sync::{Arc, mpsc};

use crate::{KERNEL_PANIC, core::camera::view_projection::CameraViewProjection};


mod settings;
mod view_projection;
mod controller;
mod resources;


// ========== Camera ========== 
pub struct Camera { tx: mpsc::Sender<Commands> }

impl Camera {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_config: wgpu::SurfaceConfiguration) -> Self {
        let (tx, rx) = mpsc::channel();

        let settings = settings::CameraSettings {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: surface_config.width as f32 / surface_config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0
        };

        let mut view_projection = view_projection::CameraViewProjection::new();
        view_projection.update_view_proj(&settings);

        let controller = controller::CameraController::new(0.01);

        let resources = resources::Resources::new(&device, view_projection);

        let mut camera_system = CameraSystem {
            rx,
            device,
            queue,
            surface_config,
            settings,
            view_projection,
            controller,
            resources
        };

        std::thread::spawn(move || {
            camera_system.run();
        });

        Self { tx }
    }

    pub fn update(&self) {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::Update { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} camera system is crashed!", KERNEL_PANIC).as_str())
    }
    
    pub fn handle_key(&self, code: KeyCode, is_pressed: bool) -> bool {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::HandleKey { code, is_pressed, response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} camera system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn view_proj(&self) -> view_projection::CameraViewProjection {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetViewProj { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} camera system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn uniform_buffer(&self) -> Arc<wgpu::Buffer> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetUniformBuffer { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} camera system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn bind_group_layout(&self) -> Arc<wgpu::BindGroupLayout> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetBindGroupLayout { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} camera system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn bind_group(&self) -> Arc<wgpu::BindGroup> {
        let (tx, rx) = oneshot::channel();
        
        self.tx.send(Commands::GetBindGroup { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} camera system is crashed!", KERNEL_PANIC).as_str())
    }
}

// ========== Camera System ==========
struct CameraSystem {
    // receiver
    rx: mpsc::Receiver<Commands>,

    // graphic context
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,

    // camera settings
    settings: settings::CameraSettings,
    
    // inverse projection matrix
    view_projection: view_projection::CameraViewProjection,
    
    // camera controller
    controller: controller::CameraController,

    // graphic resources
    resources: resources::Resources
}

impl CameraSystem {
    fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Commands::HandleKey { code, is_pressed, response }) => {
                    response.send(self.controller.handle_key(code, is_pressed)).ok();
                },
                Ok(Commands::Update { response }) => {
                    self.controller.update_camera(&mut self.settings);
                    self.view_projection.update_view_proj(&self.settings);
                    self.queue.write_buffer(
                        &self.resources.uniform_buffer,
                        0,
                        bytemuck::cast_slice(&[self.view_projection])    
                    );
                    response.send(()).ok();
                },
                Ok(Commands::GetViewProj { response }) => {
                    response.send(self.view_projection).ok();
                },
                Ok(Commands::GetUniformBuffer { response }) => {
                    response.send(self.resources.uniform_buffer.clone()).ok();
                },
                Ok(Commands::GetBindGroupLayout { response }) => {
                    response.send(self.resources.bind_group_layout.clone()).ok();
                },
                Ok(Commands::GetBindGroup { response }) => {
                    response.send(self.resources.bind_group.clone()).ok();
                }
                Err(_) => break
            }
        }
    }
}

// ========== System Commands ==========
enum Commands {
    HandleKey { code: KeyCode, is_pressed: bool, response: oneshot::Sender<bool> },
    Update { response: oneshot::Sender<()> },
    GetViewProj { response: oneshot::Sender<CameraViewProjection> },
    GetUniformBuffer { response: oneshot::Sender<Arc<wgpu::Buffer>> },
    GetBindGroupLayout { response: oneshot::Sender<Arc<wgpu::BindGroupLayout>> },
    GetBindGroup { response: oneshot::Sender<Arc<wgpu::BindGroup>> }
}
