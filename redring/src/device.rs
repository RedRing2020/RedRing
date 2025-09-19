use wgpu::{Instance, Adapter, Device, Queue, RequestAdapterOptions, DeviceDescriptor};

pub fn request_adapter(instance: &Instance, surface: &wgpu::Surface) -> Adapter {
    pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    })).expect("Failed to find adapter")
}

pub fn request_device(adapter: &Adapter) -> (Device, Queue) {
    let descriptor = DeviceDescriptor {
        label: Some("RedRing Device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::default(),
        trace: wgpu::Trace::default(),
    };
    pollster::block_on(adapter.request_device(&descriptor))
        .expect("Failed to create device")
}