use super::*;
use crate::geometry::convert;
use crate::record::Representation;
use geo_contracts::{Angle, ToleranceSettings};
use std::collections::HashMap;

/// 試験の全体の許容誤差
const TOLERANCE: f64 = 1.0e-9;

fn geometry_tolerances() -> ToleranceSettings<f64> {
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

    fn id(&mut self) -> u64 {
        let id = self.next;
        self.next += 1;
        id
    }

    fn geometry(&mut self, geometry: Geometry) -> u64 {
        let id = self.id();
        self.records.geometry.insert(id, geometry);
        id
    }

    fn topology(&mut self, topology: Topology) -> u64 {
        let id = self.id();
        self.records.topology.insert(id, topology);
        id
    }

    fn point(&mut self, [x, y, z]: [f64; 3]) -> u64 {
        self.geometry(Geometry::Point([x, y, z]))
    }

    fn direction(&mut self, direction: [f64; 3]) -> u64 {
        self.geometry(Geometry::Direction(direction))
    }

    fn placement(&mut self, origin: [f64; 3], axis: [f64; 3], reference: [f64; 3]) -> u64 {
        let location = self.point(origin);
        let axis = self.direction(axis);
        let ref_direction = self.direction(reference);
        self.geometry(Geometry::Placement {
            location,
            axis: Some(axis),
            ref_direction: Some(ref_direction),
        })
    }

    fn vertex(&mut self, point: [f64; 3]) -> u64 {
        let point = self.point(point);
        self.topology(Topology::Vertex { point })
    }

    fn line(&mut self, point: [f64; 3], direction: [f64; 3]) -> u64 {
        let point = self.point(point);
        let direction = self.direction(direction);
        let vector = self.geometry(Geometry::Vector {
            direction,
            magnitude: 1.0,
        });
        self.geometry(Geometry::Line { point, vector })
    }

    fn circle(
        &mut self,
        center: [f64; 3],
        axis: [f64; 3],
        reference: [f64; 3],
        radius: f64,
    ) -> u64 {
        let placement = self.placement(center, axis, reference);
        self.geometry(Geometry::Circle { placement, radius })
    }

    fn edge(&mut self, start: u64, end: u64, curve: u64, same_sense: bool) -> u64 {
        self.topology(Topology::Edge {
            start,
            end,
            curve,
            same_sense: Some(same_sense),
        })
    }

    /// 稜線と向きの組の並びの `FACE_BOUND`
    fn bound(&mut self, edges: &[(u64, bool)], orientation: bool) -> u64 {
        let edges = edges
            .iter()
            .map(|&(edge, orientation)| {
                self.topology(Topology::OrientedEdge {
                    edge,
                    orientation: Some(orientation),
                })
            })
            .collect();
        let bound = self.topology(Topology::EdgeLoop { edges });
        self.topology(Topology::FaceBound {
            bound,
            orientation: Some(orientation),
            outer: false,
        })
    }

    fn face(&mut self, bounds: Vec<u64>, surface: u64, same_sense: bool) -> u64 {
        self.topology(Topology::Face {
            bounds,
            surface: Some(surface),
            same_sense: Some(same_sense),
        })
    }

    /// 面 `faces` の閉じたシェルの立体を、表現の要素にする
    fn solid(&mut self, faces: Vec<u64>) -> u64 {
        let outer = self.topology(Topology::Shell {
            faces,
            closed: true,
        });
        let solid = self.topology(Topology::Solid {
            outer,
            voids: Vec::new(),
        });
        self.represent(solid);
        solid
    }

    /// 面 `faces` の開いたシェルを、表現の要素にする
    fn open_shell(&mut self, faces: Vec<u64>) -> u64 {
        let shell = self.topology(Topology::Shell {
            faces,
            closed: false,
        });
        self.represent(shell);
        shell
    }

    fn represent(&mut self, item: u64) {
        let id = self.id();
        self.records.representations.insert(
            id,
            Representation {
                name: String::new(),
                context: 0,
                items: vec![item],
            },
        );
    }

    fn build(&self) -> Topologies {
        let geometries = convert(&self.records, &geometry_tolerances());
        assert!(geometries.skipped.is_empty(), "{:?}", geometries.skipped);
        build(
            &self.records,
            &geometries,
            TopologyToleranceSettings::new(TOLERANCE),
            ImportToleranceSettings::default(),
        )
        .unwrap()
    }
}

/// 多面体（点 `points` と、外から見て反時計回りの頂点の番号の並びの面 `faces`）の記録
///
/// 稜線は、初めに出た面の向きで作る。`plane_sense` が `false` の面は、平面の法線を内向きにし、`same_sense`
/// を `.T.` のまま書く（向きの旗の誤り）。
struct Polyhedron {
    builder: Builder,
    faces: Vec<u64>,
    /// 頂点の組から、ファイルの稜線
    edges: HashMap<(usize, usize), u64>,
    item: u64,
}

