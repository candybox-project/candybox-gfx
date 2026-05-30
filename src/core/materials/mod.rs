use std::collections::HashMap;
use std::sync::{Arc, mpsc};
use image::{DynamicImage, GenericImageView};
use thiserror::Error;
use tokio::sync::oneshot;
use wgpu::wgc::device::queue;
use crate::core::materials;
use crate::core::materials::memory_graph::{GraphError, GraphEvent};
use crate::core::materials::textures::{TextureArraySize, TextureError};
use crate::materials::MaterialHandle;
use crate::{INFO, KERNEL_PANIC, OK};


mod resources;
mod textures;
mod memory_graph;


// ========== Materials ==========
pub struct Materials {
    tx: mpsc::Sender<Commands>
}

impl Materials {
    pub fn new(device: wgpu::Device, queue: wgpu::Queue, surface_config: wgpu::SurfaceConfiguration) -> Self {
        let (tx, rx) = mpsc::channel();

        let resources = resources::Resources::new(&device);  

        let mut materials_system = MaterialsSystem {
            rx,
            memory_graph: memory_graph::MemoryGraph::new(),
            cow_data: HashMap::new(),
            device,
            queue,
            surface_config,
            resources
        };

        std::thread::spawn(move || {
            materials_system.run();
        });

        Self { tx }
    }

    pub fn add_material(&self, material_uuid: usize, image: DynamicImage) -> Result<(), MaterialError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::AddMaterial { material_uuid, image, response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} materials system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn update_material(&self, material_uuid: usize, image: DynamicImage) -> Result<(), MaterialError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::UpdateMaterial { material_uuid, image, response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} materials system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn delete_material(&self, material_uuid: usize) -> Result<(), MaterialError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::DeleteMaterial { material_uuid, response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} materials system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn get_material_handle(&self, material_uuid: usize) -> Result<MaterialHandle, MaterialError> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::GetMaterialHandle { material_uuid, response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} materials system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn bind_group_layout(&self) -> Arc<wgpu::BindGroupLayout> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::GetBindGroupLayout { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} materials system is crashed!", KERNEL_PANIC).as_str())
    }

    pub fn bind_group(&self) -> Option<Arc<wgpu::BindGroup>> {
        let (tx, rx) = oneshot::channel();

        self.tx.send(Commands::GetBindGroup { response: tx }).ok();
        rx.blocking_recv()
            .expect(format!("{} materials system is crashed!", KERNEL_PANIC).as_str())
    }
}

// ========== Materials System ==========
struct MaterialsSystem {
    rx: mpsc::Receiver<Commands>,

    // memory graph
    memory_graph: memory_graph::MemoryGraph,
    
    // Copy-on-Write data: TextureArraySize -> Vec<active layer number>
    cow_data: HashMap<TextureArraySize, Vec<usize>>,

    // grahpic context
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    
    // graphic resources
    resources: resources::Resources
}


