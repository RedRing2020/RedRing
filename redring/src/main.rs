use winit::{
    event::{WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
    application::ApplicationHandler,
};

struct App<'a> {
    window: &'a Window,
    surface: &'a wgpu::Surface<'a>,
    device: &'a wgpu::Device,
    queue: &'a wgpu::Queue,
    config: &'a wgpu::SurfaceConfiguration,
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // main関数でwindow/surface等を初期化済みなので何もしない
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let WindowEvent::CloseRequested = event {
            std::process::exit(0);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let window = event_loop.create_window(Default::default()).expect("Failed to create window");

    // wgpu初期化
    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(&window).expect("Failed to create surface");
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    })).expect("No suitable GPU adapters found");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("Failed to create device");
    let size = window.inner_size();
    let caps = surface.get_capabilities(&adapter);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: caps.formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: wgpu::CompositeAlphaMode::Auto,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    // Appに参照を渡す
    let mut app = App {
        window: &window,
        surface: &surface,
        device: &device,
        queue: &queue,
        config: &config,
    };
    event_loop.run_app(&mut app).expect("Event loop failed");
}