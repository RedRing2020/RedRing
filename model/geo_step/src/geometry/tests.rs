use super::*;
use crate::record::{BSplineCurve, BSplineSurface, Records};
use geo_contracts::{
    Circle3DEvaluation, ConicalSurface3DDistance, ConicalSurface3DEvaluation,
    CylindricalSurface3DDistance, CylindricalSurface3DProperties, Ellipse3DEvaluation,
    InfiniteLine3DEvaluation, NurbsCurve3DEvaluation, NurbsSurface3DEvaluation,
    SphericalSurface3DDistance, SphericalSurface3DProperties, TorusSurface3DDistance,
    TorusSurface3DProperties,
};

const TOLERANCE: f64 = 1.0e-9;

fn tolerances() -> ToleranceSettings<f64> {
    ToleranceSettings {
        distance_tolerance: TOLERANCE,
        angle_tolerance: TOLERANCE,
        area_tolerance: TOLERANCE * TOLERANCE,
        length_tolerance: TOLERANCE,
    }
}

/// 試験の記録を組み立てる
struct Builder {
    records: Records,
    next: u64,
}

impl Builder {
    fn new() -> Self {
        Self {
            records: Records::default(),
            next: 1,
        }
    }

    fn add(&mut self, geometry: Record) -> u64 {
        let id = self.next;
        self.next += 1;
        self.records.geometry.insert(id, geometry);
        id
    }

    fn point(&mut self, x: f64, y: f64, z: f64) -> u64 {
        self.add(Record::Point([x, y, z]))
    }

    fn direction(&mut self, x: f64, y: f64, z: f64) -> u64 {
        self.add(Record::Direction([x, y, z]))
    }

    /// 原点 `origin`・軸 `axis`・基準の方向 `reference` の配置
    fn placement(
        &mut self,
        origin: [f64; 3],
        axis: Option<[f64; 3]>,
        reference: Option<[f64; 3]>,
    ) -> u64 {
        let location = self.point(origin[0], origin[1], origin[2]);
        let axis = axis.map(|[x, y, z]| self.direction(x, y, z));
        let ref_direction = reference.map(|[x, y, z]| self.direction(x, y, z));
        self.add(Record::Placement {
            location,
            axis,
            ref_direction,
        })
    }

    fn convert(&self) -> Geometries {
        convert(&self.records, &tolerances())
    }
}

fn base_curve(geometries: &Geometries, id: u64) -> &CurveRef<f64> {
    match &geometries.curves[&id] {
        Curve::Base(curve) => curve,
        Curve::Conic(_) => panic!("a base curve"),
    }
}

fn reason(geometries: &Geometries, id: u64) -> Option<GeometryError> {
    geometries
        .skipped
        .iter()
        .find(|skipped| skipped.id == id)
        .map(|skipped| skipped.reason)
}

#[test]
fn placements_become_orthonormal_frames() {
    let mut builder = Builder::new();
    // 軸と基準の方向がない配置は、Z と X
    let default = builder.placement([1.0, 2.0, 3.0], None, None);
    // 基準の方向は、軸の成分を除いて大きさを 1 にする
    let skewed = builder.placement(
        [0.0, 0.0, 0.0],
        Some([0.0, 0.0, 2.0]),
        Some([3.0, 0.0, 3.0]),
    );
    // 軸と平行な基準の方向では、座標系を作れない
    let parallel = builder.placement(
        [0.0, 0.0, 0.0],
        Some([0.0, 0.0, 1.0]),
        Some([0.0, 0.0, 5.0]),
    );
    let converter = Converter {
        records: &builder.records,
        tolerances: &tolerances(),
        geometries: Geometries::default(),
        failed: BTreeMap::new(),
        visiting: BTreeSet::new(),
    };

    let frame = converter.frame(default).unwrap();
    assert_eq!(frame.origin, Point3D::new(1.0, 2.0, 3.0));
    assert!((frame.z.as_vector() - Vector3D::new(0.0, 0.0, 1.0)).length() < 1.0e-12);
    assert!((frame.x.as_vector() - Vector3D::new(1.0, 0.0, 0.0)).length() < 1.0e-12);
    let frame = converter.frame(skewed).unwrap();
    assert!((frame.x.as_vector() - Vector3D::new(1.0, 0.0, 0.0)).length() < 1.0e-12);
    assert!((frame.y.as_vector() - Vector3D::new(0.0, 1.0, 0.0)).length() < 1.0e-12);
    assert_eq!(
        converter.frame(parallel).unwrap_err(),
        GeometryError::DegeneratePlacement
    );
}

