//! 製品形状のエンティティの保持・取り込みと、CAM の入力の形状への解決（設計: PRODUCT_SHAPE_ENTITY_DESIGN.md）。
//!
//! 製品形状のエンティティは、シェルと面単体を別の種類とする。面単体は、トリムなしの NURBS 曲面 1 枚、または
//! 三角形メッシュ 1 つを持つ。シェルは B-rep の表（読み込んだ立体・シェル）を持ち、同じ部品の出現は表を共有して、
//! 配置と面の ID の表を出現ごとに持つ（設計: BREP_AND_STEP_IMPORT_DESIGN.md 10.1・10.3）。面は全体で一意な面の
//! ID（`FaceId`）を持ち、面への参照は面の ID だけで指す。
//!
//! シェルの面の表示用の三角形は、B-rep の全体を一度に分けて（隣の面と境界の頂点をそろえる）、B-rep の表と一緒に
//! 共有する置き場に許容弦誤差ごとに残し、同じ部品の出現で使い回す（設計: TRIMMED_FACE_TESSELLATION_DESIGN.md 7 章）。

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use cam_algorithms::SolverGeometry;
/// 製品形状の面の形（NURBS 曲面・三角形メッシュ）の型
pub use geo_algorithms::{NurbsSurface3D, TriangleMesh3D};
use geo_algorithms::{
    SurfaceTessellationError, TessellationLimits, Vector3D, concatenate_triangle_meshes,
    tessellate_surfaces,
};
/// 製品形状のエンティティの表示・面と、面の表示用のメッシュのメソッドを呼ぶための trait
pub use geo_contracts::{
    EntityDisplayProperties, FaceSetEntity3DProperties, TriangleMesh3DProperties,
};
use geo_contracts::{EntityIdentity, SimilarityTransformable3D, TriangleMesh3DConstructor};
use geo_entity::{Attributes, DisplayAttributes, EntityId, FaceId, GeometricEntity, Metadata};
/// シェルの B-rep の表と、配置の型
pub use geo_primitives::SimilarityTransform3D;
pub use geo_topology::Brep;
/// シェルの面を三角形にできない理由
pub use shell_mesh::ShellFaceError;
use shell_mesh::{ShellMesh, triangulate_brep};
/// STEP の読み込みの結果・読めない理由・属性のキー
pub use step_import::{
    OCCURRENCE_NAMES_KEY, PRODUCT_NAME_KEY, SOLID_NAME_KEY, StepImportError, StepImportLimit,
    StepImportReport,
};

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

    pub(crate) fn geometry(&self) -> &ProductFaceGeometry {
        &self.geometry
    }
}

/// 面単体の製品形状のエンティティ
#[derive(Debug, Clone)]
pub struct ProductFaceEntity(GeometricEntity<f64, ProductFace>);

impl ProductFaceEntity {
    pub fn id(&self) -> EntityId {
        self.0.id()
    }

    pub(crate) fn face(&self) -> &ProductFace {
        self.0.geometry()
    }

    pub(crate) fn display(&self) -> &DisplayAttributes {
        self.0.display()
    }
}

impl EntityIdentity for ProductFaceEntity {
    type Id = EntityId;

    fn entity_id(&self) -> EntityId {
        self.0.id()
    }
}

impl EntityDisplayProperties for ProductFaceEntity {
    fn entity_visible(&self) -> bool {
        self.0.entity_visible()
    }

    fn entity_color(&self) -> [f32; 4] {
        self.0.entity_color()
    }
}

impl FaceSetEntity3DProperties<f64> for ProductFaceEntity {
    type FaceId = FaceId;
    type DisplayMesh = TriangleMesh3D<f64>;
    type DisplayMeshError = ProductShapeError;

    fn face_ids(&self) -> Vec<FaceId> {
        vec![self.face().id()]
    }

    fn face_display_mesh(
        &self,
        face_id: FaceId,
        chord_tolerance: f64,
    ) -> Result<TriangleMesh3D<f64>, ProductShapeError> {
        if face_id != self.face().id() {
            return Err(ProductShapeError::UnknownFace(face_id));
        }
        match self.face().geometry() {
            ProductFaceGeometry::Surface(surface) => tessellate_surfaces(
                std::slice::from_ref(surface),
                chord_tolerance,
                TessellationLimits::default(),
            )
            .map_err(ProductShapeError::Tessellation),
            ProductFaceGeometry::Mesh(mesh) => Ok(with_triangle_normals(mesh)),
        }
    }
}

