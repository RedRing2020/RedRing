//! STEP ファイルの読み込みの段 4（製品構造）（設計: BREP_AND_STEP_IMPORT_DESIGN.md 4.1・10 章）
//!
//! 段 1〜3（`geo_step`）で作った B-rep を、製品構造（製品・アセンブリの出現・配置）に沿って展開し、出現ごとに
//! シェルにする。同じ部品の B-rep は 1 回だけ作り、出現の間で共有する。製品構造に含まれない B-rep は、配置なしで
//! 読む。出現の配置は、`geo_step` が配置の記録から作る（座標系の作り方を 1 か所にまとめる）。

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::fmt;
use std::ops::ControlFlow;
use std::rc::Rc;
use std::sync::Arc;

use geo_contracts::{NurbsCurve3DProperties, NurbsSurface3DProperties, ToleranceSettings};
use geo_entity::Metadata;
use geo_step::geometry::{PlacementError, rigid_transformation};
use geo_step::record::Records;
use geo_step::syntax::{Limits, SyntaxError, SyntaxErrorKind};
use geo_step::topology::{BuildError, Built, Correction, ImportToleranceSettings, Topologies};
use geo_topology::{Brep, CurveRef, SurfaceRef, TopologyToleranceSettings};

use super::{ShellSource, SimilarityTransform3D};
use crate::background_task::TaskFailure;

/// アセンブリの深さの上限（循環も、ここで止まる）
const MAX_ASSEMBLY_DEPTH: usize = 32;
/// 出現をたどる回数の上限（共有した部分アセンブリでの増えすぎを止める）
const MAX_OCCURRENCE_VISITS: usize = 100_000;
/// 1 つのファイルの面の数の上限（出現ごとに数える）
const MAX_FACES_PER_FILE: usize = 5_000_000;
/// 1 つのファイルで、作った B-rep の曲線・曲面の制御点の合計の上限（同じ部品の出現は B-rep を共有するので、
/// B-rep ごとに数える）
const MAX_CONTROL_POINTS_PER_FILE: usize = 50_000_000;

/// エンティティの属性に残す、立体の名前のキー
pub const SOLID_NAME_KEY: &str = "step.solid_name";
/// エンティティの属性に残す、製品の名前のキー
pub const PRODUCT_NAME_KEY: &str = "step.product_name";
/// エンティティの属性に残す、アセンブリの出現の名前の並び（根から、` / ` でつなぐ）のキー
pub const OCCURRENCE_NAMES_KEY: &str = "step.occurrence_names";

/// STEP ファイルを読めない理由
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepImportError {
    /// ファイルが、構文の段の上限（大きさ・エンティティの数・引数の数・入れ子の深さ・複合のエンティティの部分の
    /// 数）を超える
    TooLarge,
    /// 構文の誤り
    Syntax(SyntaxError),
    /// 段 4 の上限を超えた
    Limit(StepImportLimit),
    /// 形状が 1 つも読めない
    NoShapes,
}

impl fmt::Display for StepImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("the file exceeds the limits of the syntax"),
            Self::Syntax(error) => write!(f, "{error}"),
            Self::Limit(limit) => write!(f, "the file exceeds the limit of {limit:?}"),
            Self::NoShapes => f.write_str("no shapes can be read"),
        }
    }
}

impl std::error::Error for StepImportError {}

/// 段 4 の上限（設計 4.3）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepImportLimit {
    /// アセンブリの深さ（循環を含む）
    AssemblyDepth,
    /// 出現をたどる回数
    OccurrenceVisits,
    /// 1 つのファイルの面の数
    FacesPerFile,
    /// 1 つのファイルの、曲線・曲面の制御点の合計
    ControlPointsPerFile,
}

