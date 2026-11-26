use std::collections::HashMap;
use std::f64::consts::{FRAC_PI_2, TAU};

use analysis::consts::tessellation::{MAX_CHORD_ANGLE_DEGREES, MAX_TRIANGLE_NORMAL_ANGLE_DEGREES};

use super::*;
use crate::Vector3D;

fn settings(chord_tolerance: f64) -> FaceTriangulationSettings {
    FaceTriangulationSettings {
        chord_tolerance,
        max_angle: Angle::from_degrees(MAX_CHORD_ANGLE_DEGREES),
        max_points: 200_000,
        max_passes: 64,
    }
}

/// 半径 `radius` の円の、許容弦誤差と角度の上限の角の刻み
fn angle_step(radius: f64, chord_tolerance: f64, max_angle: Angle<f64>) -> f64 {
    2.0 * (1.0 - chord_tolerance / radius)
        .max((max_angle.to_radians() / 2.0).cos())
        .acos()
}

/// 試験の曲面（点と、曲面までの距離）
trait TestSurface: TriangulationSurface {
    fn distance(&self, point: Point3D<f64>) -> f64;
    /// 点 `point` の近くの、∂u × ∂v の向き
    fn normal(&self, point: Point3D<f64>) -> Vector3D<f64>;
}

struct Plane;

impl TriangulationSurface for Plane {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        Some(Point3D::new(u, v, 0.0))
    }
    fn normal_at(&self, _: f64, _: f64) -> Option<Vector3D<f64>> {
        Some(Vector3D::new(0.0, 0.0, 1.0))
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn steps(&self, _: f64, _: Angle<f64>) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn scale(&self) -> (f64, f64) {
        (1.0, 1.0)
    }
}

impl TestSurface for Plane {
    fn distance(&self, point: Point3D<f64>) -> f64 {
        point.z().abs()
    }
    fn normal(&self, _: Point3D<f64>) -> Vector3D<f64> {
        Vector3D::new(0.0, 0.0, 1.0)
    }
}

struct Cylinder {
    radius: f64,
}

impl TriangulationSurface for Cylinder {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        Some(Point3D::new(
            self.radius * u.cos(),
            self.radius * u.sin(),
            v,
        ))
    }
    fn normal_at(&self, u: f64, _: f64) -> Option<Vector3D<f64>> {
        Some(Vector3D::new(u.cos(), u.sin(), 0.0))
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (Some(TAU), None)
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (Option<f64>, Option<f64>) {
        (
            Some(angle_step(self.radius, chord_tolerance, max_angle)),
            None,
        )
    }
    fn scale(&self) -> (f64, f64) {
        (self.radius, 1.0)
    }
}

impl TestSurface for Cylinder {
    fn distance(&self, point: Point3D<f64>) -> f64 {
        (point.x().hypot(point.y()) - self.radius).abs()
    }
    fn normal(&self, point: Point3D<f64>) -> Vector3D<f64> {
        Vector3D::new(point.x(), point.y(), 0.0)
    }
}

struct Sphere {
    radius: f64,
}

impl TriangulationSurface for Sphere {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        Some(Point3D::new(
            self.radius * v.cos() * u.cos(),
            self.radius * v.cos() * u.sin(),
            self.radius * v.sin(),
        ))
    }
    fn normal_at(&self, u: f64, v: f64) -> Option<Vector3D<f64>> {
        Some(Vector3D::new(v.cos() * u.cos(), v.cos() * u.sin(), v.sin()))
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (Some(TAU), None)
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (Some(-FRAC_PI_2), Some(FRAC_PI_2))
    }
    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (Option<f64>, Option<f64>) {
        let step = angle_step(self.radius, chord_tolerance, max_angle);
        (Some(step), Some(step))
    }
    fn scale(&self) -> (f64, f64) {
        (self.radius, self.radius)
    }
}

impl TestSurface for Sphere {
    fn distance(&self, point: Point3D<f64>) -> f64 {
        (point.distance_to(Point3D::new(0.0, 0.0, 0.0)) - self.radius).abs()
    }
    fn normal(&self, point: Point3D<f64>) -> Vector3D<f64> {
        Vector3D::new(point.x(), point.y(), point.z())
    }
}

struct Torus {
    major: f64,
    minor: f64,
}

