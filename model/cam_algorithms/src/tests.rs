use analysis::consts::test_constants::TOLERANCE_F64 as EPS;
use cam_core::{
    CoordinateFrame, CuttingDirection, LengthUnit, MachiningBoundary, OperationSpec,
    RectangleBoundary, ScanlineCutDirection, ScanlineParams, ScanlineToolRange, Tool,
};
use geo_algorithms::{
    NurbsSurface3D, Point3D, TessellationLimits, TriangleMesh3D, tessellate_surfaces,
};
use geo_contracts::{
    NurbsSurface3DConstructor, NurbsSurface3DEvaluation, TriangleMesh3DConstructor,
    TriangleMesh3DProperties, default_distance_tolerance,
};

use crate::{
    CamSolverError, CamSolverInput, CutterShape, InverseOffsetShape, ScanlineInputError,
    SolverGeometry, build_inverse_offset_shape, solve_toolpath,
};

fn mesh(vertices: &[(f64, f64, f64)], indices: &[[usize; 3]]) -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vertices
            .iter()
            .map(|&(x, y, z)| Point3D::new(x, y, z))
            .collect(),
        indices.to_vec(),
    )
    .unwrap()
}

fn flat_square(z: f64) -> TriangleMesh3D<f64> {
    mesh(
        &[
            (0.0, 0.0, z),
            (10.0, 0.0, z),
            (10.0, 10.0, z),
            (0.0, 10.0, z),
        ],
        &[[0, 1, 2], [0, 2, 3]],
    )
}

/// 中央が盛り上がった双二次 NURBS 曲面（XY: 0..40、Z: 80..90 付近）
fn dome_surface() -> NurbsSurface3D<f64> {
    let control_points = vec![
        vec![
            Point3D::new(0.0, 0.0, 80.0),
            Point3D::new(0.0, 20.0, 80.0),
            Point3D::new(0.0, 40.0, 80.0),
        ],
        vec![
            Point3D::new(20.0, 0.0, 80.0),
            Point3D::new(20.0, 20.0, 100.0),
            Point3D::new(20.0, 40.0, 80.0),
        ],
        vec![
            Point3D::new(40.0, 0.0, 80.0),
            Point3D::new(40.0, 20.0, 80.0),
            Point3D::new(40.0, 40.0, 80.0),
        ],
    ];
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    NurbsSurface3D::new(2, 2, knots.clone(), knots, &control_points, None).unwrap()
}

fn ball(radius: f64) -> CutterShape {
    CutterShape::Ball { radius }
}

fn flat(radius: f64) -> CutterShape {
    CutterShape::Flat { radius }
}

/// ボール工具中心高さ（= 先端高さ + r）
fn ball_center(cutter: &InverseOffsetShape, x: f64, y: f64) -> Option<f64> {
    cutter
        .tip_height_at(x, y)
        .map(|z| z + cutter.shape().radius())
}

fn solver_input(geometry: SolverGeometry) -> CamSolverInput {
    CamSolverInput {
        operation_id: "op-scan-1".to_string(),
        tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
        geometry,
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover: 2.0,
            feed_rate: 1200.0,
            tool_range: ScanlineToolRange::default(),
            cut_direction: ScanlineCutDirection::default(),
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.01,
        tessellation_limits: TessellationLimits::default(),
        boundary: None,
    }
}

/// 工具の中心の移動範囲を `tool_range` にした入力
fn with_tool_range(mut input: CamSolverInput, tool_range: ScanlineToolRange) -> CamSolverInput {
    let OperationSpec::Scanline(params) = &mut input.operation;
    params.tool_range = tool_range;
    input
}

/// 送り方 `cut_direction` で平らな正方形をスキャン加工した、各パス（Y の小さい順）の Y と、X 正方向に送るか
fn feed_directions(cut_direction: ScanlineCutDirection) -> (CuttingDirection, Vec<(f64, bool)>) {
    let mut input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    let OperationSpec::Scanline(params) = &mut input.operation;
    params.cut_direction = cut_direction;
    let toolpath = solve_toolpath(&input).unwrap();
    let passes = toolpath
        .contour_levels
        .iter()
        .map(|level| {
            // 段（形状の縁で工具が転がり落ちる所の、CL の面のほぼ鉛直な部分）の鉛直な線分は、送る向きを持たない
            let mut horizontal = level
                .segments
                .iter()
                .filter(|segment| segment.end_point().x() != segment.start.x());
            let first = horizontal.next().unwrap();
            let forward = first.end_point().x() > first.start.x();
            // パスの中のすべての線分が同じ向き
            assert!(
                horizontal.all(|segment| (segment.end_point().x() > segment.start.x()) == forward)
            );
            (first.start.y(), forward)
        })
        .collect();
    (toolpath.cutting_direction, passes)
}

#[test]
fn scanline_cut_direction_decides_the_feed_direction_of_each_line() {
    let (recorded, down) = feed_directions(ScanlineCutDirection::DownCut);
    assert_eq!(recorded, CuttingDirection::Down);
    assert!(down.len() > 2);
    assert!(down.iter().all(|&(_, forward)| !forward));

    let (recorded, up) = feed_directions(ScanlineCutDirection::UpCut);
    assert_eq!(recorded, CuttingDirection::Up);
    assert!(up.iter().all(|&(_, forward)| forward));

    // ジグザグはダウンカット（X 負方向）から始め、ラインごとに入れ替える
    let (recorded, zigzag) = feed_directions(ScanlineCutDirection::ZigZag);
    assert_eq!(recorded, CuttingDirection::Mixed);
    assert!(zigzag.windows(2).all(|pair| pair[0].0 < pair[1].0));
    for (line, &(_, forward)) in zigzag.iter().enumerate() {
        assert_eq!(forward, line % 2 == 1, "line {line}");
    }
}

#[test]
fn face_offset_places_ball_center_one_radius_above_flat_face() {
    let cutter = InverseOffsetShape::new(&flat_square(0.0), ball(3.0)).unwrap();
    let center = ball_center(&cutter, 5.0, 5.0).unwrap();
    assert!((center - 3.0).abs() < EPS);
    assert!(cutter.tip_point_at(5.0, 5.0).unwrap().z().abs() < EPS);
}

