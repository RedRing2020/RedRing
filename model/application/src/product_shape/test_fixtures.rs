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

/// 試験の形状: 平面の面の多面体の頂点 `points`（面は、外から見て反時計回りの頂点の並び）
struct Solid {
    brep: Brep<f64>,
    vertices: Vec<geo_topology::VertexIndex>,
    lines: std::collections::HashMap<(usize, usize), geo_topology::EdgeIndex>,
    points: Vec<geo_topology::Point3D<f64>>,
    faces: Vec<geo_topology::FaceIndex>,
}

impl Solid {
    fn new(points: &[[f64; 3]]) -> Self {
        let mut brep = Brep::new(geo_topology::TopologyToleranceSettings::new(1.0e-9)).unwrap();
        let points: Vec<_> = points
            .iter()
            .map(|&[x, y, z]| geo_topology::Point3D::new(x, y, z))
            .collect();
        let vertices = points
            .iter()
            .map(|&point| brep.add_vertex(point, 0.0).unwrap())
            .collect();
        Self {
            brep,
            vertices,
            lines: std::collections::HashMap::new(),
            points,
            faces: Vec::new(),
        }
    }

    /// 頂点 `a` から `b` への直線の稜線の使用（稜線は、番号の小さい頂点から大きい頂点の向き）
    fn line(&mut self, a: usize, b: usize) -> geo_topology::EdgeUse {
        use geo_contracts::InfiniteLine3DConstructor;
        let key = (a.min(b), a.max(b));
        let edge = match self.lines.get(&key) {
            Some(&edge) => edge,
            None => {
                let (start, end) = (self.points[key.0], self.points[key.1]);
                let line = geo_topology::TopoInfiniteLine3D::from_two_points(start, end).unwrap();
                let edge = self
                    .brep
                    .add_edge(
                        self.vertices[key.0],
                        self.vertices[key.1],
                        geo_topology::CurveRef::Line(line),
                        (0.0, start.distance_to(end)),
                        0.0,
                    )
                    .unwrap();
                self.lines.insert(key, edge);
                edge
            }
        };
        geo_topology::EdgeUse {
            edge,
            forward: a < b,
        }
    }

    /// 頂点 `corners` の平面の面（外から見て反時計回り）と、内側のループ `holes`
    fn plane_face(&mut self, corners: &[usize], holes: Vec<Vec<geo_topology::EdgeUse>>) {
        use geo_contracts::Plane3DConstructor;
        let outer: Vec<_> = (0..corners.len())
            .map(|index| self.line(corners[index], corners[(index + 1) % corners.len()]))
            .collect();
        let [a, b, c] = [0, 1, 2].map(|index| self.points[corners[index]]);
        let plane = geo_topology::TopoPlane3D::from_three_points(a, b, c).unwrap();
        let mut loops = vec![outer];
        loops.extend(holes);
        let face = self
            .brep
            .add_face(geo_topology::SurfaceRef::Plane(plane), true, loops)
            .unwrap();
        self.faces.push(face);
    }

    fn finish(mut self) -> Brep<f64> {
        let shell = self.brep.add_shell(self.faces).unwrap();
        self.brep.add_solid(vec![shell]).unwrap();
        assert!(self.brep.is_shell_closed(shell));
        self.brep
    }
}

/// 軸が z の方向の、中心 `center`・半径 `radius` の円の稜線（頂点は角度 0 の点）
fn circle_edge(brep: &mut Brep<f64>, center: [f64; 3], radius: f64) -> geo_topology::EdgeIndex {
    use geo_contracts::{Circle3DConstructor, Circle3DEvaluation};
    let [x, y, z] = center;
    let circle =
        geo_topology::TopoCircle3D::new_xy_plane(geo_topology::Point3D::new(x, y, z), radius)
            .unwrap();
    let vertex = brep
        .add_vertex(circle.point_at_parameter(0.0), 0.0)
        .unwrap();
    brep.add_edge(
        vertex,
        vertex,
        geo_topology::CurveRef::Circle(circle),
        (0.0, std::f64::consts::TAU),
        0.0,
    )
    .unwrap()
}

fn edge_use(edge: geo_topology::EdgeIndex, forward: bool) -> geo_topology::EdgeUse {
    geo_topology::EdgeUse { edge, forward }
}

/// 試験の B-rep: 一辺 10 の立方体（体積 1000）
pub(crate) fn cube_brep() -> Brep<f64> {
    let mut solid = Solid::new(&[
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, 10.0],
        [10.0, 0.0, 10.0],
        [10.0, 10.0, 10.0],
        [0.0, 10.0, 10.0],
    ]);
    for face in [
        [0, 3, 2, 1],
        [4, 5, 6, 7],
        [0, 1, 5, 4],
        [1, 2, 6, 5],
        [2, 3, 7, 6],
        [3, 0, 4, 7],
    ] {
        solid.plane_face(&face, Vec::new());
    }
    solid.finish()
}