impl TriangulationSurface for Torus {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        let radial = self.major + self.minor * v.cos();
        Some(Point3D::new(
            radial * u.cos(),
            radial * u.sin(),
            self.minor * v.sin(),
        ))
    }
    fn normal_at(&self, u: f64, v: f64) -> Option<Vector3D<f64>> {
        Some(Vector3D::new(v.cos() * u.cos(), v.cos() * u.sin(), v.sin()))
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (Some(TAU), Some(TAU))
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (Option<f64>, Option<f64>) {
        (
            Some(angle_step(
                self.major + self.minor,
                chord_tolerance,
                max_angle,
            )),
            Some(angle_step(self.minor, chord_tolerance, max_angle)),
        )
    }
    fn scale(&self) -> (f64, f64) {
        (self.major + self.minor, self.minor)
    }
}

impl TestSurface for Torus {
    fn distance(&self, point: Point3D<f64>) -> f64 {
        ((point.x().hypot(point.y()) - self.major).hypot(point.z()) - self.minor).abs()
    }
    fn normal(&self, point: Point3D<f64>) -> Vector3D<f64> {
        let radial = point.x().hypot(point.y());
        let scale = (radial - self.major) / radial;
        Vector3D::new(point.x() * scale, point.y() * scale, point.z())
    }
}

/// ループを作る（頂点の番号は、`first` から通しで付ける）
fn ring(first: usize, uv: impl IntoIterator<Item = (f64, f64)>) -> Vec<LoopPoint> {
    uv.into_iter()
        .enumerate()
        .map(|(index, (u, v))| LoopPoint {
            vertex: first + index,
            uv: Point2D::new(u, v),
        })
        .collect()
}

/// `from` から `to` への線分を `count` に分けた点（`to` を含まない）
fn line(from: (f64, f64), to: (f64, f64), count: usize) -> impl Iterator<Item = (f64, f64)> {
    (0..count).map(move |index| {
        let t = index as f64 / count as f64;
        (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)
    })
}

/// 結果を確かめる
///
/// - 三角形の内側の点（各三角形で 28 点）と曲面の離れが、許容弦誤差以下
/// - (u, v) で反時計回り（3 次元で、∂u × ∂v の側から見て反時計回り）で、三角形の法線と曲面の法線のなす角が上限
///   （`MAX_TRIANGLE_NORMAL_ANGLE_DEGREES`）以下
/// - 回りの向きがそろう（同じ向きの辺を 2 つの三角形が持たない）
/// - 1 つの三角形だけが持つ辺は、ループの辺（境界の点を結ぶ辺。同じループで行きと帰りに使う継ぎ目の稜線の辺は
///   除く）とちょうど同じ
fn check(
    loops: &[Vec<LoopPoint>],
    result: &FaceTriangulation,
    surface: &impl TestSurface,
    chord_tolerance: f64,
) {
    let boundary: HashMap<usize, Point2D<f64>> = loops
        .iter()
        .flatten()
        .map(|point| (point.vertex, point.uv))
        .collect();
    let point = |vertex: FaceVertex| match vertex {
        FaceVertex::Boundary(index) => {
            let uv = boundary[&index];
            surface.point_at(uv.x(), uv.y()).unwrap()
        }
        FaceVertex::Added(index) => result.added[index].point,
    };
    assert!(!result.triangles.is_empty());
    let mut directed: HashMap<(FaceVertex, FaceVertex), usize> = HashMap::new();
    for &triangle in &result.triangles {
        let [a, b, c] = triangle.map(point);
        for i in 0..=6 {
            for j in 0..=6 - i {
                let (s, t) = (i as f64 / 6.0, j as f64 / 6.0);
                let sample = Point3D::new(
                    a.x() * (1.0 - s - t) + b.x() * s + c.x() * t,
                    a.y() * (1.0 - s - t) + b.y() * s + c.y() * t,
                    a.z() * (1.0 - s - t) + b.z() * s + c.z() * t,
                );
                let distance = surface.distance(sample);
                assert!(
                    distance <= chord_tolerance * (1.0 + 1.0e-9),
                    "{distance} in {triangle:?}"
                );
            }
        }
        let normal = Vector3D::from_points(a, b).cross(Vector3D::from_points(a, c));
        let center = Point3D::new(
            (a.x() + b.x() + c.x()) / 3.0,
            (a.y() + b.y() + c.y()) / 3.0,
            (a.z() + b.z() + c.z()) / 3.0,
        );
        // 極で 3 次元の面積が 0 になる三角形（極と、同じ経線の 2 点）は、向きを確かめない
        let size = a.distance_to(b).max(b.distance_to(c)).max(c.distance_to(a));
        if normal.length() > 1.0e-9 * size * size {
            let surface_normal = surface.normal(center);
            let cosine = normal.dot(surface_normal) / (normal.length() * surface_normal.length());
            assert!(
                cosine >= MAX_TRIANGLE_NORMAL_ANGLE_DEGREES.to_radians().cos(),
                "{} degrees in {triangle:?}",
                cosine.acos().to_degrees()
            );
        }
        for index in 0..3 {
            *directed
                .entry((triangle[index], triangle[(index + 1) % 3]))
                .or_default() += 1;
        }
    }
    assert!(directed.values().all(|&count| count == 1));
    let mut single: Vec<(FaceVertex, FaceVertex)> = directed
        .keys()
        .filter(|(a, b)| !directed.contains_key(&(*b, *a)))
        .copied()
        .collect();
    single.sort();
    let mut expected: Vec<(FaceVertex, FaceVertex)> = loops
        .iter()
        .flat_map(|points| {
            (0..points.len()).map(|index| {
                (
                    FaceVertex::Boundary(points[index].vertex),
                    FaceVertex::Boundary(points[(index + 1) % points.len()].vertex),
                )
            })
        })
        .filter(|(a, b)| a != b)
        .collect();
    let seam: Vec<(FaceVertex, FaceVertex)> = expected
        .iter()
        .filter(|(a, b)| expected.contains(&(*b, *a)))
        .copied()
        .collect();
    expected.retain(|edge| !seam.contains(edge));
    expected.sort();
    assert_eq!(single, expected);
}

