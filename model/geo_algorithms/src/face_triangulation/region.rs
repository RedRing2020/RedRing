//! 切れ目を入れた範囲（設計書 5.3）と、多角形の辺の交わりの確かめ（6.1）
//!
//! 周期的な曲面では、ループの点を、隣り合う点の差が周期の半分以下になるようにつなげ直し（極の線の上では、範囲が
//! 左に来る向き）、ループが u・v の周期を何回まわるか（巻き数）を数える。周期をまわるループのある面と、ループが穴
//! だけの閉じた曲面の面は、ループの既にある点（と極）を結ぶ切れ目で、閉じた多角形にする。切れ目の上には、切れ目の
//! 向きの刻みで点を置き、左右で同じ頂点にする。v の向きにまわるループ（トーラスの管のまわり）は、u と v を入れ
//! 替えて同じように扱う。

use analysis::consts::tessellation::POLE_PARAMETER_TOLERANCE;

use super::{
    FaceTriangulationError, FaceTriangulationSettings, FaceVertex, LoopPoint, TriangulationSurface,
};
use crate::Point2D;

/// 範囲の多角形の点
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct RegionPoint {
    pub(super) vertex: FaceVertex,
    pub(super) uv: Point2D<f64>,
}

/// 切れ目を入れた範囲（閉じた多角形の並び。どの多角形も最後の点から最初の点へ閉じる）
#[derive(Debug, Clone, PartialEq)]
pub struct FaceRegion {
    pub(super) polygons: Vec<Vec<RegionPoint>>,
    /// 足した点（切れ目・極）の (u, v)。`FaceVertex::Added` の番号の順
    pub(super) added: Vec<Point2D<f64>>,
}

impl FaceRegion {
    /// 面のループ `loops`（それぞれ、同じ点を最後に繰り返さない点の並び）から、切れ目を入れた範囲を作る
    ///
    /// # Errors
    ///
    /// 設定・ループが正しくない、扱えないループの組、穴を避けて切れ目を置けないとき。
    pub fn new(
        loops: &[Vec<LoopPoint>],
        surface: &impl TriangulationSurface,
        settings: &FaceTriangulationSettings,
    ) -> Result<Self, FaceTriangulationError> {
        settings.check()?;
        let periods = surface.periods();
        let poles = surface.poles();
        let rings = loops
            .iter()
            .map(|points| Ring::unwrap(points, periods, poles))
            .collect::<Result<Vec<_>, _>>()?;
        let unsupported = rings.iter().any(|ring| {
            let (wu, wv) = ring.winding;
            wu.abs() > 1 || wv.abs() > 1 || (wu != 0 && wv != 0)
        });
        let swap = rings.iter().any(|ring| ring.winding.1 != 0);
        if unsupported || (swap && rings.iter().any(|ring| ring.winding.0 != 0)) {
            return Err(FaceTriangulationError::UnsupportedLoops);
        }
        let swapped = |pair: (Option<f64>, Option<f64>)| {
            if swap { (pair.1, pair.0) } else { pair }
        };
        let mut builder = Builder {
            swap,
            periods: swapped(periods),
            poles: if swap { (None, None) } else { poles },
            steps: swapped(super::checked_steps(surface, settings)?),
            max_points: settings.max_points,
            added: Vec::new(),
        };
        let (wrapping, holes): (Vec<Ring>, Vec<Ring>) = rings
            .into_iter()
            .map(|ring| ring.into_frame(swap))
            .partition(|ring| ring.winding != (0, 0));
        let polygons = match wrapping.as_slice() {
            [] => builder.without_wrapping(holes)?,
            [ring] => builder.cap(ring, holes)?,
            [first, second] => builder.band(first, second, holes)?,
            _ => return Err(FaceTriangulationError::UnsupportedLoops),
        };
        let polygons = polygons
            .iter()
            .map(|polygon| {
                Ok(builder
                    .split_pole_lines(polygon)?
                    .into_iter()
                    .map(|point| RegionPoint {
                        vertex: point.vertex,
                        uv: builder.to_uv(point.a, point.b),
                    })
                    .collect())
            })
            .collect::<Result<_, FaceTriangulationError>>()?;
        Ok(Self {
            polygons,
            added: builder.added,
        })
    }

