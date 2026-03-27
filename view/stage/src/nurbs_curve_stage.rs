//! NURBS曲線 GPU評価レンダリングステージ
//!
//! ViewModel層で作成したGPU評価用データを受け取り、
//! GPU上でNURBS曲線を直接評価・描画します。

use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use render::nurbs_eval::NurbsCurveEvalResources;
use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

use crate::stage_common::build_nurbs_eval_uniforms;
use crate::RenderStage;

static NURBS_CURVE_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = NURBS_CURVE_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

/// NURBS曲線レンダリングステージ
pub struct NurbsCurveStage {
    resources: Option<NurbsCurveEvalResources>,
    has_data: bool,
    format: TextureFormat,
}

impl NurbsCurveStage {
    /// 新しいNURBS曲線ステージを作成
    pub fn new(_device: &Device, format: TextureFormat) -> Self {
        Self {
            resources: None,
            has_data: false,
            format,
        }
    }

    /// GPU評価用データを設定
    pub fn set_eval_data(
        &mut self,
        device: &Device,
        eval_data: viewmodel::nurbs_view::NurbsCurveEvalData,
    ) {
        let num_evals = eval_data.num_eval_points();
        let num_cps = eval_data.num_control_points();

        tracing::debug!(
            "NURBS曲線GPU評価データ設定: {} eval points, {} control points, degree={}",
            num_evals,
            num_cps,
            eval_data.degree
        );

        if num_evals == 0 {
            tracing::warn!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "nurbs curve stage: no eval points, skip draw"
            );
            self.has_data = false;
            return;
        }

        if num_cps == 0 {
            tracing::warn!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "nurbs curve stage: no control points, skip draw"
            );
            self.has_data = false;
            return;
        }

        let resources = NurbsCurveEvalResources::new(device, self.format, &eval_data);
        self.resources = Some(resources);
        self.has_data = true;

        tracing::debug!("✓ NurbsCurveStage: GPU評価データ設定完了、has_data=true");
    }

    /// カメラ行列を更新
    pub fn update_camera(
        &mut self,
        queue: &Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        if let Some(resources) = &self.resources {
            let uniforms = build_nurbs_eval_uniforms(view_matrix, proj_matrix);

            resources.update_uniforms(queue, &uniforms);
        }
    }

    /// データが設定されているか確認
    pub fn has_data(&self) -> bool {
        self.has_data
    }
}

impl RenderStage for NurbsCurveStage {
    fn render(&mut self, _encoder: &mut CommandEncoder, _view: &TextureView) {
        // 深度バッファが必要なため、render_with_depthを使用してください
        if should_log_render_trace() {
            tracing::trace!("NurbsCurveStage.render(): depth_view未指定のためスキップ");
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
            if should_trace {
                tracing::trace!("NurbsCurveStage: データなし、描画スキップ");
            }
            return;
        }

        let Some(resources) = &self.resources else {
            tracing::warn!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "nurbs curve stage: resources not initialized"
            );
            return;
        };

        if should_trace {
            tracing::trace!(
                "🎨 NurbsCurveStage.render_with_depth(): {} 頂点を描画開始",
                resources.num_eval_points
            );
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("NURBS Curve Render Pass"),
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
        });

        resources.render(&mut render_pass);

        if should_trace {
            tracing::trace!("🎨 NurbsCurveStage.render_with_depth(): 描画完了")
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
}