#[test]
fn face_offset_follows_inclined_plane() {
    // z = 0.5 x の斜面: 中心高さ = 0.5 x + r / cos(theta)
    let slope = 0.5_f64;
    let inclined = mesh(
        &[
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 10.0 * slope),
            (10.0, 10.0, 10.0 * slope),
            (0.0, 10.0, 0.0),
        ],
        &[[0, 1, 2], [0, 2, 3]],
    );
    let r = 2.0;
    let cutter = InverseOffsetShape::new(&inclined, ball(r)).unwrap();
    let expected = slope * 5.0 + r * (1.0 + slope * slope).sqrt();
    assert!((ball_center(&cutter, 5.0, 5.0).unwrap() - expected).abs() < EPS);
}

#[test]
fn edge_offset_is_cylinder_outside_face() {
    // 辺 y=0（z=0）から水平距離 b の位置: 中心高さ = sqrt(r^2 - b^2)
    let cutter = InverseOffsetShape::new(&flat_square(0.0), ball(3.0)).unwrap();
    let b = 1.5;
    let expected = (9.0_f64 - b * b).sqrt();
    assert!((ball_center(&cutter, 5.0, -b).unwrap() - expected).abs() < EPS);
}

#[test]
fn vertex_offset_is_sphere_outside_edges() {
    let cutter = InverseOffsetShape::new(&flat_square(0.0), ball(3.0)).unwrap();
    let (dx, dy) = (-1.0, -2.0);
    let expected = (9.0_f64 - dx * dx - dy * dy).sqrt();
    assert!((ball_center(&cutter, dx, dy).unwrap() - expected).abs() < EPS);
}

#[test]
fn no_contact_outside_tool_radius() {
    for shape in [ball(3.0), flat(3.0)] {
        let cutter = InverseOffsetShape::new(&flat_square(0.0), shape).unwrap();
        assert!(cutter.tip_height_at(-3.5, 5.0).is_none());
    }
}

#[test]
fn ball_center_does_not_gouge_tessellated_dome() {
    let limits = TessellationLimits::default();
    let dome = tessellate_surfaces(&[dome_surface()], 0.01, limits).unwrap();
    let r = 3.0;
    let cutter = InverseOffsetShape::new(&dome, ball(r)).unwrap();

    for &(x, y) in &[
        (20.0, 20.0),
        (5.0, 7.0),
        (33.0, 12.0),
        (0.0, 40.0),
        (-2.0, 20.0),
    ] {
        let center_z = ball_center(&cutter, x, y).unwrap();
        let center = [x, y, center_z];
        let min_distance = dome
            .indices()
            .iter()
            .map(|tri| {
                let [a, b, c] = tri.map(|vi| {
                    let p = dome.vertices()[vi];
                    [p.x(), p.y(), p.z()]
                });
                distance_to_triangle(center, a, b, c)
            })
            .fold(f64::INFINITY, f64::min);
        // 球は形状に食い込まず（距離 >= r）、かつ接触している（距離 == r）
        assert!(
            (min_distance - r).abs() < default_distance_tolerance::<f64>(),
            "ball at ({x}, {y}) is not tangent: distance={min_distance}"
        );
    }
}

#[test]
fn flat_face_contact_on_horizontal_face_is_face_height() {
    let cutter = InverseOffsetShape::new(&flat_square(2.0), flat(3.0)).unwrap();
    assert!((cutter.tip_height_at(5.0, 5.0).unwrap() - 2.0).abs() < EPS);
}

#[test]
fn flat_disc_reaches_edge_outside_face() {
    // 円板が辺 x=0 に水平距離 2 (< r=3) で掛かる
    let cutter = InverseOffsetShape::new(&flat_square(2.0), flat(3.0)).unwrap();
    assert!((cutter.tip_height_at(-2.0, 5.0).unwrap() - 2.0).abs() < EPS);
}

#[test]
fn flat_face_contact_is_on_disc_rim_uphill() {
    // z = 0.5 x の斜面: 円板の上り側円周（x + r）で接触する
    let slope = 0.5_f64;
    let inclined = mesh(
        &[
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 10.0 * slope),
            (10.0, 10.0, 10.0 * slope),
            (0.0, 10.0, 0.0),
        ],
        &[[0, 1, 2], [0, 2, 3]],
    );
    let cutter = InverseOffsetShape::new(&inclined, flat(2.0)).unwrap();
    assert!((cutter.tip_height_at(5.0, 5.0).unwrap() - slope * 7.0).abs() < EPS);
    // 形状外側 (x = -1) でも円周が斜面に掛かる
    assert!((cutter.tip_height_at(-1.0, 5.0).unwrap() - slope * 1.0).abs() < EPS);
}

#[test]
fn flat_edge_sweep_takes_highest_point_within_disc() {
    // 傾斜辺 (0,0,0)-(10,0,5) から水平距離 2 の位置: 円板内区間 [5-√5, 5+√5] の上端で接触
    let triangle = mesh(
        &[(0.0, 0.0, 0.0), (10.0, 0.0, 5.0), (0.0, 10.0, 0.0)],
        &[[0, 1, 2]],
    );
    let cutter = InverseOffsetShape::new(&triangle, flat(3.0)).unwrap();
    let expected = 0.5 * (5.0 + 5.0_f64.sqrt());
    assert!((cutter.tip_height_at(5.0, -2.0).unwrap() - expected).abs() < EPS);
}

#[test]
fn flat_vertex_disc_contacts_apex() {
    let pyramid = mesh(
        &[
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 0.0),
            (10.0, 10.0, 0.0),
            (0.0, 10.0, 0.0),
            (5.0, 5.0, 4.0),
        ],
        &[[0, 1, 4], [1, 2, 4], [2, 3, 4], [3, 0, 4]],
    );
    let cutter = InverseOffsetShape::new(&pyramid, flat(3.0)).unwrap();
    assert!((cutter.tip_height_at(7.0, 5.0).unwrap() - 4.0).abs() < EPS);
}

