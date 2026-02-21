//! Octree可視化レンダリングステージ
//!
//! ViewModel層から変換されたOctreeワイヤーフレームデータを受け取り、
//! GPU上でレンダリングするステージです。
//!
//! # 使用例
//!
//! ```ignore
//! use stage::{OctreeStage, RenderStage};
//! use wgpu::{Device, TextureFormat};
//!
//! let device = /* wgpu Device */;
//! let format = TextureFormat::Bgra8UnormSrgb;
//!
//! let mut stage = OctreeStage::new(&device, format);
//!
//! // ViewModel変換データを設定
//! // stage.set_octree_data(&device, vertices);
//!
//! // カメラ行列更新
//! // stage.update_camera(&queue, view_proj_matrix);
//! ```

use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use render::toolpath::{ToolPathResources, ToolPathUniforms, ToolPathVertex};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use viewmodel::octree_converter::WireframeVertex;
use viewmodel_graphics::build_view_projection_matrix;
use wgpu::{CommandEncoder, Device, TextureFormat, TextureView};

use crate::RenderStage;

static OCTREE_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = OCTREE_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

/// Octree可視化ステージ
pub struct OctreeStage {
    resources: ToolPathResources,
    has_data: bool,
    depth_levels: Vec<Vec<WireframeVertex>>,
    current_depth: usize,
    max_depth: usize,
    animation_playing: bool,
    animation_interval_secs: f32,
    last_animation_step: Option<Instant>,
    #[allow(dead_code)]
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    #[allow(dead_code)]
    surface_size: (u32, u32),
}

impl OctreeStage {
    /// 新しいOctreeステージを作成
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `format`: レンダーターゲットのテクスチャフォーマット
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let resources = ToolPathResources::new(device, format);
        let size = (800, 600);
        let (depth_texture, depth_view) = Self::create_depth_texture(device, size);

