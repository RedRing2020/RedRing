use winit::{event_loop::ActiveEventLoop, window::Window, window::WindowAttributes};
use wgpu::{Instance, Surface};

pub fn create_window(event_loop: &ActiveEventLoop) -> &'static Window {
    Box::leak(Box::new(
        event_loop.create_window(WindowAttributes::default())
            .expect("Failed to create window"),
    ))
}

pub fn create_surface<'a>(instance: &'a Instance, window: &'a Window) -> Box<Surface<'a>> {
    Box::new(instance.create_surface(window).expect("Failed to create surface"))
}