#[test]
fn flat_bottom_does_not_gouge_and_touches_dome() {
    let chord_tolerance = 0.01;
    let surface = dome_surface();
    let dome = tessellate_surfaces(
        std::slice::from_ref(&surface),
        chord_tolerance,
        TessellationLimits::default(),
    )
    .unwrap();
    let r = 3.0;
    let cutter = InverseOffsetShape::new(&dome, flat(r)).unwrap();

    // 曲面を密にサンプルし、円板内の曲面最高点が先端高さに一致する
    // （食い込みなし・接触あり）ことを弦誤差の範囲で確認する
    let samples: Vec<[f64; 3]> = (0..=200)
        .flat_map(|i| (0..=200).map(move |j| (i as f64 / 200.0, j as f64 / 200.0)))
        .map(|(u, v)| {
            let p = surface
                .point_at_uv(u, v)
                .expect("sample parameter should be within the surface domain");
            [p.x(), p.y(), p.z()]
        })
        .collect();
    for &(x, y) in &[(20.0, 20.0), (5.0, 7.0), (33.0, 12.0), (-2.0, 20.0)] {
        let tip = cutter.tip_height_at(x, y).unwrap();
        let highest = samples
            .iter()
            .filter(|p| (p[0] - x).powi(2) + (p[1] - y).powi(2) <= r * r)
            .map(|p| p[2])
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (highest - tip).abs() <= 2.0 * chord_tolerance,
            "flat tip at ({x}, {y}) = {tip}, highest surface point in disc = {highest}"
        );
    }
}

#[test]
fn solve_toolpath_supports_flat_end_mill() {
    let mut input = with_tool_range(
        solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()])),
        ScanlineToolRange::GeometryExtent,
    );
    input.tool = Tool::flat_end_mill("EM6".to_string(), 6.0, 30.0);
    let toolpath = solve_toolpath(&input).unwrap();

    assert_eq!(toolpath.tool_id, "EM6");
    assert_eq!(toolpath.level_count(), 21);
    for level in &toolpath.contour_levels {
        for segment in level.cutting_segments() {
            let z = segment.end_point().z();
            assert!(
                (80.0 - EPS..=90.0 + EPS).contains(&z),
                "tip z out of range: {z}"
            );
        }
    }
}

/// 点と三角形の最短距離（Ericson, Real-Time Collision Detection 5.1.5）
fn distance_to_triangle(p: [f64; 3], a: [f64; 3], b: [f64; 3], c: [f64; 3]) -> f64 {
    let sub = |u: [f64; 3], v: [f64; 3]| [u[0] - v[0], u[1] - v[1], u[2] - v[2]];
    let dot = |u: [f64; 3], v: [f64; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let at = |o: [f64; 3], u: [f64; 3], s: f64| [o[0] + u[0] * s, o[1] + u[1] * s, o[2] + u[2] * s];

    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    let closest = if d1 <= 0.0 && d2 <= 0.0 {
        a
    } else {
        let bp = sub(p, b);
        let (d3, d4) = (dot(ab, bp), dot(ac, bp));
        let cp = sub(p, c);
        let (d5, d6) = (dot(ab, cp), dot(ac, cp));
        let vc = d1 * d4 - d3 * d2;
        let vb = d5 * d2 - d1 * d6;
        let va = d3 * d6 - d5 * d4;
        if d3 >= 0.0 && d4 <= d3 {
            b
        } else if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
            at(a, ab, d1 / (d1 - d3))
        } else if d6 >= 0.0 && d5 <= d6 {
            c
        } else if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
            at(a, ac, d2 / (d2 - d6))
        } else if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
            at(b, sub(c, b), (d4 - d3) / ((d4 - d3) + (d5 - d6)))
        } else {
            let denom = 1.0 / (va + vb + vc);
            at(at(a, ab, vb * denom), ac, vc * denom)
        }
    };
    let d = sub(p, closest);
    dot(d, d).sqrt()
}

#[test]
fn tessellation_respects_chord_tolerance() {
    let surface = dome_surface();
    let tolerance = 0.01;
    let dome = tessellate_surfaces(
        std::slice::from_ref(&surface),
        tolerance,
        TessellationLimits::default(),
    )
    .unwrap();
    assert!(dome.triangle_count() > 2);

    // 極小半径の逆オフセット形状でメッシュ高さを取り、曲面上の点との鉛直差を検証する。
    // 曲面の最大傾斜は 1 未満のため、法線方向の弦誤差 tol は鉛直差 2*tol 以内に収まる。
    let probe = InverseOffsetShape::new(&dome, ball(1e-9)).unwrap();
    for i in 0..=20 {
        for j in 0..=20 {
            let p = surface
                .point_at_uv(i as f64 / 20.0, j as f64 / 20.0)
                .expect("sample parameter should be within the surface domain");
            let Some(tip) = probe.tip_point_at(p.x(), p.y()) else {
                panic!("no contact at i={i} j={j} ({:?}, {:?})", p.x(), p.y())
            };
            let mesh_z = tip.z();
            assert!(
                (mesh_z - p.z()).abs() <= 2.0 * tolerance,
                "chord error at ({}, {}): {}",
                p.x(),
                p.y(),
                (mesh_z - p.z()).abs()
            );
        }
    }
}

#[test]
fn solve_toolpath_generates_scanline_passes_for_nurbs_surface() {
    let input = with_tool_range(
        solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()])),
        ScanlineToolRange::GeometryExtent,
    );
    let toolpath = solve_toolpath(&input).unwrap();

    assert_eq!(toolpath.tool_id, "BEM6");
    // 0..40 を stepover 2.0 で走査 → 21 ライン
    assert_eq!(toolpath.level_count(), 21);
    assert!(toolpath.total_cutting_length() > 0.0);

    for level in &toolpath.contour_levels {
        let cutting = level.cutting_segments();
        assert!(!cutting.is_empty());
        for segment in cutting {
            let z = segment.end_point().z();
            // 工具先端は曲面の最低点（80）より下に潜らず、頂点（~90）を大きく超えない
            assert!(
                (80.0 - EPS..=90.0 + EPS).contains(&z),
                "tip z out of range: {z}"
            );
        }
        // パスの間のエアカットは入れない
        assert!(level.segments.iter().all(|segment| segment.is_cutting()));
    }
}