        Self {
            resources,
            has_data: false,
            depth_levels: Vec::new(),
            current_depth: 0,
            max_depth: 0,
            animation_playing: false,
            animation_interval_secs: 0.5,
            last_animation_step: None,
            depth_texture,
            depth_view,
            surface_size: size,
        }
    }

    fn create_depth_texture(
        device: &Device,
        size: (u32, u32),
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Octree Depth Texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
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
        (depth_texture, depth_view)
    }

    /// ワイヤーフレームデータを設定（色無しバージョン）
    ///
    /// ViewModel層から変換された頂点データを受け取ります。
    /// Phase 1では色情報を無視し、単色（白）で描画します。
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `vertices`: ワイヤーフレームの頂点配列（位置のみ使用）
    ///
    /// # Note
    ///
    /// `vertices` は LineList トポロジー用のため、
    /// 2頂点で1線分を表現します。
    /// 指定されたデータは白色で描画されます。
    pub fn set_wireframe_data(&mut self, device: &Device, vertices: Vec<[f32; 3]>) {
        tracing::info!("Octreeワイヤーフレームデータ設定: {} 頂点", vertices.len());

        self.depth_levels.clear();
        self.current_depth = 0;
        self.max_depth = 0;
        self.animation_playing = false;
        self.last_animation_step = None;

        let white_vertices: Vec<WireframeVertex> = vertices
            .iter()
            .map(|&position| WireframeVertex::new(position, [1.0, 1.0, 1.0]))
            .collect();

        self.upload_wireframe_vertices(device, &white_vertices);
    }

    /// データが設定されているか確認
    pub fn has_data(&self) -> bool {
        self.has_data
    }

    /// 深さレベルごとのワイヤーフレームデータを設定（色付き）
    pub fn set_depth_levels(&mut self, device: &Device, depth_levels: Vec<Vec<WireframeVertex>>) {
        self.depth_levels = depth_levels;
        self.max_depth = self.depth_levels.len().saturating_sub(1);
        self.current_depth = 0;
        self.animation_playing = false;
        self.last_animation_step = None;

        self.rebuild_for_current_depth(device);
    }

    /// 現在の表示深さ
    pub fn current_depth(&self) -> usize {
        self.current_depth
    }

    /// 最大深さ
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// 指定深さへ表示を切り替え
    pub fn set_depth(&mut self, device: &Device, depth: usize) {
        self.current_depth = depth.min(self.max_depth);
        self.animation_playing = false;
        self.last_animation_step = None;
        self.rebuild_for_current_depth(device);
    }

    /// 深さを1段進める（最大到達時は0に戻る）
    pub fn cycle_next_depth(&mut self, device: &Device) {
        if self.max_depth == 0 {
            return;
        }
        let next = (self.current_depth + 1) % (self.max_depth + 1);
        self.set_depth(device, next);
    }

    /// 深さアニメーションを開始（0→max_depth）
    pub fn start_depth_animation(&mut self) {
        if self.max_depth == 0 {
            return;
        }
        self.current_depth = 0;
        self.animation_playing = true;
        self.last_animation_step = Some(Instant::now());
    }

    /// 深さアニメーションを更新
    pub fn tick_animation(&mut self, device: &Device) {
        if !self.animation_playing {
            return;
        }

        let now = Instant::now();
        let Some(last_step) = self.last_animation_step else {
            self.last_animation_step = Some(now);
            return;
        };

        if now.duration_since(last_step).as_secs_f32() < self.animation_interval_secs {
            return;
        }

        if self.current_depth >= self.max_depth {
            self.animation_playing = false;
            return;
        }

        self.current_depth += 1;
        self.last_animation_step = Some(now);
        self.rebuild_for_current_depth(device);
    }

    fn rebuild_for_current_depth(&mut self, device: &Device) {
        if self.depth_levels.is_empty() {
            return;
        }

        let vertices = self.depth_levels[self.current_depth].clone();
        tracing::info!(
            "Octree深さ表示更新: depth={}/{}, 頂点数={}",
            self.current_depth,
            self.max_depth,
            vertices.len()
        );

        self.upload_wireframe_vertices(device, &vertices);
    }

    fn upload_wireframe_vertices(&mut self, device: &Device, vertices: &[WireframeVertex]) {
        let gpu_vertices: Vec<ToolPathVertex> = vertices
            .iter()
            .map(|v| ToolPathVertex {
                position: v.position,
                color: [v.color[0], v.color[1], v.color[2], 1.0],
            })
            .collect();

        self.resources.update_vertices(device, &gpu_vertices);
        self.has_data = !gpu_vertices.is_empty();
    }

    /// 頂点数を取得
    pub fn vertex_count(&self) -> u32 {
        self.resources.vertex_count
    }
}

impl RenderStage for OctreeStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let depth_view = self.depth_view.clone();
        self.render_with_depth(encoder, view, &depth_view);
    }

    fn render_with_depth(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        let should_trace = should_log_render_trace();

        if !self.has_data {
            tracing::info!("🚨 OctreeStage: has_data=false, 描画スキップ");
            return;
        }

        tracing::info!(
            "✓ OctreeStage.render(): 描画実行, vertex_count={}",
            self.resources.vertex_count
        );

        if should_trace {
            tracing::trace!(
                "OctreeStage: 描画開始 ({} 頂点)",
                self.resources.vertex_count
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Octree Wireframe Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.1,
                        b: 0.15,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.resources.render(&mut render_pass);
    }

    /// カメラ行列を更新（RenderStage trait override）
    fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        tracing::info!("OctreeStage.update_camera() 呼び出し");
        tracing::info!(
            "  view_matrix[3]: [{:.2}, {:.2}, {:.2}, {:.2}]",
            view_matrix[3][0],
            view_matrix[3][1],
            view_matrix[3][2],
            view_matrix[3][3]
        );
        tracing::info!(
            "  proj_matrix[0]: [{:.2}, {:.2}, {:.2}, {:.2}]",
            proj_matrix[0][0],
            proj_matrix[0][1],
            proj_matrix[0][2],
            proj_matrix[0][3]
        );

        let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);
        let uniforms = ToolPathUniforms {
            view_proj,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        self.resources.update_uniforms(queue, &uniforms);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_octree_stage_creation() {
        // Note: wgpu Deviceが必要なため、実際のテストは統合テストで実施
        // このテストは構造確認のみ
    }

    #[test]
    fn test_octree_stage_no_data_initially() {
        // Note: wgpu Deviceが必要なため、実際のテストは統合テストで実施
    }
}
