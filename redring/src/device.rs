use wgpu::{Adapter, Device, Instance, Queue, Surface};

/// GPU初期化に必要な構造体
pub struct GpuContext<'a> {
    #[allow(dead_code)]
    pub surface: &'a Surface<'a>,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl<'a> GpuContext<'a> {
    pub fn new(instance: &'a Instance, surface: &'a Surface<'a>) -> Self {
        let adapter = pollster::block_on(Self::request_adapter(instance, surface));
        let (device, queue) = pollster::block_on(Self::request_device(&adapter));

        Self {
            surface,
            adapter,
            device,
            queue,
        }
    }

    /// 適切なGPUアダプタを取得
    async fn request_adapter(instance: &Instance, surface: &Surface<'_>) -> Adapter {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
            })
            .await
            .expect("No suitable adapter found")
    }

    /// デバイスとキューを取得
    async fn request_device(adapter: &Adapter) -> (Device, Queue) {
        let descriptor = wgpu::DeviceDescriptor {
            label: Some("RedRing Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::default(),
            trace: wgpu::Trace::default(),
        };

        adapter
            .request_device(&descriptor)
            .await
            .expect("Failed to create device")
    }
}