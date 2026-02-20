//! メッシュレンダリングステージ
//!
//! STLファイルから読み込んだ3Dメッシュや幾何形状をレンダリングするステージです。
//! Phase 2拡張: 線分描画モード（RenderMode::Lines）に対応しました。

use render::{line::LineResources, mesh::MeshResources, vertex_3d::MeshVertex};
use wgpu::{CommandEncoder, Device, TextureFormat, TextureView};

use crate::RenderStage;

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
            render_mode: RenderMode::Solid,
            format,
        }
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

    /// カメラ行列を更新
    pub fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        self.mesh_resources
            .update_camera(queue, view_matrix, proj_matrix);

        // 線分リソースのカメラも更新
        if let Some(line_res) = &self.line_resources {
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
        tracing::trace!("MeshStage.render() called, mode={:?}", self.render_mode);
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
            }
            RenderMode::Lines => {
                if let Some(line_res) = &self.line_resources {
                    tracing::trace!("Lines描画: vertex_count={}", line_res.vertex_count);
                    line_res.render(&mut render_pass);
                } else {
                    tracing::debug!("LineResources が None");
                }
            }
        }
    }

    fn update(&mut self) {
        // 必要に応じてアニメーション更新等を実装
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
