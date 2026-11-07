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
