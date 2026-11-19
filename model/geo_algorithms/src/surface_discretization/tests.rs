use geo_contracts::{
    NurbsSurface3DConstructor, NurbsSurface3DProperties, TriangleMesh3DProperties,
};

use super::*;

const CHORD_TOLERANCE: f64 = 0.001;

/// 2 次 × 2 次の曲面（制御点は 3 × 3。x・y の刻みは 20）
fn biquadratic(heights: [[f64; 3]; 3], u_positions: [f64; 3]) -> NurbsSurface3D<f64> {
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    let control_points: Vec<Vec<Point3D<f64>>> = (0..3)
        .map(|i| {
            (0..3)
                .map(|j| Point3D::new(u_positions[i], 20.0 * j as f64, heights[i][j]))
                .collect()
        })
        .collect();
    NurbsSurface3D::new(2, 2, knots.clone(), knots, &control_points, None).unwrap()
}

fn dome() -> NurbsSurface3D<f64> {
    biquadratic(
        [[80.0, 80.0, 80.0], [80.0, 100.0, 80.0], [80.0, 80.0, 80.0]],
        [0.0, 20.0, 40.0],
    )
}

fn saddle() -> NurbsSurface3D<f64> {
    biquadratic(
        [[80.0, 60.0, 80.0], [100.0, 80.0, 100.0], [80.0, 60.0, 80.0]],
        [0.0, 20.0, 40.0],
    )
}

/// u の刻みが偏ったドーム（パラメータの速さが場所で大きく変わる）
fn distorted_dome() -> NurbsSurface3D<f64> {
    biquadratic(
        [[80.0, 80.0, 80.0], [80.0, 100.0, 80.0], [80.0, 80.0, 80.0]],
        [0.0, 2.0, 40.0],
    )
}

/// 4 隅の高さが互い違いの面（u・v の向きの辺は直線で、マスの内側だけがねじれる）
fn twist() -> NurbsSurface3D<f64> {
    let knots = vec![0.0, 0.0, 1.0, 1.0];
    NurbsSurface3D::new(
        1,
        1,
        knots.clone(),
        knots,
        &[
            vec![Point3D::new(0.0, 0.0, 80.0), Point3D::new(0.0, 40.0, 100.0)],
            vec![
                Point3D::new(40.0, 0.0, 100.0),
                Point3D::new(40.0, 40.0, 80.0),
            ],
        ],
        None,
    )
    .unwrap()
}

/// 2 次 × 1 次の面で、v が最大の端の辺だけが u の向きに曲がる面（向かいの辺は直線で、その間は直線で結ぶ）
fn raised_at_v_max_edge() -> NurbsSurface3D<f64> {
    NurbsSurface3D::new(
        2,
        1,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        &[
            vec![Point3D::new(0.0, 0.0, 80.0), Point3D::new(0.0, 40.0, 80.0)],
            vec![
                Point3D::new(20.0, 0.0, 80.0),
                Point3D::new(20.0, 40.0, 100.0),
            ],
            vec![
                Point3D::new(40.0, 0.0, 80.0),
                Point3D::new(40.0, 40.0, 80.0),
            ],
        ],
        None,
    )
    .unwrap()
}

/// 1 次 × 2 次の面で、u が最大の端の辺だけが v の向きに曲がる面
fn raised_at_u_max_edge() -> NurbsSurface3D<f64> {
    NurbsSurface3D::new(
        1,
        2,
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        &[
            vec![
                Point3D::new(0.0, 0.0, 80.0),
                Point3D::new(0.0, 20.0, 80.0),
                Point3D::new(0.0, 40.0, 80.0),
            ],
            vec![
                Point3D::new(40.0, 0.0, 80.0),
                Point3D::new(40.0, 20.0, 100.0),
                Point3D::new(40.0, 40.0, 80.0),
            ],
        ],
        None,
    )
    .unwrap()
}