impl MaterialsSystem {
    pub fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(Commands::AddMaterial { material_uuid, image, response }) => {
                    response.send(self.add_material(material_uuid, &image)).ok();
                },
                Ok(Commands::UpdateMaterial { material_uuid, image, response }) => {
                    response.send(self.update_material(material_uuid, &image)).ok();
                },
                Ok(Commands::DeleteMaterial { material_uuid, response }) => {
                    response.send(self.delete_material(material_uuid)).ok();
                },
                Ok(Commands::GetMaterialHandle { material_uuid, response }) => {
                    response.send(self.get_material_handle(material_uuid)).ok();
                },
                Ok(Commands::GetBindGroupLayout { response }) => {
                    response.send(self.resources.bind_group_layout.clone()).ok();
                },
                Ok(Commands::GetBindGroup { response }) => {
                    response.send(self.resources.bind_group.clone()).ok();
                },
                Err(_) => break
            }
        }
    }
    
    fn add_material(&mut self, material_uuid: usize, image: &DynamicImage) -> Result<(), MaterialError> {
        let dimensions = image.dimensions();
        let texture_array_size = TextureArraySize::from_dimensions(dimensions)?;

        loop {
            match self.memory_graph.allocate_memory(material_uuid, &texture_array_size) {
                Ok(event) => {
                    match event {
                        GraphEvent::MemoryAllocated { texture_layer } => {
                            let texture_array = self.resources.get_texture_array(&texture_array_size)
                                .unwrap_or_else(|error| {
                                    panic!("{} {}", KERNEL_PANIC, error);
                                });

                            match texture_array.load_image(&self.queue, texture_layer as u32, image) {
                                Ok(()) => println!("{} material added successfully!", OK),
                                Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                            };

                            self.cow_data
                                .entry(texture_array_size)
                                .or_insert_with(Vec::new)
                                .push(texture_layer);

                            return Ok(());
                        },
                        _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                    }
                },
                Err(error) => {
                    match error {
                        GraphError::TextureArrayNotFound => {
                            self.expand_memory(&texture_array_size)?;
                        },
                        GraphError::OutOfMemmory => {
                            self.scale_memory(&texture_array_size)?;
                        },
                        GraphError::MemoryOccupied { memory_segment_token } => {
                            return Err(MaterialError::MemoryOccupied { material_uuid: memory_segment_token });
                        },
                        _ => panic!("{} {}", KERNEL_PANIC, error)
                    }
                }
            }
        }
    }

    fn delete_material(&mut self, material_uuid: usize) -> Result<(), MaterialError> {
        match self.memory_graph.deallocate_memory(material_uuid) {
            Ok(event) => {
                match event {
                    GraphEvent::MemoryDeallocated { texture_array_size, texture_layer } => {
                        let texture_array = self.resources.get_texture_array(&texture_array_size)
                            .unwrap_or_else(|error| {
                                panic!("{} {}", KERNEL_PANIC, error);
                            });
                        
                        match texture_array.clear_image(&self.queue, texture_layer as u32) {
                            Ok(()) => println!("{} material deleted successfully!", OK),
                            Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                        };

                        if self.memory_graph.can_drop_node(&texture_array_size).unwrap() {
                            self.memory_graph.drop_memory(&texture_array_size);
                            self.resources.delete_texture_array(&self.device, &texture_array_size);
                        };

                        self.cow_data
                            .get_mut(&texture_array_size)
                            .expect(format!("{} Copy-on-Write data not found!", KERNEL_PANIC).as_str())
                            .retain(|&layer| layer != texture_layer);

                        Ok(())
                    },
                    _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                }
            },
            Err(error) => {
                match error {
                    GraphError::MemorySegmentNotFound { memory_segment_token } => {
                        Err(MaterialError::SegmentationFault { material_uuid })
                    },
                    _ => panic!("{} {}", KERNEL_PANIC, error)
                }
            }
        }
    }

    fn update_material(&mut self, material_uuid: usize, image: &DynamicImage) -> Result<(), MaterialError> {
        match self.memory_graph.lookup_memory_segment(material_uuid) {
            Ok(event) => {
                match event {
                    GraphEvent::MemorySegmentFound { texture_array_size, texture_layer } => {
                        // FIXME упадет, если размер новой текстурки не будет соответствовать
                        // старому :(
                        let texture_array = self.resources.get_texture_array(&texture_array_size)
                            .unwrap_or_else(|error| {
                                panic!("{} {}", KERNEL_PANIC, error);
                            });
                        
                        match texture_array.load_image(&self.queue, texture_layer as u32, image) {
                            Ok(()) => println!("{} material updated successfully!", OK),
                            Err(error) => panic!("{} {}", KERNEL_PANIC, error)
                        };

                        Ok(())
                    },
                    _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                }
            },
            Err(error) => {
                match error {
                    GraphError::MemorySegmentNotFound { memory_segment_token } => {
                        Err(MaterialError::SegmentationFault { material_uuid })
                    },
                    _ => panic!("{} {}", KERNEL_PANIC, error)
                }
            }
        }
    }

    fn get_material_handle(&self, material_uuid: usize) -> Result<MaterialHandle, MaterialError> {
        match self.memory_graph.lookup_memory_segment(material_uuid) {
            Ok(event) => {
                match event {
                    GraphEvent::MemorySegmentFound { texture_array_size, texture_layer } => {
                        let texture_array_index = texture_array_size.to_index();
                        Ok(MaterialHandle {
                            texture_array_index: texture_array_index as u32,
                            texture_layer: texture_layer as u32
                        })
                    },
                    _ => panic!("{} unknown event from garbage collector!", KERNEL_PANIC)
                }
            },
            Err(error) => {
                match error {
                    GraphError::MemorySegmentNotFound { memory_segment_token } => {
                        Err(MaterialError::SegmentationFault { material_uuid })
                    },
                    _ => panic!("{} {}", KERNEL_PANIC, error)
                }
            }
        }
    }

    // auxiliary methods (^"◕ᴗ◕"^)
    fn expand_memory(&mut self, texture_array_size: &TextureArraySize) -> Result<(), TextureError> {
        println!("{} materials: expanding memory ...", INFO);

        self.memory_graph.expand_memory(&texture_array_size).unwrap_or_else(|error| {
            panic!("{} {}", KERNEL_PANIC, error) // segmentation fault! texture array is alredy exists
        });
        self.resources.add_texture_array(&self.device, texture_array_size)?;

        println!("{} materials: expanding completed successfully!", OK);
        Ok(())
    }

    fn scale_memory(&mut self, texture_array_size: &TextureArraySize) -> Result<(), GraphError> {
        println!("{} materials: scaling memory ...", INFO);
        
        // scaling one of the texture arrays in the memory heap ...
        self.memory_graph.scale_memory(texture_array_size)?;
        
        // scaling one of the texture arrays ...
        let active_layers = self.cow_data.get(texture_array_size)
            .expect(format!("{} Copy-on-Write data not found!", KERNEL_PANIC).as_str());
        self.resources.scale_texture_array(&self.device, &self.queue, texture_array_size, active_layers.clone());       
        
        println!("{} materials: scaling completed successfully!", OK);
        Ok(())
    }
}

// ========== System Commands ==========
enum Commands {
    AddMaterial { material_uuid: usize, image: DynamicImage, response: oneshot::Sender<Result<(), MaterialError>> },
    UpdateMaterial { material_uuid: usize, image: DynamicImage, response: oneshot::Sender<Result<(), MaterialError>> },
    DeleteMaterial { material_uuid: usize, response: oneshot::Sender<Result<(), MaterialError>> },
    GetMaterialHandle { material_uuid: usize, response: oneshot::Sender<Result<MaterialHandle, MaterialError>> },
    GetBindGroupLayout { response: oneshot::Sender<Arc<wgpu::BindGroupLayout>> },
    GetBindGroup { response: oneshot::Sender<Option<Arc<wgpu::BindGroup>>> }
}

// ========== Error Types ==========
#[derive(Error, Debug)]
pub enum MaterialError {
    #[error("memory occupied! material: {} is alredy exists!", .material_uuid)]
    MemoryOccupied { material_uuid: usize },

    #[error("segmentation fault! material: {} not found!", .material_uuid)]
    SegmentationFault { material_uuid: usize },

    #[error("memory graph error: {0}")]
    GraphError(#[from] GraphError),

    #[error("texture error: {0}")]
    TextureError(#[from] TextureError)
}