/// 読み込みの結果（画面には、外した要素・外した出現・配置なしで読んだ出現の数を種類ごとに出し、要素ごとの理由は
/// ログに出す）
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StepImportReport {
    /// 加えたシェルのエンティティの数（出現ごと）
    pub shells: usize,
    /// 読めた立体の数（同じ部品は 1 回）
    pub solids: usize,
    /// 外した面・シェル・立体の数
    pub skipped_elements: usize,
    /// 直した向きの数（面を裏返した・ループを逆にした）
    pub reoriented: usize,
    /// 広げた頂点・稜線の許容誤差の数
    pub widened_tolerances: usize,
    /// 配置を読めず、配置なしで読んだ出現の数
    pub unplaced_occurrences: usize,
    /// 回転と移動でない配置（拡大・鏡映）のため外した出現の数
    pub skipped_occurrences: usize,
}

/// STEP の読み込みの段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepImportStage {
    /// 構文を読む
    Syntax,
    /// エンティティを、幾何と位相の記録にする
    Records,
    /// 曲線・曲面を作る
    Geometry,
    /// B-rep を作り、製品構造に沿ってシェルにする
    Topology,
    /// 表示用の三角形を作る
    DisplayMeshes,
}

impl StepImportStage {
    /// 全体を [`PROGRESS_TOTAL`] としたときの、段の初めまでの量と、段の量（段にかかる時間の、おおよその割合）
    const fn span(self) -> (u32, u32) {
        match self {
            Self::Syntax => (0, 50),
            Self::Records => (50, 50),
            Self::Geometry => (100, 20),
            Self::Topology => (120, 530),
            Self::DisplayMeshes => (650, 350),
        }
    }
}

/// 読み込みの全体の量
const PROGRESS_TOTAL: u32 = 1000;

/// STEP の読み込みの進み具合（段と、段ごとの重みでまとめた全体の量）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepImportProgress {
    /// 今の段
    pub stage: StepImportStage,
    /// 全体のうち、済んだ量
    pub done: u32,
    /// 全体の量
    pub total: u32,
}

impl StepImportProgress {
    /// 段 `stage` で、全体の数 `total` のうち `done` が済んだときの、全体の進み具合
    pub(crate) fn in_stage(stage: StepImportStage, done: usize, total: usize) -> Self {
        let (start, span) = stage.span();
        let within = if total == 0 {
            0
        } else {
            // 済んだ数は、全体の数までにそろえる（段の量を超えない）
            (done.min(total) as u128 * u128::from(span) / total as u128) as u32
        };
        Self {
            stage,
            done: start + within,
            total: PROGRESS_TOTAL,
        }
    }
}

/// 読み込んだシェル（B-rep・配置・元の要素・名前）
pub(crate) struct ImportedShell {
    pub(crate) brep: Arc<Brep<f64>>,
    pub(crate) placement: SimilarityTransform3D<f64>,
    pub(crate) source: ShellSource,
    pub(crate) metadata: Metadata,
}

/// 進み具合を受け取り、続けるか止めるかを返す閉包
pub(crate) type ReportProgress<'a> = dyn FnMut(StepImportProgress) -> ControlFlow<()> + 'a;

