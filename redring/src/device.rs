use wgpu::{Adapter, Device, Instance, Queue, Surface};

pub fn request_adapter(instance: &Instance, surface: &Surface) -> Adapter {
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        compatible_surface: Some(surface),
        ..Default::default()
    })).expect("No suitable adapter found")
}

pub fn request_device(adapter: &Adapter) -> (Device, Queue) {
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        .expect("Device creation failed")
}