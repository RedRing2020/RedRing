use super::*;

fn mesh() -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 1.0),
            Point3D::new(10.0, 0.0, 1.5),
            Point3D::new(10.0, 10.0, 2.0),
            Point3D::new(0.0, 10.0, 0.25),
        ],
        vec![[0, 1, 2], [0, 2, 3]],
    )
    .unwrap()
}

/// 重みのある 1 次の曲面
fn rational() -> NurbsSurface3D<f64> {
    NurbsSurface3D::new(
        1,
        1,
        vec![0.0, 0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        &[
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 5.0, 1.0)],
            vec![Point3D::new(5.0, 0.0, 2.0), Point3D::new(5.0, 5.0, 3.0)],
        ],
        Some(vec![vec![1.0, 0.5], vec![0.75, 1.0]]),
    )
    .unwrap()
}

#[test]
fn meshes_round_trip_and_the_cursor_moves_past_them() {
    let mut bytes = Vec::new();
    write_triangle_mesh(&mut bytes, &mesh()).unwrap();
    assert_eq!(bytes.len(), 4 + 4 * 24 + 4 + 2 * 12);
    bytes.push(0xab);
    let mut cursor = bytes.as_slice();
    let read = read_triangle_mesh(&mut cursor).unwrap();
    assert_eq!(read.vertices(), mesh().vertices());
    assert_eq!(read.indices(), mesh().indices());
    assert_eq!(cursor, &[0xab]);
}

#[test]
fn surfaces_round_trip_with_and_without_weights() {
    let plain = NurbsSurface3D::new(
        2,
        1,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0, 1.0],
        &[
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 4.0, 0.0)],
            vec![Point3D::new(2.0, 0.0, 3.0), Point3D::new(2.0, 4.0, 3.0)],
            vec![Point3D::new(4.0, 0.0, 0.0), Point3D::new(4.0, 4.0, 0.0)],
        ],
        None,
    )
    .unwrap();
    for surface in [rational(), plain] {
        let mut bytes = Vec::new();
        write_nurbs_surface(&mut bytes, &surface).unwrap();
        let mut cursor = bytes.as_slice();
        let read = read_nurbs_surface(&mut cursor).unwrap();
        assert!(cursor.is_empty());
        assert_eq!(read.u_knots(), surface.u_knots());
        assert_eq!(read.v_knots(), surface.v_knots());
        assert_eq!(read.weights().is_some(), surface.weights().is_some());
        for u in 0..surface.u_count() {
            for v in 0..surface.v_count() {
                assert_eq!(read.control_point(u, v), surface.control_point(u, v));
                assert_eq!(read.weight(u, v), surface.weight(u, v));
            }
        }
    }
}