/// STEP のバイト列 `bytes`（ファイルの名前 `file_name`）を読み、出現ごとのシェルにする
///
/// 段の初めと、段の中の区切り（エンティティ 4,096 個ごと。B-rep を作る段は、表現の要素ごと）で、`progress` に
/// 進み具合を知らせる。
///
/// # Errors
///
/// 構文の誤り、上限を超える、形状が 1 つも読めない場合。`progress` が止めると返した場合
/// （[`TaskFailure::Cancelled`]）。
pub(crate) fn read_step(
    bytes: &[u8],
    file_name: &str,
    progress: &mut ReportProgress<'_>,
) -> Result<(Vec<ImportedShell>, StepImportReport), TaskFailure<StepImportError>> {
    let mut report_stage = |stage: StepImportStage, done: usize, total: usize| {
        progress(StepImportProgress::in_stage(stage, done, total))
    };

    begin(&mut report_stage, StepImportStage::Syntax)?;
    let file = geo_step::syntax::parse(bytes, &Limits::default(), &mut |done, total| {
        report_stage(StepImportStage::Syntax, done, total)
    })
    .map_err(|error| match error.kind {
        SyntaxErrorKind::Cancelled => TaskFailure::Cancelled,
        SyntaxErrorKind::TooLarge
        | SyntaxErrorKind::TooManyEntities
        | SyntaxErrorKind::TooManyParameters
        | SyntaxErrorKind::TooDeep
        | SyntaxErrorKind::TooManyRecords => TaskFailure::Failed(StepImportError::TooLarge),
        _ => TaskFailure::Failed(StepImportError::Syntax(error)),
    })?;

    begin(&mut report_stage, StepImportStage::Records)?;
    let records = geo_step::record::read(file, &mut |done, total| {
        report_stage(StepImportStage::Records, done, total)
    })
    .map_err(|geo_step::Cancelled| TaskFailure::Cancelled)?;

    begin(&mut report_stage, StepImportStage::Geometry)?;
    let tolerances = ToleranceSettings::<f64>::standard();
    let geometries = geo_step::geometry::convert(&records, &tolerances, &mut |done, total| {
        report_stage(StepImportStage::Geometry, done, total)
    })
    .map_err(|geo_step::Cancelled| TaskFailure::Cancelled)?;

    begin(&mut report_stage, StepImportStage::Topology)?;
    let topologies = geo_step::topology::build(
        &records,
        &geometries,
        TopologyToleranceSettings::new(tolerances.distance_tolerance),
        ImportToleranceSettings::default(),
        &mut |done, total| report_stage(StepImportStage::Topology, done, total),
    )
    .map_err(|error| match error {
        BuildError::Cancelled => TaskFailure::Cancelled,
        BuildError::Brep(_) => TaskFailure::Failed(StepImportError::NoShapes),
    })?;
    let mut report = report_and_log(&records, &geometries, &topologies);
    let shells = Expansion::new(&records, topologies, &tolerances, file_name, &mut report)
        .run()
        .map_err(TaskFailure::Failed)?;
    if shells.is_empty() {
        return Err(TaskFailure::Failed(StepImportError::NoShapes));
    }
    report.shells = shells.len();
    Ok((shells, report))
}

/// 段 `stage` の初めを知らせる（区切りに届かない小さなファイルでも、段の間で止められるようにする）
fn begin(
    report_stage: &mut impl FnMut(StepImportStage, usize, usize) -> ControlFlow<()>,
    stage: StepImportStage,
) -> Result<(), TaskFailure<StepImportError>> {
    match report_stage(stage, 0, 1) {
        ControlFlow::Continue(()) => Ok(()),
        ControlFlow::Break(()) => Err(TaskFailure::Cancelled),
    }
}

/// 段 1〜3 で外した要素・直したことを、件数にまとめ、要素ごとにログに出す
fn report_and_log(
    records: &Records,
    geometries: &geo_step::geometry::Geometries,
    topologies: &Topologies,
) -> StepImportReport {
    for skipped in &records.skipped {
        tracing::info!(id = skipped.id, name = %skipped.name, reason = ?skipped.reason, "STEP element skipped");
    }
    for skipped in &geometries.skipped {
        tracing::info!(id = skipped.id, reason = ?skipped.reason, "STEP geometry skipped");
    }
    for skipped in &topologies.skipped {
        tracing::warn!(id = skipped.id, cause = skipped.cause, reason = ?skipped.reason, "STEP element skipped");
    }
    let mut report = StepImportReport {
        skipped_elements: topologies.skipped.len(),
        ..StepImportReport::default()
    };
    for built in &topologies.items {
        if built.solid.is_some() {
            report.solids += 1;
        }
        for correction in &built.corrections {
            tracing::info!(item = built.item, ?correction, "STEP shape corrected");
            match correction {
                Correction::FaceReversed { .. } | Correction::LoopReversed { .. } => {
                    report.reoriented += 1;
                }
                Correction::VertexToleranceWidened { .. }
                | Correction::EdgeToleranceWidened { .. } => report.widened_tolerances += 1,
                _ => {}
            }
        }
        if !built.unoriented_faces.is_empty() {
            tracing::warn!(item = built.item, faces = ?built.unoriented_faces, "STEP faces with undecided orientation");
        }
        if !built.open_shells.is_empty() {
            tracing::warn!(item = built.item, shells = ?built.open_shells, "STEP closed shells that are not closed");
        }
    }
    if topologies.tolerance_below_precision {
        tracing::warn!("the tolerance is below the precision of the coordinates of the STEP file");
    }
    report
}

