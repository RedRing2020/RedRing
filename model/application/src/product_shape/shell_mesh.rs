//! シェルの B-rep を、隣の面と境界の頂点をそろえた三角形に分ける（設計: TRIMMED_FACE_TESSELLATION_DESIGN.md 4〜8 章）
//!
//! 前の段（B-rep の全体）: 稜線を 1 回ずつ点に分け（端は頂点の点）、面ごとにパラメータ曲線で (u, v) の多角形に
//! し、切れ目を入れた範囲を作って、多角形の辺の交わりを確かめる。交われば、その辺を含む稜線を細かく分け直して、
//! 作り直す（回数の上限まで）。
//!
//! 後の段（面ごと）: 範囲を三角形に分け、面の向きに合わせて三角形の回りと頂点の法線をそろえる。頂点と稜線の点は
//! B-rep の全体で通しの番号を持ち、隣り合う面の三角形は、境界で同じ番号を使う。

use std::collections::{HashMap, HashSet};
use std::fmt;

use analysis::consts::tessellation::MAX_CHORD_ANGLE_DEGREES;
use geo_algorithms::{
    Angle, CurveDiscretizationError, CurveDiscretizationSettings, CurvePoint, DiscretizableCurve,
    FaceRegion, FaceTriangulationError, FaceTriangulationSettings, FaceVertex, LoopPoint, Point2D,
    Point3D, TriangulationSurface, Vector3D, chord_angle_step,
};
use geo_contracts::{
    ConicalSurface3DProperties, CylindricalSurface3DProperties, NurbsSurface3DProperties,
    SphericalSurface3DProperties, TorusSurface3DProperties,
};
use geo_topology::{Brep, CurveRef, EdgeIndex, FaceIndex, SurfaceRef, VertexIndex};

/// 1 本の稜線から作る点の数の上限
const MAX_EDGE_POINTS: usize = 100_000;
/// 1 つの面の頂点の数の上限
const MAX_FACE_POINTS: usize = 2_000_000;
/// 面の内側に点を足す繰り返しの回数の上限
const MAX_REFINEMENT_PASSES: usize = 64;
/// (u, v) の多角形が交わるとき、稜線を分け直す回数の上限（1 回ごとに、その稜線の許容弦誤差を半分にする）
const MAX_RESPLITS: usize = 4;

/// 面を三角形にできない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellFaceError {
    /// 面の稜線を点に分けられない
    Edge(CurveDiscretizationError),
    /// パラメータ曲線を評価できない
    ParameterCurve,
    /// 稜線を分け直しても、(u, v) の多角形の辺が交わる
    Crossing,
    /// 切れ目を入れた範囲を作れない、または三角形に分けられない
    Triangulation(FaceTriangulationError),
    /// シェルの配置を三角形に当てられない
    Placement,
}

impl fmt::Display for ShellFaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Edge(error) => write!(f, "an edge of the face cannot be divided: {error}"),
            Self::ParameterCurve => f.write_str("a parameter curve cannot be evaluated"),
            Self::Crossing => f.write_str("the loops of the face cross"),
            Self::Triangulation(error) => write!(f, "the face cannot be triangulated: {error}"),
            Self::Placement => f.write_str("the placement cannot be applied to the triangles"),
        }
    }
}

impl std::error::Error for ShellFaceError {}

/// B-rep の三角形（点は B-rep の全体で通しの番号。面は B-rep の面の順）
#[derive(Debug)]
pub(crate) struct ShellMesh {
    pub(crate) points: Vec<Point3D<f64>>,
    pub(crate) faces: Vec<Result<FaceMesh, ShellFaceError>>,
}

/// 面の三角形
#[derive(Debug)]
pub(crate) struct FaceMesh {
    /// 面の頂点の、B-rep の全体の点の番号
    pub(crate) vertices: Vec<usize>,
    /// 頂点の法線（面の外向き。`vertices` と同じ順）
    pub(crate) normals: Vec<Vector3D<f64>>,
    /// 三角形（`vertices` の位置。面の外向きの法線の側から見て反時計回り）
    pub(crate) triangles: Vec<[usize; 3]>,
}

