//! NURBS曲面 GPU評価レンダリングステージ
//!
//! ViewModel層で作成したGPU評価用データを受け取り、
//! GPU上でNURBS曲面を直接評価・描画します。

use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use render::nurbs_eval::NurbsSurfaceEvalResources;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

use crate::stage_common::build_nurbs_eval_uniforms;
use crate::RenderStage;

static NURBS_SURFACE_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = NURBS_SURFACE_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

/// NURBS曲面レンダリングステージ
pub struct NurbsSurfaceStage {
    resources: Option<NurbsSurfaceEvalResources>,
    has_data: bool,
    format: TextureFormat,
    wireframe_mode: bool,
}

impl NurbsSurfaceStage {
    /// 新しいNURBS曲面ステージを作成
    pub fn new(_device: &Device, format: TextureFormat) -> Self {
        Self {
            resources: None,
            has_data: false,
            format,
            wireframe_mode: false,
        }
    }

    /// GPU評価用データを設定
    pub fn set_eval_data(
        &mut self,
        device: &Device,
        eval_data: viewmodel::nurbs_view::NurbsSurfaceEvalData,
    ) {
        let num_vertices = eval_data.num_vertices();
        let num_triangles = eval_data.num_triangles();

        tracing::debug!(
            "NURBS曲面GPU評価データ設定: {} vertices ({} triangles), control grid={}x{}, u_degree={}, v_degree={}",
            num_vertices,
            num_triangles,
            eval_data.u_count,
            eval_data.v_count,
            eval_data.u_degree,
            eval_data.v_degree
        );

        if num_vertices == 0 {
            tracing::warn!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "nurbs surface stage: no eval points, skip draw"
            );
            self.has_data = false;
            return;
        }

        let resources = NurbsSurfaceEvalResources::new(device, self.format, &eval_data);

        tracing::debug!(
            "✓ NurbsSurfaceStage: GPU評価データ設定完了、num_vertices={}, solid_indices={}, wireframe_indices={}",
            resources.num_vertices,
            resources.num_solid_indices,
            resources.num_wireframe_indices
        );

        self.resources = Some(resources);
        self.has_data = true;
    }

    /// ワイヤーフレーム表示モード設定
    pub fn set_wireframe_mode(&mut self, wireframe: bool) {
        self.wireframe_mode = wireframe;
    }

    /// ワイヤーフレームモード切替
    pub fn toggle_wireframe(&mut self) {
        self.wireframe_mode = !self.wireframe_mode;
    }

    /// ワイヤーフレームモード取得
    pub fn is_wireframe(&self) -> bool {
        self.wireframe_mode
    }

    /// データが設定されているか確認
    pub fn has_data(&self) -> bool {
        self.has_data
    }

    /// カメラ行列更新
    pub fn update_camera(
        &mut self,
        queue: &Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        if let Some(ref resources) = self.resources {
            let uniforms = build_nurbs_eval_uniforms(view_matrix, proj_matrix);
            resources.update_uniforms(queue, &uniforms);
        }
    }
}

impl RenderStage for NurbsSurfaceStage {
    fn render(&mut self, _encoder: &mut CommandEncoder, _view: &TextureView) {
        // 深度バッファが必要なため、render_with_depthを使用してください
        if should_log_render_trace() {
            tracing::trace!("NurbsSurfaceStage.render(): depth_view未指定のためスキップ");
        }
    }

    fn render_with_depth(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        let should_trace = should_log_render_trace();

        if !self.has_data {
            return;
        }

        let Some(ref resources) = self.resources else {
            return;
        };

        let mode_str = if self.wireframe_mode {
            "wireframe"
        } else {
            "solid"
        };

        if should_trace {
            tracing::trace!(
                "🎨 NurbsSurfaceStage.render_with_depth(): {} 頂点を{}モードで描画開始",
                resources.num_vertices,
                mode_str
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("NURBS Surface Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.05,
                        b: 0.05,
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

        resources.render(&mut render_pass, self.wireframe_mode);

        drop(render_pass);

        if should_trace {
            tracing::trace!("🎨 NurbsSurfaceStage.render_with_depth(): 描画完了");
        }
    }

    fn update_camera(
        &mut self,
        queue: &Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        self.update_camera(queue, view_matrix, proj_matrix);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn toggle_wireframe_mode(&mut self) -> Option<bool> {
        self.toggle_wireframe();
        Some(self.is_wireframe())
    }
}
