//! 制約つきのドロネー分割と、許容弦誤差による細分化（設計書 6.1〜6.3）
//!
//! 範囲の多角形の辺を制約の辺にして、三角形分割の座標（`TriangulationSurface::scale` を掛けた (u, v)）でドロネー
//! 分割する。外から制約の辺を越えるたびに内と外が入れ替わるとして、範囲の内側の三角形を選ぶ。
//!
//! u・v の両方に刻みのある曲面では、初めに範囲の内側に刻みの格子で点を置く。その後、内側の三角形ごとに、(u, v) の
//! 重心と 3 つの辺の中点を曲面の上で評価し、三角形（辺）の同じ位置の点との離れ（曲面の法線の向きの成分）を測る。
//! 重心の離れか、辺の中点の離れから見積もった三角形の中の離れの最大（2 次式で補う）が許容弦誤差を超えれば、重心と
//! 制約でない辺の中点のうち離れの大きいほうに点を足す。辺の (u, v) の幅では分けない（極や円錐の頂点の近くでは、
//! (u, v) の幅が広くても曲面の上では短く、境界の辺に接する三角形を分け続けてしまうため）。
//!
//! 制約の辺（境界・切れ目・極の線）は分けない。制約の辺の中点の離れが許容弦誤差を超える三角形では、その離れを上限
//! とする（分けられない辺に接する三角形のほかの辺と内側は、どこに点を足してもその辺の離れの程度より小さくできない
//! ので、点を足し続けないため）。

use std::collections::{HashMap, HashSet};

use spade::handles::{FixedFaceHandle, FixedVertexHandle, InnerTag};
use spade::{ConstrainedDelaunayTriangulation, Point2, PositionInTriangulation, Triangulation};

use super::{
    AddedPoint, FaceRegion, FaceTriangulation, FaceTriangulationError, FaceTriangulationSettings,
    FaceVertex, TriangulationSurface,
};
use crate::{Point2D, Point3D, Vector3D};

type Cdt = ConstrainedDelaunayTriangulation<Point2<f64>>;

impl FaceRegion {
    /// 範囲を、多角形の辺を必ず辺に持つ三角形に分け、許容弦誤差と刻みを守るまで内側に点を足す
    ///
    /// 多角形の辺どうしが交わらないことは、先に [`FaceRegion::crossing_edges`] で確かめておく（同じ座標と、丸め誤差に
    /// 強い判定を使うので、交わらないと確かめた範囲は、ここでも交わるとはしない）。
    ///
    /// # Errors
    ///
    /// 設定が正しくない、多角形の辺どうしが交わる、曲面を評価できない、頂点の数・繰り返しの回数が上限を超えるとき。
    pub fn triangulate(
        &self,
        surface: &impl TriangulationSurface,
        settings: &FaceTriangulationSettings,
    ) -> Result<FaceTriangulation, FaceTriangulationError> {
        settings.check()?;
        let scale = surface.scale();
        if !(scale.0.is_finite() && scale.0 > 0.0 && scale.1.is_finite() && scale.1 > 0.0) {
            return Err(FaceTriangulationError::Evaluation);
        }
        let steps = super::checked_steps(surface, settings)?;
        let mut mesh = Mesh {
            cdt: Cdt::new(),
            vertices: Vec::new(),
            added: Vec::with_capacity(self.added.len()),
            scale,
            max_points: settings.max_points,
            edge_deviations: HashMap::new(),
            accepted: HashSet::new(),
        };
        for &uv in &self.added {
            mesh.added.push(AddedPoint {
                uv,
                point: evaluate(surface, uv)?,
            });
        }
        let mut polygons = Vec::with_capacity(self.polygons.len());
        for polygon in &self.polygons {
            let mut handles = Vec::with_capacity(polygon.len());
            for point in polygon {
                let handle = mesh.insert(surface, point.uv, Some(point.vertex))?;
                if mesh.vertices[handle.index()].vertex != point.vertex {
                    // 違う点が (u, v) の同じ位置にある
                    return Err(FaceTriangulationError::Crossing);
                }
                handles.push(handle);
            }
            polygons.push(handles);
        }
        for handles in &polygons {
            for (index, &from) in handles.iter().enumerate() {
                let to = handles[(index + 1) % handles.len()];
                if from != to && mesh.cdt.try_add_constraint(from, to).is_empty() {
                    return Err(FaceTriangulationError::Crossing);
                }
            }
        }
        for uv in mesh.grid_points(steps)? {
            mesh.insert(surface, uv, None)?;
        }
        let mut pass = 0;
        loop {
            let points = mesh.points_to_add(surface, settings.chord_tolerance)?;
            if points.is_empty() {
                break;
            }
            if pass == settings.max_passes {
                return Err(FaceTriangulationError::NotConverged);
            }
            pass += 1;
            for uv in points {
                mesh.insert(surface, uv, None)?;
            }
        }
        let triangles = inside_faces(&mesh.cdt)
            .into_iter()
            .map(|face| {
                mesh.cdt
                    .face(face)
                    .vertices()
                    .map(|vertex| mesh.vertices[vertex.fix().index()].vertex)
            })
            .filter(|[a, b, c]| a != b && b != c && c != a)
            .collect();
        Ok(FaceTriangulation {
            added: mesh.added,
            triangles,
        })
    }
}

