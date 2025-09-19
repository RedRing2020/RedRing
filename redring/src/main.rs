mod surface;
mod device;
mod renderer;
mod pipeline;

use renderer::Renderer;
use winit::{
    event::WindowEvent,
    event_loop::{EventLoop, ActiveEventLoop},
    application::ApplicationHandler,
};
use wgpu::SurfaceConfiguration;

struct RedRingApp<'a> {
    renderer: Option<Renderer<'a>>,
    window: Option<&'static winit::window::Window>,
}

impl<'a> ApplicationHandler<()> for RedRingApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = surface::create_window(event_loop);
        let instance: &'static wgpu::Instance = Box::leak(Box::new(wgpu::Instance::default()));
        let surface = Box::leak(surface::create_surface(&instance, window));

        let adapter = device::request_adapter(&instance, surface);
        let (device, queue) = device::request_device(&adapter);

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        self.renderer = Some(Renderer::new(device, queue, surface, config));
        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                _event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.config.width = new_size.width;
                    renderer.config.height = new_size.height;
                    renderer.surface.configure(&renderer.device, &renderer.config);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &self.renderer {
                    renderer.render_frame();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = RedRingApp {
        renderer: None,
        window: None,
    };

    event_loop.run_app(&mut app).expect("Failed to run app");
}