/// 頂点法線を持たないメッシュは、三角形ごとに頂点を分け、三角形の法線を頂点法線とする
fn with_triangle_normals(mesh: &TriangleMesh3D<f64>) -> TriangleMesh3D<f64> {
    if mesh.normals().is_some() {
        return mesh.clone();
    }
    let source = mesh.vertices();
    let mut vertices = Vec::with_capacity(mesh.triangle_count() * 3);
    let mut normals = Vec::with_capacity(mesh.triangle_count() * 3);
    let mut indices = Vec::with_capacity(mesh.triangle_count());
    for &[a, b, c] in mesh.indices() {
        let normal = Vector3D::from_points(source[a], source[b])
            .cross(Vector3D::from_points(source[a], source[c]))
            .normalize();
        let first = vertices.len();
        vertices.extend([source[a], source[b], source[c]]);
        normals.extend([normal; 3]);
        indices.push([first, first + 1, first + 2]);
    }
    TriangleMesh3D::with_normals(vertices, indices, normals)
        .expect("one normal is given for each vertex of valid triangles")
}

/// シェルの元の要素（読み込んだファイルの中の対応）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSource {
    /// ファイルの名前
    pub file_name: String,
    /// アセンブリの出現の並び（根から、出現のエンティティの番号）
    pub occurrences: Vec<u64>,
    /// 面ごとの、元の面のエンティティの番号（B-rep の面の順）
    pub faces: Vec<u64>,
}

/// 置き場に残す B-rep の三角形の数（許容弦誤差の違う組。表示と CAM の入力で 1 つずつ）
const CACHED_SHELL_MESHES: usize = 2;

/// B-rep の三角形の置き場（許容弦誤差のビット列と三角形。古い順。同じ B-rep の表を持つシェルで共有する）
#[derive(Default)]
struct ShellMeshCache(Mutex<Vec<(u64, Arc<ShellMesh>)>>);

impl fmt::Debug for ShellMeshCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 三角形の点の並びは大きいので出さない
        f.debug_struct("ShellMeshCache").finish_non_exhaustive()
    }
}

/// シェル（B-rep の表と配置、B-rep の面の順の面の ID、元の要素）
#[derive(Debug, Clone)]
pub(crate) struct ProductShell {
    brep: Arc<Brep<f64>>,
    placement: SimilarityTransform3D<f64>,
    face_ids: Vec<FaceId>,
    /// 面の ID から、B-rep の面の順の位置
    face_positions: HashMap<FaceId, usize>,
    source: Option<ShellSource>,
    meshes: Arc<ShellMeshCache>,
}

impl ProductShell {
    fn new(
        brep: Arc<Brep<f64>>,
        placement: SimilarityTransform3D<f64>,
        face_ids: Vec<FaceId>,
        source: Option<ShellSource>,
        meshes: Arc<ShellMeshCache>,
    ) -> Self {
        let face_positions = face_ids
            .iter()
            .enumerate()
            .map(|(position, &id)| (id, position))
            .collect();
        Self {
            brep,
            placement,
            face_ids,
            face_positions,
            source,
            meshes,
        }
    }

    /// 許容弦誤差 `chord_tolerance` の B-rep の三角形（置き場になければ作って残し、三角形にできない面を警告に
    /// 出す。置き場が一杯なら、古いものを捨てる）
    fn mesh(&self, chord_tolerance: f64, entity_id: EntityId) -> Arc<ShellMesh> {
        let key = chord_tolerance.to_bits();
        let mut meshes = self.meshes.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((_, mesh)) = meshes.iter().find(|(cached, _)| *cached == key) {
            return Arc::clone(mesh);
        }
        let mesh = Arc::new(triangulate_brep(&self.brep, chord_tolerance));
        for (face_id, face) in self.face_ids.iter().zip(&mesh.faces) {
            if let Err(error) = face {
                tracing::warn!(%entity_id, %face_id, "a shell face is not displayed: {error}");
            }
        }
        if meshes.len() == CACHED_SHELL_MESHES {
            meshes.remove(0);
        }
        meshes.push((key, Arc::clone(&mesh)));
        mesh
    }
}

