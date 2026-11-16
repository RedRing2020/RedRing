//! STEP の位相の記録と型にした曲線・曲面から、B-rep の表を作る（設計: BREP_AND_STEP_IMPORT_DESIGN.md 8 章）
//!
//! 表現の要素（立体・シェル・面の集まり）ごとに、1 つの `Brep` を作る。稜線の区間は頂点の位置から決め（8.2）、
//! 向きの旗は形で確かめて直す（8.1）。直したことは記録する。作れない面は、理由を記録して外し、残りの面で
//! シェルを作る。
//!
//! 表に足す前に、頂点・稜線の許容誤差を、測ったずれに合わせて広げ、1 つの角にある頂点と、重なる稜線を
//! まとめる（6.1・6.2）。

#[cfg(test)]
mod tests;

use crate::geometry::{Conic, Curve, Geometries, Sense};
use crate::record::{Geometry, Records, Topology};
use analysis::consts::DERIVATIVE_ZERO_THRESHOLD_F64;
use analysis::consts::import_tolerance::{DEVIATION_FACTOR, ELEMENT_TOLERANCE_LIMIT_RATIO};
use geo_contracts::{
    Circle3DProperties, Ellipse3DProperties, InfiniteLine3DConstructor, InfiniteLine3DDistance,
    InfiniteLine3DEvaluation, NurbsCurve3DEvaluation, NurbsCurve3DProperties, Plane3DConstructor,
};
use geo_core::{Direction3D, Point3D, Vector3D};
use geo_topology::{
    Brep, BrepError, CurveRef, EdgeIndex, EdgeUse, FaceIndex, ParameterCurve, ShellIndex,
    SolidIndex, SurfaceRef, TopoEllipse3D, TopoInfiniteLine3D, TopoNurbsCurve3D, TopoPlane3D,
    TopologyToleranceSettings, VertexIndex,
};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::f64::consts::TAU;

/// 向きの旗を重ねる入れ子（`ORIENTED_FACE` など）の段の上限
const MAX_ORIENTED_DEPTH: usize = 4;
/// 曲面のない面の平面を求めるときに、稜線の使用 1 つから取る点の数
const SAMPLES_PER_USE: usize = 8;
/// NURBS 曲線の上の点のパラメータを探すときに、制御点 1 つあたりに取る点の数
const SAMPLES_PER_CONTROL_POINT: usize = 16;
/// NURBS 曲線の上の点のパラメータを、ニュートン法で詰める回数の上限
const MAX_NEWTON_STEPS: usize = 32;
/// 2 本の稜線が重なるかを確かめるときに、区間を分ける数（区間の両端を含めて この数 + 1 個の点で比べる）
const EDGE_COMPARISON_SEGMENTS: usize = 8;

/// 読み込みで、頂点・稜線の許容誤差を広げるときの設定（設計 6.1）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImportToleranceSettings {
    /// 要素の許容誤差の上限の、読み込む立体の大きさ（頂点と曲線の外形の箱の対角の長さ）に対する比（正）
    pub limit_ratio: f64,
    /// 測ったずれに掛ける倍率（1 以上）
    pub deviation_factor: f64,
}

impl Default for ImportToleranceSettings {
    fn default() -> Self {
        Self {
            limit_ratio: ELEMENT_TOLERANCE_LIMIT_RATIO,
            deviation_factor: DEVIATION_FACTOR,
        }
    }
}

impl ImportToleranceSettings {
    fn is_valid(&self) -> bool {
        self.limit_ratio > 0.0
            && self.limit_ratio.is_finite()
            && self.deviation_factor >= 1.0
            && self.deviation_factor.is_finite()
    }
}

/// 作った B-rep と、外した要素
#[derive(Debug, Clone, Default)]
pub struct Topologies {
    /// 表現の要素ごとの B-rep（面が 1 つもできなかった要素は含まない）
    pub items: Vec<Built>,
    /// 外した要素と、その理由
    pub skipped: Vec<TopologySkipped>,
}

/// 表現の要素 1 つから作った B-rep
#[derive(Debug, Clone)]
pub struct Built {
    /// 表現の要素（立体・シェル・面の集まり）のファイルの番号
    pub item: u64,
    pub brep: Brep<f64>,
    /// 作ったシェル（立体なら、初めが外側）
    pub shells: Vec<ShellIndex>,
    /// 立体の要素から作った立体（外側のシェルを作れなければ `None`）
    pub solid: Option<SolidIndex>,
    /// ファイルの面の番号から、B-rep の面への表
    pub faces: BTreeMap<u64, FaceIndex>,
    /// 直したこと
    pub corrections: Vec<Correction>,
    /// 向きを決められなかった面（ファイルの面の番号）。ループの向きを直せなかった面と、面ごとに向きが決まる
    /// 曲面の上で、ループからは表と裏のどちらとも決まらない面
    pub unoriented_faces: Vec<u64>,
    /// ファイルで閉じたシェルとされているが、作ったシェルが閉じていないもの（ファイルのシェルの番号）
    pub open_shells: Vec<u64>,
}

/// 読み込みで直したこと（番号はファイルの番号）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Correction {
    /// 頂点 `vertex` を、同じ角にある頂点 `into` にまとめた
    VertexMerged { vertex: u64, into: u64 },
    /// 稜線 `edge` を、同じ頂点の間で重なる稜線 `into` にまとめた
    EdgeMerged { edge: u64, into: u64 },
    /// 頂点 `vertex`（まとめた頂点は、まとめ先）の許容誤差を、初めの値から `tolerance` に広げた
    VertexToleranceWidened { vertex: u64, tolerance: f64 },
    /// 稜線 `edge`（まとめた稜線は、まとめ先）の許容誤差を、初めの値から `tolerance` に広げた
    EdgeToleranceWidened { edge: u64, tolerance: f64 },
    /// 開いた曲線の稜線で、`same_sense` と頂点の順が食い違ったため、頂点の順を使った
    EdgeSense { edge: u64 },
    /// 頂点が直線から外れた稜線を、頂点の間の線分にした
    EdgeToSegment { edge: u64 },
    /// 面のループが向きの規則に合わなかったため、面を裏返した
    FaceReversed { face: u64 },
    /// 内側のループが外側のループと同じ回りだったため、ループ `bound` を逆にした
    LoopReversed { face: u64, bound: u64 },
}

/// 外した要素（面・シェル・立体）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologySkipped {
    /// 外した要素のファイルの番号
    pub id: u64,
    /// 理由の起きた要素のファイルの番号（稜線・曲面など）
    pub cause: u64,
    pub reason: TopologyError,
}

/// 要素を作れない理由
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TopologyError {
    /// 頂点が曲線から、または稜線が面の曲面から、許容誤差の上限を超えてずれている（`deviation` は測った
    /// ずれ。理由の起きた要素は、頂点か稜線）
    ToleranceExceeded { deviation: f64 },
    /// 参照の先が、期待した種類の位相・幾何の記録でない
    MissingReference,
    /// 曲線・曲面を型にできなかった（理由は、幾何の段で記録してある）
    MissingGeometry,
    /// 向きの旗の入れ子が深すぎる
    TooDeep,
    /// 開いた曲線（直線・放物線・双曲線・閉じていない NURBS 曲線）の上の閉じた稜線
    ClosedOnOpenCurve,
    /// 稜線の区間を決められない（両端の点が曲線の上で同じ位置にある、閉じた NURBS 曲線の継ぎ目をまたぐなど）
    InvalidEdgeRange,
    /// 双曲線を許容誤差の中で近似できない
    ApproximationFailed,
    /// NURBS 曲面の上の面（パラメータ曲線の当てはめができるまで外す）
    NurbsSurface,
    /// 曲面のない面で、ループの載る平面を決められない
    NoLoopPlane,
    /// 同じ面を、逆の向きで 2 回使う
    ReversedReuse,
    /// B-rep の表に足せない
    Brep(BrepError),
}