    /// 交わる多角形の辺のうち、両端が境界の点の辺（頂点の番号の組）
    ///
    /// 辺どうしが端の点のほかで交わるか接すれば、交わるとする。三角形分割と同じ座標（曲面 `surface` の倍率を掛けた
    /// (u, v)）と、丸め誤差に強い向きの判定で確かめる。呼び出し元は、その辺を含む稜線を分け直して、範囲を作り直す。
    pub fn crossing_edges(&self, surface: &impl TriangulationSurface) -> Vec<[usize; 2]> {
        let scale = surface.scale();
        let edges: Vec<Edge> = self
            .polygons
            .iter()
            .flat_map(|polygon| {
                (0..polygon.len()).map(|index| {
                    let (from, to) = (polygon[index], polygon[(index + 1) % polygon.len()]);
                    Edge {
                        vertices: [from.vertex, to.vertex],
                        from: (from.uv.x() * scale.0, from.uv.y() * scale.1),
                        to: (to.uv.x() * scale.0, to.uv.y() * scale.1),
                    }
                })
            })
            .collect();
        let mut order: Vec<usize> = (0..edges.len()).collect();
        order.sort_by(|&a, &b| edges[a].min_u().total_cmp(&edges[b].min_u()));
        let mut crossing = vec![false; edges.len()];
        for (position, &first) in order.iter().enumerate() {
            for &second in &order[position + 1..] {
                if edges[second].min_u() > edges[first].max_u() {
                    break;
                }
                if segments_cross(
                    (edges[first].from, edges[first].to),
                    (edges[second].from, edges[second].to),
                ) {
                    crossing[first] = true;
                    crossing[second] = true;
                }
            }
        }
        let mut result: Vec<[usize; 2]> = edges
            .iter()
            .zip(crossing)
            .filter(|(_, crossing)| *crossing)
            .filter_map(|(edge, _)| match edge.vertices {
                [FaceVertex::Boundary(from), FaceVertex::Boundary(to)] => Some([from, to]),
                _ => None,
            })
            .collect();
        result.sort_unstable();
        result.dedup();
        result
    }
}

/// (u, v) の線分（a と b、または u と v の組の 2 点）
type Segment = ((f64, f64), (f64, f64));

/// 多角形の辺
struct Edge {
    vertices: [FaceVertex; 2],
    from: (f64, f64),
    to: (f64, f64),
}

impl Edge {
    fn min_u(&self) -> f64 {
        self.from.0.min(self.to.0)
    }

    fn max_u(&self) -> f64 {
        self.from.0.max(self.to.0)
    }
}

/// 切れ目を作るときの点（a は周期をまわる向き、b はもう一方の向き。v の向きにまわるときは u と v を入れ替える）
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    vertex: FaceVertex,
    a: f64,
    b: f64,
}

impl Point {
    fn shifted(self, da: f64, db: f64) -> Self {
        Self {
            a: self.a + da,
            b: self.b + db,
            ..self
        }
    }

    fn position(self) -> (f64, f64) {
        (self.a, self.b)
    }
}

/// 周期をつなげ直したループ
#[derive(Debug, Clone)]
struct Ring {
    points: Vec<Point>,
    /// u・v の周期をまわる回数（`into_frame` の後は (a の回数, b の回数)）
    winding: (i32, i32),
}

impl Ring {
    fn unwrap(
        points: &[LoopPoint],
        periods: (Option<f64>, Option<f64>),
        poles: (Option<f64>, Option<f64>),
    ) -> Result<Self, FaceTriangulationError> {
        let mut unwrapped: Vec<Point> = Vec::with_capacity(points.len());
        for point in points {
            let (u, v) = (point.uv.x(), point.uv.y());
            if !(u.is_finite() && v.is_finite()) {
                return Err(FaceTriangulationError::InvalidLoops);
            }
            let next = match unwrapped.last() {
                None => Point {
                    vertex: FaceVertex::Boundary(point.vertex),
                    a: u,
                    b: v,
                },
                Some(&previous) => {
                    let (du, dv) = unwrapped_step(previous, u, v, periods, poles);
                    Point {
                        vertex: FaceVertex::Boundary(point.vertex),
                        a: previous.a + du,
                        b: previous.b + dv,
                    }
                }
            };
            if unwrapped.last() != Some(&next) {
                unwrapped.push(next);
            }
        }
        let (Some(&first), Some(&last)) = (unwrapped.first(), unwrapped.last()) else {
            return Err(FaceTriangulationError::InvalidLoops);
        };
        let (du, dv) = unwrapped_step(last, first.a, first.b, periods, poles);
        let turns = |period: Option<f64>, total: f64| {
            period.map_or(0, |period| (total / period).round() as i32)
        };
        let winding = (
            turns(periods.0, last.a + du - first.a),
            turns(periods.1, last.b + dv - first.b),
        );
        if winding == (0, 0) && unwrapped.len() < 3 {
            return Err(FaceTriangulationError::InvalidLoops);
        }
        Ok(Self {
            points: unwrapped,
            winding,
        })
    }

