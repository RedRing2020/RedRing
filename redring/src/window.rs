use winit::{event_loop::ActiveEventLoop, window::Window, window::WindowAttributes};

pub fn create_window(event_loop: &ActiveEventLoop) -> &'static Window {
    Box::leak(Box::new(
        event_loop.create_window(WindowAttributes::default())
            .expect("Failed to create window"),
    ))
}