use wgpu::{Instance, SurfaceConfiguration};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::device::GpuContext;
use crate::graphic::Graphic;
use render::surface::create_surface;

/// アプリケーションの状態を保持する構造体
pub struct App<'a> {
    pub graphic: Graphic<'a>,
    pub window: &'a Window,
}

impl<'a> App<'a> {
    /// 初期化処理
    pub async fn new(window: &'a Window) -> Self {
        // 'static に昇格した GPU インスタンスとサーフェス
        let instance = Box::leak(Box::new(Instance::default()));
        let surface = Box::leak(Box::new(create_surface(instance, window)));

        // GPU コンテキストの初期化
        let gpu = GpuContext::new(instance, surface);

        // SurfaceConfiguration の構築と適用
        let config = Self::create_surface_config(surface, window, &gpu.adapter);
        surface.configure(&gpu.device, &config);

        // Graphic の初期化
        let graphic = Graphic::new(gpu.device, gpu.queue, surface, config);

        Self { graphic, window }
    }

    /// SurfaceConfiguration を構築
    fn create_surface_config(
        surface: &wgpu::Surface<'_>,
        window: &Window,
        adapter: &wgpu::Adapter,
    ) -> SurfaceConfiguration {
        let size = window.inner_size();
        let caps = surface.get_capabilities(adapter);

        SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        }
    }

    /// ウィンドウサイズ変更時の処理
    fn resize_surface(&mut self, size: PhysicalSize<u32>) {
        self.graphic.config.width = size.width;
        self.graphic.config.height = size.height;
        self.graphic
            .surface
            .configure(&self.graphic.device, &self.graphic.config);
    }
}

impl<'a> ApplicationHandler<()> for App<'a> {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize_surface(size),
            WindowEvent::RedrawRequested => self.graphic.render_frame(),
            _ => {}
        }
    }
}