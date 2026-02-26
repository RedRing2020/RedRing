//! メッシュレンダリングステージ
//!
//! STLファイルから読み込んだ3Dメッシュや幾何形状をレンダリングするステージです。
//! Phase 2拡張: 線分描画モード（RenderMode::Lines）に対応しました。

use logging_foundation::{frame_interval_from_env, should_log_every_n_frames};
use render::{line::LineResources, mesh::MeshResources, vertex_3d::MeshVertex};
use std::sync::atomic::{AtomicU64, Ordering};
use wgpu::{CommandEncoder, Device, TextureFormat, TextureView};

use crate::RenderStage;

static MESH_STAGE_RENDER_LOG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn should_log_render_trace() -> bool {
    let frame = MESH_STAGE_RENDER_LOG_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    let interval = frame_interval_from_env("REDRING_LOG_FRAME_INTERVAL", 120);
    should_log_every_n_frames(frame, interval)
}

/// レンダリングモード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// ソリッド描画（三角形メッシュ）
    Solid,
    /// ワイヤーフレーム描画（三角形の辺のみ）
    Wireframe,
    /// 線分描画（LineListトポロジー）
    Lines,
}

/// メッシュレンダリングステージ
pub struct MeshStage {
    mesh_resources: MeshResources,
    line_resources: Option<LineResources>,
    overlay_line_resources: Option<LineResources>,
    overlay_toolpath_line_resources: Option<LineResources>,
    overlay_tool_line_resources: Option<LineResources>,
    render_mode: RenderMode,
    format: TextureFormat,
}

impl MeshStage {
    /// 新しいメッシュステージを作成
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let mesh_resources = MeshResources::new(device, format);

        Self {
            mesh_resources,
            line_resources: None,
            overlay_line_resources: None,
            overlay_toolpath_line_resources: None,
            overlay_tool_line_resources: None,
            render_mode: RenderMode::Solid,
            format,
        }
    }

    /// メッシュの基本色を設定（シェーディング時）
    pub fn set_mesh_base_color(&mut self, color: [f32; 4]) {
        self.mesh_resources.set_base_color(color);
    }

    /// メッシュデータを設定（ソリッド/ワイヤーフレーム用）
    pub fn set_mesh_data(&mut self, device: &Device, vertices: Vec<MeshVertex>, indices: Vec<u32>) {
        tracing::debug!(
            "メッシュデータ設定: {} 頂点, {} インデックス",
            vertices.len(),
            indices.len()
        );

        self.mesh_resources
            .update_mesh_data(device, &vertices, &indices);
        self.render_mode = RenderMode::Solid;
    }

    /// 線分データを設定（Lines用）
    pub fn set_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>) {
        tracing::debug!("線分データ設定: {} 頂点", vertices.len());

        // LineResourcesが未初期化の場合は作成
        if self.line_resources.is_none() {
            self.line_resources = Some(LineResources::new(device, self.format));
        }

        // 線分データを更新
        if let Some(line_res) = &mut self.line_resources {
            line_res.update_line_data(device, &vertices);
        }

        self.render_mode = RenderMode::Lines;
    }

    /// ソリッド表示に重畳する線分データを設定
    pub fn set_overlay_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>) {
        if self.overlay_line_resources.is_none() {
            self.overlay_line_resources = Some(LineResources::new(device, self.format));
        }

        if let Some(line_res) = &mut self.overlay_line_resources {
            line_res.update_line_data(device, &vertices);
        }
    }

    /// ソリッド表示に重畳するツールパス線分データを設定
    pub fn set_overlay_toolpath_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>) {
        if self.overlay_toolpath_line_resources.is_none() {
            self.overlay_toolpath_line_resources = Some(LineResources::new(device, self.format));
        }

        if let Some(line_res) = &mut self.overlay_toolpath_line_resources {
            line_res.update_line_data(device, &vertices);
        }
    }

    /// ソリッド表示に重畳する工具線分データを設定
    pub fn set_overlay_tool_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>) {
        if self.overlay_tool_line_resources.is_none() {
            self.overlay_tool_line_resources = Some(LineResources::new(device, self.format));
        }

        if let Some(line_res) = &mut self.overlay_tool_line_resources {
            line_res.update_line_data(device, &vertices);
        }
    }

    /// ソリッド表示への線分重畳を無効化
    pub fn clear_overlay_line_data(&mut self) {
        self.overlay_line_resources = None;
        self.overlay_toolpath_line_resources = None;
        self.overlay_tool_line_resources = None;
    }

    /// カメラ行列を更新
    pub fn update_camera_matrices(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        self.mesh_resources
            .update_camera(queue, view_matrix, proj_matrix);

        // 線分リソースのカメラも更新
        if let Some(line_res) = &mut self.line_resources {
            line_res.update_camera(queue, view_matrix, proj_matrix);
        }

        if let Some(line_res) = &mut self.overlay_line_resources {
            line_res.update_camera(queue, view_matrix, proj_matrix);
        }

        if let Some(line_res) = &mut self.overlay_toolpath_line_resources {
            line_res.update_camera(queue, view_matrix, proj_matrix);
        }

        if let Some(line_res) = &mut self.overlay_tool_line_resources {
            line_res.update_camera(queue, view_matrix, proj_matrix);
        }
    }

    /// ワイヤーフレームモードを切り替え
    pub fn toggle_wireframe(&mut self) {
        self.mesh_resources.toggle_wireframe();
    }

    /// ワイヤーフレームモードかどうか
    pub fn is_wireframe(&self) -> bool {
        self.mesh_resources.is_wireframe()
    }
}

