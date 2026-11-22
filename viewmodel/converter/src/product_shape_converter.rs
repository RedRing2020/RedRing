//! 製品形状のエンティティの描画用の変換

use application::product_shape::{FaceSetEntity3DProperties, TriangleMesh3DProperties};

use crate::mesh_converter::VertexData;

/// 製品形状の表示の設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProductShapeDisplaySettings {
    /// 曲面を表示用の三角形に離散化する許容弦誤差（mm）
    pub chord_tolerance: f64,
}

impl Default for ProductShapeDisplaySettings {
    fn default() -> Self {
        Self {
            chord_tolerance: 0.05,
        }
    }
}

/// 1 つのエンティティの描画用のメッシュ（エンティティのすべての面をつなげる）
#[derive(Debug, Clone)]
pub struct ProductShapeMesh {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    /// エンティティの表示色（透明度を含む）
    pub color: [f32; 4],
}

/// 面を持つエンティティを、すべての面をつなげた描画用のメッシュにする（表示用の三角形を作れない面は除く。面ごとの
/// 理由は、三角形を作る Model 側が一度だけ警告に出す）
///
/// # Errors
///
/// どの面の表示用の三角形も作れない場合（初めの面の理由）。
pub fn face_set_entity_to_mesh<E>(
    entity: &E,
    chord_tolerance: f64,
) -> Result<ProductShapeMesh, E::DisplayMeshError>
where
    E: FaceSetEntity3DProperties<f64>,
{
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut first_error = None;
    let mut displayed = false;
    for face_id in entity.face_ids() {
        let mesh = match entity.face_display_mesh(face_id, chord_tolerance) {
            Ok(mesh) => mesh,
            Err(error) => {
                first_error.get_or_insert(error);
                continue;
            }
        };
        displayed = true;
        let offset = vertices.len() as u32;
        let normals = mesh.normals();
        vertices.extend(mesh.vertices().iter().enumerate().map(|(index, point)| {
            let normal = normals.map_or([0.0; 3], |normals| {
                let normal = normals[index];
                [normal.x() as f32, normal.y() as f32, normal.z() as f32]
            });
            VertexData::new(
                [point.x() as f32, point.y() as f32, point.z() as f32],
                normal,
            )
        }));
        indices.extend(
            mesh.indices()
                .iter()
                .flat_map(|triangle| triangle.map(|index| offset + index as u32)),
        );
    }
    if let (false, Some(error)) = (displayed, first_error) {
        return Err(error);
    }
    Ok(ProductShapeMesh {
        vertices,
        indices,
        color: entity.entity_color(),
    })
}

#[cfg(test)]
mod tests {
    use application::product_shape::{
        EntityDisplayProperties, ProductFaceGeometry, ProductShapeStore,
    };
    use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D};
    use geo_contracts::{NurbsSurface3DConstructor, TriangleMesh3DConstructor};

    use super::*;

    fn pyramid() -> TriangleMesh3D<f64> {
        TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(10.0, 0.0, 0.0),
                Point3D::new(10.0, 10.0, 0.0),
                Point3D::new(0.0, 10.0, 0.0),
                Point3D::new(5.0, 5.0, 8.0),
            ],
            vec![[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
        )
        .unwrap()
    }

    fn dome() -> NurbsSurface3D<f64> {
        let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let row = |x: f64, middle: f64| {
            vec![
                Point3D::new(x, 0.0, 0.0),
                Point3D::new(x, 10.0, middle),
                Point3D::new(x, 20.0, 0.0),
            ]
        };
        NurbsSurface3D::new(
            2,
            2,
            knots.clone(),
            knots,
            &[row(0.0, 0.0), row(10.0, 5.0), row(20.0, 0.0)],
            None,
        )
        .unwrap()
    }

    fn unit_length(normal: [f32; 3]) -> bool {
        let length = normal.iter().map(|value| value * value).sum::<f32>().sqrt();
        (length - 1.0).abs() < 1e-5
    }

    #[test]
    fn stl_and_surface_faces_are_converted_through_the_entity_trait() {
        let mut store = ProductShapeStore::new();
        store.add_face(ProductFaceGeometry::Mesh(pyramid()));
        store.add_face(ProductFaceGeometry::Surface(dome()));
        let [mesh_entity, surface_entity] = store.entities() else {
            panic!("two entities are held");
        };

        let mesh = face_set_entity_to_mesh(mesh_entity, 0.05).unwrap();
        assert_eq!(mesh.vertices.len(), 12);
        assert_eq!(mesh.indices.len(), 12);
        assert_eq!(mesh.color, mesh_entity.entity_color());

        let surface = face_set_entity_to_mesh(surface_entity, 0.05).unwrap();
        assert!(surface.indices.len() > 6);
        assert!(
            surface
                .indices
                .iter()
                .all(|&index| (index as usize) < surface.vertices.len())
        );
        for mesh in [&mesh, &surface] {
            assert!(
                mesh.vertices
                    .iter()
                    .all(|vertex| unit_length(vertex.normal))
            );
        }
    }
    /// 面ごとに、表示用の三角形を作れるか（`true`）を決めたエンティティ
    struct Faces(Vec<bool>);

    impl geo_contracts::EntityIdentity for Faces {
        type Id = u32;

        fn entity_id(&self) -> u32 {
            0
        }
    }

    impl EntityDisplayProperties for Faces {
        fn entity_visible(&self) -> bool {
            true
        }

        fn entity_color(&self) -> [f32; 4] {
            [1.0; 4]
        }
    }

    impl FaceSetEntity3DProperties<f64> for Faces {
        type FaceId = usize;
        type DisplayMesh = TriangleMesh3D<f64>;
        type DisplayMeshError = String;

        fn face_ids(&self) -> Vec<usize> {
            (0..self.0.len()).collect()
        }

        fn face_display_mesh(&self, face_id: usize, _: f64) -> Result<TriangleMesh3D<f64>, String> {
            if self.0[face_id] {
                Ok(pyramid())
            } else {
                Err(format!("face {face_id}"))
            }
        }
    }

    #[test]
    fn faces_without_triangles_are_left_out() {
        // 作れない面だけを除く
        let mesh = face_set_entity_to_mesh(&Faces(vec![false, true, false]), 0.05).unwrap();
        assert_eq!(mesh.indices.len(), 12);
        // どの面も作れなければ、初めの面の理由
        assert_eq!(
            face_set_entity_to_mesh(&Faces(vec![false, false]), 0.05).unwrap_err(),
            "face 0"
        );
    }
}
