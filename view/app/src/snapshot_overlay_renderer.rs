use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

const SNAPSHOT_OVERLAY_SHADER: &str = r#"
struct VsOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
) -> VsOut {
    var out: VsOut;
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct OverlayVertex {
    position: [f32; 2],
    color: [f32; 4],
}

pub struct SnapshotOverlayRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    vertex_count: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SnapshotOverlayStyle {
    pub block_count: usize,
    pub track_color: [f32; 4],
    pub done_color: [f32; 4],
    pub todo_color: [f32; 4],
    pub cursor_color: [f32; 4],
}

impl Default for SnapshotOverlayStyle {
    fn default() -> Self {
        Self {
            block_count: 20,
            track_color: [0.22, 0.22, 0.25, 0.65],
            done_color: [0.0, 0.75, 1.0, 0.95],
            todo_color: [0.35, 0.35, 0.40, 0.85],
            cursor_color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl SnapshotOverlayRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Snapshot Overlay Shader"),
            source: wgpu::ShaderSource::Wgsl(SNAPSHOT_OVERLAY_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Snapshot Overlay Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Snapshot Overlay Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<OverlayVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Snapshot Overlay Vertex Buffer"),
            contents: &[0; std::mem::size_of::<OverlayVertex>() * 256],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            pipeline,
            vertex_buffer,
            vertex_count: 0,
        }
    }

    pub fn update_progress(
        &mut self,
        queue: &wgpu::Queue,
        progress: Option<f32>,
        style: &SnapshotOverlayStyle,
        viewport_width: u32,
        viewport_height: u32,
    ) {
        let Some(progress) = progress else {
            self.vertex_count = 0;
            return;
        };
        if viewport_width == 0 || viewport_height == 0 {
            self.vertex_count = 0;
            return;
        }

        let p = progress.clamp(0.0, 1.0);
        let left = 16.0;
        let top = 16.0;
        let width = 220.0;
        let height = 14.0;
        let right = left + width;
        let bottom = top + height;

        let block_count = style.block_count.max(1);
        let gap = 1.0f32;
        let block_width = (width - gap * (block_count as f32 - 1.0)) / block_count as f32;

        let to_ndc = |x: f32, y: f32| -> [f32; 2] {
            let ndc_x = (x / viewport_width as f32) * 2.0 - 1.0;
            let ndc_y = 1.0 - (y / viewport_height as f32) * 2.0;
            [ndc_x, ndc_y]
        };

        let c_track = style.track_color;
        let c_done = style.done_color;
        let c_todo = style.todo_color;
        let c_cursor = style.cursor_color;

        let mut vertices: Vec<OverlayVertex> = Vec::with_capacity(block_count * 6 + 18);

        append_solid_quad(&mut vertices, &to_ndc, (left, top), (right, bottom), c_track);

        let filled_blocks = ((p * block_count as f32).floor() as usize).min(block_count);

        for i in 0..block_count {
            let x0 = left + i as f32 * (block_width + gap);
            let x1 = x0 + block_width;
            let color = if i < filled_blocks { c_done } else { c_todo };
            append_solid_quad(
                &mut vertices,
                &to_ndc,
                (x0, top + 1.0),
                (x1, bottom - 1.0),
                color,
            );
        }

        let cursor_idx = ((p * (block_count as f32 - 1.0)).round() as usize).min(block_count - 1);
        let cursor_x0 = left + cursor_idx as f32 * (block_width + gap) - 0.5;
        let cursor_x1 = cursor_x0 + block_width + 1.0;
        append_solid_quad(
            &mut vertices,
            &to_ndc,
            (cursor_x0, top - 1.0),
            (cursor_x1, bottom + 1.0),
            c_cursor,
        );

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        self.vertex_count = vertices.len() as u32;
    }

    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        if self.vertex_count == 0 {
            return;
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Snapshot Overlay Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }
}

fn append_solid_quad<F>(
    out: &mut Vec<OverlayVertex>,
    to_ndc: &F,
    min: (f32, f32),
    max: (f32, f32),
    color: [f32; 4],
) where
    F: Fn(f32, f32) -> [f32; 2],
{
    let (x0, y0) = min;
    let (x1, y1) = max;

    let v0 = OverlayVertex {
        position: to_ndc(x0, y0),
        color,
    };
    let v1 = OverlayVertex {
        position: to_ndc(x1, y0),
        color,
    };
    let v2 = OverlayVertex {
        position: to_ndc(x1, y1),
        color,
    };
    let v3 = OverlayVertex {
        position: to_ndc(x0, y1),
        color,
    };

    out.extend_from_slice(&[v0, v1, v2, v0, v2, v3]);
}
