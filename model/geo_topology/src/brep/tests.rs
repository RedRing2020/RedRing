use super::*;
use crate::{
    TopoCircle3D, TopoConicalSurface3D, TopoCylindricalSurface3D, TopoInfiniteLine3D, TopoPlane3D,
    TopoSphericalSurface3D, TopoTorusSurface3D, TopologyValidator,
};
use geo_contracts::{
    Angle, Circle3DConstructor, Circle3DEvaluation, ConicalSurface3DConstructor,
    CylindricalSurface3DConstructor, InfiniteLine3DConstructor, NurbsCurve3DConstructor,
    Plane3DConstructor, SphericalSurface3DConstructor, TorusSurface3DConstructor,
    TorusSurface3DEvaluation,
};
use geo_core::Direction3D;
use std::f64::consts::{FRAC_PI_4, PI, TAU};

fn tolerances() -> TopologyToleranceSettings<f64> {
    TopologyToleranceSettings::new(1.0e-9)
}

fn validator() -> TopologyValidator<f64> {
    TopologyValidator::new(tolerances())
}

/// 2 つの頂点の間の直線の稜線を足す
fn line_edge(brep: &mut Brep<f64>, start: VertexIndex, end: VertexIndex) -> EdgeIndex {
    let (from, to) = (brep.vertex(start).point(), brep.vertex(end).point());
    let line = TopoInfiniteLine3D::from_two_points(from, to).unwrap();
    brep.add_edge(
        start,
        end,
        CurveRef::Line(line),
        (0.0, from.distance_to(to)),
    )
    .unwrap()
}

/// 円 `circle` の 1 周の稜線（始点と終点が同じ頂点）を足す
fn circle_edge(brep: &mut Brep<f64>, circle: TopoCircle3D<f64>) -> EdgeIndex {
    let vertex = brep.add_vertex(circle.point_at_parameter(0.0));
    brep.add_edge(vertex, vertex, CurveRef::Circle(circle), (0.0, TAU))
        .unwrap()
}

/// 中心が (0, 0, `height`)、軸が Z の、半径 `radius` の円
fn horizontal_circle(height: f64, radius: f64) -> TopoCircle3D<f64> {
    TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, height), radius).unwrap()
}

/// 3 点を通る平面
fn plane(a: Point3D<f64>, b: Point3D<f64>, c: Point3D<f64>) -> SurfaceRef<f64> {
    SurfaceRef::Plane(TopoPlane3D::from_three_points(a, b, c).unwrap())
}

fn forward(edge: EdgeIndex) -> EdgeUse {
    EdgeUse {
        edge,
        forward: true,
    }
}

fn backward(edge: EdgeIndex) -> EdgeUse {
    EdgeUse {
        edge,
        forward: false,
    }
}

/// 外向きの面からなる四面体（頂点 0 が原点、1・2・3 が各軸の上）
fn tetrahedron() -> (Brep<f64>, Vec<FaceIndex>) {
    let mut brep = Brep::new();
    let p = [
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
        Point3D::new(0.0, 0.0, 1.0),
    ];
    let v: Vec<VertexIndex> = p.iter().map(|&point| brep.add_vertex(point)).collect();
    let e01 = line_edge(&mut brep, v[0], v[1]);
    let e02 = line_edge(&mut brep, v[0], v[2]);
    let e03 = line_edge(&mut brep, v[0], v[3]);
    let e12 = line_edge(&mut brep, v[1], v[2]);
    let e13 = line_edge(&mut brep, v[1], v[3]);
    let e23 = line_edge(&mut brep, v[2], v[3]);
    let faces = vec![
        // 底面（-z）: 0 → 2 → 1
        brep.add_face(
            plane(p[0], p[2], p[1]),
            true,
            vec![vec![forward(e02), backward(e12), backward(e01)]],
            tolerances(),
        )
        .unwrap(),
        // 側面（-y）: 0 → 1 → 3
        brep.add_face(
            plane(p[0], p[1], p[3]),
            true,
            vec![vec![forward(e01), forward(e13), backward(e03)]],
            tolerances(),
        )
        .unwrap(),
        // 側面（-x）: 0 → 3 → 2
        brep.add_face(
            plane(p[0], p[3], p[2]),
            true,
            vec![vec![forward(e03), backward(e23), backward(e02)]],
            tolerances(),
        )
        .unwrap(),
        // 斜めの面: 1 → 2 → 3
        brep.add_face(
            plane(p[1], p[2], p[3]),
            true,
            vec![vec![forward(e12), forward(e23), backward(e13)]],
            tolerances(),
        )
        .unwrap(),
    ];
    (brep, faces)
}

