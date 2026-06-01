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


use image::{DynamicImage, GenericImageView};
use std::cmp::max;
use thiserror::Error;

use crate::core::materials::textures;


// ========== Texture array ==========
pub struct TextureArray {
    // GPU resources
    texture: wgpu::Texture,
    view: wgpu::TextureView,

    // current && max layers count
    layers_count: u32,
    max_layers_count: u32,
    
    // width/height
    dimensions: (u32, u32)
}

impl TextureArray {
    pub fn new(device: &wgpu::Device, sampler: &wgpu::Sampler, size: &TextureArraySize) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size.to_u32(),
                height: size.to_u32(),
                depth_or_array_layers: 8
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_DST |
                wgpu::TextureUsages::COPY_SRC,
            view_formats: &[]
        });
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });

        Self {
            texture,
            view,
            layers_count: 0,
            max_layers_count: 8,
            dimensions: (size.to_u32(), size.to_u32())
        }
    }

    pub fn scale(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sampler: &wgpu::Sampler,
        active_layers: Vec<usize>
    ) {
        self.max_layers_count *= 2;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.dimensions.0,
                height: self.dimensions.1,
                depth_or_array_layers: self.max_layers_count
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_DST |
                wgpu::TextureUsages::COPY_SRC,
            view_formats: &[]
        });
        
        let mut encoder = device.create_command_encoder(&Default::default());
        
        for layer in active_layers {
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo{
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: layer as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: self.dimensions.0,
                    height: self.dimensions.1,
                    depth_or_array_layers: 1,
                },
            );
        }

        queue.submit([encoder.finish()]);
            
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        }); 
        
        self.texture = texture;
        self.view = view;
    }

    pub fn load_image(&self, queue: &wgpu::Queue, layer: u32, image: &DynamicImage) -> Result<(), TextureError> {
        if layer > self.max_layers_count {
            return Err(TextureError::OutOfMemory {
                data_layer: layer,
                max_layers: self.max_layers_count
            });
        }

        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();

        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d { x: 0, y: 0, z: layer }
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1)
            },
            wgpu::Extent3d {
                width: dimensions.0,
                height: dimensions.1,
                depth_or_array_layers: 1
            }
        );

        Ok(())
    }

    pub fn clear_image(&self, queue: &wgpu::Queue, layer: u32) -> Result<(), TextureError> {
        if layer > self.max_layers_count {
            return Err(TextureError::OutOfMemory {
                data_layer: layer,
                max_layers: self.max_layers_count
            });
        }

        let size = (self.dimensions.0 * self.dimensions.1 * 4) as usize;
        let transparent_data = vec![0u8; size];

        queue.write_texture(
            wgpu::TexelCopyTextureInfoBase {
                texture: &self.texture,
                mip_level: 0,
                aspect: wgpu::TextureAspect::All,
                origin: wgpu::Origin3d { x: 0, y: 0, z: layer }
            },
            &transparent_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.dimensions.0),
                rows_per_image: Some(self.dimensions.1)
            },
            wgpu::Extent3d {
                width: self.dimensions.0,
                height: self.dimensions.1,
                depth_or_array_layers: 1
            }
        );
        
        Ok(())
    }

    pub fn view(&self) -> &wgpu::TextureView { &self.view }
}

// ========== Dummy Texture ==========
pub struct DummyTexture {
    texture: wgpu::Texture,
    view: wgpu::TextureView
}

impl DummyTexture {
    pub fn new(device: &wgpu::Device, sampler: &wgpu::Sampler) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING |
                wgpu::TextureUsages::COPY_DST |
                wgpu::TextureUsages::COPY_SRC,
            view_formats: &[]
        });
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        
        Self {
            texture,
            view
        }
    }

    pub fn view(&self) -> &wgpu::TextureView { &self.view }
}

// ========== Texture array sizes ==========
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum TextureArraySize {
    Size16,
    Size32,
    Size64,
    Size128,
    Size256,
    Size512,
    Size1024,
    Size2048,
    Size4096,
    Size8192
}