/// 同じ B-rep の表を持つシェルが `shells` にあれば、その三角形の置き場。なければ新しい置き場
fn shared_meshes<'a>(
    mut shells: impl Iterator<Item = &'a ProductShellEntity>,
    brep: &Arc<Brep<f64>>,
) -> Arc<ShellMeshCache> {
    shells
        .find(|shell| Arc::ptr_eq(&shell.shell().brep, brep))
        .map_or_else(Arc::default, |shell| Arc::clone(&shell.shell().meshes))
}

/// シェルの製品形状のエンティティ
#[derive(Debug, Clone)]
pub struct ProductShellEntity(GeometricEntity<f64, ProductShell>);

impl ProductShellEntity {
    pub fn id(&self) -> EntityId {
        self.0.id()
    }

    fn shell(&self) -> &ProductShell {
        self.0.geometry()
    }
}

impl EntityIdentity for ProductShellEntity {
    type Id = EntityId;

    fn entity_id(&self) -> EntityId {
        self.0.id()
    }
}

impl EntityDisplayProperties for ProductShellEntity {
    fn entity_visible(&self) -> bool {
        self.0.entity_visible()
    }

    fn entity_color(&self) -> [f32; 4] {
        self.0.entity_color()
    }
}

impl FaceSetEntity3DProperties<f64> for ProductShellEntity {
    type FaceId = FaceId;
    type DisplayMesh = TriangleMesh3D<f64>;
    type DisplayMeshError = ProductShapeError;

    fn face_ids(&self) -> Vec<FaceId> {
        self.shell().face_ids.clone()
    }

    /// 面 `face_id` の三角形を、配置を当てて返す（B-rep の全体を一度に分けた三角形の、その面の分）
    fn face_display_mesh(
        &self,
        face_id: FaceId,
        chord_tolerance: f64,
    ) -> Result<TriangleMesh3D<f64>, ProductShapeError> {
        let shell = self.shell();
        let position = *shell
            .face_positions
            .get(&face_id)
            .ok_or(ProductShapeError::UnknownFace(face_id))?;
        let mesh = shell.mesh(chord_tolerance, self.id());
        let face = mesh.faces[position]
            .as_ref()
            .map_err(|error| ProductShapeError::ShellFace(*error))?;
        let vertices = face
            .vertices
            .iter()
            .map(|&index| mesh.points[index])
            .collect();
        TriangleMesh3D::with_normals(vertices, face.triangles.clone(), face.normals.clone())
            .expect("the triangles of a face index its vertices, with one normal each")
            .transform_similarity(&shell.placement)
            .map_err(|_| ProductShapeError::ShellFace(ShellFaceError::Placement))
    }
}

/// 加えた面単体のエンティティと、その面
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddedProductFace {
    pub entity_id: EntityId,
    pub face_id: FaceId,
}

/// 製品形状の保持・取り込み・解決の失敗
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductShapeError {
    /// STL のファイルを読めない（誤りの種類と理由）
    StlFile {
        kind: std::io::ErrorKind,
        reason: String,
    },
    /// STL の内容を形状にできない（理由）
    Stl(String),
    /// 保持していないエンティティ
    UnknownEntity(EntityId),
    /// 保持していない面
    UnknownFace(FaceId),
    /// NURBS 曲面と三角形メッシュの面が混ざっている
    MixedFaceKinds,
    /// 表示用の三角形への離散化の失敗
    Tessellation(SurfaceTessellationError),
    /// 面がない
    NoFaces,
    /// シェルの元の要素の面の数が、B-rep の面の数と合わない
    SourceFaceCountMismatch,
    /// シェルの面を三角形にできない
    ShellFace(ShellFaceError),
    /// STEP のファイルを読めない（誤りの種類と理由）
    StepFile {
        kind: std::io::ErrorKind,
        reason: String,
    },
    /// STEP の内容を形状にできない
    Step(StepImportError),
}

impl fmt::Display for ProductShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StlFile { reason, .. } => write!(f, "failed to read the STL file: {reason}"),
            Self::Stl(reason) => write!(f, "failed to load STL: {reason}"),
            Self::UnknownEntity(id) => write!(f, "unknown product shape entity: {id}"),
            Self::UnknownFace(id) => write!(f, "unknown product face: {id}"),
            Self::MixedFaceKinds => write!(f, "NURBS surface faces and mesh faces are mixed"),
            Self::Tessellation(error) => write!(f, "failed to tessellate a surface face: {error}"),
            Self::NoFaces => write!(f, "no faces are given"),
            Self::SourceFaceCountMismatch => {
                write!(f, "the source faces do not match the faces of the B-rep")
            }
            Self::ShellFace(error) => write!(f, "failed to triangulate a shell face: {error}"),
            Self::StepFile { reason, .. } => write!(f, "failed to read the STEP file: {reason}"),
            Self::Step(error) => write!(f, "failed to load STEP: {error}"),
        }
    }
}

