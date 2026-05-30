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