fn triangulate(
    loops: &[Vec<LoopPoint>],
    surface: &impl TestSurface,
    chord_tolerance: f64,
) -> FaceTriangulation {
    let settings = settings(chord_tolerance);
    let region = FaceRegion::new(loops, surface, &settings).unwrap();
    assert!(region.crossing_edges(surface).is_empty());
    let result = region.triangulate(surface, &settings).unwrap();
    check(loops, &result, surface, chord_tolerance);
    result
}

/// 長方形のループ（`clockwise` なら時計回り。辺を `count` に分ける）
fn rectangle(
    first: usize,
    low: (f64, f64),
    high: (f64, f64),
    count: usize,
    clockwise: bool,
) -> Vec<LoopPoint> {
    let corners = [low, (high.0, low.1), high, (low.0, high.1)];
    let mut points: Vec<(f64, f64)> = (0..4)
        .flat_map(|index| line(corners[index], corners[(index + 1) % 4], count))
        .collect();
    if clockwise {
        points.reverse();
    }
    ring(first, points)
}

#[test]
fn a_plane_with_many_holes_uses_only_the_boundary_points() {
    let mut loops = vec![rectangle(0, (0.0, 0.0), (10.0, 10.0), 5, false)];
    for row in 0..4 {
        for column in 0..4 {
            let low = (1.0 + 2.3 * column as f64, 1.0 + 2.3 * row as f64);
            loops.push(rectangle(
                1000 * (loops.len()),
                low,
                (low.0 + 1.0, low.1 + 0.5),
                1,
                true,
            ));
        }
    }
    let result = triangulate(&loops, &Plane, 0.01);
    // 平面には内側の点を足さない
    assert!(result.added.is_empty());
    let area: f64 = result
        .triangles
        .iter()
        .map(|triangle| {
            let [p, q, r] = triangle.map(|vertex| match vertex {
                FaceVertex::Boundary(index) => {
                    loops
                        .iter()
                        .flatten()
                        .find(|point| point.vertex == index)
                        .unwrap()
                        .uv
                }
                FaceVertex::Added(_) => unreachable!(),
            });
            ((q.x() - p.x()) * (r.y() - p.y()) - (q.y() - p.y()) * (r.x() - p.x())) / 2.0
        })
        .sum();
    assert!((area - (100.0 - 16.0 * 0.5)).abs() < 1.0e-9);
}

#[test]
fn a_long_thin_polygon_is_triangulated() {
    let loops = vec![rectangle(0, (0.0, 0.0), (100.0, 0.01), 100, false)];
    let result = triangulate(&loops, &Plane, 0.001);
    assert!(result.added.is_empty());
    assert_eq!(result.triangles.len(), 400 - 2);
}