    fn into_frame(self, swap: bool) -> Self {
        if !swap {
            return self;
        }
        Self {
            points: self
                .points
                .into_iter()
                .map(|point| Point {
                    a: point.b,
                    b: point.a,
                    ..point
                })
                .collect(),
            winding: (self.winding.1, self.winding.0),
        }
    }

    /// 周期をまわらないループの符号つきの面積（反時計回りで正）
    fn area(&self) -> f64 {
        let count = self.points.len();
        (0..count)
            .map(|index| {
                let (p, q) = (self.points[index], self.points[(index + 1) % count]);
                p.a * q.b - q.a * p.b
            })
            .sum::<f64>()
            / 2.0
    }

    fn range(&self, of: impl Fn(&Point) -> f64) -> (f64, f64) {
        self.points
            .iter()
            .map(of)
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), value| {
                (low.min(value), high.max(value))
            })
    }

    fn mean_b(&self) -> f64 {
        self.points.iter().map(|point| point.b).sum::<f64>() / self.points.len() as f64
    }

    /// 周期をまわらないループを、最初の点が `[start, start + 周期)` に入るよう、周期の分だけ動かす
    fn shifted_into(
        &self,
        start: (Option<f64>, Option<f64>),
        periods: (Option<f64>, Option<f64>),
    ) -> Self {
        let shift = |start: Option<f64>, period: Option<f64>, value: f64| match (start, period) {
            (Some(start), Some(period)) => -((value - start) / period).floor() * period,
            _ => 0.0,
        };
        let first = self.points[0];
        let (da, db) = (
            shift(start.0, periods.0, first.a),
            shift(start.1, periods.1, first.b),
        );
        Self {
            points: self
                .points
                .iter()
                .map(|point| point.shifted(da, db))
                .collect(),
            winding: self.winding,
        }
    }

    /// 周期をまわるループを、`start` 番目の点から 1 周たどった点の並び（最後に、最初の点を周期の分だけ動かした点を
    /// 含む）。すべての点を a の向きに `shift` だけ動かす
    fn rotated(&self, start: usize, period: f64, shift: f64) -> Vec<Point> {
        let count = self.points.len();
        let turn = f64::from(self.winding.0) * period;
        (0..=count)
            .map(|offset| {
                let index = (start + offset) % count;
                let da = if start + offset >= count {
                    turn + shift
                } else {
                    shift
                };
                self.points[index].shifted(da, 0.0)
            })
            .collect()
    }

    /// 閉じた多角形としての辺
    fn closed_edges(&self) -> impl Iterator<Item = Segment> + '_ {
        let count = self.points.len();
        (0..count).map(move |index| {
            (
                self.points[index].position(),
                self.points[(index + 1) % count].position(),
            )
        })
    }
}

