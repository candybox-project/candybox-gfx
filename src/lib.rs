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


mod core;
mod meshes;
mod materials;


use std::{ fmt::format, panic, sync::Arc };
use image::DynamicImage;
use tokio::sync::{ Mutex, mpsc };
use thiserror::Error;
use crate::{core::SurfaceError, materials::textures};
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{ KeyEvent, WindowEvent },
    event_loop::EventLoop,
    keyboard::{ KeyCode, PhysicalKey }
};


// FIXME это надо вынести в отдельную либу и написать методы для логирования (^"◕ᴗ◕"^)
pub const OK: &str = "\x1b[1;32m[ ok ]\x1b[0m";
pub const ERR: &str = "\x1b[1;33m[ err ]\x1b[0m";
pub const INFO: &str = "\x1b[1;36m[ info! ]\x1b[0m"; 
pub const KERNEL_PANIC: &str = "\x1b[1;31m[ kernel panic! ]\x1b[0m";

// FIXME и это тоже :)
#[macro_export]
macro_rules! include_vertex_shader {
    ($name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/vertex_shaders/",
            $name
        ))
    };
}

#[macro_export]
macro_rules! include_fragment_shader {
    ($name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/fragment_shaders/",
            $name
        ))
    };
}

#[macro_export]
macro_rules! include_compute_shader {
    ($name:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/compute_shaders/",
            $name
        ))
    };
}

#[macro_export]
macro_rules! include_shader {
    ($path:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/shaders/",
            $path
        ))
    };
}

#[macro_export]
macro_rules! include_image {
    ($path:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/textures/",
            $path
        ))
    };
}


// справка для доки! эту структурку пишет пользователь движка :)
// ========== App ==========
pub struct App {
    core: Option<core::Core>,
    window: Option<Arc<winit::window::Window>>,

    material_data: textures::Texture,
    material_data1: textures::Texture,
    last_uuid: usize,

    image: DynamicImage
}

impl App {
    pub fn new() -> Self {
        let diffuse_bytes = include_bytes!("../assets/textures/source_image_2.jpg");
        let image = image::load_from_memory(diffuse_bytes).unwrap();
        
        let material_data = match materials::textures::Texture::new(
            include_image!("source_image_2.jpg")
        ) {
            Ok(texture) => texture,
            Err(error) => panic!("{} {}", ERR, error)
        };

        let material_data1 = match materials::textures::Texture::new(
            include_image!("source_image_3.jpg")
        ) {
            Ok(texture) => texture,
            Err(error) => panic!("{} {}", ERR, error)
        };

        Self {
            core: None,
            window: None,
            last_uuid: 0,
            material_data,
            material_data1,
            image
        }
    }
    
    // сравка для доки! это boilerplate
    // launch app in the event loop
    pub fn run(&mut self) -> Result<(), AppError> {
        let event_loop = EventLoop::with_user_event().build()?;
        event_loop.run_app(self)?;   

        Ok(())
    }
}


// сравка для доки! это тоже boilerplate
impl ApplicationHandler for App {
    // restore the window's graphical state
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = winit::window::Window::default_attributes();

        let window = Arc::new(match event_loop.create_window(window_attributes) {
            Ok(wndw) => {
                println!("{} window was created successfully!", OK);
                wndw
            },
            Err(error) => panic!("{} {}", KERNEL_PANIC, error)
        });
        
        let core = core::Core::new(window.clone());
        self.core = Some(core);
        self.window = Some(window);
        
