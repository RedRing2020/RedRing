use winit::{
    event::WindowEvent,
    event_loop::{EventLoop, ActiveEventLoop},
    window::WindowAttributes,
    application::ApplicationHandler,
};
use wgpu::SurfaceConfiguration;
use render::renderer::Renderer;

struct RedRingApp<'a> {
    renderer: Option<Renderer<'a>>,
    window: Option<&'static winit::window::Window>,
}

impl<'a> ApplicationHandler<()> for RedRingApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Box::leak(Box::new(
            event_loop.create_window(WindowAttributes::default())
                .expect("Failed to create window"),
        )) as &'static winit::window::Window;

        let instance = wgpu::Instance::default();

        let surface = instance.create_surface(window).expect("Failed to create surface");
        let surface = Box::leak(Box::new(surface)); // &'static Surface<'static>

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .expect("Failed to find adapter");

        let device_descriptor = wgpu::DeviceDescriptor {
            label: Some("RedRing Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::default(),
        };

        let (device, queue) = pollster::block_on(adapter.request_device(&device_descriptor))
            .expect("Failed to create device");

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