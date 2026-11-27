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
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

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
/// STEP の読み込みの結果・進み具合・読めない理由・属性のキー
pub use step_import::{
    OCCURRENCE_NAMES_KEY, PRODUCT_NAME_KEY, SOLID_NAME_KEY, StepImportError, StepImportLimit,
    StepImportProgress, StepImportReport, StepImportStage,
};
/// STEP の読み込みの読む処理と、読んだ形状・その表示用の三角形を作る仕事
pub use step_shapes::{DisplayMeshWork, StepShapes, read_step_file};

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

/// 置き場に残す B-rep の三角形の数（許容弦誤差の違う組。表示と、CAM の入力の精度プロファイルの 2 つで 1 つずつ）
const CACHED_SHELL_MESHES: usize = 3;

/// B-rep の三角形の置き場（同じ B-rep の表を持つシェルで共有する）
#[derive(Default)]
struct ShellMeshCache {
    state: Mutex<ShellMeshes>,
    /// 作っていた三角形を作り終えた（置き場に残した、または作れずにやめた）
    built: Condvar,
}

/// 置き場の中身
#[derive(Default)]
struct ShellMeshes {
    /// 許容弦誤差のビット列と三角形（古い順）
    meshes: Vec<(u64, Arc<ShellMesh>)>,
    /// 作っている三角形の、許容弦誤差のビット列
    building: Vec<u64>,
    /// 別のスレッドで作る途中で panic した三角形の、許容弦誤差のビット列（別のスレッドで作り直さない）
    failed: Vec<u64>,
}

impl ShellMeshCache {
    fn lock(&self) -> MutexGuard<'_, ShellMeshes> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// B-rep `brep` の、許容弦誤差 `key` の三角形と、この呼び出しで作ったか（置き場になければ作って残す。置き場が
    /// 一杯なら、古いものを捨てる。別のスレッドで作っていれば、作り終えるのを待つ）
    fn mesh_or_build(self: &Arc<Self>, key: u64, brep: &Brep<f64>) -> (Arc<ShellMesh>, bool) {
        let mut meshes = self.lock();
        loop {
            if let Some(mesh) = meshes.get(key) {
                return (mesh, false);
            }
            if !meshes.building.contains(&key) {
                break;
            }
            meshes = self
                .built
                .wait(meshes)
                .unwrap_or_else(PoisonError::into_inner);
        }
        let building = Building::start(self, &mut meshes, key);
        drop(meshes);
        (building.build(brep), true)
    }
}

impl ShellMeshes {
    fn get(&self, key: u64) -> Option<Arc<ShellMesh>> {
        self.meshes
            .iter()
            .find(|(cached, _)| *cached == key)
            .map(|(_, mesh)| Arc::clone(mesh))
    }
}

/// 作っている三角形の印（作り終えたら、または作る途中で panic したら、印を外して待っているスレッドを起こす）
struct Building {
    cache: Arc<ShellMeshCache>,
    key: u64,
}

impl Building {
    /// 許容弦誤差 `key` の三角形を作り始めたことを、置き場 `meshes`（`cache` の中身）に記す
    fn start(cache: &Arc<ShellMeshCache>, meshes: &mut ShellMeshes, key: u64) -> Self {
        meshes.building.push(key);
        Self {
            cache: Arc::clone(cache),
            key,
        }
    }

    /// B-rep `brep` を許容弦誤差 `key` で三角形に分けて置き場に残す（置き場が一杯なら、古いものを捨てる）
    fn build(self, brep: &Brep<f64>) -> Arc<ShellMesh> {
        let mesh = Arc::new(triangulate_brep(brep, f64::from_bits(self.key)));
        let mut meshes = self.cache.lock();
        if meshes.meshes.len() == CACHED_SHELL_MESHES {
            meshes.meshes.remove(0);
        }
        meshes.meshes.push((self.key, Arc::clone(&mesh)));
        mesh
    }
}

