//! 走査線の区間の弦（CL の 2 点を結ぶ直線）から、逆オフセット形状の上側の面が上に出る量の最大
//!
//! 頂点・辺の要素は、範囲（隣り合う面ベクトルの間）で切らない。範囲の外の部分（球面・辺の長さの円柱面・円板の全体）
//! も工具が形状に接する位置で、逆オフセット形状の内側にあり、上側の面より上に出ないため、最大は変わらない。
//!
//! 走査線の鉛直な平面（y が一定）で要素を切った形は、面とフラットの頂点は線分、そのほかは 2 次曲線を直線の条件（区間
//! の中・辺の中）で切った部分になる（ボールの頂点は (x, z) の円、ボールの辺は (x, z) の楕円か平行な 2 本の直線、
//! フラットの辺は x と辺の上の位置 t の平面の楕円で、高さは t の 1 次式）。弦から上に出る量はその平面の 1 次の関数の
//! ため、最大は条件の境目（条件の直線と曲線の交点）か、曲線の上で関数が止まる点（接線が関数の等高線と平行な点。曲線と、
//! 曲線の勾配が関数の勾配と平行になる直線の交点）にある。それらの点を 2 次方程式で求め、条件を満たす点の最大を取る。

use geo_algorithms::{LineSegment2D, Point2D, Triangle3D};
use geo_contracts::{LineSegment2DConstructor, Triangle3DBoundaryAccess};

use super::{EdgeElement, Element, InverseOffsetShape, VertexElement, dot, sub, tolerance};

impl InverseOffsetShape {
    /// 走査線 y の上の、工具先端の 2 点 `start`・`end`（[x, z]）を結ぶ弦から、CL の面（工具先端が通る面）が上に出る
    /// 量の最大と、その x（`(x, 上に出る量)`）。CL の面が弦より下にしかない場合は負の値。2 点の x が同じ場合と、区間で
    /// 工具が形状に接しない場合は `None`
    pub fn rise_above_chord(&self, y: f64, start: [f64; 2], end: [f64; 2]) -> Option<(f64, f64)> {
        let ([x0, z0], [x1, z1]) = if start[0] <= end[0] {
            (start, end)
        } else {
            (end, start)
        };
        if x1 <= x0 {
            return None;
        }
        // 要素の高さ（基準点の高さ）にそろえた弦
        let offset = self.shape.reference_offset();
        let chord = Chord {
            start: x0,
            end: x1,
            height: z0 + offset,
            slope: (z1 - z0) / (x1 - x0),
        };
        let lowest = z0.min(z1) + offset;
        let mut best: Option<(f64, f64)> = None;
        for element in self.index.candidates_within([x0, y], [x1, y]) {
            // 候補は上限の高い順のため、上に出る量の上限（要素の上限 − 弦の最も低い所）が今の最大以下になれば、残りの
            // 候補はそれより上に出ない
            let upper = self.z_ranges[element].upper + tolerance() - lowest;
            if best.is_some_and(|(_, rise)| rise >= upper) {
                break;
            }
            let found = match self.elements[element] {
                Element::Face(face) => face_rise(&self.faces[face], y, &chord),
                Element::UpperSide(face) => face_rise(&self.upper_sides[face], y, &chord),
                Element::Vertex(vertex) => self.vertex_rise(vertex, y, &chord),
                Element::Edge(edge) => self.edge_rise(edge, y, &chord),
            };
            if let Some(found) = found
                && best.is_none_or(|(_, rise)| found.1 > rise)
            {
                best = Some(found);
            }
        }
        best
    }

    fn vertex_rise(&self, vertex: usize, y: f64, chord: &Chord) -> Option<(f64, f64)> {
        let VertexElement { point, sphere, .. } = &self.vertices[vertex];
        let radius = self.shape.radius();
        let w = y - point[1];
        if sphere.is_none() {
            // 頂点の高さの水平な円板を切った線分
            if w.abs() > radius {
                return None;
            }
            let half = (radius * radius - w * w).sqrt();
            return segment_rise(
                [[point[0] - half, point[2]], [point[0] + half, point[2]]],
                chord,
            );
        }
        // 球の中心からの差 (u, v) = (x − 中心の x, z − 中心の z) の円 u² + v² + w² = r²
        let conic = [1.0, 0.0, 1.0, 0.0, 0.0, w * w - radius * radius];
        let bounds = chord.interval_bounds(point[0]);
        let objective = chord.objective(point[0], point[2], 1.0);
        maximize(conic, radius, objective, &bounds).map(|([u, _], rise)| (point[0] + u, rise))
    }