#[test]
fn a_cylinder_band_with_a_hole_is_cut_between_its_loops() {
    let cylinder = Cylinder { radius: 5.0 };
    let tolerance = 0.01;
    let step = angle_step(5.0, tolerance, Angle::from_degrees(MAX_CHORD_ANGLE_DEGREES));
    let lower_count = (TAU / step).ceil() as usize;
    let upper_count = lower_count + 3;
    // 下のループは u が増える向き、上のループは u が減る向き。上のループは周期の分だけずれた値で始める
    let lower = ring(0, line((0.3, 0.0), (0.3 + TAU, 0.0), lower_count));
    let upper = ring(
        1000,
        line((1.0 + 2.0 * TAU, 3.0), (1.0 + TAU, 3.0), upper_count),
    );
    // 穴の u の向きの辺も、u の刻みで分ける
    let hole = rectangle(2000, (2.0, 1.0), (2.2, 2.0), 4, true);
    let loops = vec![lower, upper, hole];
    // 穴がなければ、内側に点を足さない（切れ目は母線なので、切れ目にも点を足さない）
    let result = triangulate(&loops[..2], &cylinder, tolerance);
    assert!(result.added.is_empty(), "{}", result.added.len());
    // 穴の角から斜めに渡る辺は曲面から離れるので、穴のまわりには点を足す
    triangulate(&loops, &cylinder, tolerance);
}

#[test]
fn a_hemisphere_is_cut_to_its_pole() {
    let sphere = Sphere { radius: 10.0 };
    let tolerance = 0.05;
    let loops = vec![ring(0, line((0.0, 0.0), (TAU, 0.0), 72))];
    triangulate(&loops, &sphere, tolerance);
}

#[test]
fn a_spherical_lune_through_both_poles_splits_the_pole_lines() {
    let sphere = Sphere { radius: 10.0 };
    let tolerance = 0.05;
    let (south, north) = (0, 1);
    // u = width の経線を上り、北極で u = 0 へ移り、u = 0 の経線を下り、南極で u = width へ戻る。経線の境界の 2 点と
    // 極を結ぶ、曲面に垂直に立つ三角形を残さない
    for width in [2.0, 3.0, 4.0, 5.0] {
        for divisions in [40, 60] {
            let meridian = |u: f64, from: f64, to: f64| line((u, from), (u, to), divisions).skip(1);
            let mut loop_points = vec![LoopPoint {
                vertex: south,
                uv: Point2D::new(width, -FRAC_PI_2),
            }];
            loop_points.extend(ring(10, meridian(width, -FRAC_PI_2, FRAC_PI_2)));
            loop_points.push(LoopPoint {
                vertex: north,
                uv: Point2D::new(width, FRAC_PI_2),
            });
            loop_points.push(LoopPoint {
                vertex: north,
                uv: Point2D::new(0.0, FRAC_PI_2),
            });
            loop_points.extend(ring(1000, meridian(0.0, FRAC_PI_2, -FRAC_PI_2)));
            loop_points.push(LoopPoint {
                vertex: south,
                uv: Point2D::new(0.0, -FRAC_PI_2),
            });
            triangulate(&[loop_points], &sphere, tolerance);
        }
    }
}

#[test]
fn a_whole_sphere_and_a_whole_torus_are_closed() {
    let sphere = Sphere { radius: 10.0 };
    triangulate(&[], &sphere, 0.05);
    let torus = Torus {
        major: 10.0,
        minor: 3.0,
    };
    let holes = vec![rectangle(0, (1.0, 1.0), (1.2, 1.5), 2, true)];
    triangulate(&holes, &torus, 0.05);
    triangulate(&[], &torus, 0.05);
}

#[test]
fn a_torus_band_around_the_tube_swaps_u_and_v() {
    let torus = Torus {
        major: 10.0,
        minor: 3.0,
    };
    // u = 0 では v が減る向き、u = 1 では v が増える向き（範囲は 0 < u < 1）
    let first = ring(0, line((0.0, TAU), (0.0, 0.0), 40));
    let second = ring(1000, line((1.0, 0.5), (1.0, 0.5 + TAU), 45));
    triangulate(&[first, second], &torus, 0.05);
}

