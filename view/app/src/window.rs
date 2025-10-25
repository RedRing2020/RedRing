use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

pub fn create_window(event_loop: &ActiveEventLoop) -> &'static Window {
    let attributes = WindowAttributes::default().with_title("RedRing");
    let window = event_loop
        .create_window(attributes)
        .expect("Failed to create window");

    Box::leak(Box::new(window))
}