#[test]
fn analytic_curves_keep_their_shape() {
    let mut builder = Builder::new();
    let frame = builder.placement(
        [1.0, 2.0, 3.0],
        Some([0.0, 0.0, 1.0]),
        Some([1.0, 0.0, 0.0]),
    );
    let circle = builder.add(Record::Circle {
        placement: frame,
        radius: 10.0,
    });
    // 2 つ目の半軸が長い楕円は、Y を長軸にする
    let ellipse = builder.add(Record::Ellipse {
        placement: frame,
        semi_axis_1: 2.0,
        semi_axis_2: 5.0,
    });
    let point = builder.point(0.0, 0.0, 0.0);
    let direction = builder.direction(0.0, 3.0, 0.0);
    let vector = builder.add(Record::Vector {
        direction,
        magnitude: -2.0,
    });
    let line = builder.add(Record::Line { point, vector });
    let surface_curve = builder.add(Record::SurfaceCurve { curve: circle });
    let geometries = builder.convert();

    let CurveRef::Circle(circle) = base_curve(&geometries, circle) else {
        panic!("a circle");
    };
    for t in [0.0, 1.0, 2.5] {
        let p = circle.point_at_parameter(t);
        assert!((p.distance_to(Point3D::new(1.0, 2.0, 3.0)) - 10.0).abs() < 1.0e-12);
        assert!((p.z() - 3.0).abs() < 1.0e-12);
    }
    let CurveRef::Ellipse(ellipse) = base_curve(&geometries, ellipse) else {
        panic!("an ellipse");
    };
    let major_end = ellipse.point_at_parameter(0.0);
    assert!(
        major_end.distance_to(Point3D::new(1.0, 7.0, 3.0)) < 1.0e-12
            || major_end.distance_to(Point3D::new(1.0, -3.0, 3.0)) < 1.0e-12
    );
    // 大きさが負のベクトルは、向きを逆にする
    let CurveRef::Line(line) = base_curve(&geometries, line) else {
        panic!("a line");
    };
    assert!(
        line.point_at_parameter(1.0)
            .distance_to(Point3D::new(0.0, -1.0, 0.0))
            < 1.0e-12
    );
    // 曲面の上の曲線は、その 3 次元の曲線
    assert!(matches!(
        base_curve(&geometries, surface_curve),
        CurveRef::Circle(_)
    ));
    assert!(geometries.skipped.is_empty(), "{:?}", geometries.skipped);
}