/// 三角形 `mesh` の、三角形にできない面と、内側に点を足すのを上限で終えた面（面の ID は `face_ids`。B-rep の面の
/// 順）を、別の警告に出す
fn warn_untriangulated_faces(mesh: &ShellMesh, face_ids: &[FaceId], entity_id: EntityId) {
    for (face_id, face) in face_ids.iter().zip(&mesh.faces) {
        match face {
            Err(error) => {
                tracing::warn!(%entity_id, %face_id, "a shell face cannot be triangulated: {error}");
            }
            Ok(face) => {
                if let Some(reason) = face.stopped_by {
                    tracing::warn!(
                        %entity_id,
                        %face_id,
                        "the refinement of a shell face stopped at a limit, its triangles may exceed the chord tolerance: {reason}"
                    );
                }
            }
        }
    }
}

impl Drop for Building {
    fn drop(&mut self) {
        let mut meshes = self.cache.lock();
        meshes.building.retain(|key| *key != self.key);
        if std::thread::panicking() {
            meshes.failed.push(self.key);
        }
        self.cache.built.notify_all();
    }
}

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
    /// 出す。置き場が一杯なら、古いものを捨てる。別のスレッドで作っていれば、作り終えるのを待つ）
    fn mesh(&self, chord_tolerance: f64, entity_id: EntityId) -> Arc<ShellMesh> {
        let (mesh, built) = self
            .meshes
            .mesh_or_build(chord_tolerance.to_bits(), &self.brep);
        if built {
            warn_untriangulated_faces(&mesh, &self.face_ids, entity_id);
        }
        mesh
    }

    /// 許容弦誤差 `chord_tolerance` の B-rep の三角形を、置き場に残しているか（なければ、別のスレッドで作り始める。
    /// 作っている間は、残していないとする。別のスレッドで作れなかった（panic した・スレッドを作れない）三角形は、
    /// 別のスレッドでは作り直さずに、残しているとする。使うときに、そのスレッドで作る）
    fn prepare_mesh(&self, chord_tolerance: f64, entity_id: EntityId) -> bool {
        let key = chord_tolerance.to_bits();
        let mut meshes = self.meshes.lock();
        if meshes.get(key).is_some() || meshes.failed.contains(&key) {
            return true;
        }
        if meshes.building.contains(&key) {
            return false;
        }
        let building = Building::start(&self.meshes, &mut meshes, key);
        drop(meshes);
        let (brep, face_ids) = (Arc::clone(&self.brep), self.face_ids.clone());
        let spawned = std::thread::Builder::new()
            .name("shell-mesh".to_string())
            .spawn(move || {
                let mesh = building.build(&brep);
                warn_untriangulated_faces(&mesh, &face_ids, entity_id);
            });
        match spawned {
            Ok(_) => false,
            Err(error) => {
                // スレッドを作れなければ（作る途中の印は外れる）、このスレッドで作る（確かめ中のままにしない）
                tracing::warn!(%entity_id, "cannot start a thread to triangulate a shell: {error}");
                self.mesh(chord_tolerance, entity_id);
                true
            }
        }
    }

    /// 許容弦誤差 `chord_tolerance` の B-rep の三角形を、置き場に残しているか（別のスレッドで作れなかった三角形は、
    /// 残しているとする）
    fn mesh_ready(&self, chord_tolerance: f64) -> bool {
        let key = chord_tolerance.to_bits();
        let meshes = self.meshes.lock();
        meshes.get(key).is_some() || meshes.failed.contains(&key)
    }

    /// B-rep の面の位置 `positions` の三角形を、境界の頂点を共有した 1 つの三角形メッシュにして、配置を当てる（法線は
    /// 持たない）。配置した後の許容弦誤差を `chord_tolerance` にする（配置の拡大の分、B-rep の座標では小さくする）
    ///
    /// # Errors
    ///
    /// 三角形にできない面か、内側に点を足すのを上限で終えた面が、`positions` にあるとき。
    fn solver_mesh(
        &self,
        positions: &[usize],
        chord_tolerance: f64,
        entity_id: EntityId,
    ) -> Result<TriangleMesh3D<f64>, ProductShapeError> {
        let mesh = self.mesh(chord_tolerance / self.placement.scale_factor(), entity_id);
        // シェルの点の番号から、メッシュの頂点の番号
        let mut vertex_of: Vec<Option<usize>> = vec![None; mesh.points.len()];
        let mut vertices = Vec::new();
        let mut triangles = Vec::new();
        for &position in positions {
            // 内側に点を足すのを上限で終えた面は、許容弦誤差を守っていない所があるので、加工の形状にしない
            let face = mesh
                .face_within_tolerance(position)
                .map_err(ProductShapeError::ShellFace)?;
            for triangle in &face.triangles {
                triangles.push(triangle.map(|corner| {
                    let point = face.vertices[corner];
                    *vertex_of[point].get_or_insert_with(|| {
                        vertices.push(self.placement.apply_point(mesh.points[point]));
                        vertices.len() - 1
                    })
                }));
            }
        }
        Ok(TriangleMesh3D::new(vertices, triangles)
            .expect("the triangles of the faces index the points taken from the shell"))
    }
}

