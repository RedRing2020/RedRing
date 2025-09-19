mod app;
mod device;
mod graphic;

use app::App;
use winit::event_loop::EventLoop;

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = Box::leak(Box::new(
        event_loop.create_window(Default::default()).expect("Failed to create window"),
    ));

    let mut app = pollster::block_on(App::new(window));
    event_loop.run_app(&mut app).expect("Failed to run app");
}