#[test]
fn crossing_edges_are_reported() {
    let outer = rectangle(0, (0.0, 0.0), (10.0, 10.0), 1, false);
    // 外側のループからはみ出す穴
    let hole = rectangle(100, (8.0, 4.0), (12.0, 6.0), 1, true);
    let settings = settings(0.01);
    let region = FaceRegion::new(&[outer, hole], &Plane, &settings).unwrap();
    let crossing = region.crossing_edges(&Plane);
    assert!(crossing.contains(&[1, 2]));
    assert!(crossing.contains(&[100, 101]) || crossing.contains(&[102, 103]));
    assert_eq!(
        region.triangulate(&Plane, &settings).unwrap_err(),
        FaceTriangulationError::Crossing
    );
}

#[test]
fn unsupported_loops_are_refused() {
    let settings = settings(0.01);
    let cylinder = Cylinder { radius: 5.0 };
    // 極のない円柱に、1 つだけまわるループ
    let single = ring(0, line((0.0, 0.0), (TAU, 0.0), 40));
    assert_eq!(
        FaceRegion::new(std::slice::from_ref(&single), &cylinder, &settings).unwrap_err(),
        FaceTriangulationError::UnsupportedLoops
    );
    // 2 回まわるループ
    let twice = ring(0, line((0.0, 0.0), (2.0 * TAU, 1.0), 80));
    assert_eq!(
        FaceRegion::new(&[twice], &cylinder, &settings).unwrap_err(),
        FaceTriangulationError::UnsupportedLoops
    );
    // 穴が u の全体を覆う帯
    let upper = ring(1000, line((TAU, 3.0), (0.0, 3.0), 40));
    let holes = (0..4).map(|index| {
        let low = (index as f64 * 1.6, 0.5 + (index % 2) as f64);
        rectangle(2000 + 10 * index, low, (low.0 + 2.0, low.1 + 0.4), 1, true)
    });
    let mut loops = vec![single, upper];
    loops.extend(holes);
    assert_eq!(
        FaceRegion::new(&loops, &cylinder, &settings).unwrap_err(),
        FaceTriangulationError::NoCut
    );
    // 点が足りない
    let short = ring(0, [(0.0, 0.0), (1.0, 0.0)]);
    assert_eq!(
        FaceRegion::new(&[short], &Plane, &settings).unwrap_err(),
        FaceTriangulationError::InvalidLoops
    );
}

#[test]
fn too_many_points_are_refused() {
    let sphere = Sphere { radius: 10.0 };
    let settings = FaceTriangulationSettings {
        max_points: 100,
        ..settings(0.01)
    };
    // 切れ目と極の線の点だけで上限を超えれば、範囲を作るところで断る
    assert_eq!(
        FaceRegion::new(&[], &sphere, &settings)
            .and_then(|region| region.triangulate(&sphere, &settings))
            .unwrap_err(),
        FaceTriangulationError::TooManyPoints
    );
}

#[test]
fn a_hemisphere_with_a_seam_through_the_pole_goes_once_around_the_pole() {
    // 継ぎ目の稜線を持つ半球: 赤道を u = 0 から 2π へ、継ぎ目を u = 2π で北極へ上り、北極で u = 0 へ移り、継ぎ目を
    // u = 0 で下る（北極では u が 1 周分減る）
    let sphere = Sphere { radius: 10.0 };
    let (corner, north) = (0, 1);
    let mut points = vec![LoopPoint {
        vertex: corner,
        uv: Point2D::new(0.0, 0.0),
    }];
    points.extend(ring(10, line((0.0, 0.0), (TAU, 0.0), 72).skip(1)));
    points.push(LoopPoint {
        vertex: corner,
        uv: Point2D::new(TAU, 0.0),
    });
    let seam: Vec<(f64, f64)> = line((0.0, 0.0), (0.0, FRAC_PI_2), 20).skip(1).collect();
    points.extend(seam.iter().enumerate().map(|(index, &(_, v))| LoopPoint {
        vertex: 100 + index,
        uv: Point2D::new(TAU, v),
    }));
    points.push(LoopPoint {
        vertex: north,
        uv: Point2D::new(TAU, FRAC_PI_2),
    });
    points.push(LoopPoint {
        vertex: north,
        uv: Point2D::new(0.0, FRAC_PI_2),
    });
    points.extend(
        seam.iter()
            .enumerate()
            .rev()
            .map(|(index, &(_, v))| LoopPoint {
                vertex: 100 + index,
                uv: Point2D::new(0.0, v),
            }),
    );
    triangulate(&[points], &sphere, 0.05);
}