impl std::error::Error for ProductShapeError {}

/// 面単体のエンティティの記録（保管先から取り出し、保管先に戻す中身。選択の状態は持たない）
#[derive(Debug, Clone)]
pub struct ProductFaceRecord {
    pub entity_id: EntityId,
    pub face_id: FaceId,
    pub geometry: ProductFaceGeometry,
    pub display: DisplayAttributes,
    pub attributes: Attributes,
    pub metadata: Metadata,
}

/// シェルのエンティティの記録（保管先から取り出し、保管先に戻す中身。選択の状態は持たない）
#[derive(Debug, Clone)]
pub struct ProductShellRecord {
    pub entity_id: EntityId,
    /// B-rep の表（同じ部品の出現は、同じ表を共有する）
    pub brep: Arc<Brep<f64>>,
    pub placement: SimilarityTransform3D<f64>,
    /// 面の ID（B-rep の面の順）
    pub face_ids: Vec<FaceId>,
    pub source: Option<ShellSource>,
    pub display: DisplayAttributes,
    pub attributes: Attributes,
    pub metadata: Metadata,
}

/// 記録から製品形状の保管先に戻すときの誤り
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductShapeRestoreError {
    /// エンティティの ID が重なっている
    DuplicateEntity(EntityId),
    /// 面の ID が重なっている
    DuplicateFace(FaceId),
    /// シェルの面の ID・元の要素の面の数が、B-rep の面の数と合わない
    FaceCountMismatch(EntityId),
}

impl fmt::Display for ProductShapeRestoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateEntity(id) => write!(f, "duplicate product shape entity: {id}"),
            Self::DuplicateFace(id) => write!(f, "duplicate product face: {id}"),
            Self::FaceCountMismatch(id) => {
                write!(f, "the faces of product shell {id} do not match its B-rep")
            }
        }
    }
}

impl std::error::Error for ProductShapeRestoreError {}

/// 製品形状のエンティティの保管先（面単体とシェルを、それぞれ加えた順に保持する）
#[derive(Debug, Default)]
pub struct ProductShapeStore {
    entities: Vec<ProductFaceEntity>,
    shells: Vec<ProductShellEntity>,
}