/// 三角形分割の頂点
struct Vertex {
    vertex: FaceVertex,
    uv: Point2D<f64>,
    point: Point3D<f64>,
}

struct Mesh {
    cdt: Cdt,
    /// spade の頂点の番号の順
    vertices: Vec<Vertex>,
    added: Vec<AddedPoint>,
    scale: (f64, f64),
    max_points: usize,
    /// 辺の中点の離れ（両端の spade の頂点の番号の、小さいほうから）
    edge_deviations: HashMap<[usize; 2], Vector3D<f64>>,
    /// 許容弦誤差と刻みを守っていると確かめた三角形（spade の頂点の番号を小さい順に並べたもの）
    accepted: HashSet<[usize; 3]>,
}

impl Mesh {
    /// (u, v) の点を足す（`vertex` が `None` なら、足した点として番号を付ける）。同じ位置に点があれば、その点
    fn insert(
        &mut self,
        surface: &impl TriangulationSurface,
        uv: Point2D<f64>,
        vertex: Option<FaceVertex>,
    ) -> Result<FixedVertexHandle, FaceTriangulationError> {
        let handle = self
            .cdt
            .insert(Point2::new(uv.x() * self.scale.0, uv.y() * self.scale.1))
            .map_err(|_| FaceTriangulationError::InvalidLoops)?;
        if handle.index() < self.vertices.len() {
            return Ok(handle);
        }
        if self.vertices.len() >= self.max_points {
            return Err(FaceTriangulationError::TooManyPoints);
        }
        let point = evaluate(surface, uv)?;
        let vertex = vertex.unwrap_or_else(|| {
            self.added.push(AddedPoint { uv, point });
            FaceVertex::Added(self.added.len() - 1)
        });
        self.vertices.push(Vertex { vertex, uv, point });
        Ok(handle)
    }