/// 作った B-rep の、展開に使う中身（B-rep は、初めに使うときに共有の表に移す）
struct Item {
    /// 表現の要素のファイルの番号
    item: u64,
    /// B-rep の面の順の、元の面のエンティティの番号
    faces: Vec<u64>,
    face_count: usize,
    /// まだ共有の表に移していない B-rep
    brep: Option<Brep<f64>>,
    /// 共有の表
    shared: Option<Arc<Brep<f64>>>,
}

/// 製品構造の展開
struct Expansion<'a> {
    records: &'a Records,
    tolerances: &'a ToleranceSettings<f64>,
    file_name: &'a str,
    report: &'a mut StepImportReport,
    items: Vec<Item>,
    /// 表現の要素から、その要素から作った B-rep（`items` の位置）
    items_of: HashMap<u64, Vec<usize>>,
    /// 形状の表現の間のつながり（向きを問わない）
    links: HashMap<u64, Vec<u64>>,
    /// 親の製品から、その出現
    children: BTreeMap<u64, Vec<u64>>,
    /// 製品の形状の表現（つながりをたどったものも含む。製品ごとに 1 回だけ集める）
    representations: HashMap<u64, Rc<HashSet<u64>>>,
    /// 製品構造でたどった（外した出現の下も含む）B-rep
    reached: HashSet<usize>,
    visits: usize,
    faces: usize,
    control_points: usize,
    shells: Vec<ImportedShell>,
}

/// たどっている出現の並び
struct Path {
    /// 出現の番号（根から）
    occurrences: Vec<u64>,
    /// 出現の名前（根から）
    names: Vec<String>,
    placement: SimilarityTransform3D<f64>,
}

impl Path {
    fn root() -> Self {
        Self {
            occurrences: Vec::new(),
            names: Vec::new(),
            placement: SimilarityTransform3D::identity(),
        }
    }
}

impl<'a> Expansion<'a> {
    fn new(
        records: &'a Records,
        topologies: Topologies,
        tolerances: &'a ToleranceSettings<f64>,
        file_name: &'a str,
        report: &'a mut StepImportReport,
    ) -> Self {
        let items: Vec<Item> = topologies
            .items
            .into_iter()
            .map(|built| Item {
                item: built.item,
                faces: source_faces(&built),
                face_count: built.brep.face_count(),
                brep: Some(built.brep),
                shared: None,
            })
            .collect();
        let mut items_of: HashMap<u64, Vec<usize>> = HashMap::new();
        for (position, item) in items.iter().enumerate() {
            items_of.entry(item.item).or_default().push(position);
        }
        let mut links: HashMap<u64, Vec<u64>> = HashMap::new();
        for &(first, second) in &records.representation_links {
            links.entry(first).or_default().push(second);
            links.entry(second).or_default().push(first);
        }
        let mut children: BTreeMap<u64, Vec<u64>> = BTreeMap::new();
        for (&id, occurrence) in &records.occurrences {
            children.entry(occurrence.parent).or_default().push(id);
        }
        Self {
            records,
            tolerances,
            file_name,
            report,
            items,
            items_of,
            links,
            children,
            representations: HashMap::new(),
            reached: HashSet::new(),
            visits: 0,
            faces: 0,
            control_points: 0,
            shells: Vec::new(),
        }
    }

    fn run(mut self) -> Result<Vec<ImportedShell>, StepImportError> {
        let child_products: HashSet<u64> = self
            .records
            .occurrences
            .values()
            .map(|occurrence| occurrence.child)
            .collect();
        let roots: Vec<u64> = self
            .records
            .products
            .keys()
            .copied()
            .filter(|product| !child_products.contains(product))
            .collect();
        for root in roots {
            self.visit(root, &Path::root())?;
        }
        // 製品構造でたどれなかった B-rep は、配置なしで読む
        for position in 0..self.items.len() {
            if !self.reached.contains(&position) {
                self.add(position, None, &Path::root())?;
            }
        }
        Ok(self.shells)
    }