#[test]
fn invalid_input_for_radius_end_mill() {
    let mut input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    input.tool = Tool::radius_end_mill("REM6R1".to_string(), 6.0, 1.0, 30.0);
    let error = solve_toolpath(&input).unwrap_err();
    assert_eq!(error.code(), "invalid_input");
}

#[test]
fn invalid_input_for_stepover_larger_than_diameter() {
    let mut input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    input.operation = OperationSpec::Scanline(ScanlineParams {
        stepover: 7.0,
        feed_rate: 1200.0,
        tool_range: ScanlineToolRange::default(),
        cut_direction: ScanlineCutDirection::default(),
    });
    let error = solve_toolpath(&input).unwrap_err();
    assert_eq!(error.code(), "invalid_input");
    assert_eq!(
        error,
        CamSolverError::InvalidScanline(ScanlineInputError::StepoverExceedsToolDiameter)
    );
}

#[test]
fn invalid_input_for_empty_surface_set() {
    let input = solver_input(SolverGeometry::NurbsSurfaceSet(Vec::new()));
    assert_eq!(solve_toolpath(&input).unwrap_err().code(), "invalid_input");
}

#[test]
fn no_solution_for_degenerate_geometry() {
    let collinear = mesh(
        &[(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (2.0, 0.0, 0.0)],
        &[[0, 1, 2]],
    );
    let input = solver_input(SolverGeometry::TriangleMesh(collinear));
    let error = solve_toolpath(&input).unwrap_err();
    assert!(matches!(error, CamSolverError::NoSolution(_)));
}

#[test]
fn convergence_failure_when_chord_tolerance_unreachable() {
    let mut input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
    input.chord_tolerance = 1e-9;
    input.tessellation_limits = TessellationLimits {
        max_subdivisions: 2,
        max_refinement_iterations: 1,
        max_vertices_per_surface: 1_000_000,
    };
    let error = solve_toolpath(&input).unwrap_err();
    assert_eq!(error.code(), "convergence_failure");
}

#[test]
fn cl_grid_covers_margin_and_matches_inverse_offset_shape() {
    let cutter = InverseOffsetShape::new(&flat_square(0.0), ball(1.0)).unwrap();
    let grid = crate::sample_cl_grid(&cutter, 0.5, 1.0).unwrap();

    // XY 範囲 0..10 を margin 1 で拡張 → -1..11 を 0.5 間隔で 25 点
    assert_eq!((grid.columns, grid.rows), (25, 25));
    assert_eq!(grid.xy(0, 0), [-1.0, -1.0]);
    for row in 0..grid.rows {
        for column in 0..grid.columns {
            let [x, y] = grid.xy(column, row);
            assert_eq!(grid.tip_height(column, row), cutter.tip_height_at(x, y));
        }
    }
    // 外周角 (-1, -1) は頂点 (0,0) から √2 > r で非接触
    assert!(grid.tip_height(0, 0).is_none());
}

#[test]
fn cl_grid_rejects_invalid_pitch() {
    let cutter = InverseOffsetShape::new(&flat_square(0.0), flat(1.0)).unwrap();
    assert_eq!(
        crate::sample_cl_grid(&cutter, 0.0, 0.0).unwrap_err().code(),
        "invalid_input"
    );
}

#[test]
fn cl_grid_boundary_points_lie_on_contact_limit() {
    // 平面正方形の角 (0,0) 周辺: 接触境界は角頂点から水平距離 r の円弧、辺外側は距離 r の直線
    let r = 1.0;
    let cutter = InverseOffsetShape::new(&flat_square(0.0), flat(r)).unwrap();
    let grid = crate::sample_cl_grid(&cutter, 0.3, r).unwrap();
    let tolerance = default_distance_tolerance::<f64>();

    let boundaries: Vec<[f64; 3]> = grid
        .x_edge_boundaries
        .iter()
        .chain(grid.y_edge_boundaries.iter())
        .flatten()
        .copied()
        .collect();
    assert!(!boundaries.is_empty());
    for [x, y, z] in boundaries {
        // 形状 [0,10]^2 から境界点までの水平距離は r（二分法の許容誤差内）
        let dx = (0.0 - x).max(x - 10.0).max(0.0);
        let dy = (0.0 - y).max(y - 10.0).max(0.0);
        let distance = (dx * dx + dy * dy).sqrt();
        assert!(
            (distance - r).abs() <= tolerance,
            "boundary ({x}, {y}) distance {distance}"
        );
        assert!(z.abs() < EPS);
    }
}

fn rectangle(rect_min: [f64; 2], rect_max: [f64; 2]) -> Option<MachiningBoundary> {
    Some(MachiningBoundary::Rectangle(RectangleBoundary {
        rect_min,
        rect_max,
    }))
}

#[test]
fn rectangle_boundary_limits_scanline_region() {
    let mut input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
    input.boundary = rectangle([10.0, 10.0], [20.0, 30.0]);
    let toolpath = solve_toolpath(&input).unwrap();

    // Y 10..30 を stepover 2.0 で走査 → 11 ライン
    assert_eq!(toolpath.level_count(), 11);
    for level in &toolpath.contour_levels {
        for segment in level.cutting_segments() {
            let p = segment.end_point();
            assert!((10.0..=20.0).contains(&p.x()) && (10.0..=30.0).contains(&p.y()));
        }
    }
}

#[test]
fn rectangle_within_tool_reach_outside_shape_is_accepted() {
    // 形状外側だが工具半径以内（x = -2..0）は接触し得るため受理する
    let mut input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
    input.boundary = rectangle([-2.0, 0.0], [0.0, 40.0]);
    assert!(solve_toolpath(&input).is_ok());
}

#[test]
fn rectangle_outside_tool_reach_is_operation_boundary_out_of_domain() {
    let mut input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
    input.boundary = rectangle([100.0, 100.0], [110.0, 110.0]);
    let error = solve_toolpath(&input).unwrap_err();
    assert_eq!(error.code(), "invalid_input");
    assert!(
        error
            .reason()
            .starts_with("operation_boundary_out_of_domain")
    );
}

#[test]
fn degenerate_rectangle_in_solver_input_is_invalid_input() {
    let mut input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    input.boundary = rectangle([5.0, 5.0], [5.0, 8.0]);
    let error = solve_toolpath(&input).unwrap_err();
    assert_eq!(error.code(), "invalid_input");
    assert!(error.reason().starts_with("invalid_rectangle_boundary"));
}

/// 精度プロファイル別の離散化・経路生成負荷の計測（要素絞り込みによる高速化の判断材料）
///
/// `cargo test -p cam_algorithms --release -- --ignored --nocapture profile_load`
#[test]
#[ignore]
fn profile_load_measurement() {
    use cam_core::ToleranceProfile;
    use std::time::Instant;

    for profile in [ToleranceProfile::PressRough, ToleranceProfile::MoldFinish] {
        let tolerance = profile.tolerance_mm();
        let started = Instant::now();
        let result =
            tessellate_surfaces(&[dome_surface()], tolerance, TessellationLimits::default());
        let tessellation_ms = started.elapsed().as_millis();
        match result {
            Ok(mesh) => {
                let mut input = solver_input(SolverGeometry::TriangleMesh(mesh.clone()));
                input.chord_tolerance = tolerance;
                let started = Instant::now();
                let toolpath = solve_toolpath(&input);
                println!(
                    "{}: tolerance={} vertices={} triangles={} tessellation={}ms solve={}ms ok={}",
                    profile.token(),
                    tolerance,
                    mesh.vertex_count(),
                    mesh.triangle_count(),
                    tessellation_ms,
                    started.elapsed().as_millis(),
                    toolpath.is_ok()
                );
            }
            Err(error) => println!(
                "{}: tolerance={} tessellation={}ms failed: {}",
                profile.token(),
                tolerance,
                tessellation_ms,
                error
            ),
        }
    }
}

#[test]
fn oversized_rectangle_is_clipped_to_tool_reach() {
    // 形状 0..40 に対し ±1e6 の矩形: 走査は reach（-3..43）にクリップされ、点数は形状規模に収まる
    let mut input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
    input.boundary = rectangle([-1.0e6, -1.0e6], [1.0e6, 1.0e6]);
    let toolpath = solve_toolpath(&input).unwrap();

    let r = input.tool.radius();
    // Y -3..43 を stepover 2.0 で走査 → 24 ライン（うち外周で非接触のラインは生成されない）
    assert!(toolpath.level_count() <= 24);
    for level in &toolpath.contour_levels {
        for segment in level.cutting_segments() {
            let p = segment.end_point();
            assert!((-r..=40.0 + r).contains(&p.x()) && (-r..=40.0 + r).contains(&p.y()));
        }
    }
}

#[test]
fn rectangle_touching_reach_is_out_of_domain_not_no_solution() {
    // reach は形状 0..40 ± 工具半径 3 → -3..43。辺・角で接するだけの矩形は面積を持たない
    let degenerate_cases = [
        ([43.0, 0.0], [50.0, 40.0]),    // 右辺で接する
        ([-10.0, -10.0], [-3.0, -3.0]), // 角で接する
        ([0.0, 43.0], [40.0, 60.0]),    // 上辺で接する
    ];
    for (rect_min, rect_max) in degenerate_cases {
        let mut input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
        input.boundary = rectangle(rect_min, rect_max);
        let error = solve_toolpath(&input).unwrap_err();
        assert_eq!(error.code(), "invalid_input", "{rect_min:?}-{rect_max:?}");
        assert!(
            error
                .reason()
                .starts_with("operation_boundary_out_of_domain"),
            "{rect_min:?}-{rect_max:?}: {}",
            error.reason()
        );
    }
}

/// u 方向のみ曲がる 40 x 400mm の樋形状（v 方向は直線）
fn gutter_surface() -> NurbsSurface3D<f64> {
    let control_points = vec![
        vec![Point3D::new(0.0, 0.0, 80.0), Point3D::new(0.0, 400.0, 80.0)],
        vec![
            Point3D::new(20.0, 0.0, 100.0),
            Point3D::new(20.0, 400.0, 100.0),
        ],
        vec![
            Point3D::new(40.0, 0.0, 80.0),
            Point3D::new(40.0, 400.0, 80.0),
        ],
    ];
    NurbsSurface3D::new(
        2,
        1,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        &control_points,
        None,
    )
    .unwrap()
}

/// パラメータ空間を u/v 等分した参照メッシュ
fn uniform_grid_mesh(surface: &NurbsSurface3D<f64>, nu: usize, nv: usize) -> TriangleMesh3D<f64> {
    let mut vertices = Vec::with_capacity((nu + 1) * (nv + 1));
    for i in 0..=nu {
        for j in 0..=nv {
            let p = surface
                .point_at_uv(i as f64 / nu as f64, j as f64 / nv as f64)
                .expect("sample parameter should be within the surface domain");
            vertices.push(p);
        }
    }
    let mut indices = Vec::with_capacity(nu * nv * 2);
    for i in 0..nu {
        for j in 0..nv {
            let p00 = i * (nv + 1) + j;
            let (p01, p10) = (p00 + 1, p00 + nv + 1);
            indices.push([p00, p10, p10 + 1]);
            indices.push([p00, p10 + 1, p01]);
        }
    }
    TriangleMesh3D::new(vertices, indices).unwrap()
}

#[test]
fn triangle_aspect_ratio_is_one_for_equilateral() {
    let a = Point3D::new(0.0, 0.0, 0.0);
    let b = Point3D::new(1.0, 0.0, 0.0);
    let c = Point3D::new(0.5, 3.0_f64.sqrt() / 2.0, 0.0);
    assert!((crate::triangle_aspect_ratio(a, b, c) - 1.0).abs() < EPS);

    // 直角二等辺三角形: √2 / (2√3 × (2 - √2) / 2)
    let right = crate::triangle_aspect_ratio(a, b, Point3D::new(0.0, 1.0, 0.0));
    let expected = 2.0_f64.sqrt() / (3.0_f64.sqrt() * (2.0 - 2.0_f64.sqrt()));
    assert!((right - expected).abs() < EPS);

    let collinear = crate::triangle_aspect_ratio(a, b, Point3D::new(2.0, 0.0, 0.0));
    assert!(collinear.is_infinite());
}

#[test]
fn slivers_do_not_change_cutter_location() {
    // 逆オフセットは要素ごとに厳密なため、細長三角形のメッシュでも CL は
    // 形状の整った参照メッシュ（1mm 程度の格子）と弦誤差内で一致する
    let chord_tolerance = 0.001;
    let sliver = tessellate_surfaces(
        &[gutter_surface()],
        chord_tolerance,
        TessellationLimits::default(),
    )
    .unwrap();
    let longest_aspect = sliver
        .indices()
        .iter()
        .map(|tri| {
            let [a, b, c] = tri.map(|i| sliver.vertices()[i]);
            crate::triangle_aspect_ratio(a, b, c)
        })
        .fold(0.0_f64, f64::max);
    assert!(
        longest_aspect > 100.0,
        "fixture must contain slivers: {longest_aspect}"
    );

    let reference = uniform_grid_mesh(&gutter_surface(), 256, 400);
    for shape in [ball(3.0), flat(3.0)] {
        let a = InverseOffsetShape::new(&sliver, shape).unwrap();
        let b = InverseOffsetShape::new(&reference, shape).unwrap();
        for &(x, y) in &[
            (1.0, 5.0),
            (12.5, 100.0),
            (20.0, 200.0),
            (33.3, 399.0),
            (-2.0, 50.0),
        ] {
            let (za, zb) = (
                a.tip_height_at(x, y).unwrap(),
                b.tip_height_at(x, y).unwrap(),
            );
            assert!(
                (za - zb).abs() <= 2.0 * chord_tolerance,
                "{shape:?} at ({x}, {y}): {za} vs {zb}"
            );
        }
    }
}

#[test]
fn triangle_aspect_ratio_is_stable_for_thin_triangles() {
    // 底辺 L・高さ h の二等辺三角形の厳密値と比較する（細長いほど Heron の公式は桁落ちする）
    for (length, height) in [(400.0_f64, 1.0e-3_f64), (400.0, 1.0e-6), (1.0e-3, 1.0e-9)] {
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(length, 0.0, 0.0);
        let c = Point3D::new(0.5 * length, height, 0.0);
        let side = (0.25 * length * length + height * height).sqrt();
        let semi_perimeter = 0.5 * (length + 2.0 * side);
        let inradius = 0.5 * length * height / semi_perimeter;
        let expected = length.max(side) / (2.0 * 3.0_f64.sqrt() * inradius);

        let ratio = crate::triangle_aspect_ratio(a, b, c);
        assert!(ratio.is_finite(), "L={length} h={height}");
        assert!(
            ((ratio - expected) / expected).abs() < 1.0e-9,
            "L={length} h={height}: {ratio} vs {expected}"
        );
    }
}

#[test]
fn triangle_aspect_ratio_is_infinite_for_coincident_points() {
    let p = Point3D::new(1.0, 2.0, 3.0);
    assert!(crate::triangle_aspect_ratio(p, p, p).is_infinite());
}

#[test]
fn solver_geometry_bounds_enclose_the_mesh_and_the_surface_control_points() {
    let triangle = mesh(
        &[(0.0, 0.0, 1.0), (4.0, 0.0, 2.0), (0.0, 3.0, 5.0)],
        &[[0, 1, 2]],
    );
    let aabb = SolverGeometry::TriangleMesh(triangle).aabb().unwrap();
    assert_eq!(aabb.min(), Point3D::new(0.0, 0.0, 1.0));
    assert_eq!(aabb.max(), Point3D::new(4.0, 3.0, 5.0));

    let surfaces = SolverGeometry::NurbsSurfaceSet(vec![dome_surface(), dome_surface()]);
    let aabb = surfaces.aabb().unwrap();
    let surface = dome_surface();
    for i in 0..=10 {
        for j in 0..=10 {
            let point = surface
                .point_at_uv(f64::from(i) / 10.0, f64::from(j) / 10.0)
                .unwrap();
            // 曲面の評価の丸め誤差の範囲で、境界ボックスに含まれる
            let tolerance = default_distance_tolerance::<f64>();
            let inside = |value: f64, low: f64, high: f64| {
                low - tolerance <= value && value <= high + tolerance
            };
            assert!(
                inside(point.x(), aabb.min().x(), aabb.max().x())
                    && inside(point.y(), aabb.min().y(), aabb.max().y())
                    && inside(point.z(), aabb.min().z(), aabb.max().z()),
                "{point:?}"
            );
        }
    }
    assert!(SolverGeometry::NurbsSurfaceSet(Vec::new()).aabb().is_none());
}

/// 切削の区間の端点（ツールパスの全レベル）
fn cutting_points(toolpath: &cam_core::ToolPath<f64>) -> Vec<Point3D<f64>> {
    toolpath
        .contour_levels
        .iter()
        .flat_map(|level| level.cutting_segments())
        .flat_map(|segment| [segment.start, segment.end_point()])
        .collect()
}

#[test]
fn ball_end_mill_moves_one_radius_beyond_the_geometry_by_default() {
    // 10 × 10 の平坦面（z = 0）を、直径 6 のボールエンドミルで走査する
    let input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    let extended = cutting_points(&solve_toolpath(&input).unwrap());
    let within = cutting_points(
        &solve_toolpath(&with_tool_range(
            input.clone(),
            ScanlineToolRange::GeometryExtent,
        ))
        .unwrap(),
    );
    let r = 3.0;
    let extent = |points: &[Point3D<f64>]| {
        points.iter().fold(
            [
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ],
            |[x0, x1, y0, y1], p| [x0.min(p.x()), x1.max(p.x()), y0.min(p.y()), y1.max(p.y())],
        )
    };

    // 既定では、工具の中心が平坦面の外側に半径分出る
    let [x0, x1, y0, y1] = extent(&extended);
    for (actual, expected) in [(x0, -r), (x1, 10.0 + r), (y0, -r), (y1, 10.0 + r)] {
        assert!((actual - expected).abs() < EPS, "{actual} != {expected}");
    }
    // 形状の範囲では、工具の中心は平坦面の範囲に収まる
    let [x0, x1, y0, y1] = extent(&within);
    for (actual, expected) in [(x0, 0.0), (x1, 10.0), (y0, 0.0), (y1, 10.0)] {
        assert!((actual - expected).abs() < EPS, "{actual} != {expected}");
    }
    // 平坦面の上では工具の先端は面の高さ、外側では端の稜線を転がって下がる（面より上には出ない）
    for p in &extended {
        let inside = (0.0..=10.0).contains(&p.x()) && (0.0..=10.0).contains(&p.y());
        if inside {
            assert!(p.z().abs() < EPS, "{p:?}");
        } else {
            assert!(p.z() < EPS && p.z() >= -r - EPS, "{p:?}");
        }
    }
}

/// x < 5 が高さ 4、x >= 5 が高さ 0 の段（x = 5 に鉛直な壁）
fn step_block() -> TriangleMesh3D<f64> {
    mesh(
        &[
            (0.0, 0.0, 4.0),
            (5.0, 0.0, 4.0),
            (5.0, 10.0, 4.0),
            (0.0, 10.0, 4.0),
            (5.0, 0.0, 0.0),
            (10.0, 0.0, 0.0),
            (10.0, 10.0, 0.0),
            (5.0, 10.0, 0.0),
        ],
        &[
            [0, 1, 2],
            [0, 2, 3],
            [1, 4, 7],
            [1, 7, 2],
            [4, 5, 6],
            [4, 6, 7],
        ],
    )
}

/// 格子の点の高さ 3 sin(x / 2) + 2 cos(y / 3) の、尾根と谷のある地形（0..12）
fn ridged_terrain() -> TriangleMesh3D<f64> {
    let count = 12;
    let mut vertices = Vec::new();
    for j in 0..=count {
        for i in 0..=count {
            let (x, y) = (i as f64, j as f64);
            vertices.push((x, y, 3.0 * (x / 2.0).sin() + 2.0 * (y / 3.0).cos()));
        }
    }
    let at = |i: usize, j: usize| j * (count + 1) + i;
    let mut triangles = Vec::new();
    for j in 0..count {
        for i in 0..count {
            triangles.push([at(i, j), at(i + 1, j), at(i + 1, j + 1)]);
            triangles.push([at(i, j), at(i + 1, j + 1), at(i, j + 1)]);
        }
    }
    mesh(&vertices, &triangles)
}

/// 経路の線分を細かく分けた点で、経路の高さが CL の面の高さからトレランスを超えて下を通らない
fn assert_path_does_not_gouge(input: &CamSolverInput) {
    let cutter = build_inverse_offset_shape(input).unwrap();
    let toolpath = solve_toolpath(input).unwrap();
    let tolerance = input.chord_tolerance;
    for level in &toolpath.contour_levels {
        for segment in level.cutting_segments() {
            let (start, end) = (segment.start, segment.end_point());
            for k in 0..=10 {
                let t = k as f64 / 10.0;
                let x = start.x() + t * (end.x() - start.x());
                let z = start.z() + t * (end.z() - start.z());
                if let Some(tip) = cutter.tip_height_at(x, start.y()) {
                    assert!(
                        z >= tip - tolerance - EPS,
                        "y = {}, x = {x}: path {z} below the CL surface {tip}",
                        start.y()
                    );
                }
            }
        }
    }
}

#[test]
fn scanline_paths_do_not_pass_below_the_cl_surface_beyond_the_tolerance() {
    let geometries = [
        SolverGeometry::TriangleMesh(step_block()),
        SolverGeometry::TriangleMesh(ridged_terrain()),
        SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]),
    ];
    for geometry in geometries {
        for tool in [
            Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
            Tool::flat_end_mill("EM6".to_string(), 6.0, 30.0),
        ] {
            let mut input = solver_input(geometry.clone());
            input.tool = tool;
            let OperationSpec::Scanline(params) = &mut input.operation;
            params.stepover = 1.3;
            assert_path_does_not_gouge(&input);
        }
    }
}