/// B-rep `brep` のすべての面を、許容弦誤差 `chord_tolerance` で三角形に分ける
pub(crate) fn triangulate_brep(brep: &Brep<f64>, chord_tolerance: f64) -> ShellMesh {
    let settings = FaceTriangulationSettings {
        chord_tolerance,
        max_angle: Angle::from_degrees(MAX_CHORD_ANGLE_DEGREES),
        max_points: MAX_FACE_POINTS,
        max_passes: MAX_REFINEMENT_PASSES,
    };
    let faces: Vec<FaceIndex> = brep.face_indices().collect();
    let positions: HashMap<FaceIndex, usize> = faces
        .iter()
        .enumerate()
        .map(|(position, &face)| (face, position))
        .collect();
    let mut points = Points::default();
    let mut edges = EdgePoints::default();
    for &face in &faces {
        for &loop_index in brep.face(face).loops() {
            for edge_use in brep.face_loop(loop_index).uses() {
                if !edges.edges.contains_key(&edge_use.edge) {
                    edges.discretize(brep, edge_use.edge, &settings, 1.0, &mut points);
                }
            }
        }
    }
    let mut prepared: Vec<Result<Prepared<'_>, ShellFaceError>> = faces
        .iter()
        .map(|&face| prepare_face(brep, face, &edges, &settings))
        .collect();
    // 交わる辺を含む稜線を、許容弦誤差を半分にして分け直し、その稜線を使う面だけを作り直す
    let mut divisors: HashMap<EdgeIndex, f64> = HashMap::new();
    for _ in 0..MAX_RESPLITS {
        let crossing: HashSet<EdgeIndex> = prepared
            .iter()
            .filter_map(|face| face.as_ref().ok())
            .flat_map(|face| &face.crossing)
            .filter_map(|&[a, b]| edges.segments.get(&(a.min(b), a.max(b))).copied())
            .collect();
        if crossing.is_empty() {
            break;
        }
        let mut affected = HashSet::new();
        for edge in crossing {
            let divisor = divisors.entry(edge).or_insert(1.0);
            *divisor *= 2.0;
            edges.discretize(brep, edge, &settings, *divisor, &mut points);
            affected.extend(brep.edge(edge).uses().iter().map(|location| location.face));
        }
        for face in affected {
            prepared[positions[&face]] = prepare_face(brep, face, &edges, &settings);
        }
    }
    let faces = prepared
        .into_iter()
        .map(|face| face.and_then(|face| face.triangulate(brep, &settings, &mut points)))
        .collect();
    ShellMesh {
        points: points.points,
        faces,
    }
}

/// B-rep の全体の点（頂点の点と、稜線の点と、面の内側に足した点。稜線を分け直す前の点も残る）
#[derive(Default)]
struct Points {
    points: Vec<Point3D<f64>>,
    /// 頂点の点の番号
    vertices: HashMap<VertexIndex, usize>,
}

impl Points {
    fn push(&mut self, point: Point3D<f64>) -> usize {
        self.points.push(point);
        self.points.len() - 1
    }

    fn vertex(&mut self, brep: &Brep<f64>, vertex: VertexIndex) -> usize {
        if let Some(&index) = self.vertices.get(&vertex) {
            return index;
        }
        let index = self.push(brep.vertex(vertex).point());
        self.vertices.insert(vertex, index);
        index
    }
}

/// 稜線の点（母曲線のパラメータと点の番号。区間の小さい端から）と、隣り合う点の組から稜線への対応
#[derive(Default)]
struct EdgePoints {
    edges: HashMap<EdgeIndex, Result<Vec<(f64, usize)>, CurveDiscretizationError>>,
    segments: HashMap<(usize, usize), EdgeIndex>,
}