#[test]
fn analytic_surfaces_keep_their_shape() {
    let mut builder = Builder::new();
    let frame = builder.placement([0.0, 0.0, 1.0], Some([0.0, 0.0, 1.0]), None);
    let cylinder = builder.add(Record::CylindricalSurface {
        placement: frame,
        radius: 2.0,
    });
    let cone = builder.add(Record::ConicalSurface {
        placement: frame,
        radius: 2.0,
        semi_angle: Angle::from_degrees(30.0),
    });
    let sphere = builder.add(Record::SphericalSurface {
        placement: frame,
        radius: 3.0,
    });
    let torus = builder.add(Record::ToroidalSurface {
        placement: frame,
        major_radius: 5.0,
        minor_radius: 1.0,
    });
    let plane = builder.add(Record::Plane { placement: frame });
    let broken = builder.add(Record::CylindricalSurface {
        placement: frame,
        radius: -1.0,
    });
    let geometries = builder.convert();

    let SurfaceRef::Cylinder(cylinder) = &geometries.surfaces[&cylinder].surface else {
        panic!("a cylinder");
    };
    assert_eq!(cylinder.radius(), 2.0);
    assert!(cylinder.distance_to_point(Point3D::new(2.0, 0.0, 7.0)) < 1.0e-12);
    let SurfaceRef::Cone(cone) = &geometries.surfaces[&cone].surface else {
        panic!("a cone");
    };
    // 配置の原点の高さで半径 2、そこから軸の方向に 1 上がると、半径は tan 30° 増える
    let up = 2.0 + 30.0_f64.to_radians().tan();
    assert!(cone.distance_to_point(Point3D::new(up, 0.0, 2.0)) < 1.0e-12);
    let SurfaceRef::Sphere(sphere) = &geometries.surfaces[&sphere].surface else {
        panic!("a sphere");
    };
    assert!(sphere.distance_to_point(Point3D::new(0.0, 0.0, 4.0)) < 1.0e-12);
    let SurfaceRef::Torus(torus) = &geometries.surfaces[&torus].surface else {
        panic!("a torus");
    };
    assert!(torus.distance_to_point(Point3D::new(6.0, 0.0, 1.0)) < 1.0e-12);
    assert!(matches!(
        geometries.surfaces[&plane].surface,
        SurfaceRef::Plane(_)
    ));
    assert_eq!(
        reason(&geometries, broken),
        Some(GeometryError::InvalidShape)
    );
}

/// de Boor の方法で、ノットベクトル `knots`・同次座標の制御点 `points` の B-spline の点を直接求める（両端を固定
/// していないものの、元の形の点）
fn de_boor(degree: usize, knots: &[f64], points: &[[f64; 4]], t: f64) -> [f64; 3] {
    let span = (degree..knots.len() - degree - 1)
        .rev()
        .find(|&index| knots[index] <= t)
        .unwrap();
    let mut d: Vec<[f64; 4]> = (0..=degree).map(|j| points[j + span - degree]).collect();
    for r in 1..=degree {
        for j in (r..=degree).rev() {
            let index = j + span - degree;
            let alpha = (t - knots[index]) / (knots[index + degree + 1 - r] - knots[index]);
            let previous = d[j - 1];
            for (value, before) in d[j].iter_mut().zip(previous) {
                *value = (1.0 - alpha) * before + alpha * *value;
            }
        }
    }
    let h = d[degree];
    [h[0] / h[3], h[1] / h[3], h[2] / h[3]]
}

#[test]
fn an_unclamped_b_spline_curve_is_clamped_without_changing_its_shape() {
    let mut builder = Builder::new();
    let coordinates = [
        [0.0, 0.0, 0.0],
        [1.0, 2.0, 0.0],
        [3.0, 3.0, 1.0],
        [5.0, 1.0, 2.0],
        [6.0, -1.0, 1.0],
        [8.0, 0.0, 0.0],
    ];
    let control_points: Vec<u64> = coordinates
        .iter()
        .map(|&[x, y, z]| builder.point(x, y, z))
        .collect();
    let weights = vec![1.0, 2.0, 0.5, 1.0, 1.5, 1.0];
    // 3 次で、両端を固定していない一様なノット（定義域は 3〜6）
    let curve = builder.add(Record::BSplineCurve(BSplineCurve {
        degree: 3,
        control_points,
        closed: Some(false),
        multiplicities: vec![1; 10],
        knots: (0..10).map(f64::from).collect(),
        weights: Some(weights.clone()),
    }));
    let geometries = builder.convert();
    let CurveRef::Nurbs(nurbs) = base_curve(&geometries, curve) else {
        panic!("a NURBS curve");
    };

    let knots: Vec<f64> = (0..10).map(f64::from).collect();
    let homogeneous: Vec<[f64; 4]> = coordinates
        .iter()
        .zip(&weights)
        .map(|(&[x, y, z], &w)| [x * w, y * w, z * w, w])
        .collect();
    for t in [3.0, 3.4, 4.0, 4.75, 5.5, 5.999] {
        let expected = de_boor(3, &knots, &homogeneous, t);
        let actual = nurbs.point_at_parameter(t).unwrap();
        let expected = Point3D::new(expected[0], expected[1], expected[2]);
        assert!(actual.distance_to(expected) < 1.0e-9, "t = {t}");
    }
}