/// 理由と、その起きた要素
#[derive(Debug, Clone, Copy)]
struct Failure {
    cause: u64,
    reason: TopologyError,
}

impl Failure {
    fn new(cause: u64, reason: TopologyError) -> Self {
        Self { cause, reason }
    }
}

/// 中間データの位相の記録から、表現の要素ごとに B-rep を作る
///
/// `tolerances` は、B-rep の表の全体の許容誤差（モデルの値）。頂点・稜線の許容誤差の初めの値は、表現の
/// 文脈の許容誤差（なければ全体の許容誤差）とし、全体の許容誤差と上限の間に収める。上限は、表現の要素の
/// 頂点と曲線の外形の箱の対角の長さに `settings` の比を掛けた値（全体の許容誤差より小さければ、全体の
/// 許容誤差）。
///
/// # Errors
///
/// 全体の許容誤差が正の有限の値でない、または `settings` の比が正の有限の値でない・倍率が 1 以上の有限の値で
/// ないとき（`BrepError::InvalidTolerance`）。
pub fn build(
    records: &Records,
    geometries: &Geometries,
    tolerances: TopologyToleranceSettings<f64>,
    settings: ImportToleranceSettings,
) -> Result<Topologies, BrepError> {
    // 許容誤差は、要素を作る前に確かめる
    Brep::new(tolerances)?;
    if !settings.is_valid() {
        return Err(BrepError::InvalidTolerance);
    }
    let mut topologies = Topologies::default();
    let mut seen = BTreeSet::new();
    for representation in records.representations.values() {
        let uncertainty = records
            .contexts
            .get(&representation.context)
            .and_then(|context| context.uncertainty_mm);
        for &item in &representation.items {
            if !seen.insert(item) {
                continue;
            }
            let builder = ItemBuilder {
                records,
                geometries,
                global: tolerances.distance_tolerance,
                initial: tolerances.distance_tolerance,
                limit: tolerances.distance_tolerance,
                factor: settings.deviation_factor,
                brep: Brep::new(tolerances)?,
                shapes: BTreeMap::new(),
                vertex_groups: BTreeMap::new(),
                groups: Vec::new(),
                merged_edges: BTreeMap::new(),
                face_failures: BTreeMap::new(),
                points: BTreeMap::new(),
                edges: BTreeMap::new(),
                segments: HashMap::new(),
                faces: BTreeMap::new(),
                built_faces: Vec::new(),
                corrections: Vec::new(),
                unoriented: Vec::new(),
                skipped: Vec::new(),
            };
            let builder = builder.prepare(item, uncertainty, settings.limit_ratio);
            let (built, mut skipped) = builder.item(item);
            topologies.skipped.append(&mut skipped);
            if let Some(built) = built {
                topologies.items.push(built);
            }
        }
    }
    Ok(topologies)
}

/// 作った面（ファイルの面の番号・ループのファイルの番号と外側のループか・面ごとに向きが決まる曲面の上か）
struct BuiltFace {
    id: u64,
    index: FaceIndex,
    bounds: Vec<(u64, bool)>,
    per_face: bool,
}

/// ファイルの稜線から求めた、B-rep の稜線の形
#[derive(Debug, Clone)]
struct EdgeShape {
    /// B-rep の稜線の始点・終点（ファイルの頂点）
    first: u64,
    last: u64,
    curve: CurveRef<f64>,
    range: (f64, f64),
    /// B-rep の稜線が、ファイルの始点から終点へ進むか
    along: bool,
    tolerance: f64,
    /// 稜線を表に足すときに記録する、直したこと
    correction: Option<Correction>,
}

/// 1 つの角にまとめた頂点
#[derive(Debug, Clone)]
struct VertexGroup {
    /// まとめ先（初めの頂点）のファイルの番号と点・許容誤差
    id: u64,
    point: Point3D<f64>,
    own_tolerance: f64,
    /// まとめた頂点をすべて含む許容誤差
    tolerance: f64,
    members: Vec<u64>,
    index: Option<VertexIndex>,
}

/// 前もって集めた面（ファイルの番号・型の曲面（ないときは `None`）・ループの稜線のファイルの番号）
struct PreparedFace {
    id: u64,
    surface: Option<SurfaceRef<f64>>,
    edges: Vec<u64>,
}

/// 表現の要素 1 つの B-rep を作る
struct ItemBuilder<'a> {
    records: &'a Records,
    geometries: &'a Geometries,
    /// 全体の許容誤差、要素の許容誤差の初めの値と上限、ずれに掛ける倍率
    global: f64,
    initial: f64,
    limit: f64,
    factor: f64,
    brep: Brep<f64>,
    /// ファイルの稜線の形（前もって求める）
    shapes: BTreeMap<u64, Result<EdgeShape, Failure>>,
    /// ファイルの頂点から、まとめた頂点（`groups` の位置）
    vertex_groups: BTreeMap<u64, usize>,
    groups: Vec<VertexGroup>,
    /// まとめた稜線から、まとめ先の稜線と、まとめ先の B-rep の稜線がファイルの始点から終点へ進むか
    merged_edges: BTreeMap<u64, (u64, bool)>,
    /// 稜線が面の曲面から上限を超えてずれている面
    face_failures: BTreeMap<u64, Failure>,
    /// 多角形のループの点から、B-rep の頂点
    points: BTreeMap<u64, VertexIndex>,
    /// ファイルの稜線から、B-rep の稜線と、B-rep の稜線がファイルの稜線と同じ向きか
    edges: BTreeMap<u64, Result<(EdgeIndex, bool), Failure>>,
    /// 多角形のループの辺（頂点の組）から、B-rep の稜線と、その始点
    segments: HashMap<(VertexIndex, VertexIndex), (EdgeIndex, VertexIndex)>,
    /// ファイルの面から、B-rep の面（作れなかった面は `None`）と、逆の向きで作ったか
    faces: BTreeMap<u64, (Option<FaceIndex>, bool)>,
    built_faces: Vec<BuiltFace>,
    corrections: Vec<Correction>,
    /// 向きを決められなかった面（ファイルの面の番号）
    unoriented: Vec<u64>,
    skipped: Vec<TopologySkipped>,
}