impl RenderStage for MeshStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0, // 黒に戻す
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // レンダリングモードに応じた描画
        match self.render_mode {
            RenderMode::Solid | RenderMode::Wireframe => {
                self.mesh_resources.render(&mut render_pass);

                if let Some(line_res) = &self.overlay_line_resources {
                    line_res.render(&mut render_pass);
                }
                if let Some(line_res) = &self.overlay_toolpath_line_resources {
                    line_res.render(&mut render_pass);
                }
                if let Some(line_res) = &self.overlay_tool_line_resources {
                    line_res.render(&mut render_pass);
                }
            }
            RenderMode::Lines => {
                if let Some(line_res) = &self.line_resources {
                    line_res.render(&mut render_pass);
                } else {
                    tracing::debug!("LineResources が None");
                }
            }
        }
    }

    fn render_with_depth(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,
        depth_view: &TextureView,
    ) {
        let should_trace = should_log_render_trace();
        if should_trace {
            tracing::trace!("MeshStage.render_with_depth() called, mode={:?}", self.render_mode);
        }

        if matches!(self.render_mode, RenderMode::Lines) {
            let mut line_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Line Render Pass (No Depth)"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(line_res) = &self.line_resources {
                if should_trace {
                    tracing::trace!("Lines描画(no depth): vertex_count={}", line_res.vertex_count);
                }
                line_res.render(&mut line_pass);
            } else {
                tracing::debug!("LineResources が None");
            }
            return;
        }

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Mesh Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0, // 黒に戻す
                        g: 0.0,
                        b: 0.0,
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

        // レンダリングモードに応じた描画
        match self.render_mode {
            RenderMode::Solid | RenderMode::Wireframe => {
                self.mesh_resources.render(&mut render_pass);
            }
            RenderMode::Lines => {
                if let Some(line_res) = &self.line_resources {
                    if should_trace {
                        tracing::trace!("Lines描画: vertex_count={}", line_res.vertex_count);
                    }
                    line_res.render(&mut render_pass);
                } else {
                    tracing::debug!("LineResources が None");
                }
            }
        }

        if matches!(self.render_mode, RenderMode::Solid | RenderMode::Wireframe)
            && (self.overlay_line_resources.is_some()
                || self.overlay_toolpath_line_resources.is_some()
                || self.overlay_tool_line_resources.is_some())
        {
            drop(render_pass);
            let mut overlay_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesh Overlay Line Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(line_res) = &self.overlay_line_resources {
                line_res.render(&mut overlay_pass);
            }
            if let Some(line_res) = &self.overlay_toolpath_line_resources {
                line_res.render(&mut overlay_pass);
            }
            if let Some(line_res) = &self.overlay_tool_line_resources {
                line_res.render(&mut overlay_pass);
            }
        }
    }

    fn update(&mut self) {
        // 必要に応じてアニメーション更新等を実装
    }

    fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        self.update_camera_matrices(queue, view_matrix, proj_matrix);
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