#[test]
fn two_faces_share_one_edge() {
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let c = brep.add_vertex(Point3D::new(1.0, 1.0, 0.0));
    let d = brep.add_vertex(Point3D::new(0.0, 1.0, 1.0));
    let ab = line_edge(&mut brep, a, b);
    let bc = line_edge(&mut brep, b, c);
    let ca = line_edge(&mut brep, c, a);
    let cd = line_edge(&mut brep, c, d);
    let da = line_edge(&mut brep, d, a);
    let points = |index: VertexIndex| brep.vertex(index).point();
    let first_plane = plane(points(a), points(b), points(c));
    let second_plane = plane(points(a), points(c), points(d));
    let first = brep
        .add_face(
            first_plane,
            true,
            vec![vec![forward(ab), forward(bc), forward(ca)]],
            tolerances(),
        )
        .unwrap();
    let second = brep
        .add_face(
            second_plane,
            true,
            vec![vec![backward(ca), forward(cd), forward(da)]],
            tolerances(),
        )
        .unwrap();

    let shared = brep.edge(ca);
    assert_eq!(shared.uses().len(), 2);
    assert_eq!(shared.uses()[0].face, first);
    assert_eq!(shared.uses()[1].face, second);
    assert_eq!(shared.uses()[1].position, 0);
    assert!(validator().validate(&brep).is_valid());
}

#[test]
fn a_closed_shape_uses_every_edge_twice_in_opposite_directions() {
    let (brep, _) = tetrahedron();

    for edge in 0..brep.edge_count() {
        let uses = brep.edge(EdgeIndex(edge)).uses();
        assert_eq!(uses.len(), 2);
        let forward = |position: usize| {
            let location = uses[position];
            brep.face_loop(location.loop_index).uses()[location.position].forward
        };
        assert_ne!(forward(0), forward(1));
    }
    assert!(validator().validate(&brep).is_valid());
}

#[test]
fn a_seam_edge_is_used_twice_by_one_face() {
    // 正方形の面の角から内側へ切れ目の稜線を入れ、行きと帰りで 2 回使う
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let c = brep.add_vertex(Point3D::new(1.0, 1.0, 0.0));
    let d = brep.add_vertex(Point3D::new(0.0, 1.0, 0.0));
    let inner = brep.add_vertex(Point3D::new(0.5, 0.5, 0.0));
    let ab = line_edge(&mut brep, a, b);
    let bc = line_edge(&mut brep, b, c);
    let cd = line_edge(&mut brep, c, d);
    let da = line_edge(&mut brep, d, a);
    let slit = line_edge(&mut brep, a, inner);
    let surface = plane(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    );
    let face = brep
        .add_face(
            surface,
            true,
            vec![vec![
                forward(ab),
                forward(bc),
                forward(cd),
                forward(da),
                forward(slit),
                backward(slit),
            ]],
            tolerances(),
        )
        .unwrap();

    let uses = brep.edge(slit).uses();
    assert_eq!(uses.len(), 2);
    assert!(uses.iter().all(|location| location.face == face));
    assert_eq!((uses[0].position, uses[1].position), (4, 5));
    assert!(validator().validate(&brep).is_valid());
}

#[test]
fn a_face_without_loops_needs_a_closed_surface() {
    let mut brep = Brep::new();
    let face = brep
        .add_face(
            plane(
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(1.0, 0.0, 0.0),
                Point3D::new(0.0, 1.0, 0.0),
            ),
            true,
            Vec::new(),
            tolerances(),
        )
        .unwrap();

    assert!(brep.face(face).loops().is_empty());
    // 平面は閉じていないため、境界のない面にはならない
    let report = validator().validate(&brep);
    assert_eq!(report.faces_without_boundary, vec![face]);
    assert!(!report.is_valid());
}

