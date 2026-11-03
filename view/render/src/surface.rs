use std::panic::{catch_unwind, AssertUnwindSafe};
use wgpu::{Instance, Surface, SurfaceTexture};
use winit::window::Window;

pub fn safe_get_current_texture(surface: &Surface) -> Option<SurfaceTexture> {
    match catch_unwind(AssertUnwindSafe(|| surface.get_current_texture())) {
        Ok(wgpu::CurrentSurfaceTexture::Success(texture)) => Some(texture),
        Ok(e) => {
            tracing::warn!(
                error_kind = logging_foundation::ERROR_KIND_SYSTEM,
                "surface texture acquisition failed: {:?}",
                e
            );
            None
        }
        Err(_) => {
            tracing::error!(
                error_kind = logging_foundation::ERROR_KIND_SYSTEM,
                "panic during surface texture acquisition"
            );
            None
        }
    }
}
pub fn create_surface<'a>(instance: &'a Instance, window: &'a Window) -> Surface<'a> {
    instance
        .create_surface(window)
        .expect("Failed to create surface")
}
