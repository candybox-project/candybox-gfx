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


use crate::core::camera::settings::CameraSettings;


// ========== Camera View-Projection ==========
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraViewProjection {
    view_proj: [[f32; 4]; 4]
}

impl CameraViewProjection {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        
        Self { view_proj: cgmath::Matrix4::identity().into() }
    }

    pub fn update_view_proj(&mut self, settings: &CameraSettings) {
        self.view_proj = settings.build_view_projection_matrix().into();
    }
}