#[test]
fn a_loop_that_does_not_close_is_reported() {
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let c = brep.add_vertex(Point3D::new(1.0, 1.0, 0.0));
    let ab = line_edge(&mut brep, a, b);
    let bc = line_edge(&mut brep, b, c);
    let surface = plane(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    );
    brep.add_face(
        surface,
        true,
        vec![vec![forward(ab), forward(bc)]],
        tolerances(),
    )
    .unwrap();

    let report = validator().validate(&brep);
    assert_eq!(report.disconnected_loops, vec![LoopIndex(0)]);
}

#[test]
fn an_edge_used_three_times_is_not_manifold() {
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let ab = line_edge(&mut brep, a, b);
    // 1 本の稜線のまわりに、3 つの三角形の面を付ける
    for apex in [
        Point3D::new(0.5, 1.0, 0.0),
        Point3D::new(0.5, -1.0, 0.0),
        Point3D::new(0.5, 0.0, 1.0),
    ] {
        let c = brep.add_vertex(apex);
        let bc = line_edge(&mut brep, b, c);
        let ca = line_edge(&mut brep, c, a);
        let surface = plane(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            apex,
        );
        brep.add_face(
            surface,
            true,
            vec![vec![forward(ab), forward(bc), forward(ca)]],
            tolerances(),
        )
        .unwrap();
    }

    let report = validator().validate(&brep);
    assert_eq!(report.non_manifold_edges, vec![ab]);
}

#[test]
fn elements_must_refer_to_elements_in_the_table() {
    let mut brep = Brep::<f64>::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    assert_eq!(
        brep.add_edge(a, VertexIndex(1), CurveRef::Line(line), (0.0, 1.0)),
        Err(BrepError::UnknownVertex)
    );
    let surface = plane(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    );
    assert_eq!(
        brep.add_face(
            surface.clone(),
            true,
            vec![vec![forward(EdgeIndex(0))]],
            tolerances()
        ),
        Err(BrepError::UnknownEdge)
    );
    assert_eq!(
        brep.add_face(surface, true, vec![Vec::new()], tolerances()),
        Err(BrepError::EmptyLoop)
    );
}

#[test]
fn an_edge_needs_a_non_empty_parameter_range() {
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    assert_eq!(
        brep.add_edge(a, b, CurveRef::Line(line), (1.0, 0.0)),
        Err(BrepError::InvalidParameterRange)
    );
}

#[test]
fn a_nurbs_edge_range_is_checked_against_the_curve_domain() {
    let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        1,
        vec![2.0, 2.0, 3.0, 3.0],
        &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)],
        None,
    )
    .unwrap();
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(2.0, 0.0, 0.0));

    assert_eq!(
        brep.add_edge(a, b, CurveRef::Nurbs(curve.clone()), (1.0, 2.5)),
        Err(BrepError::InvalidParameterRange)
    );
    let edge = brep
        .add_edge(a, b, CurveRef::Nurbs(curve), (2.0, 3.0))
        .unwrap();
    assert_eq!(brep.edge(edge).parameter_range(), (2.0, 3.0));
    assert!(validator().validate(&brep).is_valid());
}

#[test]
fn a_nurbs_edge_range_slightly_outside_the_domain_is_clamped() {
    let u_min = 1_000_000.0;
    let u_max = 1_000_001.0;
    let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        1,
        vec![u_min, u_min, u_max, u_max],
        &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)],
        None,
    )
    .unwrap();
    let tolerance = CurveRef::<f64>::nurbs_parameter_tolerance_for_domain(u_min, u_max);
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(2.0, 0.0, 0.0));

    let edge = brep
        .add_edge(
            a,
            b,
            CurveRef::Nurbs(curve),
            (u_min - tolerance * 0.5, u_max + tolerance * 0.5),
        )
        .unwrap();

    assert_eq!(brep.edge(edge).parameter_range(), (u_min, u_max));
}

#[test]
fn a_vertex_off_the_curve_is_reported() {
    // 直線（z = 0）の区間の端から 0.1 離れた頂点
    let line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(2.0, 0.0, 0.0),
    )
    .unwrap();
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.1));
    let b = brep.add_vertex(Point3D::new(2.0, 0.0, 0.0));
    let edge = brep
        .add_edge(a, b, CurveRef::Line(line), (0.0, 2.0))
        .unwrap();

    assert_eq!(validator().validate(&brep).edges_off_vertices, vec![edge]);
    let loose = TopologyValidator::new(TopologyToleranceSettings::new(0.11));
    assert!(loose.validate(&brep).is_valid());
}

