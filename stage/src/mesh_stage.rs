//! メッシュレンダリングステージ
//!
//! STLファイルから読み込んだ3Dメッシュをレンダリングするステージです。

use render::{mesh::MeshResources, vertex_3d::MeshVertex};
use wgpu::{CommandEncoder, Device, TextureFormat, TextureView};

use crate::RenderStage;

/// メッシュレンダリングステージ
pub struct MeshStage {
    resources: MeshResources,
}

impl MeshStage {
    /// 新しいメッシュステージを作成
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let resources = MeshResources::new(device, format);

        Self { resources }
    }

    /// メッシュデータを設定
    pub fn set_mesh_data(&mut self, device: &Device, vertices: Vec<MeshVertex>, indices: Vec<u32>) {
        tracing::info!(
            "メッシュデータ設定: {} 頂点, {} インデックス",
            vertices.len(),
            indices.len()
        );

        // リソースにメッシュデータを更新
        self.resources.update_mesh_data(device, &vertices, &indices);
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
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // メッシュをレンダリング
        self.resources.render(&mut render_pass);
    }

    fn update(&mut self) {
        // 必要に応じてアニメーション更新等を実装
    }
}
