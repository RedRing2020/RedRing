use crate::app_renderer::AppRenderer;
use std::sync::Arc;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureView};
use winit::window::Window;

pub struct Graphic {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub surface_texture: Option<SurfaceTexture>,
    pub depth_texture: Texture,
    pub depth_view: TextureView,
}

pub fn init_graphic(window: Arc<Window>) -> Graphic {
    let instance = wgpu::Instance::default();

    // 安全な方法でSurfaceを作成
    let surface = instance
        .create_surface(window.clone())
        .expect("Failed to create surface");

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .expect("No suitable GPU adapter found");

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Device"),
        required_features: wgpu::Features::POLYGON_MODE_LINE, // ワイヤーフレーム機能を有効化
        required_limits: wgpu::Limits::default(),
        memory_hints: Default::default(),
        trace: wgpu::Trace::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
    }))
    .expect("Device creation failed");

    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(caps.formats[0]);

    let config = SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    // Depth texture を作成
    let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Depth Texture"),
        size: wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });

    let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

    Graphic {
        device,
        queue,
        surface,
        config,
        surface_texture: None,
        depth_texture,
        depth_view,
    }
}

impl Graphic {
    pub fn render(&mut self, renderer: &mut AppRenderer) {
        match self.surface.get_current_texture() {
            Ok(frame) => {
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    ..Default::default()
                });

                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                // 深度ビューを渡してレンダリング
                let depth_view = &self.depth_view;
                renderer.render_with_depth(
                    &self.device,
                    &self.queue,
                    &mut encoder,
                    &view,
                    depth_view,
                    self.config.width,
                    self.config.height,
                );
                self.queue.submit(std::iter::once(encoder.finish()));
                frame.present();
                self.surface_texture = None;
            }
            Err(e) => {
                eprintln!("Failed to acquire surface texture: {:?}", e);
            }
        }
    }
}

impl Drop for Graphic {
    fn drop(&mut self) {
        println!("Dropping Graphic");
        if let Some(_frame) = self.surface_texture.take() {
            println!("⚠️ Warning: SurfaceTexture dropped without present()");
        }
    }
}