#[test]
fn an_unclamped_b_spline_surface_is_clamped_without_changing_its_shape() {
    let mut builder = Builder::new();
    // u は 2 次で両端を固定していない一様なノット（定義域 2〜3）、v は 1 次で固定したノット
    let grid: Vec<Vec<[f64; 3]>> = (0..4)
        .map(|i| {
            (0..2)
                .map(|j| [f64::from(i), f64::from(j), f64::from((i * 3 + j) % 4)])
                .collect()
        })
        .collect();
    let control_points: Vec<Vec<u64>> = grid
        .iter()
        .map(|row| {
            row.iter()
                .map(|&[x, y, z]| builder.point(x, y, z))
                .collect()
        })
        .collect();
    let surface = builder.add(Record::BSplineSurface(BSplineSurface {
        u_degree: 2,
        v_degree: 1,
        control_points,
        u_closed: Some(false),
        v_closed: Some(false),
        u_multiplicities: vec![1; 7],
        v_multiplicities: vec![2, 2],
        u_knots: (0..7).map(f64::from).collect(),
        v_knots: vec![0.0, 1.0],
        weights: None,
    }));
    let geometries = builder.convert();
    let SurfaceRef::Nurbs(nurbs) = &geometries.surfaces[&surface].surface else {
        panic!("a NURBS surface: {:?}", geometries.skipped);
    };

    let u_knots: Vec<f64> = (0..7).map(f64::from).collect();
    for u in [2.0, 2.3, 2.9] {
        for v in [0.0, 0.5, 1.0] {
            // v は 1 次なので、u の方向の 2 つの曲線を v で結ぶ
            let along = |j: usize| {
                let column: Vec<[f64; 4]> = grid
                    .iter()
                    .map(|row| {
                        let [x, y, z] = row[j];
                        [x, y, z, 1.0]
                    })
                    .collect();
                de_boor(2, &u_knots, &column, u)
            };
            let (a, b) = (along(0), along(1));
            let expected = Point3D::new(
                a[0] * (1.0 - v) + b[0] * v,
                a[1] * (1.0 - v) + b[1] * v,
                a[2] * (1.0 - v) + b[2] * v,
            );
            let actual = nurbs.point_at_uv(u, v).unwrap();
            assert!(actual.distance_to(expected) < 1.0e-9, "u = {u}, v = {v}");
        }
    }
}

#[test]
fn b_splines_with_wrong_knots_are_skipped() {
    let mut builder = Builder::new();
    let points: Vec<u64> = (0..3)
        .map(|i| builder.point(f64::from(i), 0.0, 0.0))
        .collect();
    let curve = builder.add(Record::BSplineCurve(BSplineCurve {
        degree: 2,
        control_points: points,
        closed: None,
        // 重複の合計が、制御点の数 + 次数 + 1 に合わない
        multiplicities: vec![3, 1],
        knots: vec![0.0, 1.0],
        weights: None,
    }));
    let geometries = builder.convert();

    assert_eq!(
        reason(&geometries, curve),
        Some(GeometryError::InvalidBSpline)
    );
}

#[test]
fn parabolas_and_hyperbolas_become_nurbs_segments() {
    let mut builder = Builder::new();
    let frame = builder.placement(
        [1.0, 0.0, 0.0],
        Some([0.0, 0.0, 1.0]),
        Some([1.0, 0.0, 0.0]),
    );
    let parabola = builder.add(Record::Parabola {
        placement: frame,
        focal_distance: 0.5,
    });
    let hyperbola = builder.add(Record::Hyperbola {
        placement: frame,
        semi_axis: 2.0,
        semi_imaginary_axis: 1.0,
    });
    let geometries = builder.convert();

    for (id, tolerance) in [(parabola, 0.0_f64), (hyperbola, 1.0e-6)] {
        let Curve::Conic(conic) = &geometries.curves[&id] else {
            panic!("a conic");
        };
        let (nurbs, deviation) = conic.segment((-1.5, 2.0), tolerance.max(1.0e-12)).unwrap();
        assert!(deviation <= tolerance.max(1.0e-12));
        for i in 0..=20 {
            let t = -1.5 + 3.5 * f64::from(i) / 20.0;
            let distance = nurbs
                .point_at_parameter(t)
                .unwrap()
                .distance_to(conic.point_at(t));
            assert!(distance < 1.0e-5, "t = {t}: {distance}");
        }
    }
}

