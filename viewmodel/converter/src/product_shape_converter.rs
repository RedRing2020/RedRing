//! 製品形状のエンティティの描画用の変換と、取り込んだ製品形状の保持

use std::path::Path;

use application::product_shape::{ProductShapeError, ProductShapeStore};
use geo_contracts::{EntityDisplayProperties, FaceSetEntity3DProperties, TriangleMesh3DProperties};

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

/// 面を持つエンティティを、すべての面をつなげた描画用のメッシュにする
///
/// # Errors
///
/// 面の表示用の三角形を作れない場合。
pub fn face_set_entity_to_mesh<E>(
    entity: &E,
    chord_tolerance: f64,
) -> Result<ProductShapeMesh, E::DisplayMeshError>
where
    E: FaceSetEntity3DProperties<f64>,
{
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for face_id in entity.face_ids() {
        let mesh = entity.face_display_mesh(face_id, chord_tolerance)?;
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
    Ok(ProductShapeMesh {
        vertices,
        indices,
        color: entity.entity_color(),
    })
}

/// 取り込んだ製品形状と、その表示
#[derive(Debug, Default)]
pub struct ProductShapeScene {
    store: ProductShapeStore,
    settings: ProductShapeDisplaySettings,
}

impl ProductShapeScene {
    pub fn new(settings: ProductShapeDisplaySettings) -> Self {
        Self {
            store: ProductShapeStore::new(),
            settings,
        }
    }

    /// 保持している製品形状
    pub fn store(&self) -> &ProductShapeStore {
        &self.store
    }

    /// STL ファイルを製品形状として取り込む
    ///
    /// # Errors
    ///
    /// STL の読み込みの失敗。
    pub fn import_stl(&mut self, path: &Path) -> Result<(), ProductShapeError> {
        self.store.import_stl(path).map(|_| ())
    }

    /// 表示中のエンティティの描画用のメッシュ（表示用の三角形を作れないエンティティは除き、警告を出す）
    pub fn visible_meshes(&self) -> Vec<ProductShapeMesh> {
        self.store
            .entities()
            .iter()
            .filter(|entity| entity.entity_visible())
            .filter_map(|entity| {
                face_set_entity_to_mesh(entity, self.settings.chord_tolerance)
                    .inspect_err(|error| {
                        tracing::warn!(entity_id = %entity.id(), "product shape is not displayed: {error}");
                    })
                    .ok()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use application::product_shape::ProductFaceGeometry;
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
        assert!(surface
            .indices
            .iter()
            .all(|&index| (index as usize) < surface.vertices.len()));
        for mesh in [&mesh, &surface] {
            assert!(mesh
                .vertices
                .iter()
                .all(|vertex| unit_length(vertex.normal)));
        }
    }

    #[test]
    fn imported_stl_is_displayed_until_it_is_hidden() {
        let path = std::env::temp_dir().join(format!(
            "redring_product_shape_scene_{}.stl",
            std::process::id()
        ));
        geo_io::stl::save_stl(&pyramid(), &path).unwrap();
        let mut scene = ProductShapeScene::default();
        scene.import_stl(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        let meshes = scene.visible_meshes();
        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].indices.len(), 12);

        let entity_id = scene.store.entities()[0].id();
        scene.store.set_visible(entity_id, false).unwrap();
        assert!(scene.visible_meshes().is_empty());
    }
}
