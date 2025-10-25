//! mesh_convert - TriangleMesh3D から MeshVertex への変換
//!
//! geo_primitives::TriangleMesh3D を render::MeshVertex のベクターに変換し、
//! GPU レンダリングで使用できる形式にします。

use crate::vertex_3d::MeshVertex;
use geo_foundation::Scalar;
use geo_primitives::{TriangleMesh3D, Vector3D};

/// TriangleMesh3D を MeshVertex のベクターに変換
/// 各三角形を独立した頂点として展開し、各三角形の法線を計算
pub fn triangle_mesh_to_mesh_vertices<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
) -> (Vec<MeshVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // 各三角形を個別の頂点として展開（法線の一貫性を保つため）
    for i in 0..mesh.triangle_count() {
        if let Some(triangle) = mesh.triangle(i) {
            let va = triangle.vertex_a();
            let vb = triangle.vertex_b();
            let vc = triangle.vertex_c();

            // 三角形の法線を計算
            let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
            let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());

            let normal = edge1.cross(&edge2).normalize();

            // f32 に変換
            let normal_f32 = [
                normal.x().to_f64() as f32,
                normal.y().to_f64() as f32,
                normal.z().to_f64() as f32,
            ];

            // 頂点を追加
            let base_index = vertices.len() as u32;

            vertices.push(MeshVertex::new(
                [
                    va.x().to_f64() as f32,
                    va.y().to_f64() as f32,
                    va.z().to_f64() as f32,
                ],
                normal_f32,
            ));

            vertices.push(MeshVertex::new(
                [
                    vb.x().to_f64() as f32,
                    vb.y().to_f64() as f32,
                    vb.z().to_f64() as f32,
                ],
                normal_f32,
            ));

            vertices.push(MeshVertex::new(
                [
                    vc.x().to_f64() as f32,
                    vc.y().to_f64() as f32,
                    vc.z().to_f64() as f32,
                ],
                normal_f32,
            ));

            // インデックスを追加
            indices.push(base_index);
            indices.push(base_index + 1);
            indices.push(base_index + 2);
        }
    }

    (vertices, indices)
}

/// 簡易版の頂点マージ変換
/// 元の頂点配列を使用し、インデックスベースで変換
pub fn triangle_mesh_to_mesh_vertices_indexed<T: Scalar>(
    mesh: &TriangleMesh3D<T>,
) -> (Vec<MeshVertex>, Vec<u32>) {
    let mut vertices = Vec::with_capacity(mesh.vertex_count());
    let mut indices = Vec::with_capacity(mesh.triangle_count() * 3);

    // 頂点法線を計算するためのベクター
    let mut vertex_normals: Vec<Vector3D<T>> = vec![Vector3D::zero(); mesh.vertex_count()];

    // 各三角形の面法線を計算し、頂点法線に加算
    for i in 0..mesh.triangle_count() {
        if let (Some(triangle), Some(triangle_indices)) =
            (mesh.triangle(i), mesh.triangle_indices(i))
        {
            let va = triangle.vertex_a();
            let vb = triangle.vertex_b();
            let vc = triangle.vertex_c();

            // 面法線を計算
            let edge1 = Vector3D::new(vb.x() - va.x(), vb.y() - va.y(), vb.z() - va.z());
            let edge2 = Vector3D::new(vc.x() - va.x(), vc.y() - va.y(), vc.z() - va.z());
            let face_normal = edge1.cross(&edge2);

            // 各頂点の法線に面法線を加算
            vertex_normals[triangle_indices[0]] = vertex_normals[triangle_indices[0]] + face_normal;
            vertex_normals[triangle_indices[1]] = vertex_normals[triangle_indices[1]] + face_normal;
            vertex_normals[triangle_indices[2]] = vertex_normals[triangle_indices[2]] + face_normal;
        }
    }

    // 頂点データを作成
    for (i, vertex_normal) in vertex_normals.iter().enumerate().take(mesh.vertex_count()) {
        if let Some(vertex) = mesh.vertex(i) {
            let position = [
                vertex.x().to_f64() as f32,
                vertex.y().to_f64() as f32,
                vertex.z().to_f64() as f32,
            ];

            // 法線を正規化
            let normal = vertex_normal.normalize();
            let normal_f32 = [
                normal.x().to_f64() as f32,
                normal.y().to_f64() as f32,
                normal.z().to_f64() as f32,
            ];

            vertices.push(MeshVertex::new(position, normal_f32));
        }
    }

    // インデックスデータを作成
    for i in 0..mesh.triangle_count() {
        if let Some(triangle_indices) = mesh.triangle_indices(i) {
            indices.push(triangle_indices[0] as u32);
            indices.push(triangle_indices[1] as u32);
            indices.push(triangle_indices[2] as u32);
        }
    }

    (vertices, indices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_primitives::Point3D;

    #[test]
    fn test_triangle_mesh_conversion() {
        // シンプルな三角形メッシュを作成
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let (render_vertices, render_indices) = triangle_mesh_to_mesh_vertices(&mesh);

        assert_eq!(render_vertices.len(), 3);
        assert_eq!(render_indices.len(), 3);

        // 頂点位置の確認
        assert_eq!(render_vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(render_vertices[1].position, [1.0, 0.0, 0.0]);
        assert_eq!(render_vertices[2].position, [0.0, 1.0, 0.0]);

        // インデックスの確認
        assert_eq!(render_indices, vec![0, 1, 2]);
    }

    #[test]
    fn test_triangle_mesh_indexed_conversion() {
        // 四角形メッシュ（2つの三角形）
        let vertices = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let indices = vec![[0, 1, 2], [0, 2, 3]];

        let mesh = TriangleMesh3D::new(vertices, indices).unwrap();
        let (render_vertices, render_indices) = triangle_mesh_to_mesh_vertices_indexed(&mesh);

        assert_eq!(render_vertices.len(), 4);
        assert_eq!(render_indices.len(), 6);
    }
}
