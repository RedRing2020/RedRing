use std::panic::{catch_unwind, AssertUnwindSafe};
use wgpu::{Instance, Surface, SurfaceTexture};
use winit::window::Window;

pub fn safe_get_current_texture(surface: &Surface) -> Option<SurfaceTexture> {
    match catch_unwind(AssertUnwindSafe(|| surface.get_current_texture())) {
        Ok(Ok(texture)) => Some(texture),
        Ok(Err(e)) => {
            tracing::warn!("Surface texture acquisition failed: {:?}", e);
            None
        }
        Err(_) => {
            tracing::error!("Panic occurred during surface texture acquisition");
            None
        }
    }
}
pub fn create_surface<'a>(instance: &'a Instance, window: &'a Window) -> Surface<'a> {
    instance
        .create_surface(window)
        .expect("Failed to create surface")
}
