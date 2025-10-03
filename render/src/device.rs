use wgpu::{Adapter, Device, Instance, Queue, Surface};

pub struct GpuContext<'a> {
    #[allow(dead_code)]
    pub surface: &'a Surface<'a>,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl<'a> GpuContext<'a> {
    pub fn new(instance: &'a Instance, surface: &'a Surface<'a>) -> Self {
        let adapter = pollster::block_on(
            instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
        ).expect("No suitable adapter found");

        let descriptor = wgpu::DeviceDescriptor {
            label: Some("RedRing Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::default(),
            experimental_features: wgpu::ExperimentalFeatures::default(),
        };

        let (device, queue) = pollster::block_on(adapter.request_device(&descriptor))
            .expect("Failed to create device");

        Self {
            surface,
            adapter,
            device,
            queue,
        }
    }
}