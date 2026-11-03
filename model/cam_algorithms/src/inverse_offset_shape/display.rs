//! 逆オフセット形状の表示用の三角形（表示・保存だけに使い、計算には使わない）
//!
//! 面の要素は三角形そのまま、球面・円柱面・円板は角度のきざみで分ける。範囲は正確に切り取る: 球面は、正二十面体を
//! 細かく分けた三角形を範囲の平面（球の中心を通る）で切り取ってから球面へ戻し、円柱面は範囲を円弧として求める。

use geo_contracts::Triangle3DBoundaryAccess;

use super::{
    EdgeElement, InverseOffsetShape, VertexElement, cross,
    default_orthogonality_dot_error_tolerance, dot, sub, tolerance,
};
use crate::inverse_offset::CutterShape;
use crate::solver::CamSolverError;

/// 表示用の角度のきざみの下限（1°。細かくすると三角形の数がきざみの 2 乗に反比例して増えるため）
const MIN_DISPLAY_ANGLE_STEP: f64 = std::f64::consts::PI / 180.0;

/// 表示用の三角形の種類
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InverseOffsetPartKind {
    /// 高さを求める索引に入る面（上を向く面）
    IndexedFace,
    /// 高さを求める索引に入らない面（鉛直な面・アンダーカットの面を、外向きにずらしたもの）
    UnindexedFace,
    /// 開いた塊の下を向く面を、上側にずらしたもの
    UpperSide,
    /// 外を決められなかった塊の面（面ごとに上向きにそろえたもの）
    UnorientedFace,
    /// 頂点（ボール: 切り取った球面、フラット: 工具の底が乗る頂点の円板）
    Vertex,
    /// 辺（ボール: 範囲を限った円柱面、フラット: 円板の縁のうち範囲の円弧を辺に沿って動かした面）
    Edge,
}

impl InverseOffsetPartKind {
    /// すべての種類（`display_parts` が返す順）
    const ALL: [Self; 6] = [
        Self::IndexedFace,
        Self::UnindexedFace,
        Self::UpperSide,
        Self::UnorientedFace,
        Self::Vertex,
        Self::Edge,
    ];

    /// `ALL` の中の位置
    fn slot(self) -> usize {
        match self {
            Self::IndexedFace => 0,
            Self::UnindexedFace => 1,
            Self::UpperSide => 2,
            Self::UnorientedFace => 3,
            Self::Vertex => 4,
            Self::Edge => 5,
        }
    }
}

/// 種類ごとの表示用の三角形（頂点の順は、要素の外の側から見て左回り。面積のない三角形は含めない）
#[derive(Debug, Clone, PartialEq)]
pub struct InverseOffsetPart {
    pub kind: InverseOffsetPartKind,
    pub triangles: Vec<[[f64; 3]; 3]>,
}

impl InverseOffsetShape {
    /// 要素を三角形に分けた、種類ごとの表示用の三角形（三角形のない種類は含めない）
    ///
    /// 球面・円柱面・円板は、角度 `angle_step`（ラジアン）以下のきざみで分ける。
    ///
    /// # Errors
    ///
    /// `angle_step` が有限でないか、1° から π / 2 の外の場合。
    pub fn display_parts(&self, angle_step: f64) -> Result<Vec<InverseOffsetPart>, CamSolverError> {
        if !(angle_step.is_finite()
            && (MIN_DISPLAY_ANGLE_STEP..=std::f64::consts::FRAC_PI_2).contains(&angle_step))
        {
            return Err(CamSolverError::InvalidInput(
                "display angle step must be finite and between 1 degree and pi / 2".to_string(),
            ));
        }
        let corners = |face: &geo_algorithms::Triangle3D<f64>| {
            [face.vertex_a(), face.vertex_b(), face.vertex_c()].map(|p| [p.x(), p.y(), p.z()])
        };
        let mut parts = InverseOffsetPartKind::ALL.map(|kind| InverseOffsetPart {
            kind,
            triangles: Vec::new(),
        });
        let slot = InverseOffsetPartKind::slot;
        for (face, &kind) in self.faces.iter().zip(&self.face_kinds) {
            parts[slot(kind)].triangles.push(corners(face));
        }
        // 上側の要素は外向きと逆の側から見るため、頂点の順を逆にする
        parts[slot(InverseOffsetPartKind::UpperSide)].triangles = self
            .upper_sides
            .iter()
            .map(|face| {
                let [a, b, c] = corners(face);
                [a, c, b]
            })
            .collect();
        let radius = self.shape.radius();
        // 球面の三角形は、ボールの頂点だけに使う
        let sphere =
            matches!(self.shape, CutterShape::Ball { .. }).then(|| Icosphere::new(angle_step));
        parts[slot(InverseOffsetPartKind::Vertex)].triangles = self
            .vertices
            .iter()
            .flat_map(|vertex| vertex_triangles(vertex, radius, angle_step, sphere.as_ref()))
            .collect();
        parts[slot(InverseOffsetPartKind::Edge)].triangles = self
            .edges
            .iter()
            .flat_map(|edge| edge_triangles(edge, radius, angle_step))
            .collect();
        Ok(parts
            .into_iter()
            .filter(|part| !part.triangles.is_empty())
            .collect())
    }
}