#[test]
fn counts_larger_than_the_remaining_bytes_are_rejected_before_allocating() {
    let mut bytes = Vec::new();
    write_triangle_mesh(&mut bytes, &mesh()).unwrap();
    bytes[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert_eq!(
        read_triangle_mesh(&mut bytes.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("mesh.vertices")
    );
    assert_eq!(
        read_triangle_mesh(&mut &bytes[..2]).unwrap_err(),
        BinaryGeometryError::UnexpectedEnd
    );
}

#[test]
fn surfaces_without_control_points_in_one_direction_are_rejected() {
    // u の数だけ大きく、v の数が 0（制御点の数は 0 になる）
    let mut bytes = Vec::new();
    for value in [0_u32, 0, u32::MAX, 0, 0, 0] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.push(0);
    assert_eq!(
        read_nurbs_surface(&mut bytes.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("nurbs.control_points")
    );
}

#[test]
fn corrupted_geometry_bytes_do_not_panic() {
    let mut mesh_bytes = Vec::new();
    write_triangle_mesh(&mut mesh_bytes, &mesh()).unwrap();
    let mut surface_bytes = Vec::new();
    write_nurbs_surface(&mut surface_bytes, &rational()).unwrap();
    let read_mesh: fn(&mut &[u8]) = |cursor| {
        let _ = read_triangle_mesh(cursor);
    };
    let read_surface: fn(&mut &[u8]) = |cursor| {
        let _ = read_nurbs_surface(cursor);
    };
    let cases = [
        (mesh_bytes.as_slice(), read_mesh),
        (surface_bytes.as_slice(), read_surface),
    ];
    for (bytes, read) in cases {
        for position in 0..bytes.len() {
            for value in [0x00, 0x01, 0x7f, 0x80, 0xff] {
                let mut corrupted = bytes.to_vec();
                corrupted[position] = value;
                read(&mut corrupted.as_slice());
            }
            if position + 4 <= bytes.len() {
                let mut corrupted = bytes.to_vec();
                corrupted[position..position + 4].copy_from_slice(&u32::MAX.to_le_bytes());
                read(&mut corrupted.as_slice());
            }
            read(&mut &bytes[..position]);
        }
    }
}

fn direction(x: f64, y: f64, z: f64) -> Direction3D<f64> {
    Direction3D::from_vector(Vector3D::new(x, y, z)).unwrap()
}

fn near(a: Point3D<f64>, b: Point3D<f64>) -> bool {
    a.distance_to(b) < 1.0e-12
}

fn same_direction(a: Direction3D<f64>, b: Direction3D<f64>) -> bool {
    (a.as_vector() - b.as_vector()).length() < 1.0e-12
}

/// 2 次の有理の NURBS 曲線（4 分の 1 の円）
fn rational_curve() -> NurbsCurve3D<f64> {
    let weight = std::f64::consts::FRAC_1_SQRT_2;
    <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        2,
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        &[
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ],
        Some(vec![1.0, weight, 1.0]),
    )
    .unwrap()
}

/// 傾けた座標系（中心・軸・角度 0 の方向）
fn frame() -> (Point3D<f64>, Direction3D<f64>, Direction3D<f64>) {
    (
        Point3D::new(1.0, -2.0, 3.5),
        direction(0.0, 0.6, 0.8),
        direction(1.0, 0.0, 0.0),
    )
}

#[test]
fn nurbs_curves_round_trip_with_and_without_weights() {
    let plain = <NurbsCurve3D<f64> as NurbsCurve3DConstructor<f64>>::new(
        1,
        vec![0.0, 0.0, 1.0, 2.0, 2.0],
        &[
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 2.0, 0.0),
            Point3D::new(3.0, 2.0, 1.0),
        ],
        None,
    )
    .unwrap();
    for curve in [rational_curve(), plain] {
        let mut bytes = Vec::new();
        write_nurbs_curve(&mut bytes, &curve).unwrap();
        bytes.push(0xAA);
        let mut cursor = bytes.as_slice();
        let read = read_nurbs_curve(&mut cursor).unwrap();
        assert_eq!(cursor, [0xAA]);
        assert_eq!(read.degree(), curve.degree());
        assert_eq!(read.coordinates(), curve.coordinates());
        assert_eq!(
            <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::knot_vector(&read),
            <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::knot_vector(&curve)
        );
        assert_eq!(
            <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::weights(&read),
            <NurbsCurve3D<f64> as NurbsCurve3DProperties<f64>>::weights(&curve)
        );
    }
}

#[test]
fn analytic_curves_round_trip() {
    let (center, axis, reference) = frame();
    let line = <InfiniteLine3D<f64> as InfiniteLine3DConstructor<f64>>::new(
        center,
        Vector3D::new(0.0, 3.0, 4.0),
    )
    .unwrap();
    let mut bytes = Vec::new();
    write_line(&mut bytes, &line);
    let read = read_line(&mut bytes.as_slice()).unwrap();
    assert!(near(read.point(), line.point()));
    assert!(same_direction(read.direction(), line.direction()));

    // 角度 0 の方向が、軸から決まる既定の方向でない円も、同じ方向に戻る
    let circle =
        <Circle3D<f64> as Circle3DConstructor<f64>>::from_frame(center, axis, reference, 2.5)
            .unwrap();
    let mut bytes = Vec::new();
    write_circle(&mut bytes, &circle);
    let read = read_circle(&mut bytes.as_slice()).unwrap();
    assert!(near(read.center(), circle.center()));
    assert!(same_direction(read.axis(), circle.axis()));
    assert!(same_direction(read.ref_direction(), reference));
    assert_eq!(read.radius(), 2.5);

    let ellipse =
        <Ellipse3D<f64> as Ellipse3DConstructor<f64>>::new(center, 3.0, 1.5, axis, reference)
            .unwrap();
    let mut bytes = Vec::new();
    write_ellipse(&mut bytes, &ellipse);
    let read = read_ellipse(&mut bytes.as_slice()).unwrap();
    assert!(near(read.center(), ellipse.center()));
    assert!(same_direction(read.normal(), ellipse.normal()));
    assert!(same_direction(
        read.major_axis_direction(),
        ellipse.major_axis_direction()
    ));
    assert_eq!((read.semi_major_axis(), read.semi_minor_axis()), (3.0, 1.5));
}

#[test]
fn analytic_surfaces_round_trip() {
    let (center, axis, reference) = frame();

    let plane =
        <Plane3D<f64> as Plane3DConstructor<f64>>::from_origin_and_axes(center, axis, reference)
            .unwrap();
    let mut bytes = Vec::new();
    write_plane(&mut bytes, &plane);
    let read = read_plane(&mut bytes.as_slice()).unwrap();
    assert!(near(read.origin(), plane.origin()));
    assert!(same_direction(read.normal(), plane.normal()));
    assert!(same_direction(read.u_axis(), plane.u_axis()));

    let cylinder = <CylindricalSurface3D<f64> as CylindricalSurface3DConstructor<f64>>::new(
        center, axis, reference, 2.0,
    )
    .unwrap();
    let mut bytes = Vec::new();
    write_cylinder(&mut bytes, &cylinder);
    let read = read_cylinder(&mut bytes.as_slice()).unwrap();
    assert!(near(read.center(), cylinder.center()));
    assert!(same_direction(read.axis(), cylinder.axis()));
    assert!(same_direction(
        read.ref_direction(),
        cylinder.ref_direction()
    ));
    assert_eq!(read.radius(), 2.0);

    let cone = <ConicalSurface3D<f64> as ConicalSurface3DConstructor<f64>>::new(
        center,
        axis,
        reference,
        2.0,
        Angle::from_radians(0.3),
    )
    .unwrap();
    let mut bytes = Vec::new();
    write_cone(&mut bytes, &cone);
    let read = read_cone(&mut bytes.as_slice()).unwrap();
    assert!(near(read.center(), cone.center()));
    assert!(same_direction(read.ref_direction(), cone.ref_direction()));
    assert_eq!(read.radius(), 2.0);
    assert_eq!(read.semi_angle().to_radians(), 0.3);

    let sphere = <SphericalSurface3D<f64> as SphericalSurface3DConstructor<f64>>::new(
        center, axis, reference, 4.0,
    )
    .unwrap();
    let mut bytes = Vec::new();
    write_sphere(&mut bytes, &sphere);
    let read = read_sphere(&mut bytes.as_slice()).unwrap();
    assert!(near(read.center(), sphere.center()));
    assert!(same_direction(read.axis(), sphere.axis()));
    assert!(same_direction(read.ref_direction(), sphere.ref_direction()));
    assert_eq!(read.radius(), 4.0);

    let torus = <TorusSurface3D<f64> as TorusSurface3DConstructor<f64>>::new(
        center, axis, reference, 5.0, 1.5,
    )
    .unwrap();
    let mut bytes = Vec::new();
    write_torus(&mut bytes, &torus);
    let read = read_torus(&mut bytes.as_slice()).unwrap();
    assert!(near(read.center(), torus.center()));
    assert!(same_direction(read.axis(), torus.axis()));
    assert!(same_direction(read.ref_direction(), torus.ref_direction()));
    assert_eq!((read.major_radius(), read.minor_radius()), (5.0, 1.5));
}

#[test]
fn nurbs_curves_with_values_that_are_not_finite_are_rejected() {
    let mut bytes = Vec::new();
    write_nurbs_curve(&mut bytes, &rational_curve()).unwrap();
    // 次数・制御点の数・ノットの数の後ろの、初めのノット
    let knot = 12;
    let mut broken = bytes.clone();
    broken[knot..knot + 8].copy_from_slice(&f64::NEG_INFINITY.to_le_bytes());
    assert_eq!(
        read_nurbs_curve(&mut broken.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("nurbs_curve.knots")
    );
    // 初めの制御点
    let point = knot + 6 * 8;
    let mut broken = bytes.clone();
    broken[point..point + 8].copy_from_slice(&f64::NAN.to_le_bytes());
    assert_eq!(
        read_nurbs_curve(&mut broken.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("nurbs_curve.control_points")
    );
    // 終わりの重み
    let length = bytes.len();
    let mut broken = bytes;
    broken[length - 8..].copy_from_slice(&f64::INFINITY.to_le_bytes());
    assert_eq!(
        read_nurbs_curve(&mut broken.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("nurbs_curve.weights")
    );
}

#[test]
fn values_that_do_not_make_a_shape_are_rejected() {
    // 大きさ 0 の方向
    let mut bytes = Vec::new();
    write_point(&mut bytes, Point3D::new(0.0, 0.0, 0.0));
    write_point(&mut bytes, Point3D::new(0.0, 0.0, 0.0));
    write_point(&mut bytes, Point3D::new(1.0, 0.0, 0.0));
    write_f64(&mut bytes, 1.0);
    assert_eq!(
        read_circle(&mut bytes.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("circle.axis")
    );
    // 有限でない半径
    let (center, axis, _) = frame();
    let circle = <Circle3D<f64> as Circle3DConstructor<f64>>::new(center, axis, 1.0).unwrap();
    let mut bytes = Vec::new();
    write_circle(&mut bytes, &circle);
    let length = bytes.len();
    bytes[length - 8..].copy_from_slice(&f64::NAN.to_le_bytes());
    assert_eq!(
        read_circle(&mut bytes.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("circle.radius")
    );
    // 正でない半径
    bytes[length - 8..].copy_from_slice(&(-1.0_f64).to_le_bytes());
    assert_eq!(
        read_circle(&mut bytes.as_slice()).unwrap_err(),
        BinaryGeometryError::InvalidValue("circle")
    );
}

#[test]
fn corrupted_curve_and_surface_bytes_do_not_panic() {
    let (center, axis, reference) = frame();
    type Reader = fn(&mut &[u8]);
    let mut cases: Vec<(Vec<u8>, Reader)> = Vec::new();
    let mut bytes = Vec::new();
    write_nurbs_curve(&mut bytes, &rational_curve()).unwrap();
    cases.push((bytes, |cursor| {
        let _ = read_nurbs_curve(cursor);
    }));
    let mut bytes = Vec::new();
    write_ellipse(
        &mut bytes,
        &<Ellipse3D<f64> as Ellipse3DConstructor<f64>>::new(center, 3.0, 1.5, axis, reference)
            .unwrap(),
    );
    cases.push((bytes, |cursor| {
        let _ = read_ellipse(cursor);
    }));
    let mut bytes = Vec::new();
    write_cone(
        &mut bytes,
        &<ConicalSurface3D<f64> as ConicalSurface3DConstructor<f64>>::new(
            center,
            axis,
            reference,
            2.0,
            Angle::from_radians(0.3),
        )
        .unwrap(),
    );
    cases.push((bytes, |cursor| {
        let _ = read_cone(cursor);
    }));
    let mut bytes = Vec::new();
    write_torus(
        &mut bytes,
        &<TorusSurface3D<f64> as TorusSurface3DConstructor<f64>>::new(
            center, axis, reference, 5.0, 1.5,
        )
        .unwrap(),
    );
    cases.push((bytes, |cursor| {
        let _ = read_torus(cursor);
    }));
    for (bytes, read) in cases {
        for position in 0..bytes.len() {
            for value in [0x00, 0x01, 0x7f, 0x80, 0xff] {
                let mut corrupted = bytes.clone();
                corrupted[position] = value;
                read(&mut corrupted.as_slice());
            }
            if position + 4 <= bytes.len() {
                let mut corrupted = bytes.clone();
                corrupted[position..position + 4].copy_from_slice(&u32::MAX.to_le_bytes());
                read(&mut corrupted.as_slice());
            }
            read(&mut &bytes[..position]);
        }
    }
}
