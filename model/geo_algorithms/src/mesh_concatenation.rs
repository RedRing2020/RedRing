//! 三角形メッシュの連結

use geo_contracts::{Scalar, TriangleMesh3DConstructor, TriangleMesh3DProperties};

use crate::TriangleMesh3D;

/// 複数の三角形メッシュを 1 つにつなげる（頂点番号は前のメッシュの頂点の数だけずらす）
///
/// すべてのメッシュが頂点法線を持つ場合だけ、頂点法線もつなげる。
pub fn concatenate_triangle_meshes<T: Scalar>(meshes: &[&TriangleMesh3D<T>]) -> TriangleMesh3D<T> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for mesh in meshes {
        let offset = vertices.len();
        vertices.extend_from_slice(mesh.vertices());
        indices.extend(
            mesh.indices()
                .iter()
                .map(|triangle| triangle.map(|index| index + offset)),
        );
    }
    let normals: Option<Vec<_>> = meshes.iter().map(|mesh| mesh.normals()).try_fold(
        Vec::new(),
        |mut normals, mesh_normals| {
            normals.extend_from_slice(mesh_normals?);
            Some(normals)
        },
    );
    match normals {
        Some(normals) => TriangleMesh3D::with_normals(vertices, indices, normals),
        None => TriangleMesh3D::new(vertices, indices),
    }
    .expect("concatenating valid meshes keeps every index and normal in range")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point3D, Vector3D};

    fn triangle(z: f64) -> TriangleMesh3D<f64> {
        TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, z),
                Point3D::new(1.0, 0.0, z),
                Point3D::new(0.0, 1.0, z),
            ],
            vec![[0, 1, 2]],
        )
        .unwrap()
    }

    fn with_up_normals(mesh: &TriangleMesh3D<f64>) -> TriangleMesh3D<f64> {
        TriangleMesh3D::with_normals(
            mesh.vertices().to_vec(),
            mesh.indices().to_vec(),
            vec![Vector3D::new(0.0, 0.0, 1.0); mesh.vertex_count()],
        )
        .unwrap()
    }

    #[test]
    fn indices_of_later_meshes_are_shifted() {
        let (first, second) = (triangle(0.0), triangle(2.0));
        let mesh = concatenate_triangle_meshes(&[&first, &second]);
        assert_eq!(mesh.vertex_count(), 6);
        assert_eq!(mesh.indices(), &[[0, 1, 2], [3, 4, 5]]);
        assert_eq!(mesh.vertices()[3].z(), 2.0);
        assert_eq!(mesh.normals(), None);
    }

    #[test]
    fn normals_are_kept_only_when_every_mesh_has_them() {
        let (first, second) = (
            with_up_normals(&triangle(0.0)),
            with_up_normals(&triangle(2.0)),
        );
        let mesh = concatenate_triangle_meshes(&[&first, &second]);
        assert_eq!(mesh.normals().map(<[_]>::len), Some(6));

        let without = triangle(4.0);
        assert_eq!(
            concatenate_triangle_meshes(&[&first, &without]).normals(),
            None
        );
        assert_eq!(concatenate_triangle_meshes::<f64>(&[]).vertex_count(), 0);
    }
}