/// 単位球に内接する、正二十面体の面を細かく分けた三角形（頂点の順は外から見て左回り）
struct Icosphere {
    triangles: Vec<[[f64; 3]; 3]>,
}

impl Icosphere {
    /// 三角形の辺が中心から見て `angle_step` 以下になるように分ける
    fn new(angle_step: f64) -> Self {
        let golden = (1.0 + 5.0_f64.sqrt()) / 2.0;
        let vertices: Vec<[f64; 3]> = [
            [-1.0, golden, 0.0],
            [1.0, golden, 0.0],
            [-1.0, -golden, 0.0],
            [1.0, -golden, 0.0],
            [0.0, -1.0, golden],
            [0.0, 1.0, golden],
            [0.0, -1.0, -golden],
            [0.0, 1.0, -golden],
            [golden, 0.0, -1.0],
            [golden, 0.0, 1.0],
            [-golden, 0.0, -1.0],
            [-golden, 0.0, 1.0],
        ]
        .into_iter()
        .map(normalized)
        .collect();
        let faces = [
            [0, 11, 5],
            [0, 5, 1],
            [0, 1, 7],
            [0, 7, 10],
            [0, 10, 11],
            [1, 5, 9],
            [5, 11, 4],
            [11, 10, 2],
            [10, 7, 6],
            [7, 1, 8],
            [3, 9, 4],
            [3, 4, 2],
            [3, 2, 6],
            [3, 6, 8],
            [3, 8, 9],
            [4, 9, 5],
            [2, 4, 11],
            [6, 2, 10],
            [8, 6, 7],
            [9, 8, 1],
        ];
        // 面を n 等分した長さ L / n の辺は、中心からの距離 d 以上の位置にあるため、中心から見る角は
        // 2 asin(L / (2 n d)) 以下で、asin が凸のため 2 asin(L / (2 d)) / n 以下になる。d は内接球の半径（面の中心まで）
        let edge = sub(vertices[0], vertices[11]);
        let inradius = {
            let [a, b, c] = faces[0].map(|vertex| vertices[vertex]);
            dot(normalized(cross(sub(b, a), sub(c, a))), a)
        };
        let span = 2.0 * (dot(edge, edge).sqrt() / (2.0 * inradius)).asin();
        let count = divisions(span, angle_step);
        let mut triangles = Vec::new();
        for face in faces {
            let [a, b, c] = face.map(|vertex| vertices[vertex]);
            // 面の上の格子の点（a からの b・c の向きの段数）を球面へ戻す
            let at = |i: usize, j: usize| {
                let (u, v) = (i as f64 / count as f64, j as f64 / count as f64);
                normalized([0, 1, 2].map(|k| a[k] + u * (b[k] - a[k]) + v * (c[k] - a[k])))
            };
            for i in 0..count {
                for j in 0..count - i {
                    triangles.push([at(i, j), at(i + 1, j), at(i, j + 1)]);
                    if i + j + 1 < count {
                        triangles.push([at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)]);
                    }
                }
            }
        }
        Self { triangles }
    }
}