#[test]
fn a_nurbs_edge_may_use_part_of_the_curve() {
    let curve = <TopoNurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        1,
        vec![2.0, 2.0, 3.0, 3.0],
        &[Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 0.0, 0.0)],
        None,
    )
    .unwrap();
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    brep.add_edge(a, b, CurveRef::Nurbs(curve), (2.0, 2.5))
        .unwrap();

    assert!(validator().validate(&brep).is_valid());
}

#[test]
fn two_uses_in_the_same_direction_are_reported() {
    // 2 つの三角形が、共有する稜線を同じ向きに使う（面の向きがそろっていない）
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let c = brep.add_vertex(Point3D::new(1.0, 1.0, 0.0));
    let d = brep.add_vertex(Point3D::new(0.0, 1.0, 1.0));
    let ab = line_edge(&mut brep, a, b);
    let bc = line_edge(&mut brep, b, c);
    let ca = line_edge(&mut brep, c, a);
    let ad = line_edge(&mut brep, a, d);
    let dc = line_edge(&mut brep, d, c);
    let points = |index: VertexIndex| brep.vertex(index).point();
    let first_plane = plane(points(a), points(b), points(c));
    let second_plane = plane(points(c), points(a), points(d));
    brep.add_face(
        first_plane,
        true,
        vec![vec![forward(ab), forward(bc), forward(ca)]],
        tolerances(),
    )
    .unwrap();
    brep.add_face(
        second_plane,
        true,
        vec![vec![forward(ca), forward(ad), forward(dc)]],
        tolerances(),
    )
    .unwrap();

    let report = validator().validate(&brep);
    assert_eq!(report.inconsistently_oriented_edges, vec![ca]);
    assert!(!report.is_valid());
}

/// XY 平面（法線 +Z）の上の三角形の面を、頂点 `order` の順に回るループで足す
fn triangle_face(order: [usize; 3], same_sense: bool) -> (Brep<f64>, FaceIndex) {
    let points = [
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0),
        Point3D::new(0.0, 1.0, 0.0),
    ];
    let mut brep = Brep::new();
    let vertices = points.map(|point| brep.add_vertex(point));
    let uses = (0..3)
        .map(|side| {
            forward(line_edge(
                &mut brep,
                vertices[order[side]],
                vertices[order[(side + 1) % 3]],
            ))
        })
        .collect();
    let face = brep
        .add_face(
            plane(points[0], points[1], points[2]),
            same_sense,
            vec![uses],
            tolerances(),
        )
        .unwrap();
    (brep, face)
}

#[test]
fn a_clockwise_outer_loop_seen_from_the_normal_is_reported() {
    // 法線 +Z の側から見て反時計回り（面がループの左）なら、向きの規則に合う
    let (brep, _) = triangle_face([0, 1, 2], true);
    assert!(validator().validate(&brep).is_valid());

    // 時計回り（面がループの右）は、規則に合わない
    let (brep, face) = triangle_face([0, 2, 1], true);
    let report = validator().validate(&brep);
    assert_eq!(report.misoriented_faces, vec![face]);
    assert!(!report.is_valid());

    // 面の外向きの法線が曲面の法線と逆（-Z）なら、時計回りが規則に合う
    let (brep, _) = triangle_face([0, 2, 1], false);
    assert!(validator().validate(&brep).is_valid());
}