#[test]
fn a_hole_past_the_end_of_the_period_is_avoided_by_the_cut() {
    // 球の全体に、u が周期の終わりを越えて 2.7 まで回り込む細長い穴と、その回り込んだ範囲の中の小さな穴がある。
    // 穴のない u の範囲は 2.7 から 3.0 だけ
    let sphere = Sphere { radius: 10.0 };
    let slot = rectangle(0, (3.0, 0.0), (3.0 + TAU - 0.3, 0.1), 40, true);
    let small = rectangle(1000, (0.5, 0.5), (1.0, 0.6), 4, true);
    triangulate(&[slot, small], &sphere, 0.05);
}

/// 刻みに 0 を返す曲面
struct ZeroStep;

impl TriangulationSurface for ZeroStep {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        Plane.point_at(u, v)
    }
    fn normal_at(&self, u: f64, v: f64) -> Option<Vector3D<f64>> {
        Plane.normal_at(u, v)
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn steps(&self, _: f64, _: Angle<f64>) -> (Option<f64>, Option<f64>) {
        (Some(0.0), None)
    }
    fn scale(&self) -> (f64, f64) {
        (1.0, 1.0)
    }
}

#[test]
fn wrong_steps_are_refused() {
    let square = rectangle(0, (0.0, 0.0), (1.0, 1.0), 1, false);
    assert_eq!(
        FaceRegion::new(&[square], &ZeroStep, &settings(0.01)).unwrap_err(),
        FaceTriangulationError::Evaluation
    );
}

#[test]
fn a_coarse_boundary_does_not_keep_adding_points() {
    // 境界の円を、許容弦誤差を少し超える粗さ（弦の離れが許容弦誤差の約 1.2 倍）で分けた円柱の帯。境界の辺は分け
    // られないので、その辺に接する三角形では、その辺の離れを上限にして、点を足し続けない
    let cylinder = Cylinder { radius: 5.0 };
    let lower = ring(0, line((0.0, 0.0), (TAU, 0.0), 45));
    let upper = ring(1000, line((TAU, 3.0), (0.0, 3.0), 45));
    let settings = settings(0.01);
    let region = FaceRegion::new(&[lower, upper], &cylinder, &settings).unwrap();
    region.triangulate(&cylinder, &settings).unwrap();
}

#[test]
fn triangles_tilted_by_a_boundary_chord_do_not_keep_adding_points() {
    // 斜めの平面で切った 4 分の 1 の円柱の面（下の境界は、平面との交わりの楕円を許容弦誤差で分けた点。上の境界は円）。
    // 下の境界の弦は 3 次元で曲面から離れるので、弦のそばの点と弦でできる三角形は傾く。点を足しても傾きは直らない
    // （足した点は弦により近い三角形を作る）ので、その傾きでは点を足し続けない
    let cylinder = Cylinder { radius: 3.048 };
    let cases: [(f64, &[(f64, f64)]); 2] = [
        (
            0.05,
            &[
                (5.497939, -1.457706),
                (5.531306, -1.446156),
                (5.832058, -1.270045),
                (6.132809, -1.177851),
                (6.433561, -1.177851),
                (6.734313, -1.270045),
                (7.035065, -1.447644),
                (7.068431, -1.456217),
            ],
        ),
        (
            0.02,
            &[
                (5.497939, -1.457706),
                (5.531306, -1.446156),
                (5.746129, -1.312359),
                (5.960951, -1.219591),
                (6.175774, -1.172116),
                (6.390597, -1.172116),
                (6.605419, -1.219591),
                (6.820242, -1.312359),
                (7.035065, -1.447644),
                (7.068431, -1.456217),
            ],
        ),
    ];
    for (tolerance, bottom) in cases {
        let count = circle_points(cylinder.radius, tolerance).div_ceil(4);
        let top = line((7.068431, 115.860515), (5.497939, 115.860515), count)
            .chain([(5.497939, 115.860515)]);
        let loops = vec![ring(0, bottom.iter().copied().chain(top))];
        let settings = settings(tolerance);
        let region = FaceRegion::new(&loops, &cylinder, &settings).unwrap();
        assert!(region.crossing_edges(&cylinder).is_empty());
        let result = region.triangulate(&cylinder, &settings).unwrap();
        assert!(result.added.len() < 2_000, "{}", result.added.len());
        // 傾きで分けない三角形も、内側で許容弦誤差を守る（境界の弦の離れは許容弦誤差より小さい）
        let point = |vertex: FaceVertex| match vertex {
            FaceVertex::Boundary(index) => {
                let uv = loops[0][index].uv;
                cylinder.point_at(uv.x(), uv.y()).unwrap()
            }
            FaceVertex::Added(index) => result.added[index].point,
        };
        for triangle in &result.triangles {
            let [a, b, c] = triangle.map(point);
            for i in 0..=6 {
                for j in 0..=6 - i {
                    let (s, t) = (i as f64 / 6.0, j as f64 / 6.0);
                    let sample = Point3D::new(
                        a.x() * (1.0 - s - t) + b.x() * s + c.x() * t,
                        a.y() * (1.0 - s - t) + b.y() * s + c.y() * t,
                        a.z() * (1.0 - s - t) + b.z() * s + c.z() * t,
                    );
                    let distance = cylinder.distance(sample);
                    assert!(
                        distance <= tolerance * (1.0 + 1.0e-9),
                        "{distance} in {triangle:?}"
                    );
                }
            }
        }
    }
}