    /// u・v の両方に刻みのある曲面で、範囲の内側に刻みの格子で置く点の (u, v)。境界の点と制約の辺から、刻みの半分
    /// より近い点は置かない（細い三角形を避けるため）
    fn grid_points(
        &self,
        steps: (Option<f64>, Option<f64>),
    ) -> Result<Vec<Point2D<f64>>, FaceTriangulationError> {
        let (Some(step_u), Some(step_v)) = steps else {
            return Ok(Vec::new());
        };
        let (low, high) = self.vertices.iter().fold(
            (
                (f64::INFINITY, f64::INFINITY),
                (f64::NEG_INFINITY, f64::NEG_INFINITY),
            ),
            |(low, high), vertex| {
                (
                    (low.0.min(vertex.uv.x()), low.1.min(vertex.uv.y())),
                    (high.0.max(vertex.uv.x()), high.1.max(vertex.uv.y())),
                )
            },
        );
        let count_u = ((high.0 - low.0) / step_u).ceil();
        let count_v = ((high.1 - low.1) / step_v).ceil();
        if !(count_u.is_finite() && count_v.is_finite())
            || count_u * count_v > self.max_points as f64
        {
            return Err(FaceTriangulationError::TooManyPoints);
        }
        let (count_u, count_v) = (count_u as usize, count_v as usize);
        let inside: HashSet<FixedFaceHandle<InnerTag>> =
            inside_faces(&self.cdt).into_iter().collect();
        let clearance = 0.5 * (step_u * self.scale.0).min(step_v * self.scale.1);
        let mut points = Vec::new();
        for i in 1..count_u {
            for j in 1..count_v {
                let uv = Point2D::new(
                    low.0 + (high.0 - low.0) * i as f64 / count_u as f64,
                    low.1 + (high.1 - low.1) * j as f64 / count_v as f64,
                );
                let position = Point2::new(uv.x() * self.scale.0, uv.y() * self.scale.1);
                let PositionInTriangulation::OnFace(face) = self.cdt.locate(position) else {
                    continue;
                };
                if !inside.contains(&face) {
                    continue;
                }
                // 点を含む三角形と、その隣の三角形の頂点・制約の辺との距離を見る（境界の辺の上の細い三角形の隣に
                // 入った点も、境界の辺から離す）
                let face = self.cdt.face(face);
                let mut faces = vec![face];
                faces.extend(
                    face.adjacent_edges()
                        .iter()
                        .filter_map(|edge| edge.rev().face().as_inner()),
                );
                let near = faces.iter().any(|face| {
                    face.vertices().iter().any(|vertex| {
                        vertex.position().distance_2(position) < clearance * clearance
                    }) || face.adjacent_edges().iter().any(|edge| {
                        edge.as_undirected().is_constraint_edge()
                            && edge.as_undirected().distance_2(position) < clearance * clearance
                    })
                });
                if !near {
                    points.push(uv);
                }
            }
        }
        Ok(points)
    }

    /// 許容弦誤差を超える三角形に足す点の (u, v)
    fn points_to_add(
        &mut self,
        surface: &impl TriangulationSurface,
        chord_tolerance: f64,
    ) -> Result<Vec<Point2D<f64>>, FaceTriangulationError> {
        let mut points = Vec::new();
        let mut seen = HashSet::new();
        for face in inside_faces(&self.cdt) {
            let face = self.cdt.face(face);
            let handles = face.vertices().map(|vertex| vertex.fix().index());
            let mut key = handles;
            key.sort_unstable();
            if self.accepted.contains(&key) {
                continue;
            }
            let constraints = face.adjacent_edges().map(|edge| {
                let [from, to] = edge.vertices().map(|vertex| vertex.fix().index());
                (
                    [from.min(to), from.max(to)],
                    edge.as_undirected().is_constraint_edge(),
                )
            });
            let corners = handles.map(|index| &self.vertices[index]);
            let centroid = Point2D::new(
                corners.iter().map(|corner| corner.uv.x()).sum::<f64>() / 3.0,
                corners.iter().map(|corner| corner.uv.y()).sum::<f64>() / 3.0,
            );
            let linear = Point3D::new(
                corners.iter().map(|corner| corner.point.x()).sum::<f64>() / 3.0,
                corners.iter().map(|corner| corner.point.y()).sum::<f64>() / 3.0,
                corners.iter().map(|corner| corner.point.z()).sum::<f64>() / 3.0,
            );
            let centroid_deviation = deviation(surface, centroid, linear)?.length();
            // 制約の辺は分けられないので、制約の辺の中点の離れより厳しくは求めない。その辺に接する三角形の、ほかの
            // 辺と内側の離れは、どこに点を足してもその辺の離れの程度より小さくできないため
            let mut allowed = chord_tolerance;
            // 辺の中点の離れ（角の番号の組 (0, 1)・(1, 2)・(2, 0) の順）と、制約でない辺のうち離れの最も大きい辺の中点
            let mut deviations = [Vector3D::new(0.0, 0.0, 0.0); 3];
            let mut worst_edge: Option<(f64, Point2D<f64>)> = None;
            for corner in 0..3 {
                let (from, to) = (handles[corner], handles[(corner + 1) % 3]);
                let (start, end) = (&self.vertices[from], &self.vertices[to]);
                let middle = start.uv.midpoint(end.uv);
                let edge = [from.min(to), from.max(to)];
                let deviation = match self.edge_deviations.get(&edge) {
                    Some(&deviation) => deviation,
                    None => {
                        let deviation =
                            deviation(surface, middle, start.point.midpoint(end.point))?;
                        self.edge_deviations.insert(edge, deviation);
                        deviation
                    }
                };
                let constraint = constraints
                    .iter()
                    .any(|&(key, constraint)| key == edge && constraint);
                let length = deviation.length();
                deviations[corner] = deviation;
                if constraint {
                    allowed = allowed.max(length);
                    continue;
                }
                if worst_edge.is_none_or(|(worst, _)| length > worst) {
                    worst_edge = Some((length, middle));
                }
            }
            if centroid_deviation.max(quadratic_maximum(&deviations)) <= allowed {
                self.accepted.insert(key);
                continue;
            }
            let uv = match worst_edge {
                Some((length, middle)) if length > centroid_deviation => middle,
                _ => centroid,
            };
            if seen.insert((uv.x().to_bits(), uv.y().to_bits())) {
                points.push(uv);
            }
        }
        Ok(points)
    }
}