#[test]
fn an_inner_loop_must_turn_the_other_way() {
    let square = |brep: &mut Brep<f64>, size: f64, offset: f64, counterclockwise: bool| {
        let mut corners = [
            Point3D::new(offset, offset, 0.0),
            Point3D::new(offset + size, offset, 0.0),
            Point3D::new(offset + size, offset + size, 0.0),
            Point3D::new(offset, offset + size, 0.0),
        ];
        if !counterclockwise {
            corners.reverse();
        }
        let vertices: Vec<VertexIndex> = corners
            .iter()
            .map(|&point| brep.add_vertex(point))
            .collect();
        (0..4)
            .map(|side| forward(line_edge(brep, vertices[side], vertices[(side + 1) % 4])))
            .collect::<Vec<_>>()
    };
    let surface = || {
        plane(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        )
    };
    for (hole_counterclockwise, valid) in [(false, true), (true, false)] {
        let mut brep = Brep::new();
        let outer = square(&mut brep, 4.0, 0.0, true);
        let hole = square(&mut brep, 1.0, 1.0, hole_counterclockwise);
        brep.add_face(surface(), true, vec![outer, hole], tolerances())
            .unwrap();
        assert_eq!(validator().validate(&brep).is_valid(), valid);
    }
}

#[test]
fn a_cylinder_band_has_the_upward_loop_below() {
    // 円柱の帯（z = 0 と z = 1 の円の間）: 下の円は u の正の向き、上の円は負の向きに回る
    let surface = SurfaceRef::Cylinder(
        TopoCylindricalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap(),
    );
    for (bottom_forward, valid) in [(true, true), (false, false)] {
        let mut brep = Brep::new();
        let bottom = circle_edge(&mut brep, horizontal_circle(0.0, 1.0));
        let top = circle_edge(&mut brep, horizontal_circle(1.0, 1.0));
        let loops = vec![
            vec![EdgeUse {
                edge: bottom,
                forward: bottom_forward,
            }],
            vec![EdgeUse {
                edge: top,
                forward: !bottom_forward,
            }],
        ];
        let face = brep
            .add_face(surface.clone(), true, loops, tolerances())
            .unwrap();
        // 1 周の円のパラメータ曲線は、周期で折り返さずに u が 2π 進む
        let bottom_curve = &brep
            .face_loop(brep.face(face).loops()[0])
            .parameter_curves()[0];
        let (start, end) = (
            bottom_curve.point_at(0.0).unwrap(),
            bottom_curve.point_at(TAU).unwrap(),
        );
        assert!(((end.x() - start.x()).abs() - TAU).abs() < 1.0e-9);
        assert_eq!(validator().validate(&brep).is_valid(), valid);
    }
}

#[test]
fn a_cone_face_with_one_loop_lies_on_the_apex_side() {
    // 半頂角 45° の円錐（z = 0 で半径 1、頂点は z = -1）の、z = 0 の円から頂点までの面
    let surface = SurfaceRef::Cone(
        TopoConicalSurface3D::new_standard(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
            Angle::from_radians(FRAC_PI_4),
        )
        .unwrap(),
    );
    // 面は円より下（v の小さい側）なので、ループは u の負の向き（円を逆向き）に回る
    for (forward, valid) in [(false, true), (true, false)] {
        let mut brep = Brep::new();
        let rim = circle_edge(&mut brep, horizontal_circle(0.0, 1.0));
        brep.add_face(
            surface.clone(),
            true,
            vec![vec![EdgeUse { edge: rim, forward }]],
            tolerances(),
        )
        .unwrap();
        assert_eq!(validator().validate(&brep).is_valid(), valid);
    }
}

#[test]
fn a_sphere_cap_may_lie_on_either_side_of_its_loop() {
    // 球の緯線の円 1 つで囲む面は、北の側も南の側も閉じた面になる
    let surface = SurfaceRef::Sphere(
        TopoSphericalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 2.0).unwrap(),
    );
    let height: f64 = 1.0;
    let radius = (4.0 - height * height).sqrt();
    for forward in [true, false] {
        let mut brep = Brep::new();
        let latitude = circle_edge(&mut brep, horizontal_circle(height, radius));
        brep.add_face(
            surface.clone(),
            true,
            vec![vec![EdgeUse {
                edge: latitude,
                forward,
            }]],
            tolerances(),
        )
        .unwrap();
        assert!(validator().validate(&brep).is_valid());
    }
}

#[test]
fn an_edge_without_an_exact_parameter_curve_is_refused() {
    // 円柱の軸に傾いた円は、円柱の上の正確なパラメータ曲線にならない（当てはめは別に行う）
    let surface = SurfaceRef::Cylinder(
        TopoCylindricalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap(),
    );
    let tilted = TopoCircle3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        Direction3D::from_vector(crate::Vector3D::new(0.0, 1.0, 1.0)).unwrap(),
        1.0,
    )
    .unwrap();
    let mut brep = Brep::new();
    let edge = circle_edge(&mut brep, tilted);

    assert_eq!(
        brep.add_face(surface, true, vec![vec![forward(edge)]], tolerances()),
        Err(BrepError::NoParameterCurve)
    );
}