    fn edge_rise(&self, edge: usize, y: f64, chord: &Chord) -> Option<(f64, f64)> {
        let EdgeElement {
            ends: [start, end],
            cylinder,
            ..
        } = &self.edges[edge];
        let radius = self.shape.radius();
        let d = sub(*end, *start);
        let w = y - start[1];
        let mut bounds = chord.interval_bounds(start[0]);
        let (conic, objective) = if cylinder.is_some() {
            // 円柱面を持つ辺は長さがある（長さのない辺は、作るときに外す）
            let length_squared = dot(d, d);
            let n = d.map(|component| component / length_squared.sqrt());
            // 辺の始点からの差 q = (u, w, v) の、軸からの距離の 2 乗 |q|² − (q·n)² = r²（n は辺の単位の向き）
            let conic = [
                1.0 - n[0] * n[0],
                -2.0 * n[0] * n[2],
                1.0 - n[2] * n[2],
                -2.0 * n[0] * n[1] * w,
                -2.0 * n[2] * n[1] * w,
                w * w * (1.0 - n[1] * n[1]) - radius * radius,
            ];
            // 軸への垂線の足が辺の中にある（0 <= q·d <= |d|²）
            let slack = tolerance() * length_squared.sqrt();
            bounds.push(([-d[0], -d[2], -d[1] * w], slack));
            bounds.push(([d[0], d[2], d[1] * w - length_squared], slack));
            (conic, chord.objective(start[0], start[2], 1.0))
        } else {
            // XY に投影した辺が線分にならない（ほぼ鉛直な）辺は、`edge_height` と同じく高さを決めない
            LineSegment2D::new(
                Point2D::new(start[0], start[1]),
                Point2D::new(end[0], end[1]),
            )?;
            let horizontal_squared = d[0] * d[0] + d[1] * d[1];
            // u = x − 始点の x と辺の上の位置 t の、(x, y) と辺の点の水平な距離 (u − t d_x)² + (w − t d_y)² = r²。
            // 高さは辺の高さ（始点の高さ + t d_z）
            let conic = [
                1.0,
                -2.0 * d[0],
                horizontal_squared,
                0.0,
                -2.0 * w * d[1],
                w * w - radius * radius,
            ];
            // 0 <= t <= 1
            bounds.push(([0.0, -1.0, 0.0], tolerance()));
            bounds.push(([0.0, 1.0, -1.0], tolerance()));
            (conic, chord.objective(start[0], start[2], d[2]))
        };
        maximize(conic, radius, objective, &bounds).map(|([u, _], rise)| (start[0] + u, rise))
    }
}

/// 要素の高さにそろえた弦（x が `start`〜`end` の区間）
struct Chord {
    start: f64,
    end: f64,
    /// `start` での高さ
    height: f64,
    slope: f64,
}

impl Chord {
    fn at(&self, x: f64) -> f64 {
        self.height + self.slope * (x - self.start)
    }

    /// x = `origin_x` + u の、区間の中にある条件（`maximize` の条件）
    fn interval_bounds(&self, origin_x: f64) -> Vec<([f64; 3], f64)> {
        vec![
            ([-1.0, 0.0, self.start - origin_x], tolerance()),
            ([1.0, 0.0, origin_x - self.end], tolerance()),
        ]
    }

    /// 点 (`origin_x` + u, `origin_z` + `height_per_v` v) の、弦から上に出る量（`maximize` の 1 次の関数）
    fn objective(&self, origin_x: f64, origin_z: f64, height_per_v: f64) -> [f64; 3] {
        [-self.slope, height_per_v, origin_z - self.at(origin_x)]
    }
}