#[test]
fn extrusions_revolutions_and_offsets_in_the_design_range_are_read() {
    let mut builder = Builder::new();
    let z_frame = builder.placement(
        [0.0, 0.0, 0.0],
        Some([0.0, 0.0, 1.0]),
        Some([1.0, 0.0, 0.0]),
    );
    let circle = builder.add(Record::Circle {
        placement: z_frame,
        radius: 3.0,
    });
    let up = builder.direction(0.0, 0.0, 1.0);
    let up_vector = builder.add(Record::Vector {
        direction: up,
        magnitude: 5.0,
    });
    let extruded_circle = builder.add(Record::SurfaceOfLinearExtrusion {
        curve: circle,
        vector: up_vector,
    });
    // Z の軸のまわりの回転: 軸に平行な直線（円柱）、軸と交わる直線（円錐）、XZ 平面の円（トーラス・球）
    let axis_location = builder.point(0.0, 0.0, 0.0);
    let axis = builder.add(Record::Axis {
        location: axis_location,
        axis: Some(up),
    });
    let line_point = builder.point(2.0, 0.0, 0.0);
    let parallel_line = builder.add(Record::Line {
        point: line_point,
        vector: up_vector,
    });
    let slanted_direction = builder.direction(1.0, 0.0, 1.0);
    let slanted_vector = builder.add(Record::Vector {
        direction: slanted_direction,
        magnitude: 1.0,
    });
    let slanted_line = builder.add(Record::Line {
        point: line_point,
        vector: slanted_vector,
    });
    let tube_frame = builder.placement(
        [4.0, 0.0, 0.0],
        Some([0.0, 1.0, 0.0]),
        Some([1.0, 0.0, 0.0]),
    );
    let tube = builder.add(Record::Circle {
        placement: tube_frame,
        radius: 1.0,
    });
    let meridian_frame = builder.placement(
        [0.0, 0.0, 0.0],
        Some([0.0, 1.0, 0.0]),
        Some([1.0, 0.0, 0.0]),
    );
    let meridian = builder.add(Record::Circle {
        placement: meridian_frame,
        radius: 2.5,
    });
    let cylinder = builder.add(Record::SurfaceOfRevolution {
        curve: parallel_line,
        axis,
    });
    let cone = builder.add(Record::SurfaceOfRevolution {
        curve: slanted_line,
        axis,
    });
    let torus = builder.add(Record::SurfaceOfRevolution { curve: tube, axis });
    let sphere = builder.add(Record::SurfaceOfRevolution {
        curve: meridian,
        axis,
    });
    // 軸と交わらない、ねじれた直線は範囲の外
    let skew_point = builder.point(2.0, 1.0, 0.0);
    let sideways = builder.direction(1.0, 0.0, 1.0);
    let sideways_vector = builder.add(Record::Vector {
        direction: sideways,
        magnitude: 1.0,
    });
    let skew_line = builder.add(Record::Line {
        point: skew_point,
        vector: sideways_vector,
    });
    let skew = builder.add(Record::SurfaceOfRevolution {
        curve: skew_line,
        axis,
    });
    // オフセット: 円錐を外向きの法線の側へ 0.5
    let base_cone = builder.add(Record::ConicalSurface {
        placement: z_frame,
        radius: 2.0,
        semi_angle: Angle::from_degrees(30.0),
    });
    let offset_cone = builder.add(Record::OffsetSurface {
        surface: base_cone,
        distance: 0.5,
    });
    let geometries = builder.convert();

    let SurfaceRef::Cylinder(extruded) = &geometries.surfaces[&extruded_circle].surface else {
        panic!("a cylinder");
    };
    assert_eq!(extruded.radius(), 3.0);
    let SurfaceRef::Cylinder(cylinder) = &geometries.surfaces[&cylinder].surface else {
        panic!("a cylinder");
    };
    assert!((cylinder.radius() - 2.0).abs() < 1.0e-12);
    // 回した直線の点は、円錐の上にある
    let SurfaceRef::Cone(cone) = &geometries.surfaces[&cone].surface else {
        panic!("a cone");
    };
    for t in [0.5, 1.0, 3.0] {
        let p = Point3D::new(2.0 + t, 0.0, t);
        assert!(cone.distance_to_point(p) < 1.0e-9, "t = {t}");
    }
    let SurfaceRef::Torus(torus) = &geometries.surfaces[&torus].surface else {
        panic!("a torus");
    };
    assert!((torus.major_radius() - 4.0).abs() < 1.0e-12);
    assert_eq!(torus.minor_radius(), 1.0);
    // 中心が軸の上の円を回した球は、STEP の法線との関係が面ごとに決まる
    assert!(matches!(
        geometries.surfaces[&sphere].sense,
        Sense::PerFace { .. }
    ));
    let SurfaceRef::Sphere(sphere) = &geometries.surfaces[&sphere].surface else {
        panic!("a sphere");
    };
    assert_eq!(sphere.radius(), 2.5);
    assert_eq!(reason(&geometries, skew), Some(GeometryError::Unsupported));
    // 元の円錐の点を、その法線の向きに 0.5 動かした点は、オフセットの円錐の上にある
    let (SurfaceRef::Cone(base), SurfaceRef::Cone(offset)) = (
        &geometries.surfaces[&base_cone].surface,
        &geometries.surfaces[&offset_cone].surface,
    ) else {
        panic!("cones");
    };
    for (u, v) in [(0.0, 0.0), (1.0, 2.0), (4.0, -1.0)] {
        let moved = base.point_at_uv(u, v) + base.normal_at(u, v).unwrap().as_vector() * 0.5;
        assert!(offset.distance_to_point(moved) < 1.0e-9, "u = {u}, v = {v}");
    }
}