/// 角で 0、辺の中点で離れ `deviations`（辺 (0, 1)・(1, 2)・(2, 0)）になる 2 次式の、三角形の中の最大（6 等分の
/// 格子の点で測る）。曲面と三角形の離れを、辺の中点のほかでも見積もる
fn quadratic_maximum(deviations: &[Vector3D<f64>; 3]) -> f64 {
    let mut maximum: f64 = 0.0;
    for i in 0..=6 {
        for j in 0..=6 - i {
            let weights = [i as f64 / 6.0, j as f64 / 6.0, (6 - i - j) as f64 / 6.0];
            let value = (0..3).fold(Vector3D::new(0.0, 0.0, 0.0), |sum, edge| {
                sum + deviations[edge] * (4.0 * weights[edge] * weights[(edge + 1) % 3])
            });
            maximum = maximum.max(value.length());
        }
    }
    maximum
}

/// 三角形の点 `linear` と、同じ (u, v) の曲面の点の離れ（曲面の法線の向きの成分。法線が定まらなければ、点どうしの
/// 差）
///
/// 曲面の上の向きのずれは、(u, v) の付け方のゆがみ（極の近くなど）で、三角形が曲面から離れていなくても大きくなるので
/// 測らない。
fn deviation(
    surface: &impl TriangulationSurface,
    uv: Point2D<f64>,
    linear: Point3D<f64>,
) -> Result<Vector3D<f64>, FaceTriangulationError> {
    let difference = Vector3D::from_points(linear, evaluate(surface, uv)?);
    let normal = surface
        .normal_at(uv.x(), uv.y())
        .map(|normal| (normal, normal.length()))
        .filter(|(_, length)| length.is_finite() && *length > 0.0);
    Ok(match normal {
        Some((normal, length)) => {
            let unit = normal * (1.0 / length);
            unit * difference.dot(unit)
        }
        None => difference,
    })
}

/// 曲面の点（有限の値でなければ評価できないとする）
fn evaluate(
    surface: &impl TriangulationSurface,
    uv: Point2D<f64>,
) -> Result<Point3D<f64>, FaceTriangulationError> {
    surface
        .point_at(uv.x(), uv.y())
        .filter(|point| point.x().is_finite() && point.y().is_finite() && point.z().is_finite())
        .ok_or(FaceTriangulationError::Evaluation)
}

/// 範囲の内側の三角形（外から制約の辺を越えるたびに、内と外が入れ替わる）
fn inside_faces(cdt: &Cdt) -> Vec<FixedFaceHandle<InnerTag>> {
    let mut inside: Vec<Option<bool>> = vec![None; cdt.num_all_faces()];
    let mut stack = Vec::new();
    for edge in cdt.convex_hull() {
        let face = if edge.face().is_outer() {
            edge.rev().face()
        } else {
            edge.face()
        };
        if let Some(face) = face.as_inner() {
            stack.push((face.fix(), edge.as_undirected().is_constraint_edge()));
        }
    }
    while let Some((face, value)) = stack.pop() {
        if inside[face.index()].is_some() {
            continue;
        }
        inside[face.index()] = Some(value);
        for edge in cdt.face(face).adjacent_edges() {
            if let Some(next) = edge.rev().face().as_inner()
                && inside[next.fix().index()].is_none()
            {
                stack.push((
                    next.fix(),
                    value != edge.as_undirected().is_constraint_edge(),
                ));
            }
        }
    }
    cdt.fixed_inner_faces()
        .filter(|face| inside[face.index()] == Some(true))
        .collect()
}
