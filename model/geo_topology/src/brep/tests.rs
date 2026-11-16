use super::*;
use crate::{TopoInfiniteLine3D, TopoLineSegment3D, TopologyToleranceSettings, TopologyValidator};
use geo_contracts::{
    InfiniteLine3DConstructor, LineSegment3DConstructor, LineSegment3DSupportLine,
    NurbsCurve3DConstructor, Plane3DConstructor,
};

fn validator() -> TopologyValidator<f64> {
    TopologyValidator::new(TopologyToleranceSettings::new(1.0e-9))
}

/// 2 つの頂点の間の線分の稜線を足す
fn line_edge(brep: &mut Brep<f64>, start: VertexIndex, end: VertexIndex) -> EdgeIndex {
    let line =
        TopoLineSegment3D::new(brep.vertex(start).point(), brep.vertex(end).point()).unwrap();
    brep.add_edge(
        start,
        end,
        CurveRef::Line(line),
        (line.start_param(), line.end_param()),
    )
    .unwrap()
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
        )
        .unwrap(),
        // 側面（-y）: 0 → 1 → 3
        brep.add_face(
            plane(p[0], p[1], p[3]),
            true,
            vec![vec![forward(e01), forward(e13), backward(e03)]],
        )
        .unwrap(),
        // 側面（-x）: 0 → 3 → 2
        brep.add_face(
            plane(p[0], p[3], p[2]),
            true,
            vec![vec![forward(e03), backward(e23), backward(e02)]],
        )
        .unwrap(),
        // 斜めの面: 1 → 2 → 3
        brep.add_face(
            plane(p[1], p[2], p[3]),
            true,
            vec![vec![forward(e12), forward(e23), backward(e13)]],
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
        )
        .unwrap();
    let second = brep
        .add_face(
            second_plane,
            true,
            vec![vec![backward(ca), forward(cd), forward(da)]],
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
    brep.add_face(surface, true, vec![vec![forward(ab), forward(bc)]])
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
    let line =
        TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();

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
        brep.add_face(surface.clone(), true, vec![vec![forward(EdgeIndex(0))]]),
        Err(BrepError::UnknownEdge)
    );
    assert_eq!(
        brep.add_face(surface, true, vec![Vec::new()]),
        Err(BrepError::EmptyLoop)
    );
}

#[test]
fn an_edge_needs_a_non_empty_parameter_range() {
    let mut brep = Brep::new();
    let a = brep.add_vertex(Point3D::new(0.0, 0.0, 0.0));
    let b = brep.add_vertex(Point3D::new(1.0, 0.0, 0.0));
    let line =
        TopoLineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0)).unwrap();

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
fn edge_checks_separate_vertex_binding_from_curve_endpoints() {
    // 母線（z = 0）から離れた拘束の端点（z = 0.1）に頂点を置いた線分
    let support_line = TopoInfiniteLine3D::from_two_points(
        Point3D::new(0.0, 0.0, 0.0),
        Point3D::new(2.0, 0.0, 0.0),
    )
    .unwrap();
    let start = Point3D::new(0.0, 0.0, 0.1);
    let end = Point3D::new(2.0, 0.0, 0.1);
    let line = TopoLineSegment3D::from_support_line_and_constraint_points(support_line, start, end)
        .unwrap();
    let mut brep = Brep::new();
    let a = brep.add_vertex(start);
    let b = brep.add_vertex(end);
    let edge = brep
        .add_edge(a, b, CurveRef::Line(line), (0.0, 2.0))
        .unwrap();

    let report = validator().validate_edge(&brep, edge);
    assert!(report.binding_consistent);
    assert!(!report.ideal_endpoint_consistent);
    assert!(!report.evaluated_endpoint_consistent);
    // 3 つに分けた許容誤差（1 つ 0.1 強）なら、離れ 0.1 を受け入れる
    let loose = TopologyValidator::new(TopologyToleranceSettings::new(0.31));
    assert!(loose.validate_edge(&brep, edge).is_valid());
    let tight = TopologyValidator::new(TopologyToleranceSettings::new(0.29));
    assert!(!tight.validate_edge(&brep, edge).is_valid());
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
    let edge = brep
        .add_edge(a, b, CurveRef::Nurbs(curve), (2.0, 2.5))
        .unwrap();

    assert!(validator().validate_edge(&brep, edge).is_valid());
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
    )
    .unwrap();
    brep.add_face(
        second_plane,
        true,
        vec![vec![forward(ca), forward(ad), forward(dc)]],
    )
    .unwrap();

    let report = validator().validate(&brep);
    assert_eq!(report.inconsistently_oriented_edges, vec![ca]);
    assert!(!report.is_valid());
}