#[test]
fn steps_of_the_cl_surface_become_vertical_moves_on_the_high_side() {
    // ボールの中心は壁の上の辺を回って x = 8 まで下り、そこで床の高さへ跳ぶ。経路は高い側の高さのまま進んでから下りる
    let input = solver_input(SolverGeometry::TriangleMesh(step_block()));
    let cutter = build_inverse_offset_shape(&input).unwrap();
    let toolpath = solve_toolpath(&input).unwrap();
    let vertical: Vec<_> = toolpath
        .contour_levels
        .iter()
        .flat_map(|level| level.cutting_segments())
        .filter(|segment| segment.start.x() == segment.end_point().x())
        .collect();
    assert!(!vertical.is_empty());
    for segment in vertical {
        let (start, end) = (segment.start, segment.end_point());
        // 段の下の端は、CL の面より下にならない
        let tip = cutter.tip_height_at(start.x(), start.y()).unwrap();
        assert!(start.z().min(end.z()) >= tip - input.chord_tolerance - EPS);
    }
}

#[test]
fn points_are_not_added_on_flat_faces() {
    // 平坦面の上だけを送る（工具の中心は形状の範囲）と、点は工具の半径以下の等間隔のまま（0..10 を r = 3 以下の
    // 4 等分で 5 点）
    let input = with_tool_range(
        solver_input(SolverGeometry::TriangleMesh(flat_square(0.0))),
        ScanlineToolRange::GeometryExtent,
    );
    let toolpath = solve_toolpath(&input).unwrap();
    for level in &toolpath.contour_levels {
        assert_eq!(level.cutting_segments().len(), 4);
    }
}