impl<'a> ItemBuilder<'a> {
    fn topology(&self, id: u64) -> Result<&'a Topology, Failure> {
        self.records
            .topology
            .get(&id)
            .ok_or(Failure::new(id, TopologyError::MissingReference))
    }

    /// 向きの旗の入れ子 `id` をたどり、元の要素と、向きを逆にするかを返す
    fn resolve(
        &self,
        id: u64,
        inner: impl Fn(&Topology) -> Option<(u64, Option<bool>)>,
    ) -> Result<(u64, bool), Failure> {
        let mut id = id;
        let mut reversed = false;
        for _ in 0..=MAX_ORIENTED_DEPTH {
            match inner(self.topology(id)?) {
                Some((next, orientation)) => {
                    reversed ^= orientation == Some(false);
                    id = next;
                }
                None => return Ok((id, reversed)),
            }
        }
        Err(Failure::new(id, TopologyError::TooDeep))
    }

    /// 表現の要素 `item` のシェルと、立体の要素か（立体なら、初めのシェルが外側）
    fn item_shells(&self, item: u64) -> Result<(Vec<u64>, bool), Failure> {
        match self.topology(item)? {
            Topology::Solid { outer, voids } => Ok((
                std::iter::once(*outer)
                    .chain(voids.iter().copied())
                    .collect(),
                true,
            )),
            Topology::SurfaceModel { shells } => Ok((shells.clone(), false)),
            Topology::Shell { .. } | Topology::OrientedShell { .. } => Ok((vec![item], false)),
            _ => Err(Failure::new(item, TopologyError::MissingReference)),
        }
    }

    /// 表に足す前に、表現の要素 `item` の稜線の形を求め、頂点・稜線の許容誤差を広げ、頂点と稜線をまとめる
    /// （設計 6.1 の順。頂点を使う稜線の許容誤差以上にそろえるのは、表に稜線を足すときに行う）
    fn prepare(mut self, item: u64, uncertainty: Option<f64>, limit_ratio: f64) -> Self {
        let Ok((shells, _)) = self.item_shells(item) else {
            return self;
        };
        let (faces, uses) = self.collect_faces(&shells);
        self.set_limits(&uses, uncertainty, limit_ratio);
        // 1. 頂点を、その頂点を使う曲線に合わせて広げる
        let mut vertex_tolerances: BTreeMap<u64, f64> = BTreeMap::new();
        for &edge in uses.keys() {
            let shape = self.checked_edge_shape(edge).map(|(shape, deviations)| {
                for (vertex, deviation) in deviations {
                    let tolerance = vertex_tolerances.entry(vertex).or_insert(self.initial);
                    *tolerance = tolerance.max(deviation * self.factor);
                }
                shape
            });
            self.shapes.insert(edge, shape);
        }
        // 2. 頂点をまとめ、同じ頂点の間で重なる稜線をまとめる
        self.group_vertices(&vertex_tolerances);
        self.merge_edges(&uses);
        // 3. 稜線を、面の曲面に合わせて広げる
        for face in &faces {
            self.fit_edges_to_surface(face);
        }
        for group in &self.groups {
            if group.tolerance > self.initial {
                self.corrections.push(Correction::VertexToleranceWidened {
                    vertex: group.id,
                    tolerance: group.tolerance,
                });
            }
        }
        for (&edge, shape) in &self.shapes {
            if let Ok(shape) = shape
                && !self.merged_edges.contains_key(&edge)
                && shape.tolerance > self.initial
            {
                self.corrections.push(Correction::EdgeToleranceWidened {
                    edge,
                    tolerance: shape.tolerance,
                });
            }
        }
        self
    }

    /// シェル `shells` の面（向きの旗をたどった元の面。NURBS 曲面の面と、曲面を型にできなかった面は、
    /// `face` と同じく除く）と、面のループが稜線を使う回数と初めに使う面（ファイルの稜線から）
    fn collect_faces(&self, shells: &[u64]) -> (Vec<PreparedFace>, BTreeMap<u64, (usize, u64)>) {
        let mut faces = Vec::new();
        let mut uses: BTreeMap<u64, (usize, u64)> = BTreeMap::new();
        let mut seen = HashSet::new();
        for &shell in shells {
            let Ok((shell, _)) = self.resolve(shell, |topology| match topology {
                Topology::OrientedShell { shell, orientation } => Some((*shell, *orientation)),
                _ => None,
            }) else {
                continue;
            };
            let Ok(Topology::Shell {
                faces: shell_faces, ..
            }) = self.topology(shell)
            else {
                continue;
            };
            for &face in shell_faces {
                let Ok((face, _)) = self.resolve(face, |topology| match topology {
                    Topology::OrientedFace { face, orientation } => Some((*face, *orientation)),
                    _ => None,
                }) else {
                    continue;
                };
                if !seen.insert(face) {
                    continue;
                }
                let Ok(Topology::Face {
                    bounds, surface, ..
                }) = self.topology(face)
                else {
                    continue;
                };
                let surface = match surface {
                    Some(surface) => match self.geometries.surfaces.get(surface) {
                        Some(typed) if !matches!(typed.surface, SurfaceRef::Nurbs(_)) => {
                            Some(typed.surface.clone())
                        }
                        _ => continue,
                    },
                    None => None,
                };
                let mut edges = Vec::new();
                for &bound in bounds {
                    let Ok(Topology::FaceBound { bound: inner, .. }) = self.topology(bound) else {
                        continue;
                    };
                    let Ok(Topology::EdgeLoop { edges: oriented }) = self.topology(*inner) else {
                        continue;
                    };
                    for &oriented in oriented {
                        if let Ok((edge, _)) = self.resolve(oriented, |topology| match topology {
                            Topology::OrientedEdge { edge, orientation } => {
                                Some((*edge, *orientation))
                            }
                            _ => None,
                        }) {
                            uses.entry(edge).or_insert((0, face)).0 += 1;
                            edges.push(edge);
                        }
                    }
                }
                faces.push(PreparedFace {
                    id: face,
                    surface,
                    edges,
                });
            }
        }
        (faces, uses)
    }

    /// 要素の許容誤差の上限と初めの値を決める（上限は、稜線 `edges` の頂点と曲線の外形の箱の対角の長さに比
    /// `limit_ratio` を掛けた値。円・楕円はその全体を囲む箱、NURBS 曲線は制御点で箱を広げる。初めの値は、
    /// 文脈の許容誤差 `uncertainty`（なければ全体の許容誤差））
    fn set_limits(
        &mut self,
        edges: &BTreeMap<u64, (usize, u64)>,
        uncertainty: Option<f64>,
        limit_ratio: f64,
    ) {
        let mut points = Vec::new();
        for &edge in edges.keys() {
            let Ok(Topology::Edge {
                start, end, curve, ..
            }) = self.topology(edge)
            else {
                continue;
            };
            points.extend(
                [*start, *end]
                    .iter()
                    .filter_map(|&vertex| self.vertex_point(vertex).ok()),
            );
            let around = |center: Point3D<f64>, radius: f64| {
                [
                    center + Vector3D::new(-radius, -radius, -radius),
                    center + Vector3D::new(radius, radius, radius),
                ]
            };
            match self.geometries.curves.get(curve) {
                Some(Curve::Base(CurveRef::Circle(circle))) => {
                    points.extend(around(circle.center(), circle.radius()));
                }
                Some(Curve::Base(CurveRef::Ellipse(ellipse))) => {
                    points.extend(around(ellipse.center(), ellipse.semi_major_axis()));
                }
                Some(Curve::Base(CurveRef::Nurbs(nurbs))) => {
                    points.extend(
                        (0..nurbs.control_point_count())
                            .filter_map(|index| nurbs.control_point(index)),
                    );
                }
                _ => {}
            }
        }
        let mut bounds: Option<(Point3D<f64>, Point3D<f64>)> = None;
        for point in points {
            bounds = Some(match bounds {
                Some((low, high)) => (
                    Point3D::new(
                        low.x().min(point.x()),
                        low.y().min(point.y()),
                        low.z().min(point.z()),
                    ),
                    Point3D::new(
                        high.x().max(point.x()),
                        high.y().max(point.y()),
                        high.z().max(point.z()),
                    ),
                ),
                None => (point, point),
            });
        }
        let size = bounds.map_or(0.0, |(low, high)| low.distance_to(high));
        let limit = size * limit_ratio;
        self.limit = if limit.is_finite() {
            limit.max(self.global)
        } else {
            self.global
        };
        self.initial = uncertainty
            .filter(|value| value.is_finite())
            .map_or(self.global, |value| value.clamp(self.global, self.limit));
    }

    /// 頂点をまとめる: 点が両方の許容誤差の和の中で重なる頂点を、1 つの角にまとめる（1 本の稜線の両端の
    /// 頂点はまとめない。まとめた許容誤差が上限を超えるときもまとめない）
    fn group_vertices(&mut self, tolerances: &BTreeMap<u64, f64>) {
        let ends: HashSet<(u64, u64)> = self
            .shapes
            .values()
            .filter_map(|shape| shape.as_ref().ok())
            .map(|shape| (shape.first.min(shape.last), shape.first.max(shape.last)))
            .collect();
        // 格子の大きさを許容誤差の和の最大以上にし、隣の格子までを探す
        let cell = 2.0
            * tolerances
                .values()
                .fold(self.initial, |max, &value| max.max(value));
        let key = |point: Point3D<f64>| {
            (
                (point.x() / cell).floor() as i64,
                (point.y() / cell).floor() as i64,
                (point.z() / cell).floor() as i64,
            )
        };
        let mut grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
        for (&vertex, &tolerance) in tolerances {
            let Ok(point) = self.vertex_point(vertex) else {
                continue;
            };
            let (x, y, z) = key(point);
            let mut best: Option<(f64, usize)> = None;
            for neighbor in (-1..=1).flat_map(|dx| {
                (-1..=1).flat_map(move |dy| (-1..=1).map(move |dz| (x + dx, y + dy, z + dz)))
            }) {
                for &position in grid.get(&neighbor).into_iter().flatten() {
                    let Some(group) = self.groups.get(position) else {
                        continue;
                    };
                    let distance = group.point.distance_to(point);
                    let joinable = distance <= group.own_tolerance + tolerance
                        && group.tolerance.max(distance + tolerance) <= self.limit
                        && !group.members.iter().any(|&member| {
                            ends.contains(&(member.min(vertex), member.max(vertex)))
                        });
                    if joinable && best.is_none_or(|(nearest, _)| distance < nearest) {
                        best = Some((distance, position));
                    }
                }
            }
            match best.and_then(|(distance, position)| {
                self.groups
                    .get_mut(position)
                    .map(|group| (distance, position, group))
            }) {
                Some((distance, position, group)) => {
                    group.tolerance = group.tolerance.max(distance + tolerance);
                    group.members.push(vertex);
                    self.corrections.push(Correction::VertexMerged {
                        vertex,
                        into: group.id,
                    });
                    self.vertex_groups.insert(vertex, position);
                }
                None => {
                    let position = self.groups.len();
                    self.groups.push(VertexGroup {
                        id: vertex,
                        point,
                        own_tolerance: tolerance,
                        tolerance,
                        members: vec![vertex],
                        index: None,
                    });
                    grid.entry((x, y, z)).or_default().push(position);
                    self.vertex_groups.insert(vertex, position);
                }
            }
        }
    }

    /// 稜線をまとめる: 同じ頂点の間で、区間を分けた点が両方の許容誤差の和の中で重なり、それぞれ 1 つの面の
    /// ループだけが 1 回使う稜線の組を、1 本の稜線にまとめる（両端が同じ頂点の稜線はまとめない）
    fn merge_edges(&mut self, uses: &BTreeMap<u64, (usize, u64)>) {
        // 両端の頂点の組から、まとめる候補の稜線と、それを使う面
        let mut candidates: BTreeMap<(usize, usize), Vec<(u64, u64)>> = BTreeMap::new();
        for (&edge, &(count, face)) in uses {
            if count != 1 {
                continue;
            }
            let Some(Ok(shape)) = self.shapes.get(&edge) else {
                continue;
            };
            let (Some(&first), Some(&last)) = (
                self.vertex_groups.get(&shape.first),
                self.vertex_groups.get(&shape.last),
            ) else {
                continue;
            };
            if first != last {
                candidates
                    .entry((first.min(last), first.max(last)))
                    .or_default()
                    .push((edge, face));
            }
        }
        for edges in candidates.values() {
            let mut used = vec![false; edges.len()];
            for (position, &(edge, face)) in edges.iter().enumerate() {
                for (other_position, &(other, other_face)) in
                    edges.iter().enumerate().skip(position + 1)
                {
                    if used.get(position) == Some(&true) {
                        break;
                    }
                    // 同じ面のループの中の 2 本はまとめない（ループの形が変わるため）
                    if used.get(other_position) == Some(&true) || face == other_face {
                        continue;
                    }
                    if self.merge_edge_pair(edge, other) {
                        for marked in [position, other_position] {
                            if let Some(flag) = used.get_mut(marked) {
                                *flag = true;
                            }
                        }
                    }
                }
            }
        }
    }

    /// 稜線 `other` が稜線 `edge` と重なれば、`edge` にまとめる（まとめたか）
    fn merge_edge_pair(&mut self, edge: u64, other: u64) -> bool {
        let (Some(Ok(shape)), Some(Ok(other_shape))) =
            (self.shapes.get(&edge), self.shapes.get(&other))
        else {
            return false;
        };
        // `other` の B-rep の向きが、`edge` の B-rep の向きと同じか
        let same_direction =
            self.vertex_groups.get(&other_shape.first) == self.vertex_groups.get(&shape.first);
        let point = |shape: &EdgeShape, fraction: f64| {
            let (t0, t1) = shape.range;
            shape.curve.point_at_parameter(t0 + (t1 - t0) * fraction)
        };
        let distance = (0..=EDGE_COMPARISON_SEGMENTS).fold(0.0_f64, |max, index| {
            let fraction = index as f64 / EDGE_COMPARISON_SEGMENTS as f64;
            let other_fraction = if same_direction {
                fraction
            } else {
                1.0 - fraction
            };
            max.max(point(shape, fraction).distance_to(point(other_shape, other_fraction)))
        });
        let tolerance = shape.tolerance.max(other_shape.tolerance + distance);
        if !(distance <= shape.tolerance + other_shape.tolerance && tolerance <= self.limit) {
            return false;
        }
        // `other` のファイルの始点から終点へ、まとめ先の B-rep の稜線が進むか
        let along = other_shape.along == same_direction;
        if let Some(Ok(shape)) = self.shapes.get_mut(&edge) {
            shape.tolerance = tolerance;
        }
        self.merged_edges.insert(other, (edge, along));
        self.corrections.push(Correction::EdgeMerged {
            edge: other,
            into: edge,
        });
        true
    }

    /// 面 `face` の稜線の許容誤差を、面の曲面からのずれに合わせて広げる（上限を超える稜線があれば、どの
    /// 稜線も広げず、面を外す理由を残す）
    fn fit_edges_to_surface(&mut self, face: &PreparedFace) {
        let Some(surface) = &face.surface else {
            return;
        };
        let mut widened = Vec::new();
        for &referenced in &face.edges {
            let edge = self
                .merged_edges
                .get(&referenced)
                .map_or(referenced, |&(into, _)| into);
            let Some(Ok(shape)) = self.shapes.get(&edge) else {
                continue;
            };
            // パラメータ曲線を作れない組は、面を足すときに理由を残す
            let Some(deviation) =
                ParameterCurve::surface_deviation(surface, &shape.curve, shape.range)
            else {
                continue;
            };
            let tolerance = deviation * self.factor;
            if tolerance > self.limit {
                self.face_failures.insert(
                    face.id,
                    Failure::new(referenced, TopologyError::ToleranceExceeded { deviation }),
                );
                return;
            }
            if tolerance > shape.tolerance {
                widened.push((edge, tolerance));
            }
        }
        for (edge, tolerance) in widened {
            if let Some(Ok(shape)) = self.shapes.get_mut(&edge) {
                shape.tolerance = shape.tolerance.max(tolerance);
            }
        }
    }

    /// 表現の要素 `item` の B-rep と、外した要素（面が 1 つもできなければ、B-rep は `None`）
    fn item(mut self, item: u64) -> (Option<Built>, Vec<TopologySkipped>) {
        // シェルと、立体の要素か（立体なら、初めのシェルが外側）
        let (shells, solid) = match self.item_shells(item) {
            Ok(shells) => shells,
            Err(failure) => {
                self.skip(item, failure);
                return (None, self.skipped);
            }
        };
        let mut shell_faces = Vec::with_capacity(shells.len());
        for &shell in &shells {
            match self.shell_faces(shell) {
                Ok((faces, closed)) => shell_faces.push((shell, faces, closed)),
                Err(failure) => self.skip(shell, failure),
            }
        }
        if self.built_faces.is_empty() {
            return (None, self.skipped);
        }
        self.orient_faces();

        let mut shell_indices: Vec<(u64, ShellIndex)> = Vec::new();
        let mut open_shells = Vec::new();
        for (shell, faces, closed) in shell_faces {
            match self.brep.add_shell(faces) {
                Ok(index) => {
                    if closed && !self.brep.is_shell_closed(index) {
                        open_shells.push(shell);
                    }
                    shell_indices.push((shell, index));
                }
                Err(error) => self.skip(shell, Failure::new(shell, TopologyError::Brep(error))),
            }
        }
        let built_shells: Vec<u64> = shell_indices.iter().map(|&(shell, _)| shell).collect();
        let shell_indices: Vec<ShellIndex> =
            shell_indices.into_iter().map(|(_, index)| index).collect();
        let mut solid_index = None;
        if solid {
            // 外側のシェルがなければ、立体を作らない
            match built_shells.first() {
                Some(&first) if shells.first() == Some(&first) => {
                    match self.brep.add_solid(shell_indices.clone()) {
                        Ok(index) => solid_index = Some(index),
                        Err(error) => {
                            self.skip(item, Failure::new(item, TopologyError::Brep(error)))
                        }
                    }
                }
                _ => {
                    let outer = shells.first().copied().unwrap_or(item);
                    self.skip(item, Failure::new(outer, TopologyError::MissingReference));
                }
            }
        }

        let (ids, mut indices): (Vec<u64>, Vec<FaceIndex>) = self
            .built_faces
            .iter()
            .map(|face| (face.id, face.index))
            .unzip();
        let brep = self.brep.into_canonical_order(&mut indices);
        let built = Built {
            item,
            brep,
            shells: shell_indices,
            solid: solid_index,
            faces: ids.into_iter().zip(indices).collect(),
            corrections: self.corrections,
            unoriented_faces: self.unoriented,
            open_shells,
        };
        (Some(built), self.skipped)
    }

    fn skip(&mut self, id: u64, failure: Failure) {
        self.skipped.push(TopologySkipped {
            id,
            cause: failure.cause,
            reason: failure.reason,
        });
    }

    /// シェル `shell` の面を作り、作れた面と、ファイルで閉じたシェルとされているかを返す（`ORIENTED_SHELL` の
    /// `.F.` は、面をすべて逆にする）
    fn shell_faces(&mut self, shell: u64) -> Result<(Vec<FaceIndex>, bool), Failure> {
        let (shell, reversed) = self.resolve(shell, |topology| match topology {
            Topology::OrientedShell { shell, orientation } => Some((*shell, *orientation)),
            _ => None,
        })?;
        let Topology::Shell { faces, closed } = self.topology(shell)? else {
            return Err(Failure::new(shell, TopologyError::MissingReference));
        };
        let mut indices = Vec::with_capacity(faces.len());
        let mut seen = HashSet::with_capacity(faces.len());
        for &face in faces {
            let (face, face_reversed) = match self.resolve(face, |topology| match topology {
                Topology::OrientedFace { face, orientation } => Some((*face, *orientation)),
                _ => None,
            }) {
                Ok(resolved) => resolved,
                Err(failure) => {
                    self.skip(face, failure);
                    continue;
                }
            };
            let reversed = reversed ^ face_reversed;
            let index = match self.faces.get(&face) {
                Some(&(index, built_reversed)) if built_reversed == reversed => index,
                Some(_) => {
                    self.skip(face, Failure::new(face, TopologyError::ReversedReuse));
                    None
                }
                None => {
                    let index = match self.face(face, reversed) {
                        Ok(index) => Some(index),
                        Err(failure) => {
                            self.skip(face, failure);
                            None
                        }
                    };
                    self.faces.insert(face, (index, reversed));
                    index
                }
            };
            if let Some(index) = index
                && seen.insert(index)
            {
                indices.push(index);
            }
        }
        Ok((indices, *closed))
    }

    /// 面 `id` を作る（`reversed` なら、ループと面の向きを逆にする）
    fn face(&mut self, id: u64, reversed: bool) -> Result<FaceIndex, Failure> {
        let Topology::Face {
            bounds,
            surface,
            same_sense,
        } = self.topology(id)?
        else {
            return Err(Failure::new(id, TopologyError::MissingReference));
        };
        let (bounds, surface, same_sense) = (bounds.clone(), *surface, *same_sense);
        // NURBS 曲面の面は、稜線を作る前に外す
        let typed = match surface {
            Some(surface) => {
                let typed = self
                    .geometries
                    .surfaces
                    .get(&surface)
                    .ok_or(Failure::new(surface, TopologyError::MissingGeometry))?;
                if matches!(typed.surface, SurfaceRef::Nurbs(_)) {
                    return Err(Failure::new(surface, TopologyError::NurbsSurface));
                }
                Some(typed)
            }
            None => None,
        };
        if let Some(&failure) = self.face_failures.get(&id) {
            return Err(failure);
        }
        let mut loops = Vec::with_capacity(bounds.len());
        let mut loop_bounds = Vec::with_capacity(bounds.len());
        for &bound in &bounds {
            if let Some((mut uses, outer)) = self.bound(bound)? {
                if reversed {
                    reverse(&mut uses);
                }
                loops.push(uses);
                loop_bounds.push((bound, outer));
            }
        }
        // 面の向きは、ファイルの `same_sense` と、型の曲面の法線が STEP の法線と逆かの組で決める。面ごとに
        // 決まる曲面では、ループの回りで決める（初めは表とし、後で確かめる）
        let (surface, same_sense, per_face) = match typed {
            Some(typed) => {
                let same_sense = same_sense.unwrap_or(true);
                match typed.sense {
                    Sense::Same => (typed.surface.clone(), same_sense ^ reversed, false),
                    Sense::Reversed => (typed.surface.clone(), !same_sense ^ reversed, false),
                    Sense::PerFace => (typed.surface.clone(), true, true),
                }
            }
            // ループの法線を曲面の法線にするため、面は表
            None => (self.loop_plane(id, &loops)?, true, false),
        };
        let index = self
            .brep
            .add_face(surface, same_sense, loops)
            .map_err(|error| Failure::new(id, TopologyError::Brep(error)))?;
        self.built_faces.push(BuiltFace {
            id,
            index,
            bounds: loop_bounds,
            per_face,
        });
        Ok(index)
    }

    /// ループ `bound`（`FACE_BOUND`）の稜線の使用と、外側のループ（`FACE_OUTER_BOUND`）か（頂点だけのループは
    /// `None`）
    fn bound(&mut self, bound: u64) -> Result<Option<(Vec<EdgeUse>, bool)>, Failure> {
        let Topology::FaceBound {
            bound: inner,
            orientation,
            outer,
        } = self.topology(bound)?
        else {
            return Err(Failure::new(bound, TopologyError::MissingReference));
        };
        let (inner, orientation, outer) = (*inner, *orientation, *outer);
        let mut uses = match self.topology(inner)? {
            Topology::EdgeLoop { edges } => {
                let edges = edges.clone();
                let mut uses = Vec::with_capacity(edges.len());
                for oriented in edges {
                    let (edge, reversed) = self.resolve(oriented, |topology| match topology {
                        Topology::OrientedEdge { edge, orientation } => Some((*edge, *orientation)),
                        _ => None,
                    })?;
                    let (index, along) = self.edge(edge)?;
                    uses.push(EdgeUse {
                        edge: index,
                        forward: along != reversed,
                    });
                }
                uses
            }
            Topology::PolyLoop { points } => {
                let points = points.clone();
                self.poly_loop(inner, &points)?
            }
            Topology::VertexLoop { .. } => return Ok(None),
            _ => return Err(Failure::new(inner, TopologyError::MissingReference)),
        };
        if orientation == Some(false) {
            reverse(&mut uses);
        }
        Ok(Some((uses, outer)))
    }

    /// 多角形のループ `id` の稜線の使用（点の組で稜線を共有する）
    fn poly_loop(&mut self, id: u64, points: &[u64]) -> Result<Vec<EdgeUse>, Failure> {
        let mut vertices = Vec::with_capacity(points.len());
        for &point in points {
            let vertex = match self.points.get(&point) {
                Some(&vertex) => vertex,
                None => {
                    let position = self.point(point)?;
                    let vertex = self
                        .brep
                        .add_vertex(position, self.initial)
                        .map_err(|error| Failure::new(point, TopologyError::Brep(error)))?;
                    self.points.insert(point, vertex);
                    vertex
                }
            };
            vertices.push(vertex);
        }
        let mut uses = Vec::with_capacity(vertices.len());
        for (position, &start) in vertices.iter().enumerate() {
            let Some(&end) = vertices.get((position + 1) % vertices.len()) else {
                continue;
            };
            let key = if self.segments.contains_key(&(end, start)) {
                (end, start)
            } else {
                (start, end)
            };
            let (edge, edge_start) = match self.segments.get(&key) {
                Some(&segment) => segment,
                None => {
                    let edge = self
                        .segment(start, end)
                        .map_err(|reason| Failure::new(id, reason))?;
                    self.segments.insert(key, (edge, start));
                    (edge, start)
                }
            };
            uses.push(EdgeUse {
                edge,
                forward: edge_start == start,
            });
        }
        Ok(uses)
    }

    /// 頂点 `start` から `end` への線分の稜線
    fn segment(
        &mut self,
        start: VertexIndex,
        end: VertexIndex,
    ) -> Result<EdgeIndex, TopologyError> {
        let (line, range) = segment(
            self.brep.vertex(start).point(),
            self.brep.vertex(end).point(),
        )?;
        self.brep
            .add_edge(start, end, line, range, self.initial)
            .map_err(TopologyError::Brep)
    }

    fn point(&self, id: u64) -> Result<Point3D<f64>, Failure> {
        match self.records.geometry.get(&id) {
            Some(Geometry::Point([x, y, z])) => Ok(Point3D::new(*x, *y, *z)),
            _ => Err(Failure::new(id, TopologyError::MissingReference)),
        }
    }

    /// ファイルの頂点 `id` の点
    fn vertex_point(&self, id: u64) -> Result<Point3D<f64>, Failure> {
        let Topology::Vertex { point } = self.topology(id)? else {
            return Err(Failure::new(id, TopologyError::MissingReference));
        };
        self.point(*point)
    }

    /// ファイルの頂点 `id` の B-rep の頂点（同じ角にまとめた頂点は、1 つの B-rep の頂点になる）
    fn vertex(&mut self, id: u64) -> Result<VertexIndex, Failure> {
        if !self.vertex_groups.contains_key(&id) {
            // 前もって集めなかった頂点は、1 つだけの組にする
            let point = self.vertex_point(id)?;
            self.vertex_groups.insert(id, self.groups.len());
            self.groups.push(VertexGroup {
                id,
                point,
                own_tolerance: self.initial,
                tolerance: self.initial,
                members: vec![id],
                index: None,
            });
        }
        let group = self
            .vertex_groups
            .get(&id)
            .and_then(|&position| self.groups.get(position))
            .ok_or(Failure::new(id, TopologyError::MissingReference))?;
        if let Some(index) = group.index {
            return Ok(index);
        }
        let (position, point, tolerance) = (
            self.vertex_groups.get(&id).copied(),
            group.point,
            group.tolerance,
        );
        let index = self
            .brep
            .add_vertex(point, tolerance)
            .map_err(|error| Failure::new(id, TopologyError::Brep(error)))?;
        if let Some(group) = position.and_then(|position| self.groups.get_mut(position)) {
            group.index = Some(index);
        }
        Ok(index)
    }

    /// ファイルの稜線 `id` の B-rep の稜線と、B-rep の稜線がファイルの稜線と同じ向き（ファイルの始点から終点へ
    /// 区間が増える）か（まとめた稜線は、まとめ先の B-rep の稜線）
    fn edge(&mut self, id: u64) -> Result<(EdgeIndex, bool), Failure> {
        if let Some(&result) = self.edges.get(&id) {
            return result;
        }
        let result = match self.merged_edges.get(&id) {
            Some(&(into, along)) => self.edge(into).map(|(index, _)| (index, along)),
            None => self.build_edge(id),
        };
        self.edges.insert(id, result);
        result
    }

    fn build_edge(&mut self, id: u64) -> Result<(EdgeIndex, bool), Failure> {
        let shape = match self.shapes.remove(&id) {
            Some(shape) => shape?,
            None => {
                // 前もって求めなかった稜線は、ここで求める（頂点のずれは、稜線の許容誤差に含め、表に足すときに
                // 頂点の許容誤差を広げる）
                let (mut shape, deviations) = self.checked_edge_shape(id)?;
                for (_, deviation) in deviations {
                    shape.tolerance = shape.tolerance.max(deviation * self.factor);
                }
                shape
            }
        };
        if let Some(correction) = shape.correction {
            self.corrections.push(correction);
        }
        let first = self.vertex(shape.first)?;
        let last = self.vertex(shape.last)?;
        let index = self
            .brep
            .add_edge(first, last, shape.curve, shape.range, shape.tolerance)
            .map_err(|error| Failure::new(id, TopologyError::Brep(error)))?;
        Ok((index, shape.along))
    }

    /// `edge_shape` に、頂点のずれが上限を超えないことの確かめを足したもの
    fn checked_edge_shape(&self, id: u64) -> Result<(EdgeShape, [(u64, f64); 2]), Failure> {
        let (shape, deviations) = self.edge_shape(id)?;
        for (vertex, deviation) in deviations {
            if deviation * self.factor > self.limit {
                return Err(Failure::new(
                    vertex,
                    TopologyError::ToleranceExceeded { deviation },
                ));
            }
        }
        Ok((shape, deviations))
    }

    /// ファイルの稜線 `id` の B-rep の稜線の形と、両端の頂点の、曲線の区間の端からのずれ（ファイルの頂点と
    /// ずれの組）
    fn edge_shape(&self, id: u64) -> Result<(EdgeShape, [(u64, f64); 2]), Failure> {
        let Topology::Edge {
            start,
            end,
            curve,
            same_sense,
        } = self.topology(id)?
        else {
            return Err(Failure::new(id, TopologyError::MissingReference));
        };
        let (start, end, curve_id) = (*start, *end, *curve);
        let written_sense = *same_sense;
        let same_sense = written_sense.unwrap_or(true);
        let closed = start == end;
        let from = self.vertex_point(start)?;
        let to = self.vertex_point(end)?;
        let curve = self
            .geometries
            .curves
            .get(&curve_id)
            .ok_or(Failure::new(curve_id, TopologyError::MissingGeometry))?;
        let fail = |reason| Failure::new(id, reason);
        let tolerance = self.initial;
        let mut correction = None;
        // 型の曲線・区間・近似のずれ・ファイルの向きに区間が増えるか・頂点の順で向きを決めたか（開いた曲線）
        let (curve, range, deviation, along, by_vertices) = match curve {
            Curve::Base(CurveRef::Line(line)) => {
                if closed {
                    return Err(fail(TopologyError::ClosedOnOpenCurve));
                }
                if line.distance_to_point(from).max(line.distance_to_point(to)) > tolerance {
                    // 頂点が直線から外れていれば、頂点の間の線分にする
                    let (segment, range) = segment(from, to).map_err(fail)?;
                    correction = Some(Correction::EdgeToSegment { edge: id });
                    (segment, range, 0.0, true, false)
                } else {
                    let (range, along) =
                        open_range(line.parameter_for_point(from), line.parameter_for_point(to))
                            .ok_or(fail(TopologyError::InvalidEdgeRange))?;
                    (CurveRef::Line(*line), range, 0.0, along, true)
                }
            }
            Curve::Base(CurveRef::Circle(circle)) => {
                let x = circle.ref_direction().as_vector();
                let y = circle.axis().as_vector().cross(x);
                let angle = |point: Point3D<f64>| {
                    let offset = point - circle.center();
                    offset.dot(y).atan2(offset.dot(x))
                };
                let (range, along) = closed_range(angle(from), angle(to), closed, same_sense);
                (CurveRef::Circle(*circle), range, 0.0, along, false)
            }
            Curve::Base(CurveRef::Ellipse(ellipse)) => {
                let angle = |point| ellipse_angle(ellipse, point, tolerance);
                let (range, along) = closed_range(angle(from), angle(to), closed, same_sense);
                (CurveRef::Ellipse(*ellipse), range, 0.0, along, false)
            }
            Curve::Base(CurveRef::Nurbs(nurbs)) => {
                let open = !is_closed(nurbs, tolerance);
                let (range, along) =
                    nurbs_range(nurbs, from, to, closed, same_sense, tolerance).map_err(fail)?;
                (CurveRef::Nurbs(nurbs.clone()), range, 0.0, along, open)
            }
            Curve::Conic(conic) => {
                if closed {
                    return Err(fail(TopologyError::ClosedOnOpenCurve));
                }
                let (range, along) =
                    open_range(conic_parameter(conic, from), conic_parameter(conic, to))
                        .ok_or(fail(TopologyError::InvalidEdgeRange))?;
                let (nurbs, deviation) = conic
                    .segment(range, tolerance)
                    .ok_or(fail(TopologyError::ApproximationFailed))?;
                (CurveRef::Nurbs(nurbs), range, deviation, along, true)
            }
        };
        // 開いた曲線では、`same_sense` と頂点の順が食い違えば、頂点の順を使う
        if by_vertices && written_sense.is_some_and(|written| written != along) {
            correction = Some(Correction::EdgeSense { edge: id });
        }
        let (first, last, first_point, last_point) = if along {
            (start, end, from, to)
        } else {
            (end, start, to, from)
        };
        let deviations = [
            (
                first,
                first_point.distance_to(curve.point_at_parameter(range.0)),
            ),
            (
                last,
                last_point.distance_to(curve.point_at_parameter(range.1)),
            ),
        ];
        let shape = EdgeShape {
            first,
            last,
            curve,
            range,
            along,
            tolerance: deviation.max(self.initial),
            correction,
        };
        Ok((shape, deviations))
    }

    /// 曲面のない面 `id` の、ループの載る平面（法線は、最も大きいループが反時計回りに見える向き）
    fn loop_plane(&self, id: u64, loops: &[Vec<EdgeUse>]) -> Result<SurfaceRef<f64>, Failure> {
        let fail = || Failure::new(id, TopologyError::NoLoopPlane);
        let mut best: Option<(Vector3D<f64>, Point3D<f64>)> = None;
        for uses in loops {
            let points: Vec<Point3D<f64>> = uses
                .iter()
                .flat_map(|&edge_use| self.use_points(edge_use))
                .collect();
            let Some(&first) = points.first() else {
                continue;
            };
            // ニューウェルの方法で、ループの囲む向きつきの面積の 2 倍の法線を求める
            let mut normal = Vector3D::new(0.0, 0.0, 0.0);
            let mut sum = Vector3D::new(0.0, 0.0, 0.0);
            for (position, &point) in points.iter().enumerate() {
                let next = points.get(position + 1).copied().unwrap_or(first);
                normal = normal + (point - first).cross(next - first);
                sum = sum + (point - first);
            }
            let centroid = first + sum * (1.0 / points.len() as f64);
            if best.is_none_or(|(best, _)| normal.length() > best.length()) {
                best = Some((normal, centroid));
            }
        }
        let (normal, centroid) = best.ok_or_else(fail)?;
        let normal = Direction3D::from_vector(normal).ok_or_else(fail)?;
        TopoPlane3D::from_point_and_normal(centroid, normal)
            .map(SurfaceRef::Plane)
            .ok_or_else(fail)
    }

    /// 稜線の使用に沿って取った点（終わりの点は、次の使用の初めの点なので含めない）
    fn use_points(&self, edge_use: EdgeUse) -> Vec<Point3D<f64>> {
        let edge = self.brep.edge(edge_use.edge);
        let (t0, t1) = edge.parameter_range();
        (0..SAMPLES_PER_USE)
            .map(|sample| {
                let fraction = sample as f64 / SAMPLES_PER_USE as f64;
                let t = if edge_use.forward {
                    t0 + (t1 - t0) * fraction
                } else {
                    t1 - (t1 - t0) * fraction
                };
                edge.curve().point_at_parameter(t)
            })
            .collect()
    }

    /// ループの規則に合わない面を直す: 面を裏返す、または内側のループを 1 つ逆にする。直せない面と、面ごとに
    /// 向きが決まる曲面の上で表と裏のどちらも規則に合う面は、向きを決められなかった面とする
    fn orient_faces(&mut self) {
        for face in &self.built_faces {
            let index = face.index;
            let oriented = self.brep.is_face_oriented(index);
            if oriented && !face.per_face {
                continue;
            }
            let reversed_oriented = self.brep.reverse_face(index).is_ok() && {
                let reversed_oriented = self.brep.is_face_oriented(index);
                if !reversed_oriented || oriented {
                    let _ = self.brep.reverse_face(index);
                }
                reversed_oriented
            };
            match (oriented, reversed_oriented) {
                (true, true) => {
                    self.unoriented.push(face.id);
                    continue;
                }
                (true, false) => continue,
                (false, true) => {
                    // 面ごとに向きが決まる曲面では、ファイルの旗の誤りではない
                    if !face.per_face {
                        self.corrections
                            .push(Correction::FaceReversed { face: face.id });
                    }
                    continue;
                }
                (false, false) => {}
            }
            // 内側のループ（外側のループと書かれていないもの）を、1 つずつ逆にしてみる
            let loops = self.brep.face(index).loops().to_vec();
            let mut fixed = false;
            if loops.len() >= 2 {
                for (&loop_index, &(bound, outer)) in loops.iter().zip(&face.bounds) {
                    if outer || self.brep.reverse_loop(loop_index).is_err() {
                        continue;
                    }
                    if self.brep.is_face_oriented(index) {
                        self.corrections.push(Correction::LoopReversed {
                            face: face.id,
                            bound,
                        });
                        fixed = true;
                        break;
                    }
                    let _ = self.brep.reverse_loop(loop_index);
                }
            }
            if !fixed {
                self.unoriented.push(face.id);
            }
        }
    }
}