/// 3 次 × 3 次の平面に、内側の制御点 1 つだけを持ち上げた局所的な盛り上がり
fn cubic_bump() -> NurbsSurface3D<f64> {
    let knots = vec![0.0, 0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0, 1.0];
    let count = knots.len() - 4;
    let control_points: Vec<Vec<Point3D<f64>>> = (0..count)
        .map(|i| {
            (0..count)
                .map(|j| {
                    let z = if i == 3 && j == 4 { 95.0 } else { 80.0 };
                    Point3D::new(6.0 * i as f64, 6.0 * j as f64, z)
                })
                .collect()
        })
        .collect();
    NurbsSurface3D::new(3, 3, knots.clone(), knots, &control_points, None).unwrap()
}

/// u の 5 次の多項式 k u (u - 1/4) (u - 1/2) (u - 3/4) (u - 1) の高さで波打つ面（v には直線）
///
/// 波の節が u = 0, 1/4, 1/2, 3/4, 1 にあり、2 分割を重ねた格子の点では平面と同じ高さになる。
fn wavy() -> NurbsSurface3D<f64> {
    const DEGREE: usize = 5;
    let scale = 400.0;
    // u (u - 1/4) (u - 1/2) (u - 3/4) (u - 1) の u^0..u^5 の係数
    let mut power = vec![1.0];
    for root in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let mut next = vec![0.0; power.len() + 1];
        for (k, &a) in power.iter().enumerate() {
            next[k + 1] += a;
            next[k] -= root * a;
        }
        power = next;
    }
    let binomial = |n: usize, k: usize| -> f64 {
        (0..k).fold(1.0, |acc, i| acc * (n - i) as f64 / (i + 1) as f64)
    };
    let heights: Vec<f64> = (0..=DEGREE)
        .map(|i| {
            80.0 + scale
                * (0..=i)
                    .map(|k| binomial(i, k) / binomial(DEGREE, k) * power[k])
                    .sum::<f64>()
        })
        .collect();
    let control_points: Vec<Vec<Point3D<f64>>> = heights
        .iter()
        .enumerate()
        .map(|(i, &z)| {
            let x = 40.0 * i as f64 / DEGREE as f64;
            vec![Point3D::new(x, 0.0, z), Point3D::new(x, 40.0, z)]
        })
        .collect();
    let mut u_knots = vec![0.0; DEGREE + 1];
    u_knots.extend(vec![1.0; DEGREE + 1]);
    NurbsSurface3D::new(
        DEGREE,
        1,
        u_knots,
        vec![0.0, 0.0, 1.0, 1.0],
        &control_points,
        None,
    )
    .unwrap()
}

fn named_surfaces() -> Vec<(&'static str, NurbsSurface3D<f64>)> {
    vec![
        ("dome", dome()),
        ("saddle", saddle()),
        ("distorted dome", distorted_dome()),
        ("twist", twist()),
        ("raised at v max edge", raised_at_v_max_edge()),
        ("raised at u max edge", raised_at_u_max_edge()),
        ("cubic bump", cubic_bump()),
        ("wavy", wavy()),
    ]
}

/// 初期分割を浅くし、収束に再分割が必要な条件にする
fn limits(max_refinement_iterations: u32) -> TessellationLimits {
    TessellationLimits {
        max_subdivisions: 1,
        max_refinement_iterations,
        max_vertices_per_surface: 1_000_000,
    }
}

/// 制限なしで収束に必要な再分割回数
fn required_refinements() -> u32 {
    let required = (0..64)
        .find(|&max| converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(max)).is_ok())
        .expect("dome should converge within 64 refinements");
    assert!(required >= 2, "test needs a case requiring refinement");
    required
}

#[test]
fn converges_when_limit_equals_required_refinements() {
    let required = required_refinements();
    assert!(converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(required)).is_ok());
}