/// 面の ID が指す面（面単体の番号、またはシェルの番号と B-rep の面の順の位置）
#[derive(Debug, Clone, Copy)]
enum FaceLocation {
    Single(usize),
    Shell { shell: usize, position: usize },
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

    /// 面 `face_id` の三角形を、配置を当てて返す（B-rep の全体を一度に分けた三角形の、その面の分）。内側に点を
    /// 足すのを上限で終えた面は、そこまでの三角形を返す（表示では、面を欠かさない）
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

    /// 読んだ STEP の形状 `shapes` を、アセンブリの出現ごとにシェルのエンティティとして加え、読み込みの結果を返す
    /// （同じ部品の B-rep と、作ってある表示用の三角形は、出現の間で共有する）
    pub fn add_step_shapes(&mut self, shapes: StepShapes) -> StepImportReport {
        let (shells, report) = shapes.into_parts();
        for (shell, meshes) in shells {
            self.push_shell(
                shell.brep,
                shell.placement,
                Some(shell.source),
                shell.metadata,
                meshes,
            );
        }
        report
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

    /// 表示中のすべてのエンティティの面の ID（面単体、続けてシェルの面。それぞれ加えた順で、シェルの面は B-rep の
    /// 面の順）
    pub fn visible_face_ids(&self) -> Vec<FaceId> {
        let singles = self
            .entities
            .iter()
            .filter(|entity| entity.display().visible)
            .map(|entity| entity.face().id());
        let shells = self
            .shells
            .iter()
            .filter(|shell| shell.0.display().visible)
            .flat_map(|shell| shell.shell().face_ids.iter().copied());
        singles.chain(shells).collect()
    }

    /// すべての面の ID が指す面
    fn face_locations(&self) -> HashMap<FaceId, FaceLocation> {
        let singles = self
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| (entity.face().id(), FaceLocation::Single(index)));
        let shells = self.shells.iter().enumerate().flat_map(|(shell, entity)| {
            entity
                .shell()
                .face_ids
                .iter()
                .enumerate()
                .map(move |(position, &id)| (id, FaceLocation::Shell { shell, position }))
        });
        singles.chain(shells).collect()
    }