/// 点 `from` から `to` への線分の曲線と区間
fn segment(
    from: Point3D<f64>,
    to: Point3D<f64>,
) -> Result<(CurveRef<f64>, (f64, f64)), TopologyError> {
    let line = TopoInfiniteLine3D::new(from, to - from).ok_or(TopologyError::InvalidEdgeRange)?;
    let length = line.parameter_for_point(to);
    Ok((CurveRef::Line(line), (0.0, length)))
}

/// ループを逆にする（稜線の使用を逆の順・逆の向きにする）
fn reverse(uses: &mut [EdgeUse]) {
    uses.reverse();
    for edge_use in uses {
        edge_use.forward = !edge_use.forward;
    }
}

/// 開いた曲線の、パラメータ `from` から `to` への稜線の区間と、区間が増える向きか（同じなら `None`）
fn open_range(from: f64, to: f64) -> Option<((f64, f64), bool)> {
    match from.partial_cmp(&to)? {
        std::cmp::Ordering::Less => Some(((from, to), true)),
        std::cmp::Ordering::Greater => Some(((to, from), false)),
        std::cmp::Ordering::Equal => None,
    }
}

/// 円・楕円の、角度 `from` から `to` への稜線の区間と、区間が増える向きか
///
/// `same_sense` なら角度の増える向き、そうでなければ減る向きに進む。両端が同じ頂点なら 1 周とする。
fn closed_range(from: f64, to: f64, same_vertex: bool, same_sense: bool) -> ((f64, f64), bool) {
    let (start, end) = if same_sense { (from, to) } else { (to, from) };
    let start = start.rem_euclid(TAU);
    let span = if same_vertex {
        TAU
    } else {
        (end - start).rem_euclid(TAU)
    };
    ((start, start + span), same_sense)
}