impl EdgePoints {
    /// 稜線 `edge` を、許容弦誤差を `divisor` で割って点に分ける（端は頂点の点。前の点は置き換える）
    fn discretize(
        &mut self,
        brep: &Brep<f64>,
        index: EdgeIndex,
        settings: &FaceTriangulationSettings,
        divisor: f64,
        points: &mut Points,
    ) {
        if let Some(Ok(previous)) = self.edges.get(&index) {
            for pair in previous.windows(2) {
                let (a, b) = (pair[0].1, pair[1].1);
                self.segments.remove(&(a.min(b), a.max(b)));
            }
        }
        let edge = brep.edge(index);
        let curve_settings = CurveDiscretizationSettings {
            chord_tolerance: settings.chord_tolerance / divisor,
            max_angle: settings.max_angle,
            max_points: MAX_EDGE_POINTS,
        };
        let range = edge.parameter_range();
        let discretized = match edge.curve() {
            CurveRef::Line(curve) => curve.discretize(range, &curve_settings),
            CurveRef::Circle(curve) => curve.discretize(range, &curve_settings),
            CurveRef::Ellipse(curve) => curve.discretize(range, &curve_settings),
            CurveRef::Nurbs(curve) => curve.discretize(range, &curve_settings),
        };
        let numbered = discretized.map(|curve_points: Vec<CurvePoint>| {
            let last = curve_points.len() - 1;
            curve_points
                .iter()
                .enumerate()
                .map(|(position, point)| {
                    let index = match position {
                        0 => points.vertex(brep, edge.start()),
                        _ if position == last => points.vertex(brep, edge.end()),
                        _ => points.push(point.point),
                    };
                    (point.parameter, index)
                })
                .collect::<Vec<_>>()
        });
        if let Ok(numbered) = &numbered {
            for pair in numbered.windows(2) {
                let (a, b) = (pair[0].1, pair[1].1);
                self.segments.insert((a.min(b), a.max(b)), index);
            }
        }
        self.edges.insert(index, numbered);
    }
}

/// 切れ目を入れた範囲まで作った面
struct Prepared<'a> {
    face: FaceIndex,
    surface: FaceSurface<'a>,
    loops: Vec<Vec<LoopPoint>>,
    region: FaceRegion,
    /// 交わる (u, v) の多角形の辺（境界の点の番号の組）
    crossing: Vec<[usize; 2]>,
}

/// 面 `face` のループを (u, v) の多角形にし、切れ目を入れた範囲を作って、交わる辺を確かめる
fn prepare_face<'a>(
    brep: &'a Brep<f64>,
    face: FaceIndex,
    edges: &EdgePoints,
    settings: &FaceTriangulationSettings,
) -> Result<Prepared<'a>, ShellFaceError> {
    let surface = brep.face(face).surface();
    let mut loops = Vec::with_capacity(brep.face(face).loops().len());
    for &loop_index in brep.face(face).loops() {
        let face_loop = brep.face_loop(loop_index);
        let uses = face_loop.uses();
        let mut loop_points = Vec::new();
        for (position, (edge_use, curve)) in
            uses.iter().zip(face_loop.parameter_curves()).enumerate()
        {
            let edge_points = edges.edges[&edge_use.edge]
                .as_ref()
                .map_err(|error| ShellFaceError::Edge(*error))?;
            let ordered: Vec<(f64, usize)> = if edge_use.forward {
                edge_points.clone()
            } else {
                edge_points.iter().rev().copied().collect()
            };
            // 使用の始めと終わりの頂点が極なら、v を極の値にそろえる。B-rep がパラメータ曲線をつなぐときと同じく、
            // 極を通るつなぎ目の前の使用の稜線の許容誤差で見分ける（頂点の許容誤差は、ほかの稜線を足すと広がる
            // ため使わない）
            let previous = uses[(position + uses.len() - 1) % uses.len()].edge;
            let start_pole = surface.singular_v(
                brep.vertex(brep.use_start(*edge_use)).point(),
                brep.edge(previous).tolerance(),
            );
            let end_pole = surface.singular_v(
                brep.vertex(brep.use_end(*edge_use)).point(),
                brep.edge(edge_use.edge).tolerance(),
            );
            // 使用の終わりの点は、次の使用の始めの点と同じなので入れない。ただし極では、次の使用の u が違う
            // （極の線に沿って進む）ので、この使用の u の極の点も入れる
            let count = if end_pole.is_some() {
                ordered.len()
            } else {
                ordered.len() - 1
            };
            for (offset, &(parameter, index)) in ordered[..count].iter().enumerate() {
                let uv = curve
                    .point_at(parameter)
                    .ok_or(ShellFaceError::ParameterCurve)?;
                let pole = match offset {
                    0 => start_pole,
                    _ if offset == ordered.len() - 1 => end_pole,
                    _ => None,
                };
                let uv = pole.map_or(uv, |pole| Point2D::new(uv.x(), pole));
                loop_points.push(LoopPoint { vertex: index, uv });
            }
        }
        // ループは、面の外向きの法線の側から見て面を左に置く。面が曲面の法線と逆向きなら、(u, v) では面が右に
        // あるので、範囲を作る部品に合わせて逆にする（三角形の回りは、後で面の向きに戻す）
        if !brep.face(face).same_sense() {
            loop_points.reverse();
        }
        loops.push(loop_points);
    }
    let surface = FaceSurface::new(surface, &loops);
    let region =
        FaceRegion::new(&loops, &surface, settings).map_err(ShellFaceError::Triangulation)?;
    let crossing = region.crossing_edges(&surface);
    Ok(Prepared {
        face,
        surface,
        loops,
        region,
        crossing,
    })
}