#[test]
fn pass_ends_are_at_the_contact_boundaries_within_the_tolerance() {
    // X に 0..10 と 20..30 の 2 つの平坦面（Y は 0..10）。ボール r = 3 の工具が接する区間は、面からの Y の距離 d で
    // 面の X の範囲を √(r² − d²) だけ広げた区間で、間（13..17 付近）は接しない。点の並び（r 以下の間隔）は区間の
    // 端に重ならない
    let squares = mesh(
        &[
            (0.0, 0.0, 0.0),
            (10.0, 0.0, 0.0),
            (10.0, 10.0, 0.0),
            (0.0, 10.0, 0.0),
            (20.0, 0.0, 0.0),
            (30.0, 0.0, 0.0),
            (30.0, 10.0, 0.0),
            (20.0, 10.0, 0.0),
        ],
        &[[0, 1, 2], [0, 2, 3], [4, 5, 6], [4, 6, 7]],
    );
    let input = solver_input(SolverGeometry::TriangleMesh(squares));
    let radius = 3.0;
    let toolpath = solve_toolpath(&input).unwrap();
    let mut passes_per_line = std::collections::BTreeMap::new();
    for level in &toolpath.contour_levels {
        let segments = level.cutting_segments();
        let y = segments[0].start.y();
        let xs = segments
            .iter()
            .flat_map(|segment| [segment.start.x(), segment.end_point().x()]);
        let (start, end) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(low, high), x| {
            (low.min(x), high.max(x))
        });
        let d = (-y).max(y - 10.0).max(0.0);
        let half = (radius * radius - d * d).sqrt();
        let face = if start < 15.0 { 0.0 } else { 20.0 };
        assert!(
            (start - (face - half)).abs() <= input.chord_tolerance + EPS,
            "{y}: {start}"
        );
        assert!(
            (end - (face + 10.0 + half)).abs() <= input.chord_tolerance + EPS,
            "{y}: {end}"
        );
        *passes_per_line.entry(y.to_bits()).or_insert(0) += 1;
    }
    // 走査線は Y = -3..13 の 2 mm ごとの 9 本で、どれも間で分かれた 2 本のパスになる（両端の d = r でも、面の辺に
    // 沿って接する）
    assert_eq!(passes_per_line.len(), 9);
    assert!(passes_per_line.values().all(|&count| count == 2));
}