/// 頂点の要素の三角形（ボール: 球面のうち範囲の部分、フラット: 工具の底が乗る頂点の高さの円板）
fn vertex_triangles(
    vertex: &VertexElement,
    radius: f64,
    angle_step: f64,
    sphere: Option<&Icosphere>,
) -> Vec<[[f64; 3]; 3]> {
    let center = vertex.point;
    let at = |direction: [f64; 3]| [0, 1, 2].map(|k| center[k] + radius * direction[k]);
    let (Some(_), Some(sphere)) = (&vertex.sphere, sphere) else {
        let count = divisions(std::f64::consts::TAU, angle_step);
        let rim = |k: usize| {
            let angle = std::f64::consts::TAU * k as f64 / count as f64;
            at([angle.cos(), angle.sin(), 0.0])
        };
        return (0..count).map(|k| [center, rim(k), rim(k + 1)]).collect();
    };
    // 範囲: つながる辺の向き along ごとに dot(向き, along) <= 0（球の中心を通る平面の片側）。高さの計算の内積の余裕は
    // 境目の丸め誤差のためのもので、表示では持たない（持つと、範囲のない頂点にも余裕の幅の見えない細い三角形が残る）
    let planes: Vec<[f64; 3]> = vertex
        .neighbors
        .iter()
        .map(|&neighbor| normalized(sub(neighbor, center)))
        .collect();
    let mut triangles = Vec::new();
    for triangle in &sphere.triangles {
        // どれかの平面の外の側に 3 つの頂点がすべてある三角形は、範囲に入らない
        if planes
            .iter()
            .any(|&plane| triangle.iter().all(|&p| dot(p, plane) > 0.0))
        {
            continue;
        }
        let mut polygon = triangle.to_vec();
        for &plane in &planes {
            polygon = clipped(&polygon, plane);
            if polygon.len() < 3 {
                break;
            }
        }
        // 切り取った多角形（凸）を、最初の頂点から扇形に分けて球面へ戻す
        let points: Vec<[f64; 3]> = polygon.into_iter().map(|p| at(normalized(p))).collect();
        for k in 1..points.len().saturating_sub(1) {
            let piece = [points[0], points[k], points[k + 1]];
            if has_area(&piece, radius) {
                triangles.push(piece);
            }
        }
    }
    triangles
}

/// 凸多角形 `polygon` のうち、`dot(p, plane) <= 0` の部分（平面は原点を通る）
fn clipped(polygon: &[[f64; 3]], plane: [f64; 3]) -> Vec<[f64; 3]> {
    let side = |p: [f64; 3]| dot(p, plane);
    let mut result = Vec::with_capacity(polygon.len() + 1);
    for (k, &p) in polygon.iter().enumerate() {
        let q = polygon[(k + 1) % polygon.len()];
        let (sp, sq) = (side(p), side(q));
        if sp <= 0.0 {
            result.push(p);
        }
        if (sp < 0.0 && sq > 0.0) || (sp > 0.0 && sq < 0.0) {
            let t = sp / (sp - sq);
            result.push([0, 1, 2].map(|i| p[i] + t * (q[i] - p[i])));
        }
    }
    result
}

