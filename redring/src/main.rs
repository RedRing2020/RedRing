use winit::{
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};
use winit::application::ApplicationHandler;

struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(event_loop.create_window(Default::default()).expect("Failed to create window"));
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            // 終了
            std::process::exit(0);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = App { window: None };
    event_loop.run_app(&mut app).expect("Event loop failed");
}