#[test]
fn fails_without_exceeding_limit_when_one_short() {
    let required = required_refinements();
    let failure =
        converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(required - 1)).unwrap_err();
    // 上限回数ちょうどで打ち切り、上限を超える再分割は行わない
    assert_eq!(failure.refinements, required - 1);
}

#[test]
fn zero_limit_verifies_initial_grid_only() {
    let failure = converge_param_grid_counted(&dome(), CHORD_TOLERANCE, limits(0)).unwrap_err();
    assert_eq!(failure.refinements, 0);
}

#[test]
fn tessellate_surfaces_reports_convergence_failure_at_boundary() {
    let required = required_refinements();
    assert!(tessellate_surfaces(&[dome()], CHORD_TOLERANCE, limits(required)).is_ok());
    assert!(matches!(
        tessellate_surfaces(&[dome()], CHORD_TOLERANCE, limits(required - 1)).unwrap_err(),
        SurfaceTessellationError::ConvergenceFailure {
            surface_index: 0,
            ..
        }
    ));
}

#[test]
fn vertex_normals_are_the_surface_normals() {
    let surface = dome();
    let mesh = tessellate_surfaces(
        std::slice::from_ref(&surface),
        0.1,
        TessellationLimits::default(),
    )
    .unwrap();
    let normals = mesh
        .normals()
        .expect("tessellated meshes have vertex normals");
    assert_eq!(normals.len(), mesh.vertex_count());
    for normal in normals {
        assert!((normal.length() - 1.0).abs() < 1e-9);
    }
    // 三角形の向き（頂点の並び）と頂点法線の向きがそろう
    for &[a, b, c] in mesh.indices() {
        let vertices = mesh.vertices();
        let face = Vector3D::from_points(vertices[a], vertices[b])
            .cross(Vector3D::from_points(vertices[a], vertices[c]));
        assert!(face.dot(normals[a]) > 0.0);
    }
    // ドームの頂上（u = v = 0.5）の法線は +Z か -Z
    let top = surface.normal_at(0.5, 0.5).unwrap().to_vector();
    assert!(top.x().abs() < 1e-9 && top.y().abs() < 1e-9);
}

/// 三角形の辺を分ける数。重心座標が (i, j, k) / n の点（n = 6 で頂点・辺・内側の 28 点。重心を含む）をとる
const SAMPLE_DIVISIONS: usize = 6;

/// 曲面への最短距離を探す刻みを半分にする回数
const PROJECTION_HALVINGS: u32 = 40;

/// 点 `point` から曲面への最短距離を、パラメータ `start` の近くで探す。
///
/// 刻み `step` で u・v の 4 方向を試し、近づかなくなったら刻みを半分にする。
fn distance_to_surface(
    surface: &NurbsSurface3D<f64>,
    point: Point3D<f64>,
    start: (f64, f64),
    step: f64,
) -> f64 {
    let ((u_min, u_max), (v_min, v_max)) = surface.parameter_domain();
    let squared = |u: f64, v: f64| distance_squared(evaluate(surface, u, v), point);
    let (mut u, mut v) = start;
    let mut best = squared(u, v);
    let mut step = step;
    for _ in 0..PROJECTION_HALVINGS {
        let mut moved = true;
        while moved {
            moved = false;
            for (du, dv) in [(step, 0.0), (-step, 0.0), (0.0, step), (0.0, -step)] {
                let (cu, cv) = ((u + du).clamp(u_min, u_max), (v + dv).clamp(v_min, v_max));
                let candidate = squared(cu, cv);
                if candidate < best {
                    (u, v, best) = (cu, cv, candidate);
                    moved = true;
                }
            }
        }
        step *= 0.5;
    }
    best.sqrt()
}