/// NURBS 曲線が閉じているか（両端の点が、許容誤差の中で重なる）
fn is_closed(curve: &TopoNurbsCurve3D<f64>, tolerance: f64) -> bool {
    let (start, end) = curve.parameter_domain();
    match (
        curve.point_at_parameter(start),
        curve.point_at_parameter(end),
    ) {
        (Some(a), Some(b)) => a.distance_to(b) <= tolerance,
        _ => false,
    }
}

/// NURBS 曲線の、点 `from` から `to` への稜線の区間と、区間が増える向きか
fn nurbs_range(
    curve: &TopoNurbsCurve3D<f64>,
    from: Point3D<f64>,
    to: Point3D<f64>,
    same_vertex: bool,
    same_sense: bool,
    tolerance: f64,
) -> Result<((f64, f64), bool), TopologyError> {
    let (lower, upper) = curve.parameter_domain();
    let closed = is_closed(curve, tolerance);
    let seam = curve
        .point_at_parameter(lower)
        .ok_or(TopologyError::InvalidEdgeRange)?;
    let on_seam = |point: Point3D<f64>| point.distance_to(seam) <= tolerance;
    if same_vertex {
        // 閉じた NURBS 曲線の全体の稜線は、頂点が継ぎ目にあるものだけ読む（区間を回せないため）
        return if !closed {
            Err(TopologyError::ClosedOnOpenCurve)
        } else if on_seam(from) {
            Ok(((lower, upper), same_sense))
        } else {
            Err(TopologyError::InvalidEdgeRange)
        };
    }
    let start = nurbs_parameter(curve, from, tolerance).ok_or(TopologyError::InvalidEdgeRange)?;
    let end = nurbs_parameter(curve, to, tolerance).ok_or(TopologyError::InvalidEdgeRange)?;
    if !closed {
        return open_range(start, end).ok_or(TopologyError::InvalidEdgeRange);
    }
    // 閉じた曲線では、`same_sense` の向きに進む。継ぎ目の点は、進む向きに合う端（初めは下の端、終わりは
    // 上の端）にする
    let (first, last, first_point, last_point) = if same_sense {
        (start, end, from, to)
    } else {
        (end, start, to, from)
    };
    let first = if on_seam(first_point) { lower } else { first };
    let last = if on_seam(last_point) { upper } else { last };
    if first < last {
        Ok(((first, last), same_sense))
    } else {
        Err(TopologyError::InvalidEdgeRange)
    }
}