    /// 製品 `product` を、出現の並び `path` で置き、その下の出現をたどる
    fn visit(&mut self, product: u64, path: &Path) -> Result<(), StepImportError> {
        self.count_visit(path)?;
        for position in self.positions_of(product) {
            self.reached.insert(position);
            self.add(position, Some(product), path)?;
        }
        for id in self.children.get(&product).cloned().unwrap_or_default() {
            let Some(occurrence) = self.records.occurrences.get(&id) else {
                continue;
            };
            let local = match occurrence.placement.as_ref() {
                None => {
                    tracing::warn!(
                        occurrence = id,
                        "a STEP occurrence without a placement is read in place"
                    );
                    self.report.unplaced_occurrences += 1;
                    SimilarityTransform3D::identity()
                }
                Some(placement) => {
                    // 関係の 2 つ目の表現が子の形状なら、逆の変換にする
                    let child = self.representations_of(occurrence.child);
                    let reversed = child.contains(&placement.representation_2)
                        && !child.contains(&placement.representation_1);
                    match rigid_transformation(
                        self.records,
                        &placement.transformation,
                        reversed,
                        self.tolerances,
                    ) {
                        Ok(local) => local,
                        Err(PlacementError::Geometry(reason)) => {
                            tracing::warn!(
                                occurrence = id,
                                ?reason,
                                "a STEP occurrence with an unreadable placement is read in place"
                            );
                            self.report.unplaced_occurrences += 1;
                            SimilarityTransform3D::identity()
                        }
                        Err(PlacementError::NotRigid) => {
                            tracing::warn!(
                                occurrence = id,
                                "a STEP occurrence placed with a scale or a mirror is skipped"
                            );
                            self.report.skipped_occurrences += 1;
                            // 外した出現の下の B-rep は、配置なしでも読まない
                            self.mark_reached(occurrence.child, path.occurrences.len() + 1)?;
                            continue;
                        }
                    }
                }
            };
            let mut occurrences = path.occurrences.clone();
            occurrences.push(id);
            let mut names = path.names.clone();
            names.push(occurrence.name.clone());
            let child = Path {
                occurrences,
                names,
                placement: local.then(&path.placement),
            };
            self.visit(occurrence.child, &child)?;
        }
        Ok(())
    }

    /// 製品 `product` とその下の出現の B-rep を、たどったものとする（加えない）
    fn mark_reached(&mut self, product: u64, depth: usize) -> Result<(), StepImportError> {
        if depth > MAX_ASSEMBLY_DEPTH {
            return Err(StepImportError::Limit(StepImportLimit::AssemblyDepth));
        }
        self.visits += 1;
        if self.visits > MAX_OCCURRENCE_VISITS {
            return Err(StepImportError::Limit(StepImportLimit::OccurrenceVisits));
        }
        let positions = self.positions_of(product);
        self.reached.extend(positions);
        for id in self.children.get(&product).cloned().unwrap_or_default() {
            if let Some(occurrence) = self.records.occurrences.get(&id) {
                self.mark_reached(occurrence.child, depth + 1)?;
            }
        }
        Ok(())
    }

    /// 出現の並び `path` をたどることを、深さと回数の上限で確かめる
    fn count_visit(&mut self, path: &Path) -> Result<(), StepImportError> {
        if path.occurrences.len() > MAX_ASSEMBLY_DEPTH {
            return Err(StepImportError::Limit(StepImportLimit::AssemblyDepth));
        }
        self.visits += 1;
        if self.visits > MAX_OCCURRENCE_VISITS {
            return Err(StepImportError::Limit(StepImportLimit::OccurrenceVisits));
        }
        Ok(())
    }

    /// 製品 `product` の形状の表現の要素から作った B-rep（`items` の位置）
    fn positions_of(&mut self, product: u64) -> Vec<usize> {
        let representations = self.representations_of(product);
        let mut elements: Vec<u64> = representations
            .iter()
            .filter_map(|representation| self.records.representations.get(representation))
            .flat_map(|representation| representation.items.iter().copied())
            .collect();
        elements.sort_unstable();
        elements.dedup();
        elements
            .into_iter()
            .flat_map(|element| self.items_of.get(&element).cloned().unwrap_or_default())
            .collect()
    }