    /// 面の ID の集合を、ツールパス生成の入力の形状にする
    ///
    /// 面単体の三角形メッシュの面とシェルの面は 1 つのメッシュにつなげ、NURBS 曲面の面単体は曲面の集合にする。シェル
    /// の面は、許容弦誤差 `chord_tolerance` で三角形に分け、シェルごとに境界の頂点を共有する。同じ面の ID は 1 回
    /// だけ使う。
    ///
    /// # Errors
    ///
    /// 面がない場合、保持していない面がある場合、NURBS 曲面の面単体とほかの面が混ざっている場合、シェルの面を
    /// 三角形に分けられない場合。
    pub(crate) fn resolve_solver_geometry(
        &self,
        face_ids: &[FaceId],
        chord_tolerance: f64,
    ) -> Result<SolverGeometry, ProductShapeError> {
        let locations = self.face_locations();
        let mut seen = HashSet::new();
        let mut singles = Vec::new();
        // シェルの番号と、選んだ面の位置（シェルを初めに選んだ順）
        let mut shell_faces: Vec<(usize, Vec<usize>)> = Vec::new();
        for &face_id in face_ids {
            if !seen.insert(face_id) {
                continue;
            }
            match *locations
                .get(&face_id)
                .ok_or(ProductShapeError::UnknownFace(face_id))?
            {
                FaceLocation::Single(index) => singles.push(self.entities[index].face().geometry()),
                FaceLocation::Shell { shell, position } => {
                    match shell_faces.iter_mut().find(|(index, _)| *index == shell) {
                        Some((_, positions)) => positions.push(position),
                        None => shell_faces.push((shell, vec![position])),
                    }
                }
            }
        }
        if singles.is_empty() && shell_faces.is_empty() {
            return Err(ProductShapeError::NoFaces);
        }

        if shell_faces.is_empty()
            && singles
                .iter()
                .all(|geometry| matches!(geometry, ProductFaceGeometry::Surface(_)))
        {
            let surfaces = singles
                .into_iter()
                .filter_map(|geometry| match geometry {
                    ProductFaceGeometry::Surface(surface) => Some(surface.clone()),
                    ProductFaceGeometry::Mesh(_) => None,
                })
                .collect();
            return Ok(SolverGeometry::NurbsSurfaceSet(surfaces));
        }

        let single_meshes = singles
            .into_iter()
            .map(|geometry| match geometry {
                ProductFaceGeometry::Mesh(mesh) => Ok(mesh),
                ProductFaceGeometry::Surface(_) => Err(ProductShapeError::MixedFaceKinds),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let shell_meshes = shell_faces
            .iter()
            .map(|(index, positions)| {
                let entity = &self.shells[*index];
                entity
                    .shell()
                    .solver_mesh(positions, chord_tolerance, entity.id())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let meshes: Vec<&TriangleMesh3D<f64>> =
            single_meshes.into_iter().chain(&shell_meshes).collect();
        Ok(SolverGeometry::TriangleMesh(concatenate_triangle_meshes(
            &meshes,
        )))
    }

    /// 面の ID の集合 `face_ids` を [`Self::resolve_solver_geometry`] で形状にするときに、シェルの面を許容弦誤差
    /// `chord_tolerance` で三角形に分け終えているか（分けていないシェルは、別のスレッドで分け始める。シェルの面が
    /// なければ、分け終えているとする）
    pub(crate) fn prepare_solver_geometry(
        &self,
        face_ids: &[FaceId],
        chord_tolerance: f64,
    ) -> bool {
        // 加工の対象の面を持つシェルと、B-rep の座標の許容弦誤差
        let targets: Vec<(&ProductShellEntity, f64)> = self
            .shells
            .iter()
            .filter(|entity| {
                face_ids
                    .iter()
                    .any(|face_id| entity.shell().face_positions.contains_key(face_id))
            })
            .map(|entity| {
                let shell = entity.shell();
                (entity, chord_tolerance / shell.placement.scale_factor())
            })
            .collect();
        // 同じ置き場で、置き場に入らない数の許容弦誤差が要る（同じ B-rep を違う拡大で置いた）ときは、作り終えた
        // 三角形が互いを捨て合うため、別のスレッドでは作らない（形状にするときに、そのスレッドで作る）
        let mut keys: Vec<(*const ShellMeshCache, u64)> = targets
            .iter()
            .map(|(entity, tolerance)| (Arc::as_ptr(&entity.shell().meshes), tolerance.to_bits()))
            .collect();
        keys.sort_unstable();
        keys.dedup();
        let crowded = keys
            .chunk_by(|a, b| a.0 == b.0)
            .any(|same| same.len() > CACHED_SHELL_MESHES);
        if crowded {
            return true;
        }
        // すべてのシェルを分け始めるため、途中でやめない
        let started = targets.iter().fold(true, |ready, (entity, tolerance)| {
            let prepared = entity.shell().prepare_mesh(*tolerance, entity.id());
            ready && prepared
        });
        // 分け終えた三角形が、ほかの三角形を作り終えて捨てられていないかを、最後にまとめて確かめる
        started
            && targets
                .iter()
                .all(|(entity, tolerance)| entity.shell().mesh_ready(*tolerance))
    }
}

mod shell_mesh;
mod step_import;
mod step_shapes;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod test_fixtures;
