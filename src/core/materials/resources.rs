use std::{collections::HashMap, sync::Arc};

use crate::core::materials::textures::{self, TextureArray, TextureArraySize, TextureError};


// ========== Graphics Resources ==========
pub struct Resources {
    // texture sampler
    sampler: wgpu::Sampler,

    // unified bind group layout && bind group
    pub bind_group_layout: Arc<wgpu::BindGroupLayout>,
    pub bind_group: Option<Arc<wgpu::BindGroup>>,

    // texture arrays
    texture_arrays: HashMap<TextureArraySize, TextureArray>,

    // dummy texture
    dummy_texture: textures::DummyTexture
}

impl Resources {
    pub fn new(device: &wgpu::Device) -> Self {
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::MipmapFilterMode::Nearest,
                ..Default::default()
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // sampler
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                },
                // 16 px
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 32 px
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 64 px
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 128 px
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 256 px
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 512 px
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 1024 px
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 2048 px
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 4096 px
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                // 8192 px
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                }
            ]
        });

        let dummy_texture = textures::DummyTexture::new(device, &sampler);

        Self {
            sampler,
            bind_group_layout: Arc::new(bind_group_layout),
            bind_group: None,
            texture_arrays: HashMap::new(),
            dummy_texture
        }
    }

    pub fn add_texture_array(
        &mut self,
        device: &wgpu::Device,
        texture_array_size: &TextureArraySize
    ) -> Result<(), TextureError> {
        if self.texture_arrays.contains_key(texture_array_size) {
            Err(TextureError::TextureArrayAlredyExists { texture_array_size: *texture_array_size })
        } else {
            let texture_array = match texture_array_size {
                TextureArraySize::Size16 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size32 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size64 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size128 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size256 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size512 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size1024 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size2048 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size4096 => TextureArray::new(device, &self.sampler, texture_array_size),
                TextureArraySize::Size8192 => TextureArray::new(device, &self.sampler, texture_array_size),
            };

            self.texture_arrays.insert(*texture_array_size, texture_array);
            self.update_bind_group(device);           

            Ok(())
        }
    }

    pub fn scale_texture_array(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_array_size: &TextureArraySize,
        active_layers: Vec<usize>
    ) -> Result<(), TextureError> {
        if let Some(texture_array) = self.texture_arrays.get_mut(texture_array_size) {
            texture_array.scale(device, queue, &self.sampler, active_layers);
            self.update_bind_group(device);
            Ok(())
        } else { Err(TextureError::TextureArrayNotFound { texture_array_size: *texture_array_size }) }
    }

    pub fn delete_texture_array(
        &mut self,
        device: &wgpu::Device,
        texture_array_size: &TextureArraySize
    ) -> Result<(), TextureError> {
        self.texture_arrays
            .remove(texture_array_size)
            .ok_or(TextureError::TextureArrayNotFound {
                texture_array_size: *texture_array_size
            })?;

        self.update_bind_group(device);
        
        Ok(())
    }

    pub fn get_texture_array(&self, texture_array_size: &TextureArraySize) -> Result<&TextureArray, TextureError> {
        if let Some(texture_array) = self.texture_arrays.get(texture_array_size) {
            Ok(texture_array)
        } else { Err(TextureError::TextureArrayNotFound { texture_array_size: *texture_array_size }) }
    }

    fn update_bind_group(&mut self, device: &wgpu::Device) {
        let mut entries = Vec::new();

        entries.push(wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Sampler(&self.sampler)
        });

        for texture_array_size in TextureArraySize::all() {
            let view = self.texture_arrays
                .get(&texture_array_size)
                .map(|texture_array| texture_array.view())
                .unwrap_or_else(|| self.dummy_texture.view());
            
            entries.push(wgpu::BindGroupEntry {
                binding: texture_array_size.to_binding() as u32,
                resource: wgpu::BindingResource::TextureView(view)
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout,
            entries: &entries
        });

        self.bind_group = Some(Arc::new(bind_group));
    }
}