    /// 製品 `product` の形状の表現（形状の表現の間のつながりをたどったものも含む）
    fn representations_of(&mut self, product: u64) -> Rc<HashSet<u64>> {
        if let Some(found) = self.representations.get(&product) {
            return Rc::clone(found);
        }
        let mut found = HashSet::new();
        let mut queue: VecDeque<u64> = self
            .records
            .products
            .get(&product)
            .map(|product| product.representations.iter().copied().collect())
            .unwrap_or_default();
        while let Some(representation) = queue.pop_front() {
            if found.insert(representation) {
                queue.extend(self.links.get(&representation).into_iter().flatten());
            }
        }
        let found = Rc::new(found);
        self.representations.insert(product, Rc::clone(&found));
        found
    }

    /// B-rep `position` を、製品 `product` の出現の並び `path` で置くシェルにする
    fn add(
        &mut self,
        position: usize,
        product: Option<u64>,
        path: &Path,
    ) -> Result<(), StepImportError> {
        let Some(item) = self.items.get_mut(position) else {
            return Ok(());
        };
        self.faces += item.face_count;
        if self.faces > MAX_FACES_PER_FILE {
            return Err(StepImportError::Limit(StepImportLimit::FacesPerFile));
        }
        let brep = match (&item.shared, item.brep.take()) {
            (Some(shared), _) => Arc::clone(shared),
            (None, Some(brep)) => {
                self.control_points += control_points(&brep);
                if self.control_points > MAX_CONTROL_POINTS_PER_FILE {
                    return Err(StepImportError::Limit(
                        StepImportLimit::ControlPointsPerFile,
                    ));
                }
                let shared = Arc::new(brep);
                item.shared = Some(Arc::clone(&shared));
                shared
            }
            (None, None) => return Ok(()),
        };
        let solid_name = self
            .records
            .item_names
            .get(&item.item)
            .cloned()
            .unwrap_or_default();
        let faces = item.faces.clone();
        let product_name = product
            .and_then(|product| self.records.products.get(&product))
            .map(|product| product.name.clone())
            .unwrap_or_default();
        let occurrence_name = path.names.last().cloned().unwrap_or_default();
        let name = [&solid_name, &product_name, &occurrence_name]
            .into_iter()
            .find(|name| !name.is_empty())
            .cloned();
        let mut metadata = Metadata {
            name,
            ..Metadata::default()
        };
        metadata
            .custom
            .insert(SOLID_NAME_KEY.to_string(), solid_name);
        metadata
            .custom
            .insert(PRODUCT_NAME_KEY.to_string(), product_name);
        metadata
            .custom
            .insert(OCCURRENCE_NAMES_KEY.to_string(), path.names.join(" / "));
        self.shells.push(ImportedShell {
            source: ShellSource {
                file_name: self.file_name.to_string(),
                occurrences: path.occurrences.clone(),
                faces,
            },
            brep,
            placement: path.placement,
            metadata,
        });
        Ok(())
    }
}

/// B-rep の面の順の、元の面のエンティティの番号（見つからない面は 0）
fn source_faces(built: &Built) -> Vec<u64> {
    let of_face: HashMap<_, u64> = built
        .faces
        .iter()
        .map(|(&source, &face)| (face, source))
        .collect();
    built
        .brep
        .face_indices()
        .map(|face| of_face.get(&face).copied().unwrap_or(0))
        .collect()
}

/// B-rep の曲線・曲面の制御点の数の合計
fn control_points(brep: &Brep<f64>) -> usize {
    let records = brep.records();
    let curves: usize = records
        .edges
        .iter()
        .map(|edge| match &edge.curve {
            CurveRef::Nurbs(curve) => curve.control_point_count(),
            _ => 0,
        })
        .sum();
    let surfaces: usize = records
        .faces
        .iter()
        .map(|face| match &face.surface {
            SurfaceRef::Nurbs(surface) => surface.u_count() * surface.v_count(),
            _ => 0,
        })
        .sum();
    curves + surfaces
}

#[cfg(test)]
pub(super) mod tests;