/// 点 `previous` から (u, v) への差を、周期の分だけ動かして近い向きにする。極の線の上では、範囲が左に来る向き
/// （大きい側の極では u が減る向き、小さい側の極では u が増える向き）にし、u の値が周期の整数倍だけ違えば、極の線を
/// 1 周する（継ぎ目の稜線が極を通るループ）
fn unwrapped_step(
    previous: Point,
    u: f64,
    v: f64,
    periods: (Option<f64>, Option<f64>),
    poles: (Option<f64>, Option<f64>),
) -> (f64, f64) {
    let near = |pole: Option<f64>| {
        pole.is_some_and(|pole| {
            (previous.b - pole).abs() <= POLE_PARAMETER_TOLERANCE
                && (v - pole).abs() <= POLE_PARAMETER_TOLERANCE
        })
    };
    let mut du = u - previous.a;
    if let Some(period) = periods.0 {
        let upper = near(poles.1);
        if upper || near(poles.0) {
            // u が増える向き・減る向きに進む量。どちらかがほぼ 0 なら、値が周期の整数倍だけ違うので 1 周する
            let forward = du.rem_euclid(period);
            let backward = (-du).rem_euclid(period);
            let whole = forward <= POLE_PARAMETER_TOLERANCE || backward <= POLE_PARAMETER_TOLERANCE;
            du = if du.abs() <= POLE_PARAMETER_TOLERANCE {
                0.0
            } else if whole {
                if upper { -period } else { period }
            } else if upper {
                -backward
            } else {
                forward
            };
        } else {
            du -= (du / period).round() * period;
        }
    }
    let mut dv = v - previous.b;
    if let Some(period) = periods.1 {
        dv -= (dv / period).round() * period;
    }
    (du, dv)
}

/// 切れ目を入れた範囲を作るときの曲面の情報と、足した点
struct Builder {
    swap: bool,
    periods: (Option<f64>, Option<f64>),
    poles: (Option<f64>, Option<f64>),
    steps: (Option<f64>, Option<f64>),
    /// 1 つの面の頂点の数の上限（切れ目と極の線に置く点の数を抑える）
    max_points: usize,
    added: Vec<Point2D<f64>>,
}

impl Builder {
    fn to_uv(&self, a: f64, b: f64) -> Point2D<f64> {
        if self.swap {
            Point2D::new(b, a)
        } else {
            Point2D::new(a, b)
        }
    }

    fn add(&mut self, a: f64, b: f64) -> FaceVertex {
        self.added.push(self.to_uv(a, b));
        FaceVertex::Added(self.added.len() - 1)
    }

    /// 周期をまわるループのない面: 外側のループが 1 つなら、そのまま。ループが穴だけなら、閉じた曲面の全体
    fn without_wrapping(
        &mut self,
        rings: Vec<Ring>,
    ) -> Result<Vec<Vec<Point>>, FaceTriangulationError> {
        let (outer, holes): (Vec<Ring>, Vec<Ring>) =
            rings.into_iter().partition(|ring| ring.area() > 0.0);
        match outer.as_slice() {
            [outer] => {
                let start = (Some(outer.range(|p| p.a).0), Some(outer.range(|p| p.b).0));
                let mut polygons = vec![outer.points.clone()];
                polygons.extend(
                    holes
                        .iter()
                        .map(|hole| hole.shifted_into(start, self.periods).points),
                );
                Ok(polygons)
            }
            [] => match (self.periods, self.poles) {
                ((Some(period), _), (Some(low), Some(high))) => {
                    self.whole_with_poles(period, (low, high), &holes)
                }
                ((Some(period_a), Some(period_b)), _) => {
                    self.whole_torus((period_a, period_b), &holes)
                }
                _ => Err(FaceTriangulationError::UnsupportedLoops),
            },
            _ => Err(FaceTriangulationError::UnsupportedLoops),
        }
    }

    /// 1 つのループが 1 回まわる面（極までの面）
    fn cap(
        &mut self,
        ring: &Ring,
        holes: Vec<Ring>,
    ) -> Result<Vec<Vec<Point>>, FaceTriangulationError> {
        let Some(period) = self.periods.0 else {
            return Err(FaceTriangulationError::UnsupportedLoops);
        };
        let turn = f64::from(ring.winding.0) * period;
        // 範囲は、まわる向きの左の側（a が増える向きなら b の大きい側）
        let pole = if ring.winding.0 > 0 {
            self.poles.1
        } else {
            self.poles.0
        }
        .ok_or(FaceTriangulationError::UnsupportedLoops)?;
        for start in candidates(ring, &holes, period) {
            let first = ring.points[start];
            let window = first.a.min(first.a + turn);
            let holes: Vec<Ring> = holes
                .iter()
                .map(|hole| hole.shifted_into((Some(window), None), self.periods))
                .collect();
            let boundary = ring.rotated(start, period, 0.0);
            let top = (first.a, pole);
            let cuts = [
                (first.position(), top),
                (first.shifted(turn, 0.0).position(), (top.0 + turn, top.1)),
            ];
            if blocked(&cuts, &holes, &[&boundary]) {
                continue;
            }
            let pole_vertex = self.add(top.0, top.1);
            let cut = self.cut_points(first.position(), top, 1)?;
            let mut polygon = boundary;
            polygon.extend(cut.iter().map(|point| point.shifted(turn, 0.0)));
            polygon.push(Point {
                vertex: pole_vertex,
                a: top.0 + turn,
                b: top.1,
            });
            polygon.push(Point {
                vertex: pole_vertex,
                a: top.0,
                b: top.1,
            });
            polygon.extend(cut.iter().rev());
            let mut polygons = vec![polygon];
            polygons.extend(holes.into_iter().map(|hole| hole.points));
            return Ok(polygons);
        }
        Err(FaceTriangulationError::NoCut)
    }

