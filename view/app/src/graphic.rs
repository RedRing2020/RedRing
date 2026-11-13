use crate::app_renderer::{AppRenderer, RenderFrameContext};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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
    /// GPU のデバイスが失われたか（失われた後はバッファを作れない）
    device_lost: Arc<AtomicBool>,
}

/// 描画の仕組み `backends` で、ウィンドウ `window` に描ける GPU を探す（見つからなければ `None`）
fn find_adapter(
    window: &Arc<Window>,
    backends: wgpu::Backends,
) -> Option<(wgpu::Instance, Surface<'static>, wgpu::Adapter)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let surface = instance.create_surface(window.clone()).ok()?;
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
        ..Default::default()
    }))
    .ok()?;
    Some((instance, surface, adapter))
}

pub fn init_graphic(window: Arc<Window>) -> Graphic {
    // 描画は Vulkan・DX12・Metal で行い、OpenGL は、それらで GPU が見つからないときだけ使う。OpenGL も同時に準備すると、
    // 使わなくても OpenGL のドライバを読み込み、ドライバによっては終了のときにその後始末で強制終了するため。
    // 環境変数 WGPU_BACKEND で描画の仕組みを指定できる（読めない値のときは既定にする）
    let requested = wgpu::Backends::from_env().filter(|backends| !backends.is_empty());
    let found = match requested {
        Some(backends) => find_adapter(&window, backends),
        None => find_adapter(&window, wgpu::Backends::PRIMARY).or_else(|| {
            tracing::warn!("No Vulkan, DX12 or Metal GPU adapter was found; trying OpenGL");
            find_adapter(&window, wgpu::Backends::GL)
        }),
    };
    let (_instance, surface, adapter) = found.unwrap_or_else(|| {
        panic!(
            "No GPU adapter was found for the window (backends: {}; set WGPU_BACKEND to choose another)",
            requested.map_or("Vulkan, DX12, Metal, then OpenGL".to_string(), |backends| {
                format!("{backends:?}")
            })
        )
    });
    let adapter_info = adapter.get_info();
    let driver = [
        adapter_info.driver.as_str(),
        adapter_info.driver_info.as_str(),
    ]
    .iter()
    .filter(|text| !text.is_empty())
    .copied()
    .collect::<Vec<_>>()
    .join(" ");
    if driver.is_empty() {
        tracing::info!(
            "GPU adapter: {} ({:?})",
            adapter_info.name,
            adapter_info.backend
        );
    } else {
        tracing::info!(
            "GPU adapter: {} ({:?}, driver {driver})",
            adapter_info.name,
            adapter_info.backend
        );
    }

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Device"),
        required_features: wgpu::Features::POLYGON_MODE_LINE, // ワイヤーフレーム機能を有効化
        required_limits: wgpu::Limits::default(),
        memory_hints: Default::default(),
        trace: wgpu::Trace::default(),
        experimental_features: wgpu::ExperimentalFeatures::default(),
    }))
    .expect("Device creation failed");

    let device_lost = Arc::new(AtomicBool::new(false));
    let lost = device_lost.clone();
    device.set_device_lost_callback(move |reason, message| {
        // 終了のときにデバイスを破棄した場合も呼ばれる
        if reason == wgpu::DeviceLostReason::Destroyed {
            return;
        }
        lost.store(true, Ordering::Release);
        tracing::error!("The GPU device was lost ({reason:?}): {message}");
    });
    device.on_uncaptured_error(Arc::new(|error| {
        tracing::error!("GPU error: {error}");
        panic!("wgpu error: {error}");
    }));

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
        // sRGB の形式では sRGB に出す（今までと同じ）
        color_space: wgpu::SurfaceColorSpace::Auto,
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
        device_lost,
    }
}

impl Graphic {
    /// GPU のデバイスが失われたか
    pub fn is_device_lost(&self) -> bool {
        self.device_lost.load(Ordering::Acquire)
    }

    pub fn render(&mut self, renderer: &mut AppRenderer) {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => {
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
                let render_context = RenderFrameContext {
                    device: &self.device,
                    queue: &self.queue,
                    viewport_width: self.config.width,
                    viewport_height: self.config.height,
                };
                renderer.render_with_depth(&render_context, &mut encoder, &view, depth_view);
                self.queue.submit(std::iter::once(encoder.finish()));
                self.queue.present(frame);
                self.surface_texture = None;
            }
            e => {
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
