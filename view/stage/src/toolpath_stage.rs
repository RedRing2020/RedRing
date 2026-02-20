//! CAM工具経路レンダリングステージ
//!
//! ViewModel層から変換された工具経路データを受け取り、
//! GPU上でレンダリングするステージです。
//!
//! # 使用例
//!
//! ```ignore
//! use stage::{ToolPathStage, RenderStage};
//! use wgpu::{Device, TextureFormat};
//!
//! let device = /* wgpu Device */;
//! let format = TextureFormat::Bgra8UnormSrgb;
//!
//! let mut stage = ToolPathStage::new(&device, format);
//!
//! // ViewModel変換データを設定
//! // stage.set_toolpath_data(&device, vertices);
//!
//! // カメラ行列更新
//! // stage.update_camera(&queue, view_proj_matrix);
//! ```

use analysis::linalg::matrix::Matrix4x4;
use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use render::toolpath::{ToolPathResources, ToolPathUniforms, ToolPathVertex};
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

use crate::RenderStage;

static TOOLPATH_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = TOOLPATH_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

/// CAM工具経路レンダリングステージ
pub struct ToolPathStage {
    resources: ToolPathResources,
    has_data: bool,
    #[allow(dead_code)]
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
    #[allow(dead_code)]
    surface_size: (u32, u32),
}

impl ToolPathStage {
    /// 新しい工具経路ステージを作成
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `format`: レンダーターゲットのテクスチャフォーマット
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let resources = ToolPathResources::new(device, format);

        // 初期深度テクスチャ（800x600）
        let size = (800, 600);
        let (depth_texture, depth_view) = Self::create_depth_texture(device, size);

        Self {
            resources,
            has_data: false,
            depth_texture,
            depth_view,
            surface_size: size,
        }
    }

    /// 深度テクスチャを作成
    fn create_depth_texture(
        device: &Device,
        size: (u32, u32),
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ToolPath Depth Texture"),
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

    /// 工具経路データを設定
    ///
    /// ViewModel層から変換された頂点データを受け取ります。
    ///
    /// # 引数
    ///
    /// - `device`: wgpu Device
    /// - `vertices`: 工具経路の頂点配列（位置+色）
    ///
    /// # Note
    ///
    /// `vertices` は LineList トポロジー用のため、
    /// 2頂点で1線分を表現します。
    pub fn set_toolpath_data(&mut self, device: &Device, vertices: Vec<ToolPathVertex>) {
        tracing::info!("工具経路データ設定: {} 頂点", vertices.len());

        self.resources.update_vertices(device, &vertices);
        self.has_data = !vertices.is_empty();
    }

    /// カメラ行列を更新
    ///
    /// # 引数
    ///
    /// - `queue`: wgpu Queue
    /// - `view_proj_matrix`: ビュー・プロジェクション結合行列
    pub fn update_camera(&mut self, queue: &Queue, view_proj_matrix: [[f32; 4]; 4]) {
        let uniforms = ToolPathUniforms {
            view_proj: view_proj_matrix,
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };

        self.resources.update_uniforms(queue, &uniforms);
    }

    /// カメラ行列を更新（View/Projection分離版）
    ///
    /// # 引数
    ///
    /// - `queue`: wgpu Queue
    /// - `view_matrix`: ビュー行列
    /// - `proj_matrix`: プロジェクション行列
    pub fn update_camera_separate(
        &mut self,
        queue: &Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        // Projection * View の順序（OpenGL/wgpu標準）
        // シェーダーでは: position_clip = projection * view * position_world
        // camera.rs は列優先配列を返すため、列優先として復元する
        let view = Matrix4x4::from_column_major(view_matrix);
        let proj = Matrix4x4::from_column_major(proj_matrix);
        let view_proj = (proj * view).to_column_major();
        self.update_camera(queue, view_proj);
    }

    /// データが設定されているか確認
    pub fn has_data(&self) -> bool {
        self.has_data
    }

    /// 頂点数を取得
    pub fn vertex_count(&self) -> u32 {
        self.resources.vertex_count
    }
}

impl RenderStage for ToolPathStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let should_trace = should_log_render_trace();

        if !self.has_data {
            if should_trace {
                tracing::trace!("ToolPathStage: データなし、描画スキップ");
            }
            return;
        }

        if should_trace {
            tracing::trace!(
                "ToolPathStage: 描画開始 ({} 頂点)",
                self.resources.vertex_count
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ToolPath Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load, // 既存の描画を保持
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
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

    fn update_camera(
        &mut self,
        queue: &Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        self.update_camera_separate(queue, view_matrix, proj_matrix);
    }

    fn update(&mut self) {
        // 必要に応じてアニメーション更新等を実装
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