    /// 2 つのループが逆向きに 1 回ずつまわる面（円柱の帯など）
    fn band(
        &mut self,
        first: &Ring,
        second: &Ring,
        holes: Vec<Ring>,
    ) -> Result<Vec<Vec<Point>>, FaceTriangulationError> {
        let Some(period) = self.periods.0 else {
            return Err(FaceTriangulationError::UnsupportedLoops);
        };
        // 範囲は、まわる向きの左の側。u と v を入れ替えたときは右の側
        let side = |ring: &Ring| ring.winding.0 * if self.swap { -1 } else { 1 };
        let (lower, upper) = match (side(first), side(second)) {
            (1, -1) => (first, second),
            (-1, 1) => (second, first),
            _ => return Err(FaceTriangulationError::UnsupportedLoops),
        };
        let mut upper = upper.clone();
        if let Some(period_b) = self.periods.1 {
            // 上のループを、下のループの 1 周の中に動かす
            let start = lower.points[0].b;
            let shift = -((upper.points[0].b - start) / period_b).ceil() * period_b + period_b;
            upper = Ring {
                points: upper.points.iter().map(|p| p.shifted(0.0, shift)).collect(),
                winding: upper.winding,
            };
        }
        if upper.mean_b() <= lower.mean_b() {
            return Err(FaceTriangulationError::UnsupportedLoops);
        }
        let turn = f64::from(lower.winding.0) * period;
        let low_b = lower.range(|p| p.b).0;
        for start in candidates(lower, &holes, period) {
            let from = lower.points[start];
            // 上のループの点のうち、a が最も近い点。周期の分だけ動かして、a の差を周期の半分以下にする
            let (target, offset) = upper
                .points
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    let offset = -((point.a - from.a) / period).round() * period;
                    (index, offset, (point.a + offset - from.a).abs())
                })
                .min_by(|x, y| x.2.total_cmp(&y.2))
                .map(|(index, offset, _)| (index, offset))
                .expect("a ring has points");
            let upper_ring = Ring {
                points: upper
                    .points
                    .iter()
                    .map(|p| p.shifted(offset, 0.0))
                    .collect(),
                winding: upper.winding,
            };
            let to = upper_ring.points[target];
            let left_a = |b: f64| {
                let span = to.b - from.b;
                if span == 0.0 {
                    return from.a;
                }
                from.a + (to.a - from.a) * ((b - from.b) / span).clamp(0.0, 1.0)
            };
            let holes: Vec<Ring> = holes
                .iter()
                .map(|hole| {
                    let b = hole.points[0].b;
                    hole.shifted_into(
                        (Some(left_a(b).min(left_a(b) + turn)), Some(low_b)),
                        self.periods,
                    )
                })
                .collect();
            let lower_path = lower.rotated(start, period, 0.0);
            let upper_path = upper_ring.rotated(target, period, turn);
            let cuts = [
                (from.position(), to.position()),
                (
                    from.shifted(turn, 0.0).position(),
                    to.shifted(turn, 0.0).position(),
                ),
            ];
            if blocked(&cuts, &holes, &[&lower_path, &upper_path]) {
                continue;
            }
            let cut = self.cut_points(from.position(), to.position(), 1)?;
            let mut polygon = lower_path;
            polygon.extend(cut.iter().map(|point| point.shifted(turn, 0.0)));
            polygon.extend(upper_path);
            polygon.extend(cut.iter().rev());
            let mut polygons = vec![polygon];
            polygons.extend(holes.into_iter().map(|hole| hole.points));
            return Ok(polygons);
        }
        Err(FaceTriangulationError::NoCut)
    }

    /// 両側に極のある閉じた曲面（球）の全体。極から極への切れ目を入れた長方形にする
    fn whole_with_poles(
        &mut self,
        period: f64,
        (low, high): (f64, f64),
        holes: &[Ring],
    ) -> Result<Vec<Vec<Point>>, FaceTriangulationError> {
        for start in gap_centers(holes, period, |p| p.a) {
            let holes: Vec<Ring> = holes
                .iter()
                .map(|hole| hole.shifted_into((Some(start), None), self.periods))
                .collect();
            let cuts = [
                ((start, low), (start, high)),
                ((start + period, low), (start + period, high)),
            ];
            if blocked(&cuts, &holes, &[]) {
                continue;
            }
            let south = self.add(start, low);
            let north = self.add(start, high);
            let cut = self.cut_points((start, low), (start, high), 1)?;
            let mut polygon = vec![
                Point {
                    vertex: south,
                    a: start,
                    b: low,
                },
                Point {
                    vertex: south,
                    a: start + period,
                    b: low,
                },
            ];
            polygon.extend(cut.iter().map(|point| point.shifted(period, 0.0)));
            polygon.push(Point {
                vertex: north,
                a: start + period,
                b: high,
            });
            polygon.push(Point {
                vertex: north,
                a: start,
                b: high,
            });
            polygon.extend(cut.iter().rev());
            let mut polygons = vec![polygon];
            polygons.extend(holes.into_iter().map(|hole| hole.points));
            return Ok(polygons);
        }
        Err(FaceTriangulationError::NoCut)
    }

    /// u・v の両方に周期のある閉じた曲面（トーラス）の全体。u・v の切れ目を入れた長方形にする
    fn whole_torus(
        &mut self,
        (period_a, period_b): (f64, f64),
        holes: &[Ring],
    ) -> Result<Vec<Vec<Point>>, FaceTriangulationError> {
        let starts_b = gap_centers(holes, period_b, |p| p.b);
        for start_a in gap_centers(holes, period_a, |p| p.a) {
            for &start_b in &starts_b {
                let holes: Vec<Ring> = holes
                    .iter()
                    .map(|hole| hole.shifted_into((Some(start_a), Some(start_b)), self.periods))
                    .collect();
                let (end_a, end_b) = (start_a + period_a, start_b + period_b);
                let cuts = [
                    ((start_a, start_b), (end_a, start_b)),
                    ((start_a, end_b), (end_a, end_b)),
                    ((start_a, start_b), (start_a, end_b)),
                    ((end_a, start_b), (end_a, end_b)),
                ];
                if blocked(&cuts, &holes, &[]) {
                    continue;
                }
                let corner = self.add(start_a, start_b);
                let bottom = self.cut_points((start_a, start_b), (end_a, start_b), 3)?;
                let left = self.cut_points((start_a, start_b), (start_a, end_b), 3)?;
                let at = |a: f64, b: f64| Point {
                    vertex: corner,
                    a,
                    b,
                };
                let mut polygon = vec![at(start_a, start_b)];
                polygon.extend(bottom.iter().copied());
                polygon.push(at(end_a, start_b));
                polygon.extend(left.iter().map(|point| point.shifted(period_a, 0.0)));
                polygon.push(at(end_a, end_b));
                polygon.extend(
                    bottom
                        .iter()
                        .rev()
                        .map(|point| point.shifted(0.0, period_b)),
                );
                polygon.push(at(start_a, end_b));
                polygon.extend(left.iter().rev());
                let mut polygons = vec![polygon];
                polygons.extend(holes.into_iter().map(|hole| hole.points));
                return Ok(polygons);
            }
        }
        Err(FaceTriangulationError::NoCut)
    }

    /// 長さ `delta` を刻み `step` で分ける数（刻みがなければ 0）。頂点の数の上限を超えればエラー
    fn segments(&self, delta: f64, step: Option<f64>) -> Result<usize, FaceTriangulationError> {
        let Some(step) = step else {
            return Ok(0);
        };
        let count = (delta.abs() / step).ceil();
        if count.is_finite() && count <= self.max_points as f64 {
            Ok(count as usize)
        } else {
            Err(FaceTriangulationError::TooManyPoints)
        }
    }

    /// 切れ目 `from`→`to` の、両端を除く点（切れ目の向きの刻みで、少なくとも `min_segments` に分ける）
    fn cut_points(
        &mut self,
        from: (f64, f64),
        to: (f64, f64),
        min_segments: usize,
    ) -> Result<Vec<Point>, FaceTriangulationError> {
        let count = self
            .segments(to.0 - from.0, self.steps.0)?
            .max(self.segments(to.1 - from.1, self.steps.1)?)
            .max(min_segments);
        Ok((1..count)
            .map(|index| {
                let t = index as f64 / count as f64;
                let (a, b) = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
                Point {
                    vertex: self.add(a, b),
                    a,
                    b,
                }
            })
            .collect())
    }

    /// 極の線の上の辺（両端が同じ頂点で、同じ極にある辺）を、a の刻みで分ける（足す点は同じ頂点）
    fn split_pole_lines(&self, polygon: &[Point]) -> Result<Vec<Point>, FaceTriangulationError> {
        let on_pole = |point: Point, pole: Option<f64>| {
            pole.is_some_and(|pole| (point.b - pole).abs() <= POLE_PARAMETER_TOLERANCE)
        };
        let mut result = Vec::with_capacity(polygon.len());
        for (index, &from) in polygon.iter().enumerate() {
            result.push(from);
            let to = polygon[(index + 1) % polygon.len()];
            let same_pole = (on_pole(from, self.poles.0) && on_pole(to, self.poles.0))
                || (on_pole(from, self.poles.1) && on_pole(to, self.poles.1));
            if !same_pole || from.vertex != to.vertex {
                continue;
            }
            let count = self.segments(to.a - from.a, self.steps.0)?;
            result.extend((1..count).map(|index| Point {
                a: from.a + (to.a - from.a) * index as f64 / count as f64,
                ..from
            }));
        }
        Ok(result)
    }
}