/// 三角形を平面 y で切った線分の、弦から上に出る量の最大
fn face_rise(face: &Triangle3D<f64>, y: f64, chord: &Chord) -> Option<(f64, f64)> {
    let corners =
        [face.vertex_a(), face.vertex_b(), face.vertex_c()].map(|p| [p.x(), p.y(), p.z()]);
    let mut ends: Vec<[f64; 2]> = Vec::new();
    for k in 0..3 {
        let (p, q) = (corners[k], corners[(k + 1) % 3]);
        if p[1] == y {
            ends.push([p[0], p[2]]);
        }
        if (p[1] - y) * (q[1] - y) < 0.0 {
            let s = (y - p[1]) / (q[1] - p[1]);
            ends.push([p[0] + s * (q[0] - p[0]), p[2] + s * (q[2] - p[2])]);
        }
    }
    let by_x = |a: &&[f64; 2], b: &&[f64; 2]| a[0].total_cmp(&b[0]);
    let low = *ends.iter().min_by(by_x)?;
    let high = *ends.iter().max_by(by_x)?;
    segment_rise([low, high], chord)
}

/// 線分 [`low`, `high`]（[x, z]。x の小さい順。x が同じ場合は、平面に頂点だけが乗る 1 点）の、弦から上に出る量の
/// 最大（区間で切った線分の端にある）
fn segment_rise([low, high]: [[f64; 2]; 2], chord: &Chord) -> Option<(f64, f64)> {
    let from = low[0].max(chord.start);
    let to = high[0].min(chord.end);
    if from > to {
        return None;
    }
    let height = |x: f64| {
        if high[0] > low[0] {
            low[1] + (x - low[0]) / (high[0] - low[0]) * (high[1] - low[1])
        } else {
            low[1].max(high[1])
        }
    };
    [from, to]
        .map(|x| (x, height(x) - chord.at(x)))
        .into_iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
}

/// 2 次曲線 `conic`（[a, b, c, d, e, f] で a u² + b uv + c v² + d u + e v + f = 0）のうち、すべての条件（[α, β, γ]
/// と余裕 s で α u + β v + γ <= s）を満たす部分の、1 次の関数 `objective`（[g_u, g_v, g_0] で g_u u + g_v v + g_0）の
/// 最大の点と値
///
/// 曲線の式の値は、中心・軸からの距離の 2 乗 − `radius` の 2 乗。直線が曲線に交差演算の許容誤差まで近づけば、接する
/// とする（`tip_height_at` の交差演算と同じく、接する所を落とさない）
fn maximize(
    conic: [f64; 6],
    radius: f64,
    objective: [f64; 3],
    bounds: &[([f64; 3], f64)],
) -> Option<([f64; 2], f64)> {
    let [a, b, c, d, e, _] = conic;
    let [g_u, g_v, _] = objective;
    // 曲線の勾配 (2a u + b v + d, b u + 2c v + e) が関数の勾配と平行になる直線
    let stationary = [
        2.0 * a * g_v - b * g_u,
        b * g_v - 2.0 * c * g_u,
        d * g_v - e * g_u,
    ];
    // 距離が許容誤差だけ違う場合の、曲線の式の値の違い
    let level = 2.0 * radius * tolerance();
    std::iter::once(stationary)
        .chain(bounds.iter().map(|&(line, _)| line))
        .flat_map(|line| line_points(conic, level, line))
        .filter(|&point| {
            bounds
                .iter()
                .all(|&(line, slack)| linear(line, point) <= slack)
        })
        .map(|point| (point, linear(objective, point)))
        .max_by(|first, second| first.1.total_cmp(&second.1))
}