/// 円の上の点 `point` の、円のパラメータ（角度）
fn angle_on(circle: &TopoCircle3D<f64>, point: Point3D<f64>) -> f64 {
    use geo_contracts::Circle3DProperties;
    let offset = point - circle.center();
    let x = circle.ref_direction().as_vector();
    let y = circle.axis().as_vector().cross(x);
    offset.dot(y).atan2(offset.dot(x))
}

/// 円 `circle` の上の、頂点 `from` から `to` までの短い方の円弧の稜線を足し、`from` から `to` へ進む使用を返す
fn arc_use(
    brep: &mut Brep<f64>,
    circle: TopoCircle3D<f64>,
    from: VertexIndex,
    to: VertexIndex,
) -> EdgeUse {
    let start = angle_on(&circle, brep.vertex(from).point());
    let end = angle_on(&circle, brep.vertex(to).point());
    let mut delta = end - start;
    if delta > PI {
        delta -= TAU;
    } else if delta <= -PI {
        delta += TAU;
    }
    if delta > 0.0 {
        let edge = brep
            .add_edge(from, to, CurveRef::Circle(circle), (start, start + delta))
            .unwrap();
        forward(edge)
    } else {
        let edge = brep
            .add_edge(to, from, CurveRef::Circle(circle), (end, end - delta))
            .unwrap();
        backward(edge)
    }
}

#[test]
fn a_circle_edge_may_not_go_around_more_than_once() {
    let mut brep = Brep::new();
    let circle = horizontal_circle(0.0, 1.0);
    let vertex = brep.add_vertex(circle.point_at_parameter(0.0));

    assert_eq!(
        brep.add_edge(vertex, vertex, CurveRef::Circle(circle), (0.0, 2.0 * TAU)),
        Err(BrepError::InvalidParameterRange)
    );
}

#[test]
fn a_sphere_lune_wider_than_half_a_turn_turns_the_right_way_at_the_poles() {
    // 南極から北極への経線（u = 3.5）と、北極から南極への経線（u = 0.1）で囲む、幅 3.4（> π）の月形の面。
    // ループを逆にした、残りの幅 2.88 の月形の面も正しい
    let surface = SurfaceRef::Sphere(
        TopoSphericalSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 1.0).unwrap(),
    );
    let meridian = |u: f64| {
        TopoCircle3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Direction3D::from_vector(crate::Vector3D::new(-u.sin(), u.cos(), 0.0)).unwrap(),
            1.0,
        )
        .unwrap()
    };
    for reversed in [false, true] {
        let mut brep = Brep::new();
        let south = brep.add_vertex(Point3D::new(0.0, 0.0, -1.0));
        let north = brep.add_vertex(Point3D::new(0.0, 0.0, 1.0));
        let mut half = |u: f64, from: VertexIndex, to: VertexIndex| {
            // 赤道の点を通る、極から極までの半周
            let circle = meridian(u);
            let equator = angle_on(&circle, Point3D::new(u.cos(), u.sin(), 0.0));
            let range = (equator - PI / 2.0, equator + PI / 2.0);
            let starts_at_north = circle.point_at_parameter(range.0).z() > 0.0;
            let (start, end) = if starts_at_north {
                (north, south)
            } else {
                (south, north)
            };
            let edge = brep
                .add_edge(start, end, CurveRef::Circle(circle), range)
                .unwrap();
            EdgeUse {
                edge,
                forward: start == from && end == to,
            }
        };
        let loop_uses = if reversed {
            vec![half(0.1, south, north), half(3.5, north, south)]
        } else {
            vec![half(3.5, south, north), half(0.1, north, south)]
        };
        brep.add_face(surface.clone(), true, vec![loop_uses], tolerances())
            .unwrap();
        assert!(
            validator().validate(&brep).is_valid(),
            "reversed = {reversed}"
        );
    }
}

