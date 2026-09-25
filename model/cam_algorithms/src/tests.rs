use analysis::consts::test_constants::TOLERANCE_F64 as EPS;
use cam_core::{CoordinateFrame, LengthUnit, SegmentType, Tool};
use geo_algorithms::{NurbsSurface3D, Point3D, TriangleMesh3D};
use geo_contracts::{NurbsSurface3DConstructor, default_distance_tolerance};

use crate::{
    BallDropCutter, CamSolverError, CamSolverInput, OperationSpec, ScanlineParams, SolverGeometry,
    TessellationLimits, solve_toolpath, tessellate_surfaces,
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
        vec![(0.0, 0.0, 80.0), (0.0, 20.0, 80.0), (0.0, 40.0, 80.0)],
        vec![(20.0, 0.0, 80.0), (20.0, 20.0, 100.0), (20.0, 40.0, 80.0)],
        vec![(40.0, 0.0, 80.0), (40.0, 20.0, 80.0), (40.0, 40.0, 80.0)],
    ];
    let knots = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    NurbsSurface3D::new(control_points, None, knots.clone(), knots, 2, 2).unwrap()
}

fn solver_input(geometry: SolverGeometry) -> CamSolverInput {
    CamSolverInput {
        operation_id: "op-scan-1".to_string(),
        tool: Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0),
        geometry,
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover: 2.0,
            sample_pitch: 1.0,
            feed_rate: 1200.0,
            clearance_height: 5.0,
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.01,
        tessellation_limits: TessellationLimits::default(),
    }
}

#[test]
fn face_offset_places_ball_center_one_radius_above_flat_face() {
    let cutter = BallDropCutter::new(&flat_square(0.0), 3.0).unwrap();
    let center = cutter.center_height_at(5.0, 5.0).unwrap();
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
    let cutter = BallDropCutter::new(&inclined, r).unwrap();
    let expected = slope * 5.0 + r * (1.0 + slope * slope).sqrt();
    assert!((cutter.center_height_at(5.0, 5.0).unwrap() - expected).abs() < EPS);
}

#[test]
fn edge_offset_is_cylinder_outside_face() {
    // 辺 y=0（z=0）から水平距離 b の位置: 中心高さ = sqrt(r^2 - b^2)
    let cutter = BallDropCutter::new(&flat_square(0.0), 3.0).unwrap();
    let b = 1.5;
    let expected = (9.0_f64 - b * b).sqrt();
    assert!((cutter.center_height_at(5.0, -b).unwrap() - expected).abs() < EPS);
}

#[test]
fn vertex_offset_is_sphere_outside_edges() {
    let cutter = BallDropCutter::new(&flat_square(0.0), 3.0).unwrap();
    let (dx, dy) = (-1.0, -2.0);
    let expected = (9.0_f64 - dx * dx - dy * dy).sqrt();
    assert!((cutter.center_height_at(dx, dy).unwrap() - expected).abs() < EPS);
}

#[test]
fn no_contact_outside_tool_radius() {
    let cutter = BallDropCutter::new(&flat_square(0.0), 3.0).unwrap();
    assert!(cutter.center_height_at(-3.5, 5.0).is_none());
}

#[test]
fn ball_center_does_not_gouge_tessellated_dome() {
    let limits = TessellationLimits::default();
    let dome = tessellate_surfaces(&[dome_surface()], 0.01, limits).unwrap();
    let r = 3.0;
    let cutter = BallDropCutter::new(&dome, r).unwrap();

    for &(x, y) in &[
        (20.0, 20.0),
        (5.0, 7.0),
        (33.0, 12.0),
        (0.0, 40.0),
        (-2.0, 20.0),
    ] {
        let center_z = cutter.center_height_at(x, y).unwrap();
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

    // 極小半径の drop-cutter でメッシュ高さを取り、曲面上の点との鉛直差を検証する。
    // 曲面の最大傾斜は 1 未満のため、法線方向の弦誤差 tol は鉛直差 2*tol 以内に収まる。
    let probe = BallDropCutter::new(&dome, 1e-9).unwrap();
    for i in 0..=20 {
        for j in 0..=20 {
            let p = surface.evaluate_at(i as f64 / 20.0, j as f64 / 20.0);
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
    let input = solver_input(SolverGeometry::NurbsSurfaceSet(vec![dome_surface()]));
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
        assert!(matches!(
            level.segments.first().unwrap().segment_type,
            SegmentType::Approach { .. } | SegmentType::Rapid
        ));
    }
}

#[test]
fn invalid_input_for_non_ball_tool() {
    let mut input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    input.tool = Tool::flat_end_mill("EM6".to_string(), 6.0, 30.0);
    let error = solve_toolpath(&input).unwrap_err();
    assert_eq!(error.code(), "invalid_input");
}

#[test]
fn invalid_input_for_stepover_larger_than_diameter() {
    let mut input = solver_input(SolverGeometry::TriangleMesh(flat_square(0.0)));
    input.operation = OperationSpec::Scanline(ScanlineParams {
        stepover: 7.0,
        sample_pitch: 1.0,
        feed_rate: 1200.0,
        clearance_height: 5.0,
    });
    assert_eq!(solve_toolpath(&input).unwrap_err().code(), "invalid_input");
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