/// 試験の B-rep: 半径 10・高さ 20 の円柱（底の中心が原点、軸が z）
pub(crate) fn cylinder_brep() -> Brep<f64> {
    use geo_contracts::{CylindricalSurface3DConstructor, Plane3DConstructor};
    let mut brep = Brep::new(geo_topology::TopologyToleranceSettings::new(1.0e-9)).unwrap();
    let bottom = circle_edge(&mut brep, [0.0, 0.0, 0.0], 10.0);
    let top = circle_edge(&mut brep, [0.0, 0.0, 20.0], 10.0);
    let origin = geo_topology::Point3D::new(0.0, 0.0, 0.0);
    let side = geo_topology::SurfaceRef::Cylinder(
        geo_topology::TopoCylindricalSurface3D::new_standard(origin, 10.0).unwrap(),
    );
    let plane = |z: f64, up: bool| {
        let normal = geo_primitives::Direction3D::from_vector(geo_topology::Vector3D::new(
            0.0,
            0.0,
            if up { 1.0 } else { -1.0 },
        ))
        .unwrap();
        geo_topology::SurfaceRef::Plane(
            geo_topology::TopoPlane3D::from_point_and_normal(
                geo_topology::Point3D::new(0.0, 0.0, z),
                normal,
            )
            .unwrap(),
        )
    };
    let faces = vec![
        brep.add_face(
            side,
            true,
            vec![vec![edge_use(bottom, true)], vec![edge_use(top, false)]],
        )
        .unwrap(),
        brep.add_face(plane(0.0, false), true, vec![vec![edge_use(bottom, false)]])
            .unwrap(),
        brep.add_face(plane(20.0, true), true, vec![vec![edge_use(top, true)]])
            .unwrap(),
    ];
    let shell = brep.add_shell(faces).unwrap();
    brep.add_solid(vec![shell]).unwrap();
    assert!(brep.is_shell_closed(shell));
    brep
}

/// 試験の B-rep: 半径 10 の球（ループのない 1 つの面）
pub(crate) fn sphere_brep() -> Brep<f64> {
    use geo_contracts::SphericalSurface3DConstructor;
    let mut brep = Brep::new(geo_topology::TopologyToleranceSettings::new(1.0e-9)).unwrap();
    let sphere = geo_topology::TopoSphericalSurface3D::new_standard(
        geo_topology::Point3D::new(0.0, 0.0, 0.0),
        10.0,
    )
    .unwrap();
    let face = brep
        .add_face(geo_topology::SurfaceRef::Sphere(sphere), true, Vec::new())
        .unwrap();
    let shell = brep.add_shell(vec![face]).unwrap();
    brep.add_solid(vec![shell]).unwrap();
    brep
}

/// 試験の B-rep: 中心の円の半径 20・管の半径 5 のトーラス（ループのない 1 つの面）
pub(crate) fn torus_brep() -> Brep<f64> {
    use geo_contracts::TorusSurface3DConstructor;
    let mut brep = Brep::new(geo_topology::TopologyToleranceSettings::new(1.0e-9)).unwrap();
    let torus = geo_topology::TopoTorusSurface3D::new_standard(
        geo_topology::Point3D::new(0.0, 0.0, 0.0),
        20.0,
        5.0,
    )
    .unwrap();
    let face = brep
        .add_face(geo_topology::SurfaceRef::Torus(torus), true, Vec::new())
        .unwrap();
    let shell = brep.add_shell(vec![face]).unwrap();
    brep.add_solid(vec![shell]).unwrap();
    brep
}

/// 試験の B-rep: 40 × 30 × 10 の板に、中心 (20, 15)・半径 5 の穴を z の方向に開けたもの
pub(crate) fn plate_with_hole_brep() -> Brep<f64> {
    use geo_contracts::CylindricalSurface3DConstructor;
    let mut solid = Solid::new(&[
        [0.0, 0.0, 0.0],
        [40.0, 0.0, 0.0],
        [40.0, 30.0, 0.0],
        [0.0, 30.0, 0.0],
        [0.0, 0.0, 10.0],
        [40.0, 0.0, 10.0],
        [40.0, 30.0, 10.0],
        [0.0, 30.0, 10.0],
    ]);
    let bottom = circle_edge(&mut solid.brep, [20.0, 15.0, 0.0], 5.0);
    let top = circle_edge(&mut solid.brep, [20.0, 15.0, 10.0], 5.0);
    // 外から見て、穴のループは時計回り（下の面は下から見るので、z のまわりの反時計回りの円をそのまま使う）
    solid.plane_face(&[0, 3, 2, 1], vec![vec![edge_use(bottom, true)]]);
    solid.plane_face(&[4, 5, 6, 7], vec![vec![edge_use(top, false)]]);
    for face in [[0, 1, 5, 4], [1, 2, 6, 5], [2, 3, 7, 6], [3, 0, 4, 7]] {
        solid.plane_face(&face, Vec::new());
    }
    // 穴の面の外向きの法線は軸の向き（円柱の法線と逆）
    let hole = geo_topology::SurfaceRef::Cylinder(
        geo_topology::TopoCylindricalSurface3D::new_standard(
            geo_topology::Point3D::new(20.0, 15.0, 0.0),
            5.0,
        )
        .unwrap(),
    );
    let face = solid
        .brep
        .add_face(
            hole,
            false,
            vec![vec![edge_use(bottom, false)], vec![edge_use(top, true)]],
        )
        .unwrap();
    solid.faces.push(face);
    solid.finish()
}
