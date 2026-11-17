//! 試験で使う形状

use geo_topology::Brep;

/// 試験の B-rep: 原点と各軸の上の点を頂点にした四面体の、閉じたシェルの立体
pub(crate) fn tetrahedron_brep() -> Brep<f64> {
    use geo_contracts::{InfiniteLine3DConstructor, Plane3DConstructor};
    use geo_topology::{
        CurveRef, EdgeUse, SurfaceRef, TopoInfiniteLine3D, TopoPlane3D, TopologyToleranceSettings,
    };
    let mut brep = Brep::new(TopologyToleranceSettings::new(1.0e-9)).unwrap();
    let points = [
        geo_topology::Point3D::new(0.0, 0.0, 0.0),
        geo_topology::Point3D::new(1.0, 0.0, 0.0),
        geo_topology::Point3D::new(0.0, 1.0, 0.0),
        geo_topology::Point3D::new(0.0, 0.0, 1.0),
    ];
    let vertices: Vec<_> = points
        .iter()
        .map(|&point| brep.add_vertex(point, 0.0).unwrap())
        .collect();
    let mut edge = |a: usize, b: usize| {
        let line = TopoInfiniteLine3D::from_two_points(points[a], points[b]).unwrap();
        brep.add_edge(
            vertices[a],
            vertices[b],
            CurveRef::Line(line),
            (0.0, points[a].distance_to(points[b])),
            0.0,
        )
        .unwrap()
    };
    let (e01, e02, e03, e12, e13, e23) = (
        edge(0, 1),
        edge(0, 2),
        edge(0, 3),
        edge(1, 2),
        edge(1, 3),
        edge(2, 3),
    );
    let uses = |list: &[(geo_topology::EdgeIndex, bool)]| {
        list.iter()
            .map(|&(edge, forward)| EdgeUse { edge, forward })
            .collect::<Vec<_>>()
    };
    let plane = |a: usize, b: usize, c: usize| {
        SurfaceRef::Plane(TopoPlane3D::from_three_points(points[a], points[b], points[c]).unwrap())
    };
    let faces = vec![
        brep.add_face(
            plane(0, 2, 1),
            true,
            vec![uses(&[(e02, true), (e12, false), (e01, false)])],
        )
        .unwrap(),
        brep.add_face(
            plane(0, 1, 3),
            true,
            vec![uses(&[(e01, true), (e13, true), (e03, false)])],
        )
        .unwrap(),
        brep.add_face(
            plane(0, 3, 2),
            true,
            vec![uses(&[(e03, true), (e23, false), (e02, false)])],
        )
        .unwrap(),
        brep.add_face(
            plane(1, 2, 3),
            true,
            vec![uses(&[(e12, true), (e23, true), (e13, false)])],
        )
        .unwrap(),
    ];
    let shell = brep.add_shell(faces).unwrap();
    brep.add_solid(vec![shell]).unwrap();
    brep
}