/// 周期をまわるループ `ring` の、切れ目の始まりの点の候補（よいものから）。穴があれば、穴のない a の範囲の中ほどに
/// 近い点。穴がなければ、ループの点の間の最も広い a の範囲の中ほどに近い点
fn candidates(ring: &Ring, holes: &[Ring], period: f64) -> Vec<usize> {
    let gaps = if holes.is_empty() {
        let mut values: Vec<f64> = ring.points.iter().map(|p| p.a.rem_euclid(period)).collect();
        values.sort_by(f64::total_cmp);
        let widest = (0..values.len())
            .map(|index| {
                let next = values.get(index + 1).copied().unwrap_or(values[0] + period);
                (values[index], next - values[index])
            })
            .max_by(|x, y| x.1.total_cmp(&y.1))
            .expect("a ring has points");
        vec![(widest.0, widest.1, true)]
    } else {
        free_gaps(holes, period, |p| p.a)
            .into_iter()
            .map(|(start, length)| (start, length, false))
            .collect()
    };
    let mut result = Vec::new();
    for (start, length, everywhere) in gaps {
        let center = start + length / 2.0;
        let mut inside: Vec<(usize, f64)> = ring
            .points
            .iter()
            .enumerate()
            .filter(|(_, point)| {
                let offset = (point.a - start).rem_euclid(period);
                everywhere || (offset > 0.0 && offset < length)
            })
            .map(|(index, point)| {
                let distance = (point.a - center).rem_euclid(period);
                (index, distance.min(period - distance))
            })
            .collect();
        inside.sort_by(|x, y| x.1.total_cmp(&y.1));
        result.extend(inside.into_iter().map(|(index, _)| index));
    }
    result
}

