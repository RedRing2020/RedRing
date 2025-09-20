use winit::window::Window;
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

pub struct Graphic<'a> {
    pub device: Device,
    pub queue: Queue,
    pub surface: &'a Surface<'a>,
    pub config: SurfaceConfiguration,
}

pub fn init_wgpu(_window: &Window) -> (wgpu::Device, wgpu::Queue, wgpu::Surface<'_>, SurfaceConfiguration)
 {
    // wgpu初期化処理を書く（略）
    todo!()
}