/// 辺の要素の三角形（ボール: 円柱面のうち範囲の円弧の部分、フラット: 円板の縁のうち範囲の円弧を辺に沿って動かした面）
fn edge_triangles(edge: &EdgeElement, radius: f64, angle_step: f64) -> Vec<[[f64; 3]; 3]> {
    let [start, end] = edge.ends;
    let direction = sub(end, start);
    let length = dot(direction, direction).sqrt();
    if length <= tolerance() {
        return Vec::new();
    }
    if edge.cylinder.is_none() {
        // 円板の縁のうち、辺の点から (x, y) への水平な向きが範囲の円弧を、辺の点の高さのまま辺に沿って動かした面
        // （斜円柱の一部。鉛直な辺は XY に面積を持たない）
        if direction[0].hypot(direction[1]) <= tolerance() {
            return Vec::new();
        }
        let Some((from, span)) = edge_arc(&edge.range, [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]) else {
            return Vec::new();
        };
        let count = divisions(span, angle_step);
        let rim = |p: [f64; 3], k: usize| {
            let angle = from + span * k as f64 / count as f64;
            [
                p[0] + radius * angle.cos(),
                p[1] + radius * angle.sin(),
                p[2],
            ]
        };
        let mut triangles = Vec::new();
        for k in 0..count {
            let (start_from, start_to) = (rim(start, k), rim(start, k + 1));
            let (end_from, end_to) = (rim(end, k), rim(end, k + 1));
            // 水平な辺では、辺と平行な縁の弦の三角形が面積を持たない
            triangles.extend(
                [
                    [start_from, start_to, end_to],
                    [start_from, end_to, end_from],
                ]
                .into_iter()
                .filter(|triangle| has_area(triangle, radius))
                .map(facing_up),
            );
        }
        return triangles;
    }
    let axis = direction.map(|component| component / length);
    let helper = if axis[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let first = normalized(cross(axis, helper));
    let second = cross(axis, first);
    let Some((from, span)) = edge_arc(&edge.range, first, second) else {
        return Vec::new();
    };
    let count = divisions(span, angle_step);
    let outward = |k: usize| {
        let angle = from + span * k as f64 / count as f64;
        [0, 1, 2].map(|i| angle.cos() * first[i] + angle.sin() * second[i])
    };
    let at = |p: [f64; 3], direction: [f64; 3]| [0, 1, 2].map(|i| p[i] + radius * direction[i]);
    let mut triangles = Vec::new();
    for k in 0..count {
        let (from, to) = (outward(k), outward(k + 1));
        let (start_from, start_to) = (at(start, from), at(start, to));
        let (end_from, end_to) = (at(end, from), at(end, to));
        triangles.push([start_from, start_to, end_to]);
        triangles.push([start_from, end_to, end_from]);
    }
    triangles
}

/// 向き cos θ · `first` + sin θ · `second` のうち、範囲を決めるどの向き（`range`）とも内積が 0 以下の範囲の円弧（始まりの
/// 角と長さ。`range` が空なら全周）。範囲がなければ `None`
///
/// 頂点の範囲と同じく、高さの計算の内積の余裕は持たない。
fn edge_arc(range: &[[f64; 3]], first: [f64; 3], second: [f64; 3]) -> Option<(f64, f64)> {
    use std::f64::consts::{PI, TAU};
    let mut arc = (0.0, TAU);
    for &bound in range {
        // 範囲を決める向きの角 φ に対して、範囲は [φ + π/2, φ + 3π/2] の半円
        let angle = dot(bound, second).atan2(dot(bound, first));
        let half = angle + PI / 2.0;
        let (start, span) = arc;
        let offset = (half - start).rem_euclid(TAU);
        arc = if span >= TAU {
            (half, PI)
        } else if offset <= span {
            (half, (span - offset).min(PI))
        } else if offset > PI {
            // 半円が、円弧の始まりの側に回り込んで重なる
            (start, span.min(offset - PI))
        } else {
            return None;
        };
    }
    (arc.1 > default_orthogonality_dot_error_tolerance::<f64>()).then_some(arc)
}

/// 角度 `span` を、きざみ `angle_step` 以下に分ける数
fn divisions(span: f64, angle_step: f64) -> usize {
    ((span / angle_step).ceil() as usize).max(1)
}

fn normalized(v: [f64; 3]) -> [f64; 3] {
    let norm = dot(v, v).sqrt();
    v.map(|component| component / norm)
}

/// 面積のある三角形か（大きさ `scale` の形の、数値の誤差の分の面積は除く）
fn has_area(&[a, b, c]: &[[f64; 3]; 3], scale: f64) -> bool {
    let n = cross(sub(b, a), sub(c, a));
    dot(n, n).sqrt() > tolerance() * scale
}

/// 法線が上を向く頂点の順にした三角形
fn facing_up([a, b, c]: [[f64; 3]; 3]) -> [[f64; 3]; 3] {
    if cross(sub(b, a), sub(c, a))[2] < 0.0 {
        [a, c, b]
    } else {
        [a, b, c]
    }
}

#[cfg(test)]
mod tests {
    use super::{Icosphere, cross, dot, sub};

    #[test]
    fn icosphere_triangles_face_outward_within_the_angle_step() {
        for degrees in [1.0_f64, 10.0, 90.0] {
            let step = degrees.to_radians();
            let sphere = Icosphere::new(step);
            for &[a, b, c] in &sphere.triangles {
                let centroid = [0, 1, 2].map(|k| a[k] + b[k] + c[k]);
                assert!(dot(cross(sub(b, a), sub(c, a)), centroid) > 0.0);
                for (p, q) in [(a, b), (b, c), (c, a)] {
                    assert!(dot(p, q).clamp(-1.0, 1.0).acos() <= step + 1e-12);
                }
            }
        }
    }
}