impl Prepared<'_> {
    /// 範囲を三角形に分け、面の向きに合わせて三角形の回りと頂点の法線をそろえる（足した点は `points` に加える）
    fn triangulate(
        self,
        brep: &Brep<f64>,
        settings: &FaceTriangulationSettings,
        points: &mut Points,
    ) -> Result<FaceMesh, ShellFaceError> {
        if !self.crossing.is_empty() {
            return Err(ShellFaceError::Crossing);
        }
        let triangulation = self
            .region
            .triangulate(&self.surface, settings)
            .map_err(ShellFaceError::Triangulation)?;
        let same_sense = brep.face(self.face).same_sense();
        let boundary_uv: HashMap<usize, Point2D<f64>> = self
            .loops
            .iter()
            .flatten()
            .map(|point| (point.vertex, point.uv))
            .collect();
        let oriented = |normal: Vector3D<f64>| if same_sense { normal } else { -normal };
        let mut local: HashMap<FaceVertex, usize> = HashMap::new();
        let mut uvs = Vec::new();
        let mut mesh = FaceMesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::with_capacity(triangulation.triangles.len()),
        };
        // 曲面の法線が決まらない頂点（円錐の頂点）
        let mut undetermined = Vec::new();
        for triangle in &triangulation.triangles {
            let corners = triangle.map(|vertex| {
                *local.entry(vertex).or_insert_with(|| {
                    let (index, uv) = match vertex {
                        FaceVertex::Boundary(index) => (index, boundary_uv[&index]),
                        FaceVertex::Added(added) => {
                            let added = triangulation.added[added];
                            (points.push(added.point), added.uv)
                        }
                    };
                    let normal = self.surface.normal_at(uv.x(), uv.y());
                    if normal.is_none() {
                        undetermined.push(mesh.vertices.len());
                    }
                    mesh.vertices.push(index);
                    mesh.normals
                        .push(normal.map_or(Vector3D::new(0.0, 0.0, 0.0), oriented));
                    uvs.push(uv);
                    mesh.vertices.len() - 1
                })
            });
            mesh.triangles.push(if same_sense {
                corners
            } else {
                [corners[0], corners[2], corners[1]]
            });
        }
        // 法線が決まらない頂点は、その頂点を含む三角形の法線の平均にする（和が 0 なら、その頂点を含む三角形の
        // (u, v) の重心の曲面の法線）
        for vertex in undetermined {
            let around: Vec<[usize; 3]> = mesh
                .triangles
                .iter()
                .filter(|triangle| triangle.contains(&vertex))
                .copied()
                .collect();
            let sum = around
                .iter()
                .fold(Vector3D::new(0.0, 0.0, 0.0), |sum, &[a, b, c]| {
                    let [a, b, c] = [a, b, c].map(|corner| points.points[mesh.vertices[corner]]);
                    sum + Vector3D::from_points(a, b).cross(Vector3D::from_points(a, c))
                });
            let length = sum.length();
            mesh.normals[vertex] = if length.is_finite() && length > 0.0 {
                sum * (1.0 / length)
            } else {
                around
                    .iter()
                    .find_map(|triangle| {
                        let [u, v] = triangle.iter().fold([0.0, 0.0], |[u, v], &corner| {
                            [u + uvs[corner].x() / 3.0, v + uvs[corner].y() / 3.0]
                        });
                        self.surface.normal_at(u, v)
                    })
                    .map_or(Vector3D::new(0.0, 0.0, 1.0), oriented)
            };
        }
        Ok(mesh)
    }
}

