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
use render::toolpath::{ToolPathResources, ToolPathVertex};
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use viewmodel::octree_converter::WireframeVertex;
use wgpu::{CommandEncoder, Device, TextureFormat, TextureView};

use crate::stage_common::{build_toolpath_uniforms, create_depth_texture};
use crate::RenderStage;

static OCTREE_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);
static OCTREE_STAGE_CAMERA_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = OCTREE_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

fn should_log_camera_debug() -> bool {
    let frame = OCTREE_STAGE_CAMERA_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

fn next_depth_index(current_depth: usize, max_depth: usize) -> usize {
    if max_depth == 0 {
        0
    } else {
        (current_depth + 1) % (max_depth + 1)
    }
}

fn should_step_animation(elapsed_secs: f32, interval_secs: f32) -> bool {
    elapsed_secs >= interval_secs
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
        let (depth_texture, depth_view) =
            create_depth_texture(device, size, "Octree Depth Texture");

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
        tracing::debug!("Octreeワイヤーフレームデータ設定: {} 頂点", vertices.len());

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
        let next = next_depth_index(self.current_depth, self.max_depth);
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

        if !should_step_animation(
            now.duration_since(last_step).as_secs_f32(),
            self.animation_interval_secs,
        ) {
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
        tracing::debug!(
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

    fn resize_depth_resources(&mut self, device: &Device, size: (u32, u32)) {
        if size.0 == 0 || size.1 == 0 || self.surface_size == size {
            return;
        }

        let (depth_texture, depth_view) =
            create_depth_texture(device, size, "Octree Depth Texture (Resized)");
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
        self.surface_size = size;
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
            if should_trace {
                tracing::trace!("OctreeStage: has_data=false, 描画スキップ");
            }
            return;
        }

        if should_trace {
            tracing::trace!(
                "OctreeStage.render(): 描画実行, vertex_count={}",
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
            multiview_mask: None,
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
        if should_log_camera_debug() {
            tracing::debug!("OctreeStage.update_camera() 呼び出し");
            tracing::debug!(
                "  view_matrix[3]: [{:.2}, {:.2}, {:.2}, {:.2}]",
                view_matrix[3][0],
                view_matrix[3][1],
                view_matrix[3][2],
                view_matrix[3][3]
            );
            tracing::debug!(
                "  proj_matrix[0]: [{:.2}, {:.2}, {:.2}, {:.2}]",
                proj_matrix[0][0],
                proj_matrix[0][1],
                proj_matrix[0][2],
                proj_matrix[0][3]
            );
        }

        let uniforms = build_toolpath_uniforms(view_matrix, proj_matrix);
        self.resources.update_uniforms(queue, &uniforms);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_with_device(&mut self, device: &wgpu::Device) {
        self.tick_animation(device);
    }

    fn on_surface_resized(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.resize_depth_resources(device, size);
    }

    fn cycle_octree_depth(&mut self, device: &wgpu::Device) -> Option<(usize, usize)> {
        self.cycle_next_depth(device);
        Some((self.current_depth(), self.max_depth()))
    }

    fn start_octree_depth_animation(&mut self) -> bool {
        self.start_depth_animation();
        true
    }

    fn apply_snapshot_wireframe_frame(
        &mut self,
        device: &wgpu::Device,
        snapshot_wireframes: Vec<Vec<WireframeVertex>>,
        frame_index: usize,
    ) -> bool {
        if snapshot_wireframes.is_empty() {
            return false;
        }

        if self.max_depth().saturating_add(1) != snapshot_wireframes.len() {
            self.set_depth_levels(device, snapshot_wireframes.clone());
        }

        let index = frame_index.min(snapshot_wireframes.len().saturating_sub(1));
        self.set_depth(device, index);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{next_depth_index, should_step_animation};

    #[test]
    fn next_depth_index_wraps_at_max_depth() {
        assert_eq!(next_depth_index(0, 3), 1);
        assert_eq!(next_depth_index(2, 3), 3);
        assert_eq!(next_depth_index(3, 3), 0);
    }

    #[test]
    fn next_depth_index_stays_zero_when_max_is_zero() {
        assert_eq!(next_depth_index(0, 0), 0);
        assert_eq!(next_depth_index(10, 0), 0);
    }

    #[test]
    fn should_step_animation_respects_threshold() {
        assert!(!should_step_animation(0.49, 0.5));
        assert!(should_step_animation(0.5, 0.5));
        assert!(should_step_animation(0.75, 0.5));
    }
}