/// v の線が u によって横にずれる平面（点は (u, v − `sag` × 4 u (1 − u), 0)。v が一定の線は、3 次元で放物線になる）。
/// u・v の両方に刻み `step` を持ち、内側に格子の点を置く
struct ShearedPlane {
    sag: f64,
    step: f64,
}

impl TriangulationSurface for ShearedPlane {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        Some(Point3D::new(u, v - self.sag * 4.0 * u * (1.0 - u), 0.0))
    }
    fn normal_at(&self, _: f64, _: f64) -> Option<Vector3D<f64>> {
        Some(Vector3D::new(0.0, 0.0, 1.0))
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }
    fn steps(&self, _: f64, _: Angle<f64>) -> (Option<f64>, Option<f64>) {
        (Some(self.step), Some(self.step))
    }
    fn scale(&self) -> (f64, f64) {
        (1.0, 1.0)
    }
}

#[test]
fn triangles_flipped_beside_a_boundary_chord_do_not_keep_adding_points() {
    // 3 次元でまっすぐな境界の辺（下の辺。両端の 2 点だけ）が、(u, v) では曲がる曲面。(u, v) の線分を曲面に写した
    // 線は、3 次元の弦の横（曲面に沿う向き）に 0.03 ふくらむので、そのふくらみの内側にある格子の点と弦でできる
    // 三角形は、3 次元で裏返る。点を足しても直らない（足した点もふくらみの内側に入る）ので、その三角形と、弦に
    // 沿って並ぶ、高さが許容弦誤差より小さい三角形には、点を足し続けない
    let surface = ShearedPlane {
        sag: 0.03,
        step: 0.02,
    };
    let loops = vec![ring(0, [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])];
    let settings = settings(0.01);
    let region = FaceRegion::new(&loops, &surface, &settings).unwrap();
    assert!(region.crossing_edges(&surface).is_empty());
    let grid = (1.0 / (surface.step * 0.95 * std::f64::consts::FRAC_1_SQRT_2)).ceil() as usize;
    let result = region.triangulate(&surface, &settings).unwrap();
    assert!(
        result.added.len() < 2 * grid * grid,
        "{}",
        result.added.len()
    );
}

/// 頂点が原点にある円錐（v の位置の半径は v × 0.5。刻みは面の中の最も大きい半径 `max_radius` で決める）
struct Cone {
    max_radius: f64,
}

const CONE_SLOPE: f64 = 0.5;

impl TriangulationSurface for Cone {
    fn point_at(&self, u: f64, v: f64) -> Option<Point3D<f64>> {
        let radius = CONE_SLOPE * v;
        Some(Point3D::new(radius * u.cos(), radius * u.sin(), v))
    }
    fn normal_at(&self, u: f64, v: f64) -> Option<Vector3D<f64>> {
        let radius = CONE_SLOPE * v;
        Some(Vector3D::new(
            radius * u.cos(),
            radius * u.sin(),
            -CONE_SLOPE * radius,
        ))
    }
    fn periods(&self) -> (Option<f64>, Option<f64>) {
        (Some(TAU), None)
    }
    fn poles(&self) -> (Option<f64>, Option<f64>) {
        (Some(0.0), None)
    }
    fn steps(&self, chord_tolerance: f64, max_angle: Angle<f64>) -> (Option<f64>, Option<f64>) {
        (
            Some(angle_step(self.max_radius, chord_tolerance, max_angle)),
            None,
        )
    }
    fn scale(&self) -> (f64, f64) {
        (self.max_radius, (1.0 + CONE_SLOPE * CONE_SLOPE).sqrt())
    }
}