/// NURBS 曲線の上で、点 `point` に最も近い点のパラメータ（点を取って最も近いものから、ニュートン法で、曲線の
/// 上を動く距離が `tolerance` 以下になるまで詰める）
fn nurbs_parameter(
    curve: &TopoNurbsCurve3D<f64>,
    point: Point3D<f64>,
    tolerance: f64,
) -> Option<f64> {
    let (lower, upper) = curve.parameter_domain();
    let samples = curve.control_point_count().max(2) * SAMPLES_PER_CONTROL_POINT;
    let mut best = (f64::INFINITY, lower);
    for sample in 0..=samples {
        let t = lower + (upper - lower) * (sample as f64 / samples as f64);
        let distance = curve.point_at_parameter(t)?.distance_to(point);
        if distance < best.0 {
            best = (distance, t);
        }
    }
    let mut t = best.1;
    for _ in 0..MAX_NEWTON_STEPS {
        let offset = curve.point_at_parameter(t)? - point;
        let derivative = curve.derivative_at(t)?;
        let speed = derivative.length();
        if speed <= DERIVATIVE_ZERO_THRESHOLD_F64 {
            break;
        }
        let next = (t - offset.dot(derivative) / (speed * speed)).clamp(lower, upper);
        let moved = (next - t).abs() * speed;
        t = next;
        if moved <= tolerance {
            break;
        }
    }
    Some(t)
}

