use redring::app::App;

fn main() -> Result<(), winit::error::EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::builder().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)
}