/// 面の曲面を、三角形に分ける部品が受ける形にしたもの（刻みと座標の式は、設計書 6.3）
struct FaceSurface<'a> {
    surface: &'a SurfaceRef<f64>,
    /// 面の範囲の v の小さい側と大きい側の極
    poles: (Option<f64>, Option<f64>),
    /// 円錐の面の中の半径の絶対値の最大（頂点の先の、半径が負の側も含む）
    cone_radius: f64,
    /// NURBS 曲面の面の中で測った、u・v の向きの長さと曲がり
    nurbs: Option<NurbsMeasure>,
}

/// NURBS 曲面の面の (u, v) の範囲に格子で点を取って測った、u・v の向きの長さと曲がり（設計書 6.3）
struct NurbsMeasure {
    /// 面の (u, v) の範囲の幅
    spans: (f64, f64),
    /// u・v の向きの、パラメータあたりの曲面の上の長さの平均（三角形分割の座標の倍率）
    speeds: (f64, f64),
    /// u・v の向きの、測った点ごとの曲がり（曲がっていない点は除く）
    bends: (Vec<Bend>, Vec<Bend>),
}

/// NURBS 曲面の面の中で測った 1 点の、u か v の向きの曲がり
struct Bend {
    /// 曲がりの半径
    radius: f64,
    /// パラメータあたりの曲面の上の長さ
    speed: f64,
}

/// NURBS 曲面の面の長さと曲がりを測る格子の、u・v の向きの区間の数
const NURBS_MEASURE_SEGMENTS: usize = 16;

impl NurbsMeasure {
    /// 面の (u, v) の範囲 `low`〜`high` に格子で点を取って測る（測れなければ `None`）
    fn new(surface: &SurfaceRef<f64>, low: (f64, f64), high: (f64, f64)) -> Option<Self> {
        let spans = (high.0 - low.0, high.1 - low.1);
        if !(spans.0 > 0.0 && spans.1 > 0.0 && spans.0.is_finite() && spans.1.is_finite()) {
            return None;
        }
        let count = NURBS_MEASURE_SEGMENTS;
        let steps = (spans.0 / count as f64, spans.1 / count as f64);
        let mut grid = Vec::with_capacity((count + 1) * (count + 1));
        for i in 0..=count {
            for j in 0..=count {
                let uv = (low.0 + steps.0 * i as f64, low.1 + steps.1 * j as f64);
                grid.push(surface.point_at_uv(uv.0, uv.1));
            }
        }
        let at = |i: usize, j: usize| grid[i * (count + 1) + j];
        // u の向き（`along_u`）と v の向きに 3 点ずつ取り、差で 1 階と 2 階の微分を見積もる
        let measure = |along_u: bool| {
            let step = if along_u { steps.0 } else { steps.1 };
            let point = |line: usize, position: usize| {
                if along_u {
                    at(position, line)
                } else {
                    at(line, position)
                }
            };
            let (mut length, mut samples, mut bends) = (0.0, 0usize, Vec::new());
            for line in 0..=count {
                for position in 1..count {
                    let (Some(before), Some(middle), Some(after)) = (
                        point(line, position - 1),
                        point(line, position),
                        point(line, position + 1),
                    ) else {
                        continue;
                    };
                    let first = Vector3D::from_points(before, after) * (0.5 / step);
                    let second = (Vector3D::from_points(middle, after)
                        - Vector3D::from_points(before, middle))
                        * (1.0 / (step * step));
                    let speed = first.length();
                    length += speed;
                    samples += 1;
                    // 曲がりの半径は |S'|³ / |S' × S''|（曲がっていない点・速さが 0 の点は値が有限にならない）
                    let radius = speed.powi(3) / first.cross(second).length();
                    if radius.is_finite() && speed > 0.0 {
                        bends.push(Bend { radius, speed });
                    }
                }
            }
            (samples > 0).then(|| (length / samples as f64, bends))
        };
        let (speed_u, bends_u) = measure(true)?;
        let (speed_v, bends_v) = measure(false)?;
        (speed_u > 0.0 && speed_v > 0.0).then_some(Self {
            spans,
            speeds: (speed_u, speed_v),
            bends: (bends_u, bends_v),
        })
    }