/// 三角形の頂点・辺・内側の点から曲面への最短距離の、許容弦誤差に対する比の最大
///
/// 三角形の点と、同じ重心座標の (u, v) の曲面の点との距離は、最短距離の上限になる。
/// 上限が許容弦誤差を超える点だけ、最短距離を探す。
fn max_deviation_ratio(
    surface: &NurbsSurface3D<f64>,
    chord_tolerance: f64,
    limits: TessellationLimits,
) -> f64 {
    let mesh = tessellate_surfaces(std::slice::from_ref(surface), chord_tolerance, limits)
        .expect("surface should converge");
    let (u_params, v_params) = converge_param_grid(surface, chord_tolerance, limits).unwrap();
    assert_eq!(mesh.vertex_count(), u_params.len() * v_params.len());
    let v_count = v_params.len();
    let parameter = |index: usize| (u_params[index / v_count], v_params[index % v_count]);
    let vertices = mesh.vertices();

    let mut max_ratio: f64 = 0.0;
    for &triangle in mesh.indices() {
        let corners = triangle.map(|index| vertices[index]);
        let parameters = triangle.map(parameter);
        let step = parameters
            .iter()
            .flat_map(|a| {
                parameters
                    .iter()
                    .map(move |b| (a.0 - b.0).abs().max((a.1 - b.1).abs()))
            })
            .fold(0.0, f64::max);
        for i in 0..=SAMPLE_DIVISIONS {
            for j in 0..=SAMPLE_DIVISIONS - i {
                let k = SAMPLE_DIVISIONS - i - j;
                let weights = [i, j, k].map(|w| w as f64 / SAMPLE_DIVISIONS as f64);
                let combine = |values: [f64; 3]| -> f64 {
                    values.iter().zip(weights).map(|(value, w)| value * w).sum()
                };
                let point = Point3D::new(
                    combine(corners.map(|c| c.x())),
                    combine(corners.map(|c| c.y())),
                    combine(corners.map(|c| c.z())),
                );
                let start = (
                    combine(parameters.map(|p| p.0)),
                    combine(parameters.map(|p| p.1)),
                );
                let mut deviation = distance(evaluate(surface, start.0, start.1), point);
                if deviation > chord_tolerance {
                    deviation = distance_to_surface(surface, point, start, step);
                }
                max_ratio = max_ratio.max(deviation / chord_tolerance);
            }
        }
    }
    max_ratio
}

#[test]
fn triangles_stay_within_chord_tolerance_including_interior() {
    for (name, surface) in named_surfaces() {
        for chord_tolerance in [0.1, 0.01] {
            let ratio =
                max_deviation_ratio(&surface, chord_tolerance, TessellationLimits::default());
            assert!(
                ratio <= 1.0,
                "{name}: deviation is {ratio} times the chord tolerance {chord_tolerance}"
            );
        }
    }
}

/// 1 マスの格子で、u・v が最大の端の辺だけが許容弦誤差を超える面を分ける向きに選ぶ
///
/// 端の辺の中点のずれは 10、マスの中心のずれは 5 で、許容弦誤差 7 は端の辺だけが超える。
#[test]
fn cells_check_the_edges_at_the_parameter_maximum() {
    let tolerance = 7.0;
    let (u_split, v_split) =
        find_cells_over_tolerance(&raised_at_v_max_edge(), &[0.0, 1.0], &[0.0, 1.0], tolerance);
    assert_eq!((u_split, v_split), (vec![true], vec![false]));
    let (u_split, v_split) =
        find_cells_over_tolerance(&raised_at_u_max_edge(), &[0.0, 1.0], &[0.0, 1.0], tolerance);
    assert_eq!((u_split, v_split), (vec![false], vec![true]));
}

/// 2 分割を重ねた格子の点だけでは平らに見える波を、ノットの区間を次数の数に分けた点で見つける
#[test]
fn initial_grid_divides_knot_spans_by_degree() {
    let (u_params, _) = converge_param_grid(&wavy(), 0.1, TessellationLimits::default()).unwrap();
    for k in 0..=5 {
        let u = f64::from(k) / 5.0;
        assert!(
            u_params.contains(&u),
            "u = {u} is missing from {u_params:?}"
        );
    }
    assert!(u_params.len() > 6);
}