impl TextureArraySize {
    // get index of SoA list by self
    pub fn to_index(&self) -> usize {
        match self {
            Self::Size16 => 0,
            Self::Size32 => 1,
            Self::Size64 => 2,
            Self::Size128 => 3,
            Self::Size256 => 4,
            Self::Size512 => 5,
            Self::Size1024 => 6,
            Self::Size2048 => 7,
            Self::Size4096 => 8,
            Self::Size8192 => 9
        }
    }

    // get u32-size by self
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Size16 => 16,
            Self::Size32 => 32,
            Self::Size64 => 64,
            Self::Size128 => 128,
            Self::Size256 => 256,
            Self::Size512 => 512,
            Self::Size1024 => 1024,
            Self::Size2048 => 2048,
            Self::Size4096 => 4096,
            Self::Size8192 => 8192
        }
    }

    // get self by index in SoA list
    pub fn from_index(index: usize) -> Result<Self, TextureError> {
        match index {
            0 => Ok(Self::Size16),
            1 => Ok(Self::Size32),
            2 => Ok(Self::Size64),
            3 => Ok(Self::Size128),
            4 => Ok(Self::Size256),
            5 => Ok(Self::Size512),
            6 => Ok(Self::Size1024),
            7 => Ok(Self::Size2048),
            8 => Ok(Self::Size4096),
            9 => Ok(Self::Size8192),
            _ => Err(TextureError::TextureArraySizeIndexOutOfBounds { index })
        }
    }

    // get self by required size
    pub fn from_dimensions(dimensions: (u32, u32)) -> Result<Self, TextureError> {
        let max_dimension = max(dimensions.0, dimensions.1);

        match max_dimension {
            d if d <= 16 => Ok(Self::Size16),
            d if d <= 32 => Ok(Self::Size32),
            d if d <= 64 => Ok(Self::Size64),
            d if d <= 128 => Ok(Self::Size128),
            d if d <= 256 => Ok(Self::Size256),
            d if d <= 512 => Ok(Self::Size512),
            d if d <= 1024 => Ok(Self::Size1024),
            d if d <= 2048 => Ok(Self::Size2048),
            d if d <= 4096 => Ok(Self::Size4096),
            d if d <= 8192 => Ok(Self::Size8192),
            _ => Err(TextureError::DimensionsOutOfBounds { dimensions })
        }
    }

    // get index of binding by self
    pub fn to_binding(&self) -> u32 {
        self.to_index() as u32 + 1 
    }

    pub fn all() -> Vec<Self> {
        vec![
            TextureArraySize::Size16,
            TextureArraySize::Size32,
            TextureArraySize::Size64,
            TextureArraySize::Size128,
            TextureArraySize::Size256,
            TextureArraySize::Size512,
            TextureArraySize::Size1024,
            TextureArraySize::Size2048,
            TextureArraySize::Size4096,
            TextureArraySize::Size8192
        ]
    }
}

// ========== Error types ==========
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("out of memory! data layer: {}, max texture layers: {}", .data_layer, .max_layers)]
    OutOfMemory {
        data_layer: u32,
        max_layers: u32,
    },

    #[error("invalid dimensions! expected: {:?}, found: {:?}", .expected, .found)]
    UnexpectedDimensions { expected: (u32, u32), found: (u32, u32) },

    #[error("invalid dimensions! texture dimensions: {:?} out of bounds(0 - 8192 px)!", .dimensions)]
    DimensionsOutOfBounds { dimensions: (u32, u32) },
    
    #[error("invalid texture array size index! texture array size index: {} out of bounds(0 - 9)!", .index)]
    TextureArraySizeIndexOutOfBounds { index: usize },

    #[error("segmentation fault! texture array with size: {:?} not found!", .texture_array_size)]
    TextureArrayNotFound { texture_array_size: TextureArraySize },

    #[error("memory occupied! texture array with size: {:?} is alredy exists!", .texture_array_size)]
    TextureArrayAlredyExists { texture_array_size: TextureArraySize }
}
