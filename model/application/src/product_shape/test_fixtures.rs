//! 試験で使う形状

use geo_topology::Brep;

/// 試験の B-rep: 原点と各軸の上の点を頂点にした四面体の、閉じたシェルの立体
pub(crate) fn tetrahedron_brep() -> Brep<f64> {
    use geo_contracts::{InfiniteLine3DConstructor, Plane3DConstructor};
    use geo_shapes::{CurveRef, SurfaceRef};
    use geo_topology::{EdgeUse, TopoInfiniteLine3D, TopoPlane3D, TopologyToleranceSettings};
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
                        geo_shapes::CurveRef::Line(line),
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
            .add_face(geo_shapes::SurfaceRef::Plane(plane), true, loops)
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
        geo_shapes::CurveRef::Circle(circle),
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
    let side = geo_shapes::SurfaceRef::Cylinder(
        geo_topology::TopoCylindricalSurface3D::new_standard(origin, 10.0).unwrap(),
    );
    let plane = |z: f64, up: bool| {
        let normal = geo_primitives::Direction3D::from_vector(geo_topology::Vector3D::new(
            0.0,
            0.0,
            if up { 1.0 } else { -1.0 },
        ))
        .unwrap();
        geo_shapes::SurfaceRef::Plane(
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
        .add_face(geo_shapes::SurfaceRef::Sphere(sphere), true, Vec::new())
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
        .add_face(geo_shapes::SurfaceRef::Torus(torus), true, Vec::new())
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
    let hole = geo_shapes::SurfaceRef::Cylinder(
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

/// 試験の B-rep: 半径 10 の球で、南極から北極への半円の継ぎ目の稜線を持つ 1 つの面（ループは継ぎ目の行きと帰り）
pub(crate) fn seamed_sphere_brep() -> Brep<f64> {
    use geo_contracts::{Circle3DConstructor, SphericalSurface3DConstructor};
    let mut brep = Brep::new(geo_topology::TopologyToleranceSettings::new(1.0e-9)).unwrap();
    let origin = geo_topology::Point3D::new(0.0, 0.0, 0.0);
    let south = brep
        .add_vertex(geo_topology::Point3D::new(0.0, 0.0, -10.0), 0.0)
        .unwrap();
    let north = brep
        .add_vertex(geo_topology::Point3D::new(0.0, 0.0, 10.0), 0.0)
        .unwrap();
    let direction = |x: f64, y: f64, z: f64| {
        geo_primitives::Direction3D::from_vector(geo_topology::Vector3D::new(x, y, z)).unwrap()
    };
    // x と z の平面の円（角度 -π/2 が南極、π/2 が北極、0 が (10, 0, 0)）
    let meridian = geo_topology::TopoCircle3D::from_frame(
        origin,
        direction(0.0, -1.0, 0.0),
        direction(1.0, 0.0, 0.0),
        10.0,
    )
    .unwrap();
    let seam = brep
        .add_edge(
            south,
            north,
            geo_shapes::CurveRef::Circle(meridian),
            (-std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2),
            0.0,
        )
        .unwrap();
    let sphere = geo_topology::TopoSphericalSurface3D::new_standard(origin, 10.0).unwrap();
    let face = brep
        .add_face(
            geo_shapes::SurfaceRef::Sphere(sphere),
            true,
            vec![vec![edge_use(seam, true), edge_use(seam, false)]],
        )
        .unwrap();
    let shell = brep.add_shell(vec![face]).unwrap();
    brep.add_solid(vec![shell]).unwrap();
    assert!(brep.is_shell_closed(shell));
    brep
}

/// 試験の B-rep: 頂点 (0, 0, -10)・底の円の半径 10（z = 0）の円錐で、底の円の頂点から円錐の頂点への継ぎ目の
/// 直線の稜線を持つもの（体積 1000π / 3）
pub(crate) fn cone_brep() -> Brep<f64> {
    use geo_contracts::{
        ConicalSurface3DConstructor, InfiniteLine3DConstructor, Plane3DConstructor,
    };
    let mut brep = Brep::new(geo_topology::TopologyToleranceSettings::new(1.0e-9)).unwrap();
    let base = circle_edge(&mut brep, [0.0, 0.0, 0.0], 10.0);
    // 継ぎ目は、底の円の頂点から円錐の頂点へ
    let rim_vertex = brep.edge(base).start();
    let rim = brep.vertex(rim_vertex).point();
    let apex_point = geo_topology::Point3D::new(0.0, 0.0, -10.0);
    let apex = brep.add_vertex(apex_point, 0.0).unwrap();
    let line = geo_topology::TopoInfiniteLine3D::from_two_points(rim, apex_point).unwrap();
    let seam = brep
        .add_edge(
            rim_vertex,
            apex,
            geo_shapes::CurveRef::Line(line),
            (0.0, rim.distance_to(apex_point)),
            0.0,
        )
        .unwrap();
    let cone = geo_topology::TopoConicalSurface3D::new_standard(
        geo_topology::Point3D::new(0.0, 0.0, 0.0),
        10.0,
        geo_primitives::Angle::from_degrees(45.0),
    )
    .unwrap();
    let up = geo_primitives::Direction3D::from_vector(geo_topology::Vector3D::new(0.0, 0.0, 1.0))
        .unwrap();
    let top = geo_topology::TopoPlane3D::from_point_and_normal(
        geo_topology::Point3D::new(0.0, 0.0, 0.0),
        up,
    )
    .unwrap();
    // 側面: 底の円を u の減る向きに、継ぎ目を頂点へ下り、同じ継ぎ目を上る
    let faces = vec![
        brep.add_face(
            geo_shapes::SurfaceRef::Cone(cone),
            true,
            vec![vec![
                edge_use(base, false),
                edge_use(seam, true),
                edge_use(seam, false),
            ]],
        )
        .unwrap(),
        brep.add_face(
            geo_shapes::SurfaceRef::Plane(top),
            true,
            vec![vec![edge_use(base, true)]],
        )
        .unwrap(),
    ];
    let shell = brep.add_shell(faces).unwrap();
    brep.add_solid(vec![shell]).unwrap();
    assert!(brep.is_shell_closed(shell));
    brep
}

/// 試験の B-rep の、NURBS 曲面の山の高さを決める内側の制御点の z
pub(crate) const BUMP_HEIGHT: f64 = 3.0;

/// 試験の B-rep: 一辺 10 の正方形（z = 0）を底にし、上を 3 次の NURBS 曲面の山で閉じた立体。曲面は
/// (10 u, 10 v, 9 h u (1 − u) v (1 − v))（h は `BUMP_HEIGHT`。外側の制御点の z が 0、内側の 4 点の z が h）で、
/// 縁は z = 0 の直線になる（体積 25 h）
pub(crate) fn bump_brep() -> Brep<f64> {
    use geo_contracts::NurbsSurface3DConstructor;
    let mut solid = Solid::new(&[
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [0.0, 10.0, 0.0],
    ]);
    solid.plane_face(&[0, 3, 2, 1], Vec::new());
    let control_points: Vec<Vec<_>> = (0..4)
        .map(|i| {
            (0..4)
                .map(|j| {
                    let inner = (1..3).contains(&i) && (1..3).contains(&j);
                    geo_topology::Point3D::new(
                        10.0 * i as f64 / 3.0,
                        10.0 * j as f64 / 3.0,
                        if inner { BUMP_HEIGHT } else { 0.0 },
                    )
                })
                .collect()
        })
        .collect();
    let knots = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    let surface =
        geo_topology::TopoNurbsSurface3D::new(3, 3, knots.clone(), knots, &control_points, None)
            .unwrap();
    let outer = vec![
        solid.line(0, 1),
        solid.line(1, 2),
        solid.line(2, 3),
        solid.line(3, 0),
    ];
    let face = solid
        .brep
        .add_face(geo_shapes::SurfaceRef::Nurbs(surface), true, vec![outer])
        .unwrap();
    solid.faces.push(face);
    solid.finish()
}