        println!("{} graphic context resumed!", OK);
    }
    
    // handling window events
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let core = match &mut self.core {
            Some(kernel) => { kernel },
            None => return
        };

        let window = match &self.window {
            Some(wndw) => wndw,
            None => return
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => core.resize_surface(size.width, size.height),
            WindowEvent::RedrawRequested => {
                core.update();
                match core.render_frame() {
                    Ok(()) => {},
                    Err(error) => {
                        match error {
                            SurfaceError::NotConfigured => println!("{} waiting for surface configuration ...", INFO),
                            SurfaceError::Timeout => println!("{} {}, skipping frame ...", INFO, error),
                            SurfaceError::Outdated => {
                                println!("{} surface outdated, resizing ...", INFO);
                                
                                let size = window.inner_size();
                                core.resize_surface(size.width, size.height);
                            },
                            SurfaceError::Lost => {
                                println!("{} surface lost, rebuilding ...", INFO);
                                
                                let size = window.inner_size();
                                core.reconfigure_surface(size.width, size.height);
                            },
                            SurfaceError::Suboptimal => {
                                println!("{} surface suboptimal, resizing ...", INFO);
                                
                                let size = window.inner_size();
                                core.resize_surface(size.width, size.height);
                            },
                            SurfaceError::Occluded => {} // we do nothing :) 
                        }
                    }
                }
            },
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => {
                core.handle_key(event_loop, code, key_state.is_pressed());
                match (code, key_state.is_pressed()) {
                    // тесты :)
                    (KeyCode::KeyZ, true) => {
                        // add material
                        match core.materials.add_material(0, self.material_data.image.clone()) {
                            Ok(()) => {},
                            Err(error) => println!("{} {}", ERR, error)
                        };
                    },
                    (KeyCode::KeyX, true) => {
                        // delete material
                        match core.materials.delete_material(0) {
                            Ok(()) => {},
                            Err(error) => println!("{} {}", ERR, error)
                        };
                    },
                    (KeyCode::KeyV, true) => {
                        // add another material
                        match core.materials.add_material(1, self.material_data1.image.clone()) {
                            Ok(()) => {},
                            Err(error) => println!("{} {}", ERR, error)
                        };
                    } 
                    (KeyCode::KeyQ, true) => {
                        // add mesh
                        let mut mesh_data = meshes::rectangle::Rectangle::new(
                            0.0,
                            0.0,
                            1.0,
                            1.0,
                            1.0,
                        );
                        
                        // match core.materials.add_material(0, self.material_data.image.clone()) {
                        //     Ok(()) => {},
                        //     Err(error) => println!("{} {}", ERR, error)
                        // };
 
                        let material_handle = match core.materials.get_material_handle(0) {
                            Ok(handle) => handle,
                            Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                        };

                        mesh_data.set_material(material_handle);
                        
                        match core.meshes.rectangle.add_mesh(self.last_uuid, mesh_data) {
                            Ok(()) => {},
                            Err(error) => println!("{} {}", ERR, error)
                        };
                        
                        self.last_uuid += 1;
                    },
                    (KeyCode::KeyW, true) => {
                        // update
                        let mut mesh_data = meshes::rectangle::Rectangle::new(
                            0.5,
                            0.0,
                            1.0,
                            1.0,
                            1.0,
                        );

                        let material_data = match materials::textures::Texture::new(
                            include_image!("source_image_1.jpg")
                        ) {
                            Ok(texture) => texture,
                            Err(error) => panic!("{} {}", ERR, error)
                        };

                        match core.materials.add_material(material_data.uuid, material_data.image) {
                            Ok(()) => {},
                            Err(error) => println!("{} {}", ERR, error)
                        };

                        let material_handle = core.materials.get_material_handle(material_data.uuid).unwrap();
                        mesh_data.set_material(material_handle);

                        match core.meshes.rectangle.update_mesh(1, mesh_data) {
                            Ok(()) => {},
                            Err(error) => println!("{} {}", ERR, error)
                        };
                    },
                    (KeyCode::KeyE, true) => {
                        // delete
                        
                        // match core.materials.delete_material(1) {
                        //     Ok(()) => {},
                        //     Err(error) => println!("{} {}", ERR, error)
                        // };
                        //
                        // match core.meshes.rectangle.delete_mesh(1) {
                        //     Ok(()) => {},
                        //     Err(error) => println!("{} {}", ERR, error)
                        // };
                    }
                    _ => {}
                }
            },
            _ => {}
        }
    }
}

// справка для доки! это перечисление тоже пишет пользователь движка, оно не входит в api :)
// ========== Error types ==========
#[derive(Error, Debug)]
pub enum AppError {
    #[error("event loop error: {0}")]
    EventLoopError(#[from] EventLoopError),
    
    #[error("kernel error: {0}")]
    KernelError(#[from] core::KernelError)
}