/// 2 次曲線 `conic` と直線 `line`（[α, β, γ] で α u + β v + γ = 0）の交点（直線が曲線に含まれる場合は返さない）。
/// 直線の上の曲線の式の値の、0 からの離れが `level` 以下の場合は、接する点とする
fn line_points(conic: [f64; 6], level: f64, [alpha, beta, gamma]: [f64; 3]) -> Vec<[f64; 2]> {
    let norm_squared = alpha * alpha + beta * beta;
    if norm_squared == 0.0 {
        return Vec::new();
    }
    // 直線の点 origin + s direction を代入した s の 2 次式
    let origin = [-gamma * alpha / norm_squared, -gamma * beta / norm_squared];
    let direction = [-beta, alpha];
    let [a, b, c, d, e, f] = conic;
    let [u, v] = origin;
    let [p, q] = direction;
    let quadratic = a * p * p + b * p * q + c * q * q;
    let gradient = [2.0 * a * u + b * v + d, b * u + 2.0 * c * v + e];
    let slope = gradient[0] * p + gradient[1] * q;
    let constant = a * u * u + b * u * v + c * v * v + d * u + e * v + f;
    quadratic_roots(quadratic, slope, constant, level)
        .into_iter()
        .map(|s| [u + s * p, v + s * q])
        .collect()
}

/// a s² + b s + c = 0 の実数の解（桁落ちしない式）。解がなくても、2 次式の値の 0 からの離れの最小（|判別式| / 4|a|）
/// が `level` 以下の場合は、その点を接する重解とする
fn quadratic_roots(a: f64, b: f64, c: f64, level: f64) -> Vec<f64> {
    let mut discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        if -discriminant > 4.0 * a.abs() * level {
            return Vec::new();
        }
        discriminant = 0.0;
    }
    let q = -0.5 * (b + discriminant.sqrt().copysign(b));
    let mut roots = Vec::new();
    if a != 0.0 {
        roots.push(q / a);
    }
    if q != 0.0 {
        roots.push(c / q);
    }
    roots
}

fn linear([alpha, beta, gamma]: [f64; 3], [u, v]: [f64; 2]) -> f64 {
    alpha * u + beta * v + gamma
}

#[cfg(test)]
mod tests {
    use super::{maximize, quadratic_roots};

    fn assert_near(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
    }

    #[test]
    fn nearly_tangent_quadratics_have_a_double_root() {
        // (s + 1)² + 1e-13 は、値の 0 からの離れの最小が 1e-13
        let roots = quadratic_roots(1.0, 2.0, 1.0 + 1e-13, 1e-12);
        assert!(!roots.is_empty());
        for root in roots {
            assert_near(root, -1.0);
        }
        assert!(quadratic_roots(1.0, 2.0, 1.0 + 1e-11, 1e-12).is_empty());
    }

    #[test]
    fn maxima_on_conics_are_at_stationary_points_or_on_the_bounds() {
        let circle = [1.0, 0.0, 1.0, 0.0, 0.0, -1.0];
        // 単位円の v の最大は真上、u >= 0.6 で切ると境目
        let (point, value) = maximize(circle, 1.0, [0.0, 1.0, 0.0], &[]).unwrap();
        assert_near(point[0], 0.0);
        assert_near(value, 1.0);
        let (point, value) =
            maximize(circle, 1.0, [0.0, 1.0, 0.0], &[([-1.0, 0.0, 0.6], 0.0)]).unwrap();
        assert_near(point[0], 0.6);
        assert_near(value, 0.8);
        // 楕円 u² / 4 + v² = 1 の u + v の最大は (4 / √5, 1 / √5)
        let ellipse = [0.25, 0.0, 1.0, 0.0, 0.0, -1.0];
        let (point, value) = maximize(ellipse, 1.0, [1.0, 1.0, 0.0], &[]).unwrap();
        assert_near(point[0], 4.0 / 5.0_f64.sqrt());
        assert_near(value, 5.0_f64.sqrt());
        // 平行な 2 本の直線 v = ±1 の、0 <= u <= 2 の v + 0.5 u の最大は (2, 1)
        let lines = [0.0, 0.0, 1.0, 0.0, 0.0, -1.0];
        let bounds = [([-1.0, 0.0, 0.0], 0.0), ([1.0, 0.0, -2.0], 0.0)];
        let (point, value) = maximize(lines, 1.0, [0.5, 1.0, 0.0], &bounds).unwrap();
        assert_near(point[0], 2.0);
        assert_near(value, 2.0);
        // 条件を満たす部分がない
        assert!(maximize(circle, 1.0, [0.0, 1.0, 0.0], &[([-1.0, 0.0, 2.0], 0.0)]).is_none());
    }
}
