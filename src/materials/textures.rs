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


use std::sync::atomic::{AtomicUsize, Ordering};


// ========== Texture's UUID generator
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
struct TextureUUID { id: usize }

impl TextureUUID {
    fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self { id: COUNTER.fetch_add(1, Ordering::SeqCst) }
    }
}

// ========== Texture ==========
pub struct Texture {
    pub uuid: usize,
    pub image: image::DynamicImage
}

impl Texture {
    pub fn new(bytes: &[u8]) -> Result<Self, image::ImageError> {
        let uuid = TextureUUID::new().id;
        let image = image::load_from_memory(bytes)?;

        Ok(Self { uuid, image })
    }   
}
