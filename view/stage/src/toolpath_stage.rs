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

use render::toolpath::{ToolPathResources, ToolPathUniforms, ToolPathVertex};
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

use crate::RenderStage;

/// CAM工具経路レンダリングステージ
pub struct ToolPathStage {
    resources: ToolPathResources,
    has_data: bool,
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

        Self {
            resources,
            has_data: false,
        }
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
        // View * Projection の行列乗算
        let view_proj = multiply_matrices(&view_matrix, &proj_matrix);
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
        if !self.has_data {
            tracing::debug!("ToolPathStage: データなし、描画スキップ");
            return;
        }

        tracing::debug!(
            "ToolPathStage: 描画開始 ({} 頂点)",
            self.resources.vertex_count
        );

        // Depth attachment用のテクスチャが必要
        // Note: 現在は簡易実装のため、depth_stencil_attachment は None
        // 実際の統合時には適切なDepthテクスチャを用意する必要がある
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
            depth_stencil_attachment: None, // TODO: Depthバッファ統合時に修正
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        self.resources.render(&mut render_pass);
    }

    fn update(&mut self) {
        // 必要に応じてアニメーション更新等を実装
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// 4x4行列の乗算（View * Projection）
///
/// # Note
///
/// 行優先（row-major）の行列を前提としています。
#[allow(clippy::needless_range_loop)]
fn multiply_matrices(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
    let mut result = [[0.0; 4]; 4];

    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                result[i][j] += a[i][k] * b[k][j];
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_matrix_multiplication() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let result = multiply_matrices(&identity, &identity);

        for i in 0..4 {
            for j in 0..4 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((result[i][j] - expected).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_translation_matrix_multiplication() {
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let translation = [
            [1.0, 0.0, 0.0, 10.0],
            [0.0, 1.0, 0.0, 20.0],
            [0.0, 0.0, 1.0, 30.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let result = multiply_matrices(&identity, &translation);

        assert_eq!(result, translation);
    }
}