impl ProductShapeStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// 面単体の記録 `records` とシェルの記録 `shells` を、その順に持つ保管先を作る（ID は記録のまま）
    ///
    /// # Errors
    ///
    /// エンティティの ID、または面の ID が（面単体とシェルをまたいでも）重なっている場合、シェルの面の ID・元の
    /// 要素の面の数が、B-rep の面の数と合わない場合。
    pub fn from_records(
        records: Vec<ProductFaceRecord>,
        shells: Vec<ProductShellRecord>,
    ) -> Result<Self, ProductShapeRestoreError> {
        let mut entity_ids = HashSet::new();
        let mut face_ids = HashSet::new();
        let mut entities = Vec::with_capacity(records.len());
        for record in records {
            if !entity_ids.insert(record.entity_id) {
                return Err(ProductShapeRestoreError::DuplicateEntity(record.entity_id));
            }
            if !face_ids.insert(record.face_id) {
                return Err(ProductShapeRestoreError::DuplicateFace(record.face_id));
            }
            entities.push(ProductFaceEntity(GeometricEntity::restore(
                record.entity_id,
                ProductFace {
                    id: record.face_id,
                    geometry: record.geometry,
                },
                record.display,
                record.attributes,
                record.metadata,
            )));
        }
        let mut shell_entities = Vec::with_capacity(shells.len());
        // 同じ B-rep の表（同じ部品の出現）の三角形の置き場
        let mut shared: HashMap<*const Brep<f64>, Arc<ShellMeshCache>> = HashMap::new();
        for record in shells {
            if !entity_ids.insert(record.entity_id) {
                return Err(ProductShapeRestoreError::DuplicateEntity(record.entity_id));
            }
            let face_count = record.brep.face_count();
            let source_matches = record
                .source
                .as_ref()
                .is_none_or(|source| source.faces.len() == face_count);
            if record.face_ids.len() != face_count || !source_matches {
                return Err(ProductShapeRestoreError::FaceCountMismatch(
                    record.entity_id,
                ));
            }
            if let Some(&duplicate) = record.face_ids.iter().find(|&&id| !face_ids.insert(id)) {
                return Err(ProductShapeRestoreError::DuplicateFace(duplicate));
            }
            let meshes = Arc::clone(shared.entry(Arc::as_ptr(&record.brep)).or_default());
            shell_entities.push(ProductShellEntity(GeometricEntity::restore(
                record.entity_id,
                ProductShell::new(
                    record.brep,
                    record.placement,
                    record.face_ids,
                    record.source,
                    meshes,
                ),
                record.display,
                record.attributes,
                record.metadata,
            )));
        }
        Ok(Self {
            entities,
            shells: shell_entities,
        })
    }

    /// 保持しているシェルのエンティティの記録（加えた順）
    pub fn shell_records(&self) -> Vec<ProductShellRecord> {
        self.shells
            .iter()
            .map(|entity| {
                let shell = entity.shell();
                ProductShellRecord {
                    entity_id: entity.id(),
                    brep: Arc::clone(&shell.brep),
                    placement: shell.placement,
                    face_ids: shell.face_ids.clone(),
                    source: shell.source.clone(),
                    display: entity.0.display().clone(),
                    attributes: entity.0.attributes().clone(),
                    metadata: entity.0.metadata().clone(),
                }
            })
            .collect()
    }

    /// B-rep の表 `brep` を配置 `placement` で置くシェルのエンティティを加える（面ごとに新しい面の ID を作る。
    /// 同じ部品の出現は、同じ表を渡して共有する。配置は、合成の丸めで回転が直交からずれていても、直交に直して
    /// 持つ）
    ///
    /// # Errors
    ///
    /// 元の要素 `source` の面の数が、B-rep の面の数と合わない場合。
    pub fn add_shell(
        &mut self,
        brep: Arc<Brep<f64>>,
        placement: SimilarityTransform3D<f64>,
        source: Option<ShellSource>,
    ) -> Result<EntityId, ProductShapeError> {
        let face_count = brep.face_count();
        if source
            .as_ref()
            .is_some_and(|source| source.faces.len() != face_count)
        {
            return Err(ProductShapeError::SourceFaceCountMismatch);
        }
        let meshes = shared_meshes(self.shells.iter(), &brep);
        Ok(self.push_shell(brep, placement, source, Metadata::default(), meshes))
    }

    /// シェルのエンティティを、三角形の置き場 `meshes` と一緒に加える（面ごとに新しい面の ID を作る。面の数は
    /// 確かめてある）
    fn push_shell(
        &mut self,
        brep: Arc<Brep<f64>>,
        placement: SimilarityTransform3D<f64>,
        source: Option<ShellSource>,
        metadata: Metadata,
        meshes: Arc<ShellMeshCache>,
    ) -> EntityId {
        let face_count = brep.face_count();
        let entity = ProductShellEntity(GeometricEntity::restore(
            EntityId::new_random(),
            ProductShell::new(
                brep,
                placement.renormalized(),
                (0..face_count).map(|_| FaceId::new_random()).collect(),
                source,
                meshes,
            ),
            DisplayAttributes::default(),
            Attributes::new(),
            metadata,
        ));
        let id = entity.id();
        self.shells.push(entity);
        id
    }

    /// STEP ファイルを読み、アセンブリの出現ごとにシェルのエンティティとして加える（同じ部品の B-rep は共有する。
    /// 外した要素・直したことは、要素ごとにログに出す）
    ///
    /// # Errors
    ///
    /// ファイルを読めない、構文の誤り、上限を超える、形状が 1 つも読めない場合（何も加えない）。
    pub fn import_step(&mut self, path: &Path) -> Result<StepImportReport, ProductShapeError> {
        let file_error = |error: std::io::Error| ProductShapeError::StepFile {
            kind: error.kind(),
            reason: error.to_string(),
        };
        let size = std::fs::metadata(path).map_err(file_error)?.len();
        let limits = geo_step::syntax::Limits::default();
        if size > limits.max_bytes as u64 {
            return Err(ProductShapeError::Step(StepImportError::TooLarge));
        }
        let bytes = std::fs::read(path).map_err(file_error)?;
        let file_name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        let (shells, report) =
            step_import::read_step(&bytes, &file_name).map_err(ProductShapeError::Step)?;
        // 読み込んだ B-rep は保管先のどのシェルとも共有しないので、同じ部品の出現の間でだけ置き場を共有する
        let mut shared: HashMap<*const Brep<f64>, Arc<ShellMeshCache>> = HashMap::new();
        for shell in shells {
            let meshes = Arc::clone(shared.entry(Arc::as_ptr(&shell.brep)).or_default());
            self.push_shell(
                shell.brep,
                shell.placement,
                Some(shell.source),
                shell.metadata,
                meshes,
            );
        }
        Ok(report)
    }

    /// 保持しているエンティティの記録（加えた順）
    pub fn records(&self) -> Vec<ProductFaceRecord> {
        self.entities
            .iter()
            .map(|entity| ProductFaceRecord {
                entity_id: entity.id(),
                face_id: entity.face().id(),
                geometry: entity.face().geometry().clone(),
                display: entity.display().clone(),
                attributes: entity.0.attributes().clone(),
                metadata: entity.0.metadata().clone(),
            })
            .collect()
    }

    /// 形状 `geometry` の面を 1 つ持つ面単体のエンティティを加える
    pub fn add_face(&mut self, geometry: ProductFaceGeometry) -> AddedProductFace {
        let face_id = FaceId::new_random();
        let entity = ProductFaceEntity(GeometricEntity::new(ProductFace {
            id: face_id,
            geometry,
        }));
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
        let mesh = geo_io::stl::load_stl::<f64>(path).map_err(|error| match &error {
            geo_io::StlError::Io(geo_io::IoError::FileSystem(io)) => ProductShapeError::StlFile {
                kind: io.kind(),
                reason: error.to_string(),
            },
            _ => ProductShapeError::Stl(error.to_string()),
        })?;
        if mesh.is_empty() {
            return Err(ProductShapeError::Stl(
                "the STL file has no triangles".to_string(),
            ));
        }
        Ok(self.add_face(ProductFaceGeometry::Mesh(mesh)))
    }

    /// 保持している面単体のエンティティ（加えた順）
    pub fn entities(&self) -> &[ProductFaceEntity] {
        &self.entities
    }

    /// 保持しているシェルのエンティティ（加えた順）
    pub fn shells(&self) -> &[ProductShellEntity] {
        &self.shells
    }

    /// エンティティ `entity_id`（面単体かシェル）の表示・非表示を切り替える
    ///
    /// # Errors
    ///
    /// エンティティを保持していない場合。
    pub fn set_visible(
        &mut self,
        entity_id: EntityId,
        visible: bool,
    ) -> Result<(), ProductShapeError> {
        if let Some(entity) = self
            .entities
            .iter_mut()
            .find(|entity| entity.id() == entity_id)
        {
            entity.0.display_mut().visible = visible;
            return Ok(());
        }
        let shell = self
            .shells
            .iter_mut()
            .find(|shell| shell.id() == entity_id)
            .ok_or(ProductShapeError::UnknownEntity(entity_id))?;
        shell.0.display_mut().visible = visible;
        Ok(())
    }

    /// 表示中のすべての面単体のエンティティの面の ID（加えた順。シェルの面は、トリムした面を CAM の入力にできる
    /// まで含めない）
    pub fn visible_face_ids(&self) -> Vec<FaceId> {
        self.entities
            .iter()
            .filter(|entity| entity.display().visible)
            .map(|entity| entity.face().id())
            .collect()
    }

    /// 面の ID の集合を、ツールパス生成の入力の形状にする
    ///
    /// 三角形メッシュの面は 1 つのメッシュにつなげ、NURBS 曲面の面は曲面の集合にする。同じ面の ID は 1 回だけ使う。
    ///
    /// # Errors
    ///
    /// 面がない場合、保持していない面がある場合、NURBS 曲面と三角形メッシュの面が混ざっている場合。
    pub(crate) fn resolve_solver_geometry(
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
                .map(ProductFaceEntity::face)
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

        let meshes = faces
            .into_iter()
            .map(|geometry| match geometry {
                ProductFaceGeometry::Mesh(mesh) => Ok(mesh),
                ProductFaceGeometry::Surface(_) => Err(ProductShapeError::MixedFaceKinds),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mesh = concatenate_triangle_meshes(&meshes);
        Ok(SolverGeometry::TriangleMesh(mesh))
    }
}

mod shell_mesh;
mod step_import;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_fixtures;