fn polyhedron(points: &[[f64; 3]], faces: &[&[usize]], plane_sense: &[bool]) -> Polyhedron {
    let mut builder = Builder::new();
    let vertices: Vec<u64> = points.iter().map(|&point| builder.vertex(point)).collect();
    let mut edges: HashMap<(usize, usize), u64> = HashMap::new();
    let mut face_ids = Vec::new();
    for (face, corners) in faces.iter().enumerate() {
        let mut uses = Vec::new();
        for (position, &a) in corners.iter().enumerate() {
            let b = corners[(position + 1) % corners.len()];
            if let Some(&edge) = edges.get(&(b, a)) {
                uses.push((edge, false));
                continue;
            }
            let [ax, ay, az] = points[a];
            let [bx, by, bz] = points[b];
            let line = builder.line(points[a], [bx - ax, by - ay, bz - az]);
            let edge = builder.edge(vertices[a], vertices[b], line, true);
            edges.insert((a, b), edge);
            uses.push((edge, true));
        }
        let bound = builder.bound(&uses, true);
        // 外向きの法線（初めの 3 点から）
        let [p, q, r] = [corners[0], corners[1], corners[2]].map(|index| points[index]);
        let u = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
        let v = [r[0] - p[0], r[1] - p[1], r[2] - p[2]];
        let mut normal = [
            u[1] * v[2] - u[2] * v[1],
            u[2] * v[0] - u[0] * v[2],
            u[0] * v[1] - u[1] * v[0],
        ];
        if !plane_sense.get(face).copied().unwrap_or(true) {
            normal = normal.map(|value| -value);
        }
        let placement = builder.placement(p, normal, u);
        let plane = builder.geometry(Geometry::Plane { placement });
        face_ids.push(builder.face(vec![bound], plane, true));
    }
    let item = builder.solid(face_ids.clone());
    Polyhedron {
        builder,
        faces: face_ids,
        edges,
        item,
    }
}

