use wgpu::{Device, Queue, Surface, SurfaceConfiguration};

pub struct Graphic<'a> {
    pub device: Device,
    pub queue: Queue,
    pub surface: &'a Surface<'a>,
    pub config: SurfaceConfiguration,
}