    /// 許容弦誤差と角度の上限から決まる u・v の刻み（測った点ごとの角の刻みを、パラメータの幅にした最小。曲がらな
    /// ければ範囲の幅）
    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (f64, f64) {
        let step = |bends: &[Bend], span: f64| {
            bends
                .iter()
                .map(|bend| {
                    chord_angle_step(bend.radius, chord_tolerance, max_angle) * bend.radius
                        / bend.speed
                })
                // 半径が許容弦誤差に比べて大きすぎて角の刻みが 0 になる点は、曲がっていない点とみなす
                .filter(|step| step.is_finite() && *step > 0.0)
                .fold(span, f64::min)
        };
        (
            step(&self.bends.0, self.spans.0),
            step(&self.bends.1, self.spans.1),
        )
    }
}

impl<'a> FaceSurface<'a> {
    fn new(surface: &'a SurfaceRef<f64>, loops: &[Vec<LoopPoint>]) -> Self {
        let (low, high) = loops
            .iter()
            .flatten()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), point| {
                (low.min(point.uv.y()), high.max(point.uv.y()))
            });
        let mut poles = (None, None);
        for pole in surface.poles() {
            // ループのない面（球の全体）は、両側の極を持つ
            if loops.is_empty() || pole <= low {
                poles.0 = poles.0.or(Some(pole));
            }
            if loops.is_empty() || pole >= high {
                poles.1 = Some(pole);
            }
        }
        let cone_radius = match surface {
            SurfaceRef::Cone(cone) => {
                let slope = cone.semi_angle().to_radians().tan();
                loops
                    .iter()
                    .flatten()
                    .map(|point| (cone.radius() + point.uv.y() * slope).abs())
                    .fold(0.0, f64::max)
            }
            _ => 0.0,
        };
        let nurbs = match surface {
            SurfaceRef::Nurbs(nurbs) if loops.is_empty() => {
                let ((u_min, u_max), (v_min, v_max)) = nurbs.parameter_domain();
                NurbsMeasure::new(surface, (u_min, v_min), (u_max, v_max))
            }
            SurfaceRef::Nurbs(_) => {
                let (u_low, u_high) = loops
                    .iter()
                    .flatten()
                    .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), point| {
                        (low.min(point.uv.x()), high.max(point.uv.x()))
                    });
                NurbsMeasure::new(surface, (u_low, low), (u_high, high))
            }
            _ => None,
        };
        Self {
            surface,
            poles,
            cone_radius,
            nurbs,
        }
    }
}

impl TriangulationSurface for FaceSurface<'_> {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        self.surface.point_at_uv(u, v)
    }

    fn normal_at(&self, u: f64, v: f64) -> Option<Vector3D<f64>> {
        self.surface.normal_at_uv(u, v)
    }

    fn periods(&self) -> (Option<f64>, Option<f64>) {
        self.surface.periods()
    }

    fn poles(&self) -> (Option<f64>, Option<f64>) {
        self.poles
    }

    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (Option<f64>, Option<f64>) {
        let step = |radius: f64| Some(chord_angle_step(radius, chord_tolerance, max_angle));
        match self.surface {
            SurfaceRef::Plane(_) => (None, None),
            SurfaceRef::Nurbs(_) => self.nurbs.as_ref().map_or((None, None), |nurbs| {
                let (u, v) = nurbs.steps(chord_tolerance, max_angle);
                (Some(u), Some(v))
            }),
            SurfaceRef::Cylinder(cylinder) => (step(cylinder.radius()), None),
            SurfaceRef::Cone(_) => (step(self.cone_radius), None),
            SurfaceRef::Sphere(sphere) => (step(sphere.radius()), step(sphere.radius())),
            SurfaceRef::Torus(torus) => (
                step(torus.major_radius() + torus.minor_radius()),
                step(torus.minor_radius()),
            ),
        }
    }

    fn scale(&self) -> (f64, f64) {
        match self.surface {
            SurfaceRef::Plane(_) => (1.0, 1.0),
            SurfaceRef::Nurbs(_) => self.nurbs.as_ref().map_or((1.0, 1.0), |nurbs| nurbs.speeds),
            SurfaceRef::Cylinder(cylinder) => (cylinder.radius(), 1.0),
            SurfaceRef::Cone(cone) => {
                (self.cone_radius, 1.0 / cone.semi_angle().to_radians().cos())
            }
            SurfaceRef::Sphere(sphere) => (sphere.radius(), sphere.radius()),
            SurfaceRef::Torus(torus) => (
                torus.major_radius() + torus.minor_radius(),
                torus.minor_radius(),
            ),
        }
    }
}

#[cfg(test)]
mod tests;