#[test]
fn scanline_toolpaths_require_a_positive_finite_tolerance() {
    let cutter = InverseOffsetShape::new(&flat_square(0.0), ball(3.0)).unwrap();
    let OperationSpec::Scanline(params) =
        solver_input(SolverGeometry::TriangleMesh(flat_square(0.0))).operation;
    for tolerance in [0.0, -0.01, f64::NAN, f64::INFINITY] {
        let result = crate::generate_scanline_toolpath(
            &cutter,
            "BEM6",
            &params,
            cutter.xy_bounds(),
            tolerance,
        );
        assert!(
            matches!(result, Err(CamSolverError::InvalidInput(_))),
            "{tolerance}"
        );
    }
}

/// 床（高さ 0、x = 10..30）と、両側の急な壁（傾き 80°、高さ 10）と、壁の上の平らな縁（高さ 10）の溝（y = 0..10）
fn pocket_with_steep_walls() -> TriangleMesh3D<f64> {
    let run = 10.0 / 80f64.to_radians().tan();
    let xs = [0.0, 10.0 - run, 10.0, 30.0, 30.0 + run, 40.0];
    let zs = [10.0, 10.0, 0.0, 0.0, 10.0, 10.0];
    let mut vertices = Vec::new();
    for y in [0.0, 10.0] {
        for (x, z) in xs.iter().zip(zs) {
            vertices.push((*x, y, z));
        }
    }
    let n = xs.len();
    let mut triangles = Vec::new();
    for i in 0..n - 1 {
        triangles.push([i, i + 1, n + i + 1]);
        triangles.push([i, n + i + 1, n + i]);
    }
    mesh(&vertices, &triangles)
}

