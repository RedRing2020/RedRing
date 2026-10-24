//! 製品形状のエンティティの保持・取り込みと、CAM の入力の形状への解決（設計: PRODUCT_SHAPE_ENTITY_DESIGN.md）。
//!
//! 製品形状のエンティティは、シェルと面単体を別の種類とする。ここでは面単体（トリムなしの NURBS 曲面 1 枚、
//! または三角形メッシュ 1 つ）を扱う。面は全体で一意な面の ID（`FaceId`）を持ち、面への参照は面の ID だけで指す。

use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use cam_algorithms::SolverGeometry;
use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D};
use geo_contracts::{TriangleMesh3DConstructor, TriangleMesh3DProperties};
use geo_entity::{EntityId, FaceId, GeometricEntity};

/// 面の形状
#[derive(Debug, Clone)]
pub enum ProductFaceGeometry {
    /// トリムなしの NURBS 曲面
    Surface(NurbsSurface3D<f64>),
    /// 三角形メッシュ
    Mesh(TriangleMesh3D<f64>),
}

/// 面（面の ID と形状）
#[derive(Debug, Clone)]
pub struct ProductFace {
    id: FaceId,
    geometry: ProductFaceGeometry,
}

impl ProductFace {
    pub fn id(&self) -> FaceId {
        self.id
    }

    pub fn geometry(&self) -> &ProductFaceGeometry {
        &self.geometry
    }
}

/// 面単体の製品形状のエンティティ
pub type ProductFaceEntity = GeometricEntity<f64, ProductFace>;

/// 加えた面単体のエンティティと、その面
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddedProductFace {
    pub entity_id: EntityId,
    pub face_id: FaceId,
}

/// 製品形状の保持・取り込み・解決の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductShapeError {
    /// STL の読み込みの失敗（理由）
    Stl(String),
    /// 保持していないエンティティ
    UnknownEntity(EntityId),
    /// 保持していない面
    UnknownFace(FaceId),
    /// NURBS 曲面と三角形メッシュの面が混ざっている
    MixedFaceKinds,
    /// 面がない
    NoFaces,
}

impl fmt::Display for ProductShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stl(reason) => write!(f, "failed to load STL: {reason}"),
            Self::UnknownEntity(id) => write!(f, "unknown product shape entity: {id}"),
            Self::UnknownFace(id) => write!(f, "unknown product face: {id}"),
            Self::MixedFaceKinds => write!(f, "NURBS surface faces and mesh faces are mixed"),
            Self::NoFaces => write!(f, "no faces are given"),
        }
    }
}

impl std::error::Error for ProductShapeError {}

/// 製品形状のエンティティの保管先（加えた順に保持する）
#[derive(Debug, Default)]
pub struct ProductShapeStore {
    entities: Vec<ProductFaceEntity>,
}

impl ProductShapeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 形状 `geometry` の面を 1 つ持つ面単体のエンティティを加える
    pub fn add_face(&mut self, geometry: ProductFaceGeometry) -> AddedProductFace {
        let face_id = FaceId::new_random();
        let entity = GeometricEntity::new(ProductFace {
            id: face_id,
            geometry,
        });
        let added = AddedProductFace {
            entity_id: entity.id(),
            face_id,
        };
        self.entities.push(entity);
        added
    }

    /// STL ファイルを読み、面単体のエンティティ（メッシュ全体を面 1 つ）として加える
    ///
    /// # Errors
    ///
    /// STL の読み込みの失敗。
    pub fn import_stl(&mut self, path: &Path) -> Result<AddedProductFace, ProductShapeError> {
        let mesh = geo_io::stl::load_stl::<f64>(path)
            .map_err(|error| ProductShapeError::Stl(error.to_string()))?;
        Ok(self.add_face(ProductFaceGeometry::Mesh(mesh)))
    }

    /// 保持しているエンティティ（加えた順）
    pub fn entities(&self) -> &[ProductFaceEntity] {
        &self.entities
    }

    /// エンティティ `entity_id` の表示・非表示を切り替える
    ///
    /// # Errors
    ///
    /// エンティティを保持していない場合。
    pub fn set_visible(
        &mut self,
        entity_id: EntityId,
        visible: bool,
    ) -> Result<(), ProductShapeError> {
        let entity = self
            .entities
            .iter_mut()
            .find(|entity| entity.id() == entity_id)
            .ok_or(ProductShapeError::UnknownEntity(entity_id))?;
        entity.display_mut().visible = visible;
        Ok(())
    }

    /// 表示中のすべてのエンティティの面の ID（加えた順）
    pub fn visible_face_ids(&self) -> Vec<FaceId> {
        self.entities
            .iter()
            .filter(|entity| entity.display().visible)
            .map(|entity| entity.geometry().id())
            .collect()
    }

    /// 面の ID の集合を、ツールパス生成の入力の形状にする
    ///
    /// 三角形メッシュの面は 1 つのメッシュにつなげ、NURBS 曲面の面は曲面の集合にする。同じ面の ID は 1 回だけ使う。
    ///
    /// # Errors
    ///
    /// 面がない場合、保持していない面がある場合、NURBS 曲面と三角形メッシュの面が混ざっている場合。
    pub fn resolve_solver_geometry(
        &self,
        face_ids: &[FaceId],
    ) -> Result<SolverGeometry, ProductShapeError> {
        let mut seen = HashSet::new();
        let mut faces = Vec::new();
        for &face_id in face_ids {
            if !seen.insert(face_id) {
                continue;
            }
            let face = self
                .entities
                .iter()
                .map(GeometricEntity::geometry)
                .find(|face| face.id() == face_id)
                .ok_or(ProductShapeError::UnknownFace(face_id))?;
            faces.push(face.geometry());
        }
        if faces.is_empty() {
            return Err(ProductShapeError::NoFaces);
        }

        if faces
            .iter()
            .all(|geometry| matches!(geometry, ProductFaceGeometry::Surface(_)))
        {
            let surfaces = faces
                .into_iter()
                .filter_map(|geometry| match geometry {
                    ProductFaceGeometry::Surface(surface) => Some(surface.clone()),
                    ProductFaceGeometry::Mesh(_) => None,
                })
                .collect();
            return Ok(SolverGeometry::NurbsSurfaceSet(surfaces));
        }

        let mut vertices: Vec<Point3D<f64>> = Vec::new();
        let mut indices: Vec<[usize; 3]> = Vec::new();
        for geometry in faces {
            let ProductFaceGeometry::Mesh(mesh) = geometry else {
                return Err(ProductShapeError::MixedFaceKinds);
            };
            let offset = vertices.len();
            vertices.extend_from_slice(mesh.vertices());
            indices.extend(
                mesh.indices()
                    .iter()
                    .map(|triangle| triangle.map(|index| index + offset)),
            );
        }
        let mesh = TriangleMesh3D::new(vertices, indices)
            .expect("concatenating valid meshes keeps every index in range");
        Ok(SolverGeometry::TriangleMesh(mesh))
    }
}

#[cfg(test)]
mod tests;
