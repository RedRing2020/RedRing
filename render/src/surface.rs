use std::panic::{catch_unwind, AssertUnwindSafe};
use wgpu::{Surface, SurfaceTexture};

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