/// 経路の線分を細かく分けた点で、経路が CL の面から上に離れる量の最大
///
/// 段（鉛直な線分）と、段の前後で高い側のまま進む、水平にトレランス以下の線分を除く。線分の端（CL の面の上の点か、
/// 段の端）は測らない。
fn max_rise_of_path_above_cl_surface(input: &CamSolverInput) -> f64 {
    let cutter = build_inverse_offset_shape(input).unwrap();
    let toolpath = solve_toolpath(input).unwrap();
    let mut worst = 0.0_f64;
    for level in &toolpath.contour_levels {
        for segment in level.cutting_segments() {
            let (start, end) = (segment.start, segment.end_point());
            if (end.x() - start.x()).abs() <= input.chord_tolerance {
                continue;
            }
            for k in 1..200 {
                let t = k as f64 / 200.0;
                let x = start.x() + t * (end.x() - start.x());
                let z = start.z() + t * (end.z() - start.z());
                if let Some(tip) = cutter.tip_height_at(x, start.y()) {
                    worst = worst.max(z - tip);
                }
            }
        }
    }
    worst
}

#[test]
fn scanline_paths_do_not_pass_above_the_cl_surface_beyond_the_tolerance() {
    // 谷（壁と床の境目・尾根と谷）と段差をまたぐ所で、経路が CL の面より上を通らない（削り残しの側）
    let geometries = [
        ("step block", SolverGeometry::TriangleMesh(step_block())),
        (
            "ridged terrain",
            SolverGeometry::TriangleMesh(ridged_terrain()),
        ),
        (
            "pocket",
            SolverGeometry::TriangleMesh(pocket_with_steep_walls()),
        ),
        (
            "dome",
            SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]),
        ),
    ];
    for (name, geometry) in geometries {
        for tool in [
            Tool::ball_end_mill("BEM10".to_string(), 10.0, 30.0),
            Tool::flat_end_mill("EM10".to_string(), 10.0, 30.0),
        ] {
            let mut input = solver_input(geometry.clone());
            input.tool = tool;
            let rise = max_rise_of_path_above_cl_surface(&input);
            assert!(
                rise <= input.chord_tolerance + EPS,
                "{name}, {}: the path passes {rise} above the CL surface",
                input.tool.id
            );
            assert_path_does_not_gouge(&input);
        }
    }
}