const CUBE_POINTS: [[f64; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [1.0, 1.0, 1.0],
    [0.0, 1.0, 1.0],
];

const CUBE_FACES: [&[usize]; 6] = [
    &[0, 3, 2, 1],
    &[4, 5, 6, 7],
    &[0, 1, 5, 4],
    &[1, 2, 6, 5],
    &[2, 3, 7, 6],
    &[3, 0, 4, 7],
];

/// 閉じたシェル 1 つの立体ができ、どの面も向きの規則に合い、表の確かめを通ることを確かめる
fn assert_closed_solid(topologies: &Topologies, faces: usize) -> &Built {
    assert!(topologies.skipped.is_empty(), "{:?}", topologies.skipped);
    assert_eq!(topologies.items.len(), 1);
    let built = &topologies.items[0];
    assert_eq!(built.faces.len(), faces);
    assert!(built.unoriented_faces.is_empty());
    assert!(built.open_shells.is_empty());
    assert_eq!(built.shells.len(), 1);
    assert!(built.brep.is_shell_closed(built.shells[0]));
    let solid = built.solid.expect("a solid");
    assert_eq!(built.brep.solid(solid).shells(), built.shells.as_slice());
    let report = built.brep.validate();
    assert!(report.is_valid(), "{report:?}");
    built
}

#[test]
fn a_cube_becomes_a_closed_solid() {
    let cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    let topologies = cube.builder.build();
    let built = assert_closed_solid(&topologies, 6);
    assert_eq!(built.item, cube.item);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
    // 面は、シェルをたどる順に並ぶ
    let shell_faces = built.brep.shell(built.shells[0]).faces();
    for (position, face) in cube.faces.iter().enumerate() {
        assert_eq!(built.faces[face], shell_faces[position]);
    }
}

/// 記録の稜線 `edge` の曲線を `curve` にする
fn set_edge_curve(builder: &mut Builder, edge: u64, curve: u64) {
    let Some(Topology::Edge { curve: written, .. }) = builder.records.topology.get_mut(&edge)
    else {
        panic!("an edge");
    };
    *written = curve;
}

#[test]
fn wrong_orientation_flags_are_corrected_and_recorded() {
    // 上の面（1 番）の平面の法線を内向きにし、same_sense を .T. のまま書く
    let mut cube = polyhedron(
        &CUBE_POINTS,
        &CUBE_FACES,
        &[true, false, true, true, true, true],
    );
    // 稜線 0 → 3 の直線を逆の向きにし、same_sense を .T. のまま書く
    let edge = cube.edges[&(0, 3)];
    let line = cube.builder.line([0.0, 1.0, 0.0], [0.0, -1.0, 0.0]);
    set_edge_curve(&mut cube.builder, edge, line);
    // 稜線 4 → 5 の直線を、頂点から外す
    let edge_off = cube.edges[&(4, 5)];
    let off_line = cube.builder.line([0.0, 0.001, 1.0], [1.0, 0.0, 0.0]);
    set_edge_curve(&mut cube.builder, edge_off, off_line);

    let topologies = cube.builder.build();
    let built = assert_closed_solid(&topologies, 6);
    assert_eq!(
        built.corrections,
        vec![
            Correction::EdgeSense { edge },
            Correction::EdgeToSegment { edge: edge_off },
            Correction::FaceReversed {
                face: cube.faces[1]
            },
        ]
    );
}

#[test]
fn an_oriented_face_and_a_reversed_bound_are_followed() {
    // 底の面を、ORIENTED_FACE(.F.) で、ループと面の向きを逆に書いた面を指す
    let mut cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    let bottom = cube.faces[0];
    let Some(Topology::Face {
        bounds, surface, ..
    }) = cube.builder.records.topology.get(&bottom).cloned()
    else {
        panic!("a face");
    };
    let Some(Topology::FaceBound { bound, .. }) =
        cube.builder.records.topology.get(&bounds[0]).cloned()
    else {
        panic!("a bound");
    };
    let reversed_bound = cube.builder.topology(Topology::FaceBound {
        bound,
        orientation: Some(false),
        outer: true,
    });
    let reversed_face = cube.builder.topology(Topology::Face {
        bounds: vec![reversed_bound],
        surface,
        same_sense: Some(false),
    });
    let oriented = cube.builder.topology(Topology::OrientedFace {
        face: reversed_face,
        orientation: Some(false),
    });
    let shell = match cube.builder.records.topology.get(&cube.item) {
        Some(Topology::Solid { outer, .. }) => *outer,
        _ => panic!("a solid"),
    };
    let Some(Topology::Shell { faces, .. }) = cube.builder.records.topology.get_mut(&shell) else {
        panic!("a shell");
    };
    faces[0] = oriented;

    let topologies = cube.builder.build();
    let built = assert_closed_solid(&topologies, 6);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
    assert!(built.faces.contains_key(&reversed_face));
}

fn plane_at(builder: &mut Builder, origin: [f64; 3], normal: [f64; 3]) -> u64 {
    let placement = builder.placement(origin, normal, [1.0, 0.0, 0.0]);
    builder.geometry(Geometry::Plane { placement })
}

#[test]
fn a_cylinder_with_a_seam_becomes_a_closed_solid() {
    // 半径 1・高さ 2 の円柱（底は z = 0）
    let mut builder = Builder::new();
    let bottom_vertex = builder.vertex([1.0, 0.0, 0.0]);
    let top_vertex = builder.vertex([1.0, 0.0, 2.0]);
    let bottom_circle = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let top_circle = builder.circle([0.0, 0.0, 2.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let seam_line = builder.line([1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
    let bottom = builder.edge(bottom_vertex, bottom_vertex, bottom_circle, true);
    let top = builder.edge(top_vertex, top_vertex, top_circle, true);
    let seam = builder.edge(bottom_vertex, top_vertex, seam_line, true);
    let bottom_plane = plane_at(&mut builder, [0.0; 3], [0.0, 0.0, -1.0]);
    let top_plane = plane_at(&mut builder, [0.0, 0.0, 2.0], [0.0, 0.0, 1.0]);
    let side_surface = {
        let placement = builder.placement([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        builder.geometry(Geometry::CylindricalSurface {
            placement,
            radius: 1.0,
        })
    };
    let bottom_bound = builder.bound(&[(bottom, false)], true);
    let top_bound = builder.bound(&[(top, true)], true);
    // 側面: 下の円 → 継ぎ目を上へ → 上の円を逆に → 継ぎ目を下へ
    let side_bound = builder.bound(
        &[(bottom, true), (seam, true), (top, false), (seam, false)],
        true,
    );
    let faces = vec![
        builder.face(vec![bottom_bound], bottom_plane, true),
        builder.face(vec![top_bound], top_plane, true),
        builder.face(vec![side_bound], side_surface, true),
    ];
    builder.solid(faces);

    let topologies = builder.build();
    let built = assert_closed_solid(&topologies, 3);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
}

#[test]
fn a_cone_up_to_its_apex_becomes_a_closed_solid() {
    // 底の半径 1、頂点の高さ 1（軸の向きに半径が広がるため、軸は下向き）
    let mut builder = Builder::new();
    let base_vertex = builder.vertex([1.0, 0.0, 0.0]);
    let apex = builder.vertex([0.0, 0.0, 1.0]);
    let base_circle = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let base = builder.edge(base_vertex, base_vertex, base_circle, true);
    let base_plane = plane_at(&mut builder, [0.0; 3], [0.0, 0.0, -1.0]);
    let cone_surface = {
        let placement = builder.placement([0.0; 3], [0.0, 0.0, -1.0], [1.0, 0.0, 0.0]);
        builder.geometry(Geometry::ConicalSurface {
            placement,
            radius: 1.0,
            semi_angle: Angle::from_degrees(45.0),
        })
    };
    let base_bound = builder.bound(&[(base, false)], true);
    let side_bound = builder.bound(&[(base, true)], true);
    let apex_loop = builder.topology(Topology::VertexLoop { vertex: apex });
    let apex_bound = builder.topology(Topology::FaceBound {
        bound: apex_loop,
        orientation: Some(true),
        outer: false,
    });
    let faces = vec![
        builder.face(vec![base_bound], base_plane, true),
        builder.face(vec![side_bound, apex_bound], cone_surface, true),
    ];
    builder.solid(faces);

    let topologies = builder.build();
    let built = assert_closed_solid(&topologies, 2);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
}

#[test]
fn two_hemispheres_become_a_closed_solid() {
    let mut builder = Builder::new();
    let vertex = builder.vertex([1.0, 0.0, 0.0]);
    let equator_circle = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let equator = builder.edge(vertex, vertex, equator_circle, true);
    let sphere = {
        let placement = builder.placement([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        builder.geometry(Geometry::SphericalSurface {
            placement,
            radius: 1.0,
        })
    };
    let upper_bound = builder.bound(&[(equator, true)], true);
    let lower_bound = builder.bound(&[(equator, false)], true);
    let faces = vec![
        builder.face(vec![upper_bound], sphere, true),
        builder.face(vec![lower_bound], sphere, true),
    ];
    builder.solid(faces);

    let topologies = builder.build();
    let built = assert_closed_solid(&topologies, 2);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
}

#[test]
fn a_torus_with_two_seams_becomes_a_closed_solid() {
    // 中心の円の半径 3、管の半径 1
    let mut builder = Builder::new();
    let vertex = builder.vertex([4.0, 0.0, 0.0]);
    let around_axis = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 4.0);
    let around_tube = builder.circle([3.0, 0.0, 0.0], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], 1.0);
    let u_seam = builder.edge(vertex, vertex, around_axis, true);
    let v_seam = builder.edge(vertex, vertex, around_tube, true);
    let torus = {
        let placement = builder.placement([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
        builder.geometry(Geometry::ToroidalSurface {
            placement,
            major_radius: 3.0,
            minor_radius: 1.0,
        })
    };
    let bound = builder.bound(
        &[
            (u_seam, true),
            (v_seam, true),
            (u_seam, false),
            (v_seam, false),
        ],
        true,
    );
    let face = builder.face(vec![bound], torus, true);
    builder.solid(vec![face]);

    let topologies = builder.build();
    let built = assert_closed_solid(&topologies, 1);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
}

/// 4 × 4 の正方形の板に 1 × 1 の正方形の穴を開けた面と、穴のループ（`hole_clockwise` が `false` なら、穴を
/// 外側と同じ回りに書く）
fn plate(builder: &mut Builder, hole_clockwise: bool) -> (u64, u64) {
    let square = |builder: &mut Builder, size: f64, offset: f64, clockwise: bool| {
        let mut corners = [
            [offset, offset, 0.0],
            [offset + size, offset, 0.0],
            [offset + size, offset + size, 0.0],
            [offset, offset + size, 0.0],
        ];
        if clockwise {
            corners.reverse();
        }
        let vertices: Vec<u64> = corners.iter().map(|&point| builder.vertex(point)).collect();
        let edges: Vec<(u64, bool)> = (0..4)
            .map(|side| {
                let (a, b) = (corners[side], corners[(side + 1) % 4]);
                let line = builder.line(a, [b[0] - a[0], b[1] - a[1], 0.0]);
                let edge = builder.edge(vertices[side], vertices[(side + 1) % 4], line, true);
                (edge, true)
            })
            .collect();
        builder.bound(&edges, true)
    };
    let outer = square(builder, 4.0, 0.0, false);
    let hole = square(builder, 1.0, 1.0, hole_clockwise);
    let plane = plane_at(builder, [0.0; 3], [0.0, 0.0, 1.0]);
    (builder.face(vec![outer, hole], plane, true), hole)
}

#[test]
fn an_inner_loop_turning_like_the_outer_loop_is_reversed() {
    let mut builder = Builder::new();
    let (good, _) = plate(&mut builder, true);
    builder.open_shell(vec![good]);
    let topologies = builder.build();
    assert!(topologies.items[0].corrections.is_empty());
    assert!(topologies.items[0].unoriented_faces.is_empty());

    let mut builder = Builder::new();
    let (face, hole) = plate(&mut builder, false);
    builder.open_shell(vec![face]);
    let topologies = builder.build();
    let built = &topologies.items[0];
    assert_eq!(
        built.corrections,
        vec![Correction::LoopReversed { face, bound: hole }]
    );
    assert!(built.unoriented_faces.is_empty());
    assert!(built.solid.is_none());
    assert!(!built.brep.is_shell_closed(built.shells[0]));
}

#[test]
fn a_faceted_brep_shares_poly_loop_edges_by_point() {
    // 四面体を、曲面のない面と多角形のループで書く
    let mut builder = Builder::new();
    let points: Vec<u64> = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ]
    .iter()
    .map(|&point| builder.point(point))
    .collect();
    let faces = [[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]]
        .iter()
        .map(|corners| {
            let poly = builder.topology(Topology::PolyLoop {
                points: corners.iter().map(|&corner| points[corner]).collect(),
            });
            let bound = builder.topology(Topology::FaceBound {
                bound: poly,
                orientation: Some(true),
                outer: true,
            });
            builder.topology(Topology::Face {
                bounds: vec![bound],
                surface: None,
                same_sense: None,
            })
        })
        .collect();
    builder.solid(faces);

    let topologies = builder.build();
    let built = assert_closed_solid(&topologies, 4);
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
}

#[test]
fn faces_that_cannot_be_built_are_skipped_with_the_reason() {
    let mut cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    // 上の面の曲面を、NURBS 曲面にする
    let control_points: Vec<Vec<u64>> = (0..2)
        .map(|row| {
            (0..2)
                .map(|column| cube.builder.point([column as f64, row as f64, 1.0]))
                .collect()
        })
        .collect();
    let nurbs = cube
        .builder
        .geometry(Geometry::BSplineSurface(crate::record::BSplineSurface {
            u_degree: 1,
            v_degree: 1,
            control_points,
            u_closed: Some(false),
            v_closed: Some(false),
            u_multiplicities: vec![2, 2],
            v_multiplicities: vec![2, 2],
            u_knots: vec![0.0, 1.0],
            v_knots: vec![0.0, 1.0],
            weights: None,
        }));
    let top = cube.faces[1];
    let Some(Topology::Face { surface, .. }) = cube.builder.records.topology.get_mut(&top) else {
        panic!("a face");
    };
    *surface = Some(nurbs);
    // 稜線 2 → 1 を、開いた直線の上の閉じた稜線にする
    let edge = cube.edges[&(2, 1)];
    let Some(Topology::Edge { end, start, .. }) = cube.builder.records.topology.get_mut(&edge)
    else {
        panic!("an edge");
    };
    *end = *start;

    let topologies = cube.builder.build();
    let built = &topologies.items[0];
    // 外した面のあるシェルは開いたまま作り、立体も作る（外した面の扱いは、後の段で決める）
    assert_eq!(built.faces.len(), 3);
    let shell = match cube.builder.records.topology.get(&cube.item) {
        Some(Topology::Solid { outer, .. }) => *outer,
        _ => panic!("a solid"),
    };
    assert_eq!(built.open_shells, vec![shell]);
    let skipped: Vec<(u64, u64, TopologyError)> = topologies
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.cause, skipped.reason))
        .collect();
    assert_eq!(
        skipped,
        vec![
            (cube.faces[0], edge, TopologyError::ClosedOnOpenCurve),
            (top, nurbs, TopologyError::NurbsSurface),
            (cube.faces[3], edge, TopologyError::ClosedOnOpenCurve),
        ]
    );
}

#[test]
fn an_arc_range_follows_the_vertices() {
    // 円の 4 分の 1 の弧（角度 0 から 90 度）を、b から a へ角度の減る向き（same_sense が .F.）で書く
    let mut builder = Builder::new();
    let a = builder.vertex([1.0, 0.0, 0.0]);
    let b = builder.vertex([0.0, 1.0, 0.0]);
    let circle = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let line = builder.line([1.0, 0.0, 0.0], [-1.0, 1.0, 0.0]);
    let arc = builder.edge(b, a, circle, false);
    let chord = builder.edge(a, b, line, true);
    let plane = plane_at(&mut builder, [0.0; 3], [0.0, 0.0, 1.0]);
    // 弧を a から b へ（ファイルの稜線を逆に使う）、弦を b から a へ（面は弧と弦の間で、反時計回り）
    let bound = builder.bound(&[(arc, false), (chord, false)], true);
    let face = builder.face(vec![bound], plane, true);
    builder.open_shell(vec![face]);

    let topologies = builder.build();
    let built = &topologies.items[0];
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
    assert!(built.unoriented_faces.is_empty());
    let face_loop = built
        .brep
        .face_loop(built.brep.face(built.faces[&face]).loops()[0]);
    let arc_use = face_loop.uses()[0];
    // 区間は、a から b への 4 分の 1 周
    let arc_edge = built.brep.edge(arc_use.edge);
    let (t0, t1) = arc_edge.parameter_range();
    assert!((t1 - t0 - std::f64::consts::FRAC_PI_2).abs() < 1.0e-12);
    let near = |t: f64, point: Point3D<f64>| {
        arc_edge.curve().point_at_parameter(t).distance_to(point) < 1.0e-12
    };
    assert!(near(t0, Point3D::new(1.0, 0.0, 0.0)));
    assert!(near(t1, Point3D::new(0.0, 1.0, 0.0)));
    // ファイルの稜線は区間の減る向きで、ループはそれを逆に使うので、区間の増える向き
    assert!(arc_use.forward);
    assert!(built.brep.validate().is_valid());
}

#[test]
fn an_invalid_tolerance_is_refused() {
    let result = build(
        &Records::default(),
        &Geometries::default(),
        TopologyToleranceSettings::new(f64::NAN),
        ImportToleranceSettings::default(),
    );
    assert_eq!(result.err(), Some(BrepError::InvalidTolerance));
}

/// ループ `bound` を、外側のループ（`FACE_OUTER_BOUND`）とする
fn mark_outer(builder: &mut Builder, bound: u64) {
    let Some(Topology::FaceBound { outer, .. }) = builder.records.topology.get_mut(&bound) else {
        panic!("a bound");
    };
    *outer = true;
}

#[test]
fn an_outer_bound_is_not_reversed_as_an_inner_loop() {
    // 外側のループも穴も時計回り（外側のループの書き誤り）。外側と書かれたループは逆にせず、直せない面とする
    let mut builder = Builder::new();
    let (face, hole) = plate(&mut builder, true);
    let Some(Topology::Face { bounds, .. }) = builder.records.topology.get(&face).cloned() else {
        panic!("a face");
    };
    let outer = bounds[0];
    let Some(Topology::FaceBound { orientation, .. }) = builder.records.topology.get_mut(&outer)
    else {
        panic!("a bound");
    };
    *orientation = Some(false);
    mark_outer(&mut builder, outer);
    builder.open_shell(vec![face]);

    let topologies = builder.build();
    let built = &topologies.items[0];
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
    assert_eq!(built.unoriented_faces, vec![face]);
    assert_ne!(outer, hole);
}

#[test]
fn a_face_used_in_both_directions_is_skipped_the_second_time() {
    // 開いたシェル 2 つを持つ面のモデルで、同じ面を、そのままと ORIENTED_FACE(.F.) で使う
    let mut builder = Builder::new();
    let (face, _) = plate(&mut builder, true);
    let oriented = builder.topology(Topology::OrientedFace {
        face,
        orientation: Some(false),
    });
    let first = builder.topology(Topology::Shell {
        faces: vec![face],
        closed: false,
    });
    let second = builder.topology(Topology::Shell {
        faces: vec![oriented],
        closed: false,
    });
    let model = builder.topology(Topology::SurfaceModel {
        shells: vec![first, second],
    });
    builder.represent(model);

    let topologies = builder.build();
    let built = &topologies.items[0];
    assert_eq!(built.shells.len(), 1);
    let skipped: Vec<(u64, u64, TopologyError)> = topologies
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.cause, skipped.reason))
        .collect();
    assert_eq!(
        skipped,
        vec![
            (face, face, TopologyError::ReversedReuse),
            (second, second, TopologyError::Brep(BrepError::EmptyShell)),
        ]
    );
}

#[test]
fn hemispheres_of_a_revolved_sphere_are_reported_as_undecided() {
    // 軸の上に中心のある円を回した球（面ごとに向きが決まる曲面）。ループが 1 つの面は、表と裏のどちらも規則に
    // 合うため、向きを決められなかった面とする
    let mut builder = Builder::new();
    let vertex = builder.vertex([1.0, 0.0, 0.0]);
    let equator_circle = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let equator = builder.edge(vertex, vertex, equator_circle, true);
    let meridian = builder.circle([0.0; 3], [0.0, -1.0, 0.0], [1.0, 0.0, 0.0], 1.0);
    let axis = {
        let location = builder.point([0.0; 3]);
        let up = builder.direction([0.0, 0.0, 1.0]);
        builder.geometry(Geometry::Axis {
            location,
            axis: Some(up),
        })
    };
    let sphere = builder.geometry(Geometry::SurfaceOfRevolution {
        curve: meridian,
        axis,
    });
    let upper_bound = builder.bound(&[(equator, true)], true);
    let lower_bound = builder.bound(&[(equator, false)], true);
    let faces = vec![
        builder.face(vec![upper_bound], sphere, true),
        builder.face(vec![lower_bound], sphere, true),
    ];
    builder.solid(faces.clone());

    let topologies = builder.build();
    let built = &topologies.items[0];
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
    assert_eq!(built.unoriented_faces, faces);
}

#[test]
fn a_closed_edge_on_a_closed_b_spline_must_start_at_its_seam() {
    // 1 次の B-spline の正方形（制御点 0 が継ぎ目）。全体の稜線の頂点が継ぎ目にあれば読み、ほかの角なら外す
    for (corner, readable) in [(0, true), (2, false)] {
        let mut builder = Builder::new();
        let corners = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let control_points = corners.iter().map(|&point| builder.point(point)).collect();
        let square = builder.geometry(Geometry::BSplineCurve(crate::record::BSplineCurve {
            degree: 1,
            control_points,
            closed: Some(true),
            multiplicities: vec![2, 1, 1, 1, 2],
            knots: vec![0.0, 1.0, 2.0, 3.0, 4.0],
            weights: None,
        }));
        let vertex = builder.vertex(corners[corner]);
        let edge = builder.edge(vertex, vertex, square, true);
        let plane = plane_at(&mut builder, [0.0; 3], [0.0, 0.0, 1.0]);
        let bound = builder.bound(&[(edge, true)], true);
        let face = builder.face(vec![bound], plane, true);
        builder.open_shell(vec![face]);

        let topologies = builder.build();
        if readable {
            assert!(topologies.skipped.is_empty(), "{:?}", topologies.skipped);
            assert!(topologies.items[0].brep.validate().is_valid());
        } else {
            assert!(topologies.items.is_empty());
            assert_eq!(
                topologies.skipped[0],
                TopologySkipped {
                    id: face,
                    cause: edge,
                    reason: TopologyError::InvalidEdgeRange,
                }
            );
        }
    }
}

/// 表現の文脈（番号 0）の許容誤差を `uncertainty_mm` にする
fn set_uncertainty(builder: &mut Builder, uncertainty_mm: f64) {
    builder.records.contexts.insert(
        0,
        crate::record::Context {
            dimension: Some(3),
            length_to_mm: 1.0,
            angle_to_degrees: 1.0,
            uncertainty_mm: Some(uncertainty_mm),
            default_units: false,
        },
    );
}

/// 面 `face` の曲面を、点 `origin` を通り法線 `normal` の平面にする
fn set_face_plane(builder: &mut Builder, face: u64, origin: [f64; 3], normal: [f64; 3]) {
    let plane = plane_at(builder, origin, normal);
    let Some(Topology::Face { surface, .. }) = builder.records.topology.get_mut(&face) else {
        panic!("a face");
    };
    *surface = Some(plane);
}

/// 立方体の上の面の稜線（ファイルの番号の順）
fn cube_top_edges(cube: &Polyhedron) -> Vec<u64> {
    let mut edges: Vec<u64> = [(4, 5), (5, 6), (6, 7), (7, 4)]
        .iter()
        .map(|key| cube.edges[key])
        .collect();
    edges.sort();
    edges
}

#[test]
fn edges_off_their_face_surface_are_widened() {
    // 上の面の平面を、稜線から 2e-4 だけ上げる（上限は、立方体の対角 √3 × 1e-3）
    let mut cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    let top = cube.faces[1];
    set_face_plane(&mut cube.builder, top, [0.0, 0.0, 1.0002], [0.0, 0.0, 1.0]);

    let topologies = cube.builder.build();
    let built = assert_closed_solid(&topologies, 6);
    let widened: Vec<u64> = built
        .corrections
        .iter()
        .map(|correction| match *correction {
            Correction::EdgeToleranceWidened { edge, tolerance } => {
                // ずれの 1.5 倍
                assert!((tolerance - 3.0e-4).abs() < 1.0e-12, "{tolerance}");
                edge
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(widened, cube_top_edges(&cube));
}

#[test]
fn a_face_beyond_the_tolerance_limit_is_skipped() {
    // 上の面の平面を、稜線から 0.01 だけ上げる（1.5 倍の 0.015 は、上限 √3 × 1e-3 を超える）
    let mut cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    let top = cube.faces[1];
    set_face_plane(&mut cube.builder, top, [0.0, 0.0, 1.01], [0.0, 0.0, 1.0]);

    let topologies = cube.builder.build();
    assert_eq!(topologies.items[0].faces.len(), 5);
    assert!(topologies.items[0].corrections.is_empty());
    let [skipped] = topologies.skipped.as_slice() else {
        panic!("{:?}", topologies.skipped);
    };
    assert_eq!(skipped.id, top);
    assert_eq!(skipped.cause, cube.edges[&(4, 5)]);
    let TopologyError::ToleranceExceeded { deviation } = skipped.reason else {
        panic!("{:?}", skipped.reason);
    };
    assert!((deviation - 0.01).abs() < 1.0e-12, "{deviation}");
}

/// 半径 1 の円の 4 分の 1 の弧と弦の面（弧の両端の頂点は、半径 `vertex_radius` の上）の記録と、面と弧の頂点
fn arc_face(vertex_radius: f64) -> (Builder, u64, [u64; 2]) {
    let mut builder = Builder::new();
    let a = builder.vertex([vertex_radius, 0.0, 0.0]);
    let b = builder.vertex([0.0, vertex_radius, 0.0]);
    let circle = builder.circle([0.0; 3], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0], 1.0);
    let line = builder.line([vertex_radius, 0.0, 0.0], [-1.0, 1.0, 0.0]);
    let arc = builder.edge(a, b, circle, true);
    let chord = builder.edge(b, a, line, false);
    let plane = plane_at(&mut builder, [0.0; 3], [0.0, 0.0, 1.0]);
    let bound = builder.bound(&[(arc, true), (chord, true)], true);
    let face = builder.face(vec![bound], plane, true);
    builder.open_shell(vec![face]);
    (builder, face, [a, b])
}

#[test]
fn vertices_off_their_curve_are_widened() {
    // 弧の頂点を、円から 2e-4 だけ外す（上限は、頂点と円の外形の箱の対角 2√3 × 1e-3 ≒ 3.5e-3）
    let (builder, _, [a, b]) = arc_face(1.0002);

    let topologies = builder.build();
    assert!(topologies.skipped.is_empty(), "{:?}", topologies.skipped);
    let built = &topologies.items[0];
    assert_eq!(built.corrections.len(), 2, "{:?}", built.corrections);
    for (correction, vertex) in built.corrections.iter().zip([a, b]) {
        let Correction::VertexToleranceWidened {
            vertex: widened,
            tolerance,
        } = *correction
        else {
            panic!("{correction:?}");
        };
        assert_eq!(widened, vertex);
        assert!((tolerance - 3.0e-4).abs() < 1.0e-12, "{tolerance}");
    }
    let report = built.brep.validate();
    assert!(report.is_valid(), "{report:?}");
}

#[test]
fn a_vertex_beyond_the_tolerance_limit_skips_its_faces() {
    // 弧の頂点を、円から 0.01 だけ外す（1.5 倍の 0.015 は、上限 ≒ 3.5e-3 を超える）
    let (builder, face, [a, _]) = arc_face(1.01);

    let topologies = builder.build();
    assert!(topologies.items.is_empty());
    let [skipped] = topologies.skipped.as_slice() else {
        panic!("{:?}", topologies.skipped);
    };
    assert_eq!((skipped.id, skipped.cause), (face, a));
    let TopologyError::ToleranceExceeded { deviation } = skipped.reason else {
        panic!("{:?}", skipped.reason);
    };
    assert!((deviation - 0.01).abs() < 1.0e-12, "{deviation}");
}

#[test]
fn vertices_at_one_corner_are_merged() {
    // 角 6 に、5e-7 だけ離れた 2 つ目の頂点を置き、稜線 5 → 6 と 6 → 7 はそちらを使う（文脈の許容誤差は 1e-6）
    let mut cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    set_uncertainty(&mut cube.builder, 1.0e-6);
    let second = cube.builder.vertex([1.0 + 5.0e-7, 1.0, 1.0]);
    let mut first = None;
    for (key, is_end) in [((5, 6), true), ((6, 7), false)] {
        let edge = cube.edges[&key];
        let Some(Topology::Edge { start, end, .. }) = cube.builder.records.topology.get_mut(&edge)
        else {
            panic!("an edge");
        };
        let corner = if is_end { end } else { start };
        first = Some(*corner);
        *corner = second;
    }

    let topologies = cube.builder.build();
    // まとめなければ、ループがつながらず、表の確かめを通らない
    let built = assert_closed_solid(&topologies, 6);
    let first = first.unwrap();
    assert_eq!(built.corrections.len(), 2, "{:?}", built.corrections);
    assert_eq!(
        built.corrections[0],
        Correction::VertexMerged {
            vertex: second,
            into: first
        }
    );
    let Correction::VertexToleranceWidened { vertex, tolerance } = built.corrections[1] else {
        panic!("{:?}", built.corrections[1]);
    };
    // まとめた頂点をすべて含む: 離れ 5e-7 と 2 つ目の頂点の許容誤差 1e-6 の和
    assert_eq!(vertex, first);
    assert!((tolerance - 1.5e-6).abs() < 1.0e-15, "{tolerance}");
}

#[test]
fn the_two_vertices_of_a_short_edge_are_not_merged() {
    // 5e-7 の長さの辺を持つ五角形の面（文脈の許容誤差は 1e-6 で、両端は許容誤差の和の中で重なる）
    let points = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5 - 5.0e-7, 1.0, 0.0],
    ];
    let mut pentagon = polyhedron(&points, &[&[0, 1, 2, 3, 4]], &[]);
    set_uncertainty(&mut pentagon.builder, 1.0e-6);

    let topologies = pentagon.builder.build();
    assert!(topologies.skipped.is_empty(), "{:?}", topologies.skipped);
    let built = &topologies.items[0];
    assert!(built.corrections.is_empty(), "{:?}", built.corrections);
    let face = built.brep.face(built.faces[&pentagon.faces[0]]);
    assert_eq!(built.brep.face_loop(face.loops()[0]).uses().len(), 5);
    let report = built.brep.validate();
    assert!(report.is_valid(), "{report:?}");
}

#[test]
fn overlapping_edges_between_the_same_vertices_are_merged() {
    // 右の面（3 番）は、下の面と共有する稜線 2 → 1 の代わりに、同じ直線の上の別の稜線 1 → 2 を使う
    let mut cube = polyhedron(&CUBE_POINTS, &CUBE_FACES, &[]);
    let shared = cube.edges[&(2, 1)];
    let Some(&Topology::Edge { start, end, .. }) = cube.builder.records.topology.get(&shared)
    else {
        panic!("an edge");
    };
    let line = cube.builder.line([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    let duplicate = cube.builder.edge(end, start, line, true);
    let oriented = cube
        .builder
        .records
        .topology
        .iter()
        .find_map(|(&id, topology)| match topology {
            Topology::OrientedEdge {
                edge,
                orientation: Some(false),
            } if *edge == shared => Some(id),
            _ => None,
        })
        .expect("the right face uses the shared edge backward");
    cube.builder.records.topology.insert(
        oriented,
        Topology::OrientedEdge {
            edge: duplicate,
            orientation: Some(true),
        },
    );

    let topologies = cube.builder.build();
    // まとめなければ、シェルが閉じない
    let built = assert_closed_solid(&topologies, 6);
    assert_eq!(
        built.corrections,
        vec![Correction::EdgeMerged {
            edge: duplicate,
            into: shared
        }]
    );
}

#[test]
fn invalid_import_tolerance_settings_are_refused() {
    for settings in [
        ImportToleranceSettings {
            limit_ratio: 0.0,
            ..ImportToleranceSettings::default()
        },
        ImportToleranceSettings {
            deviation_factor: 0.5,
            ..ImportToleranceSettings::default()
        },
        ImportToleranceSettings {
            limit_ratio: f64::INFINITY,
            ..ImportToleranceSettings::default()
        },
    ] {
        let result = build(
            &Records::default(),
            &Geometries::default(),
            TopologyToleranceSettings::new(TOLERANCE),
            settings,
        );
        assert_eq!(result.err(), Some(BrepError::InvalidTolerance));
    }
}

#[test]
fn the_nearest_angle_on_an_ellipse_is_found() {
    use geo_contracts::Ellipse3DConstructor;
    let ellipse = TopoEllipse3D::new(
        Point3D::new(0.0, 0.0, 0.0),
        10.0,
        1.0,
        Direction3D::from_vector(Vector3D::new(0.0, 0.0, 1.0)).unwrap(),
        Direction3D::from_vector(Vector3D::new(1.0, 0.0, 0.0)).unwrap(),
    )
    .unwrap();
    // 角度 45 度の点から、法線の向きに 1e-4 だけ外した点
    let angle = std::f64::consts::FRAC_PI_4;
    let on_curve = Point3D::new(10.0 * angle.cos(), angle.sin(), 0.0);
    let normal = Vector3D::new(angle.cos(), 10.0 * angle.sin(), 0.0);
    let point = on_curve + normal * (1.0e-4 / normal.length());
    let found = ellipse_angle(&ellipse, point, TOLERANCE);
    assert!((found - angle).abs() < 1.0e-9, "{found}");
}

#[test]
fn coincident_edges_in_one_loop_are_not_merged() {
    // 1 つのループが、同じ 2 つの頂点の間の重なる 2 本の稜線を、行きと帰りに使う
    let mut builder = Builder::new();
    let a = builder.vertex([0.0, 0.0, 0.0]);
    let b = builder.vertex([1.0, 0.0, 0.0]);
    let forward_line = builder.line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0]);
    let backward_line = builder.line([1.0, 0.0, 0.0], [-1.0, 0.0, 0.0]);
    let forward = builder.edge(a, b, forward_line, true);
    let backward = builder.edge(b, a, backward_line, true);
    let plane = plane_at(&mut builder, [0.0; 3], [0.0, 0.0, 1.0]);
    let bound = builder.bound(&[(forward, true), (backward, true)], true);
    let face = builder.face(vec![bound], plane, true);
    builder.open_shell(vec![face]);

    let topologies = builder.build();
    let merged = topologies.items.iter().any(|built| {
        built
            .corrections
            .iter()
            .any(|correction| matches!(correction, Correction::EdgeMerged { .. }))
    });
    assert!(!merged);
}
