//! 逆オフセット形状のデバッグ表示用データ変換
//!
//! 逆オフセット形状の要素の役割ごとの三角形を、MeshStage で重ねて描けるシェーディングメッシュへ変換する。

/// 表示用の三角形の種類の再エクスポート
pub use cam_algorithms::InverseOffsetPartKind;
use cam_algorithms::{CamSolverError, InverseOffsetShape};

use crate::mesh_converter::VertexData;

/// 種類ごとのシェーディングメッシュ（三角形ごとに独立した頂点・面法線。法線は要素の外の側を向く）
#[derive(Debug, Clone)]
pub struct InverseOffsetShapeMesh {
    pub kind: InverseOffsetPartKind,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

/// 逆オフセット形状を、角度 `angle_step`（ラジアン）のきざみで種類ごとのメッシュにする
///
/// # Errors
///
/// `angle_step` が表示用のきざみの範囲の外の場合。
pub fn inverse_offset_shape_meshes(
    shape: &InverseOffsetShape,
    angle_step: f64,
) -> Result<Vec<InverseOffsetShapeMesh>, CamSolverError> {
    Ok(shape
        .display_parts(angle_step)?
        .into_iter()
        .map(|part| {
            let mut vertices = Vec::with_capacity(part.triangles.len() * 3);
            for [a, b, c] in &part.triangles {
                let normal = normal(*a, *b, *c);
                for p in [a, b, c] {
                    vertices.push(VertexData::new(p.map(|v| v as f32), normal));
                }
            }
            let indices = (0..vertices.len() as u32).collect();
            InverseOffsetShapeMesh {
                kind: part.kind,
                vertices,
                indices,
            }
        })
        .collect())
}

/// 三角形の、頂点の順（右ねじ）の単位法線
fn normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    n.map(|component| (component / length) as f32)
}

#[cfg(test)]
mod tests {
    use super::{inverse_offset_shape_meshes, InverseOffsetPartKind};
    use cam_algorithms::{CutterShape, InverseOffsetShape};
    use geo_algorithms::{Point3D, TriangleMesh3D};
    use geo_contracts::TriangleMesh3DConstructor;

    #[test]
    fn meshes_keep_the_outward_normals_of_the_parts() {
        // 張り出しの下の面（外向きは下）を持つ、開いた形
        let profile = [(0.0, 4.0), (10.0, 4.0), (10.0, 3.0), (6.0, 3.0)];
        let mut vertices = Vec::new();
        for (x, z) in profile {
            vertices.push(Point3D::new(x, 0.0, z));
            vertices.push(Point3D::new(x, 10.0, z));
        }
        let triangles = (0..profile.len() - 1)
            .flat_map(|k| {
                let (a, b, c, d) = (2 * k, 2 * k + 1, 2 * k + 2, 2 * k + 3);
                [[a, c, d], [a, d, b]]
            })
            .collect();
        let mesh = TriangleMesh3D::new(vertices, triangles).unwrap();
        let shape = InverseOffsetShape::new(&mesh, CutterShape::Ball { radius: 1.0 }).unwrap();
        let meshes = inverse_offset_shape_meshes(&shape, 10.0_f64.to_radians()).unwrap();
        let normals_z = |kind| {
            meshes
                .iter()
                .find(|mesh| mesh.kind == kind)
                .unwrap()
                .vertices
                .iter()
                .map(|vertex| vertex.normal[2])
                .collect::<Vec<_>>()
        };
        assert!(normals_z(InverseOffsetPartKind::IndexedFace)
            .iter()
            .all(|&z| z > 0.99));
        // 下の面は外向き（下）、その上側の要素は上を向く
        assert!(normals_z(InverseOffsetPartKind::UnindexedFace)
            .iter()
            .any(|&z| z < -0.99));
        assert!(normals_z(InverseOffsetPartKind::UpperSide)
            .iter()
            .all(|&z| z > 0.99));
        for mesh in &meshes {
            assert_eq!(mesh.indices.len(), mesh.vertices.len());
            assert_eq!(mesh.vertices.len() % 3, 0);
        }
        assert!(inverse_offset_shape_meshes(&shape, 0.0).is_err());
    }
}