#[test]
fn extruded_and_revolved_surfaces_report_a_reversed_normal() {
    let mut builder = Builder::new();
    let frame = builder.placement(
        [0.0, 0.0, 0.0],
        Some([0.0, 0.0, 1.0]),
        Some([1.0, 0.0, 0.0]),
    );
    let circle = builder.add(Record::Circle {
        placement: frame,
        radius: 3.0,
    });
    let up = builder.direction(0.0, 0.0, 1.0);
    let down = builder.direction(0.0, 0.0, -1.0);
    let up_vector = builder.add(Record::Vector {
        direction: up,
        magnitude: 1.0,
    });
    let down_vector = builder.add(Record::Vector {
        direction: down,
        magnitude: 1.0,
    });
    // 円の軸の向きに押し出すと、STEP の法線は外向き（円柱と同じ）、逆の向きなら内向き
    let along = builder.add(Record::SurfaceOfLinearExtrusion {
        curve: circle,
        vector: up_vector,
    });
    let against = builder.add(Record::SurfaceOfLinearExtrusion {
        curve: circle,
        vector: down_vector,
    });
    // 軸の向きの直線を回すと、STEP の法線は外向き、逆の向きの直線なら内向き
    let axis_location = builder.point(0.0, 0.0, 0.0);
    let axis = builder.add(Record::Axis {
        location: axis_location,
        axis: Some(up),
    });
    let line_point = builder.point(2.0, 0.0, 0.0);
    let up_line = builder.add(Record::Line {
        point: line_point,
        vector: up_vector,
    });
    let down_line = builder.add(Record::Line {
        point: line_point,
        vector: down_vector,
    });
    let outward = builder.add(Record::SurfaceOfRevolution {
        curve: up_line,
        axis,
    });
    let inward = builder.add(Record::SurfaceOfRevolution {
        curve: down_line,
        axis,
    });
    // 逆の向きの面のオフセットは、型の曲面では内側（半径が小さくなる）へ動く
    let offset_inward = builder.add(Record::OffsetSurface {
        surface: inward,
        distance: 0.5,
    });
    let geometries = builder.convert();

    assert_eq!(geometries.surfaces[&along].sense, Sense::Same);
    assert_eq!(geometries.surfaces[&against].sense, Sense::Reversed);
    assert_eq!(geometries.surfaces[&outward].sense, Sense::Same);
    assert_eq!(geometries.surfaces[&inward].sense, Sense::Reversed);
    let offset = &geometries.surfaces[&offset_inward];
    assert_eq!(offset.sense, Sense::Reversed);
    let SurfaceRef::Cylinder(cylinder) = &offset.surface else {
        panic!("a cylinder");
    };
    assert!((cylinder.radius() - 1.5).abs() < 1.0e-12);
}