impl TestSurface for Cone {
    fn distance(&self, point: Point3D<f64>) -> f64 {
        (point.x().hypot(point.y()) - CONE_SLOPE * point.z()).abs()
            / (1.0 + CONE_SLOPE * CONE_SLOPE).sqrt()
    }
    fn normal(&self, point: Point3D<f64>) -> Vector3D<f64> {
        let radius = point.x().hypot(point.y());
        Vector3D::new(point.x(), point.y(), -CONE_SLOPE * radius)
    }
}

/// 半径 `radius` の円を、許容弦誤差と角度の上限で等しく分ける数
fn circle_points(radius: f64, chord_tolerance: f64) -> usize {
    let step = angle_step(
        radius,
        chord_tolerance,
        Angle::from_degrees(MAX_CHORD_ANGLE_DEGREES),
    );
    (TAU / step).ceil() as usize
}

#[test]
fn a_small_circle_near_a_pole_is_triangulated_without_flipped_triangles() {
    // 極の近くの緯線の境界は、自分の小さな半径で分けるので、辺の u の幅が球の u の刻みより広い
    let sphere = Sphere { radius: 10.0 };
    let tolerance = 0.005;
    for latitude in [1.45, 1.5, 1.52, 1.55] {
        let count = circle_points(10.0 * f64::cos(latitude), tolerance);
        // 北極までの面（u が増える向き）と、南極までの面（u が減る向き）
        let north = ring(0, line((0.0, latitude), (TAU, latitude), count));
        triangulate(&[north], &sphere, tolerance);
        let south = ring(0, line((TAU, -latitude), (0.0, -latitude), count));
        triangulate(&[south], &sphere, tolerance);
    }
}

#[test]
fn a_cone_frustum_with_a_small_top_circle_is_triangulated() {
    let cone = Cone { max_radius: 2.5 };
    let tolerance = 0.001;
    for small in [0.2, 0.4, 1.0, 2.0] {
        // 小さい円（v = small）は u が増える向き、大きい円（v = 5）は u が減る向き
        let lower = ring(
            0,
            line(
                (0.0, small),
                (TAU, small),
                circle_points(CONE_SLOPE * small, tolerance),
            ),
        );
        let upper = ring(
            1000,
            line((TAU, 5.0), (0.0, 5.0), circle_points(2.5, tolerance)),
        );
        triangulate(&[lower, upper], &cone, tolerance);
    }
}

#[test]
fn a_whole_sphere_and_a_whole_torus_meet_machining_tolerances_with_the_grid_alone() {
    // 工程の精度（0.001 と、細かい仕上げの 0.0001）で、内側に点を足さずに格子だけで許容弦誤差を守る。0.0001 は、
    // 大きさを 1/10 にした曲面で同じ細かさを確かめる（許容弦誤差と半径の比が同じなら、点の数も同じ）
    let grid_only = |tolerance: f64| FaceTriangulationSettings {
        max_passes: 0,
        ..settings(tolerance)
    };
    for (scale, tolerance) in [(1.0, 0.001), (0.1, 0.0001)] {
        let sphere = Sphere {
            radius: 10.0 * scale,
        };
        let torus = Torus {
            major: 20.0 * scale,
            minor: 5.0 * scale,
        };
        let settings = grid_only(tolerance);
        let result = FaceRegion::new(&[], &sphere, &settings)
            .unwrap()
            .triangulate(&sphere, &settings)
            .unwrap();
        check(&[], &result, &sphere, tolerance);
        let result = FaceRegion::new(&[], &torus, &settings)
            .unwrap()
            .triangulate(&torus, &settings)
            .unwrap();
        check(&[], &result, &torus, tolerance);
    }
}

#[test]
fn a_torus_with_a_hole_converges_at_a_machining_tolerance() {
    // 穴のまわりの、格子の点を置かない所に足す点が、隣の三角形の細分化を呼び続けない
    let torus = Torus {
        major: 2.0,
        minor: 0.5,
    };
    let holes = vec![rectangle(0, (1.0, 1.0), (1.2, 1.5), 50, true)];
    triangulate(&holes, &torus, 0.0001);
}