/// 穴のない範囲の中ほど（広いものから）。穴がなければ 0
fn gap_centers(holes: &[Ring], period: f64, of: impl Fn(&Point) -> f64) -> Vec<f64> {
    if holes.is_empty() {
        return vec![0.0];
    }
    free_gaps(holes, period, of)
        .into_iter()
        .map(|(start, length)| start + length / 2.0)
        .collect()
}

/// 穴の範囲（`of` の値の範囲）が覆わない、周期 `period` の中の範囲（始まりと長さ。広いものから）
fn free_gaps(holes: &[Ring], period: f64, of: impl Fn(&Point) -> f64) -> Vec<(f64, f64)> {
    let mut intervals: Vec<(f64, f64)> = Vec::with_capacity(holes.len());
    for hole in holes {
        let (low, high) = hole.range(&of);
        if high - low >= period {
            return Vec::new();
        }
        let start = low.rem_euclid(period);
        let end = start + high - low;
        if end > period {
            // 周期の終わりを越える範囲は、2 つに分ける
            intervals.push((start, period));
            intervals.push((0.0, end - period));
        } else {
            intervals.push((start, end));
        }
    }
    intervals.sort_by(|x, y| x.0.total_cmp(&y.0));
    let mut merged: Vec<(f64, f64)> = Vec::with_capacity(intervals.len());
    for (start, end) in intervals {
        match merged.last_mut() {
            Some(last) if start <= last.1 => last.1 = last.1.max(end),
            _ => merged.push((start, end)),
        }
    }
    let mut gaps: Vec<(f64, f64)> = (0..merged.len())
        .map(|index| {
            let next = merged
                .get(index + 1)
                .map_or(merged[0].0 + period, |next| next.0);
            (merged[index].1, next - merged[index].1)
        })
        .filter(|gap| gap.1 > 0.0)
        .map(|(start, length)| (start.rem_euclid(period), length))
        .collect();
    gaps.sort_by(|x, y| y.1.total_cmp(&x.1));
    gaps
}

