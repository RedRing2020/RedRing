use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

pub struct Graphic<'a> {
    pub device: Device,
    #[allow(dead_code)]
    pub queue: Queue,
    pub surface: &'a Surface<'a>,
    pub config: SurfaceConfiguration,
}

impl<'a> Graphic<'a> {
    pub fn new(
        device: Device,
        queue: Queue,
        surface: &'a Surface<'a>,
        config: SurfaceConfiguration,
    ) -> Self {
        Self {
            device,
            queue,
            surface,
            config,
        }
    }

    pub fn render_frame(&self) {
        // 描画処理（未実装なら空でOK）
    }
}