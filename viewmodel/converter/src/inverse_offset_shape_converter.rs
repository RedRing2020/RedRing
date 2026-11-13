//! 逆オフセット形状のデバッグ表示用データ変換
//!
//! 逆オフセット形状の要素の役割ごとの三角形を、MeshStage で重ねて描けるシェーディングメッシュへ変換する。

use application::cam_simulation_workflow::InverseOffsetPart;
pub use application::cam_simulation_workflow::InverseOffsetPartKind;

use crate::mesh_converter::VertexData;

/// 種類ごとのシェーディングメッシュ（三角形ごとに独立した頂点・面法線。法線は要素の外の側を向く）
#[derive(Debug, Clone)]
pub struct InverseOffsetShapeMesh {
    pub kind: InverseOffsetPartKind,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
}

/// 逆オフセット形状の種類ごとの表示用の三角形 `parts` を、種類ごとのメッシュにする
pub fn inverse_offset_shape_meshes(parts: Vec<InverseOffsetPart>) -> Vec<InverseOffsetShapeMesh> {
    parts
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
        .collect()
}

/// 三角形の、頂点の順（右ねじ）の単位法線（面積のない三角形は上向き）
fn normal(a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let length = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if length == 0.0 {
        return [0.0, 0.0, 1.0];
    }
    n.map(|component| (component / length) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meshes_hold_each_triangle_with_its_face_normal_by_the_vertex_order() {
        // 上向き（右ねじで +Z）と下向きの三角形
        let up = [[0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [0.0, 1.0, 1.0]];
        let down = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let meshes = inverse_offset_shape_meshes(vec![InverseOffsetPart {
            kind: InverseOffsetPartKind::UpperSide,
            triangles: vec![up, down],
        }]);
        let [mesh] = meshes.as_slice() else {
            panic!("one mesh per part");
        };
        assert_eq!(mesh.kind, InverseOffsetPartKind::UpperSide);
        assert_eq!(mesh.indices, (0..6).collect::<Vec<u32>>());
        assert!(
            mesh.vertices[..3]
                .iter()
                .all(|vertex| vertex.normal == [0.0, 0.0, 1.0])
        );
        assert!(
            mesh.vertices[3..]
                .iter()
                .all(|vertex| vertex.normal == [0.0, 0.0, -1.0])
        );
        assert_eq!(mesh.vertices[1].position, [1.0, 0.0, 1.0]);
    }

    #[test]
    fn triangles_without_area_get_an_upward_normal() {
        let line = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let meshes = inverse_offset_shape_meshes(vec![InverseOffsetPart {
            kind: InverseOffsetPartKind::UpperSide,
            triangles: vec![line],
        }]);
        assert!(
            meshes[0]
                .vertices
                .iter()
                .all(|vertex| vertex.normal == [0.0, 0.0, 1.0])
        );
    }
}