#[test]
fn a_hole_in_a_torus_may_be_the_only_loop() {
    // トーラスの (u, v) の四角（u = 0〜1、v = -0.3〜0.3）を囲むループ。反時計回りなら四角の面、
    // 時計回りなら、四角を除いたトーラスの残りの面で、どちらも正しい
    let torus = TopoTorusSurface3D::new_standard(Point3D::new(0.0, 0.0, 0.0), 3.0, 1.0).unwrap();
    let surface = SurfaceRef::Torus(torus.clone());
    let parallel = |v: f64| {
        TopoCircle3D::new_xy_plane(Point3D::new(0.0, 0.0, v.sin()), 3.0 + v.cos()).unwrap()
    };
    let tube = |u: f64| {
        TopoCircle3D::new(
            Point3D::new(3.0 * u.cos(), 3.0 * u.sin(), 0.0),
            Direction3D::from_vector(crate::Vector3D::new(-u.sin(), u.cos(), 0.0)).unwrap(),
            1.0,
        )
        .unwrap()
    };
    for counterclockwise in [true, false] {
        let mut brep = Brep::new();
        let corner =
            |brep: &mut Brep<f64>, u: f64, v: f64| brep.add_vertex(torus.point_at_uv(u, v));
        let a = corner(&mut brep, 0.0, -0.3);
        let b = corner(&mut brep, 1.0, -0.3);
        let c = corner(&mut brep, 1.0, 0.3);
        let d = corner(&mut brep, 0.0, 0.3);
        let mut uses = vec![
            arc_use(&mut brep, parallel(-0.3), a, b),
            arc_use(&mut brep, tube(1.0), b, c),
            arc_use(&mut brep, parallel(0.3), c, d),
            arc_use(&mut brep, tube(0.0), d, a),
        ];
        if !counterclockwise {
            uses.reverse();
            for edge_use in &mut uses {
                edge_use.forward = !edge_use.forward;
            }
        }
        brep.add_face(surface.clone(), true, vec![uses], tolerances())
            .unwrap();
        assert!(
            validator().validate(&brep).is_valid(),
            "counterclockwise = {counterclockwise}"
        );
    }
}

#[test]
fn a_loop_through_the_cone_apex_is_checked_from_any_starting_use() {
    // 半頂角 45° の円錐（z = 0 で半径 1、頂点は z = -1）の、頂点までの母線 2 本と底の円弧で囲む、幅 `width` の扇形
    let surface = SurfaceRef::Cone(
        TopoConicalSurface3D::new_standard(
            Point3D::new(0.0, 0.0, 0.0),
            1.0,
            Angle::from_radians(FRAC_PI_4),
        )
        .unwrap(),
    );
    let rim = horizontal_circle(0.0, 1.0);
    // 幅が π を超える扇形も確かめる
    for width in [1.0, 5.0] {
        for reversed in [false, true] {
            // ループの初めの使用を、頂点から始まらない 2 通りと、頂点から始まる 1 通りにずらす
            for rotation in 0..3 {
                let mut brep = Brep::new();
                let rim0 = brep.add_vertex(rim.point_at_parameter(0.0));
                let rim1 = brep.add_vertex(rim.point_at_parameter(width));
                let apex = brep.add_vertex(Point3D::new(0.0, 0.0, -1.0));
                let down = line_edge(&mut brep, rim0, apex);
                let up = line_edge(&mut brep, apex, rim1);
                let arc = brep
                    .add_edge(rim0, rim1, CurveRef::Circle(rim), (0.0, width))
                    .unwrap();
                // (u, v) で反時計回り: 母線を下り、母線を上り、底の円弧を戻る。逆にしたものは規則に合わない
                let mut uses = if reversed {
                    vec![forward(arc), backward(up), backward(down)]
                } else {
                    vec![forward(down), forward(up), backward(arc)]
                };
                uses.rotate_left(rotation);
                let face = brep
                    .add_face(surface.clone(), true, vec![uses], tolerances())
                    .unwrap();
                let report = validator().validate(&brep);
                let case = format!("width = {width}, reversed = {reversed}, rotation = {rotation}");
                assert_eq!(report.is_valid(), !reversed, "{case}");
                if reversed {
                    assert_eq!(report.misoriented_faces, vec![face], "{case}");
                }
            }
        }
    }
}
