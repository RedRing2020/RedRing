//! mesh_converter - TriangleMesh3D から GPU用頂点データへの変換
//!
//! MVVMアーキテクチャにおけるViewModel層の責務として、
//! geo_primitives の具体型を使用して TriangleMesh3D を GPU レンダリング用の頂点データに変換します。

// 具体型はgeo_primitivesから
use geo_primitives::{TriangleMesh3D, Vector3D};

/// GPU用頂点データ（renderクレートのMeshVertexと同じ構造）
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct VertexData {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl VertexData {
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, normal }
    }
}

/// CCWと法線の整合性を検証（簡易版）
fn validate_ccw_normal_consistency(
    edge1: &Vector3D<f64>,
    edge2: &Vector3D<f64>,
    computed_normal: &Vector3D<f64>,
) -> bool {
    // CCW順序での外積による法線計算
    let ccw_normal = edge1.cross(edge2).normalize();

    // 内積による方向性チェック（同じ方向なら正の値）
    let dot_product = ccw_normal.dot(computed_normal);

    // 直接f64で比較
    dot_product > 0.9
}

/// TriangleMesh3D を GPU用頂点データのみに変換（render用の簡易版）
pub fn triangle_mesh_to_vertices(mesh: &TriangleMesh3D<f64>) -> Vec<VertexData> {
    let mut vertices = Vec::new();

    // 各三角形を個別の頂点として展開（法線の一貫性を保つため）
    for i in 0..mesh.triangle_count() {
        if let Some(triangle) = mesh.triangle(i) {
            let va = triangle.vertex_a();
            let vb = triangle.vertex_b();
            let vc = triangle.vertex_c();

            // 三角形の法線を計算（CCW順序を前提）
            let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
            let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());

            let normal = edge1.cross(&edge2).normalize();

            // CCWと法線の整合性をチェック（デバッグ用）
            let is_consistent = validate_ccw_normal_consistency(&edge1, &edge2, &normal);
            if i < 5 {
                // 最初の5つの三角形のみログ出力
                tracing::debug!(
                    "Triangle {}: CCW consistent: {}, normal={:?}",
                    i,
                    is_consistent,
                    [normal.x(), normal.y(), normal.z()]
                );
            }

            // f32 に変換
            let normal_f32 = [normal.x() as f32, normal.y() as f32, normal.z() as f32];

            // 頂点を追加
            vertices.push(VertexData::new(
                [va.x() as f32, va.y() as f32, va.z() as f32],
                normal_f32,
            ));

            vertices.push(VertexData::new(
                [vb.x() as f32, vb.y() as f32, vb.z() as f32],
                normal_f32,
            ));

            vertices.push(VertexData::new(
                [vc.x() as f32, vc.y() as f32, vc.z() as f32],
                normal_f32,
            ));
        }
    }

    vertices
}

/// TriangleMesh3D を GPU用頂点データとインデックスに変換
pub fn triangle_mesh_to_vertices_with_indices(
    mesh: &TriangleMesh3D<f64>,
) -> (Vec<VertexData>, Vec<u32>) {
    let vertices = triangle_mesh_to_vertices(mesh);
    let indices: Vec<u32> = (0..vertices.len() as u32).collect();

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_primitives::{Point3D, TriangleMesh3D};

    #[test]
    fn test_triangle_mesh_conversion() {
        // シンプルな三角形を作成
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let gpu_vertices = triangle_mesh_to_vertices(&mesh);

        assert_eq!(gpu_vertices.len(), 3);
    }

    #[test]
    fn test_triangle_mesh_with_indices_conversion() {
        // シンプルな三角形を作成
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let (gpu_vertices, gpu_indices) = triangle_mesh_to_vertices_with_indices(&mesh);

        assert_eq!(gpu_vertices.len(), 3);
        assert_eq!(gpu_indices.len(), 3);
        assert_eq!(gpu_indices, vec![0, 1, 2]);
    }
}