/// 楕円の上で、点 `point` に最も近い点の角度（半軸で割った座標の角度から、ニュートン法で、楕円の上を動く
/// 距離が `tolerance` 以下になるまで詰める）
fn ellipse_angle(ellipse: &TopoEllipse3D<f64>, point: Point3D<f64>, tolerance: f64) -> f64 {
    let offset = point - ellipse.center();
    let (a, b) = (ellipse.semi_major_axis(), ellipse.semi_minor_axis());
    let x = offset.dot(ellipse.major_axis_direction().as_vector());
    let y = offset.dot(ellipse.minor_axis_direction().as_vector());
    let mut angle = (y / b).atan2(x / a);
    for _ in 0..MAX_NEWTON_STEPS {
        let (sin, cos) = angle.sin_cos();
        // 楕円の点から点 `point` への向き・接線・2 階の微分（楕円の座標で）
        let (dx, dy) = (a * cos - x, b * sin - y);
        let (tx, ty) = (-a * sin, b * cos);
        let slope = dx * tx + dy * ty;
        let curvature = tx * tx + ty * ty - (dx * a * cos + dy * b * sin);
        if curvature <= DERIVATIVE_ZERO_THRESHOLD_F64 {
            break;
        }
        let step = slope / curvature;
        angle -= step;
        if step.abs() * (tx * tx + ty * ty).sqrt() <= tolerance {
            break;
        }
    }
    angle
}

/// 放物線・双曲線の上の点 `point` のパラメータ（`y` の軸の成分から求める）
fn conic_parameter(conic: &Conic, point: Point3D<f64>) -> f64 {
    match *conic {
        Conic::Parabola {
            center,
            y,
            focal_distance,
            ..
        } => (point - center).dot(y) / (2.0 * focal_distance),
        Conic::Hyperbola {
            center,
            y,
            semi_imaginary_axis,
            ..
        } => ((point - center).dot(y) / semi_imaginary_axis).asinh(),
    }
}