/// 切れ目 `cuts` のどれかが、穴 `holes` か、周期をまわるループの点の並び `paths` の辺と交わるか
fn blocked(cuts: &[Segment], holes: &[Ring], paths: &[&Vec<Point>]) -> bool {
    let path_edges = paths.iter().flat_map(|path| {
        path.windows(2)
            .map(|pair| (pair[0].position(), pair[1].position()))
    });
    let edges: Vec<Segment> = holes
        .iter()
        .flat_map(Ring::closed_edges)
        .chain(path_edges)
        .collect();
    cuts.iter()
        .any(|&cut| edges.iter().any(|&edge| segments_cross(cut, edge)))
}

/// 線分 `p`・`q` が、共有する端の点のほかで交わるか接するか
fn segments_cross(p: Segment, q: Segment) -> bool {
    let shared = [(p.0, q.0), (p.0, q.1), (p.1, q.0), (p.1, q.1)]
        .into_iter()
        .filter(|(x, y)| x == y)
        .count();
    if shared >= 2 {
        // 同じ線分（行って戻る辺）
        return p.0 != p.1;
    }
    let orient = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        let coord = |point: (f64, f64)| robust::Coord {
            x: point.0,
            y: point.1,
        };
        let value = robust::orient2d(coord(a), coord(b), coord(c));
        value.partial_cmp(&0.0).map_or(0, |order| order as i32)
    };
    let on_segment = |a: (f64, f64), b: (f64, f64), c: (f64, f64)| {
        c.0 >= a.0.min(b.0) && c.0 <= a.0.max(b.0) && c.1 >= a.1.min(b.1) && c.1 <= a.1.max(b.1)
    };
    if shared == 1 {
        // 端の点を共有する 2 つの辺は、同じ向きに重なるときだけ交わる
        let (common, x, y) = if p.0 == q.0 || p.0 == q.1 {
            (p.0, p.1, if p.0 == q.0 { q.1 } else { q.0 })
        } else {
            (p.1, p.0, if p.1 == q.0 { q.1 } else { q.0 })
        };
        let along = (x.0 - common.0) * (y.0 - common.0) + (x.1 - common.1) * (y.1 - common.1);
        return orient(common, x, y) == 0 && along > 0.0;
    }
    let (o1, o2) = (orient(p.0, p.1, q.0), orient(p.0, p.1, q.1));
    let (o3, o4) = (orient(q.0, q.1, p.0), orient(q.0, q.1, p.1));
    if o1 * o2 < 0 && o3 * o4 < 0 {
        return true;
    }
    (o1 == 0 && on_segment(p.0, p.1, q.0))
        || (o2 == 0 && on_segment(p.0, p.1, q.1))
        || (o3 == 0 && on_segment(q.0, q.1, p.0))
        || (o4 == 0 && on_segment(q.0, q.1, p.1))
}