#[test]
fn self_referencing_curves_and_surfaces_are_skipped_once() {
    let mut builder = Builder::new();
    // 自分を指す曲面の上の曲線と、互いを指すオフセットの面
    let curve = builder.next;
    builder.add(Record::SurfaceCurve { curve });
    let first = builder.next;
    builder.add(Record::OffsetSurface {
        surface: first + 1,
        distance: 1.0,
    });
    builder.add(Record::OffsetSurface {
        surface: first,
        distance: 1.0,
    });
    let geometries = builder.convert();

    assert_eq!(
        reason(&geometries, curve),
        Some(GeometryError::CyclicReference)
    );
    assert_eq!(
        reason(&geometries, first),
        Some(GeometryError::CyclicReference)
    );
    assert_eq!(
        geometries
            .skipped
            .iter()
            .filter(|skipped| skipped.id == curve)
            .count(),
        1
    );
}

#[test]
fn knots_are_checked_before_expanding_and_extra_outer_knots_are_dropped() {
    let mut builder = Builder::new();
    let points: Vec<u64> = (0..5)
        .map(|i| builder.point(f64::from(i), f64::from(i % 2), 0.0))
        .collect();
    // 重複の数が大きすぎる（展開する前に、合計で弾く）
    let huge = builder.add(Record::BSplineCurve(BSplineCurve {
        degree: 3,
        control_points: points[..4].to_vec(),
        closed: None,
        multiplicities: vec![4_000_000_000_000_000_000, 4],
        knots: vec![0.0, 1.0],
        weights: None,
    }));
    // 始まりのノットがすでに次数 + 1 個重なっていて、その前にも定義域の外のノットがある
    let extra = builder.add(Record::BSplineCurve(BSplineCurve {
        degree: 3,
        control_points: points.clone(),
        closed: None,
        multiplicities: vec![1, 4, 4],
        knots: vec![-1.0, 0.0, 1.0],
        weights: None,
    }));
    let geometries = builder.convert();

    assert_eq!(
        reason(&geometries, huge),
        Some(GeometryError::InvalidBSpline)
    );
    let CurveRef::Nurbs(nurbs) = base_curve(&geometries, extra) else {
        panic!("a NURBS curve: {:?}", geometries.skipped);
    };
    // 定義域 0〜1 の曲線は、2 番目から 5 番目の制御点の 3 次の Bézier 曲線
    let start = nurbs.point_at_parameter(0.0).unwrap();
    let end = nurbs.point_at_parameter(1.0).unwrap();
    assert!(start.distance_to(Point3D::new(1.0, 1.0, 0.0)) < 1.0e-12);
    assert!(end.distance_to(Point3D::new(4.0, 0.0, 0.0)) < 1.0e-12);
}

#[test]
fn an_overflowing_hyperbola_cannot_be_approximated() {
    let conic = Conic::Hyperbola {
        center: Point3D::new(0.0, 0.0, 0.0),
        x: Vector3D::new(1.0, 0.0, 0.0),
        y: Vector3D::new(0.0, 1.0, 0.0),
        semi_axis: 1.0,
        semi_imaginary_axis: 1.0,
    };

    assert!(conic.segment((700.0, 712.0), 1.0e-6).is_none());
}
