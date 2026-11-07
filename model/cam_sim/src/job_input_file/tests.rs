use super::*;

use cam_core::{CoordinateFrame, LengthUnit};
use geo_algorithms::{
    NurbsSurface3D, NurbsSurface3DConstructor, TriangleMesh3D, TriangleMesh3DConstructor,
};

fn mesh() -> SolverGeometry {
    SolverGeometry::TriangleMesh(
        TriangleMesh3D::new(
            vec![
                Point3D::new(0.0, 0.0, 1.0),
                Point3D::new(10.0, 0.0, 1.5),
                Point3D::new(10.0, 10.0, 2.0),
                Point3D::new(0.0, 10.0, 0.25),
            ],
            vec![[0, 1, 2], [0, 2, 3]],
        )
        .unwrap(),
    )
}

/// 重みのある 1 次の曲面と、重みのない 2×1 次の曲面
fn nurbs() -> SolverGeometry {
    let rational = NurbsSurface3D::new(
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
    .unwrap();
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
    SolverGeometry::NurbsSurfaceSet(vec![rational, plain])
}

fn cam_input(geometry: SolverGeometry, tool: Tool<f64>) -> CamSolverInput {
    CamSolverInput {
        operation_id: "工程-1".to_string(),
        tool,
        geometry,
        operation: OperationSpec::Scanline(ScanlineParams {
            stepover: 1.25,
            feed_rate: 900.0,
            tool_range: ScanlineToolRange::GeometryExtent,
            cut_direction: ScanlineCutDirection::ZigZag,
            angle_degrees: -37.5,
        }),
        units: LengthUnit::Millimeter,
        coordinate_frame: CoordinateFrame::WorldRightHandedZUp,
        chord_tolerance: 0.001,
        tessellation_limits: TessellationLimits {
            max_subdivisions: 7,
            max_refinement_iterations: 3,
            max_vertices_per_surface: 123_456,
        },
        boundary: Some(MachiningBoundary::Rectangle(RectangleBoundary {
            rect_min: [-1.0, -2.0],
            rect_max: [11.0, 12.0],
        })),
    }
}

fn sim_input(snapshot_interval: SnapshotInterval) -> CuttingSimulationInput {
    CuttingSimulationInput {
        stock: Aabb3D::new(
            Point3D::new(-1.0, -2.0, -3.0),
            Point3D::new(11.0, 12.0, 4.0),
        ),
        tool: Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 40.0),
        precision: 0.05,
        snapshot_interval,
        product: mesh(),
        tolerance: 0.001,
    }
}

fn cam_bytes(input: &CamSolverInput) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_cam_solver_input_v1(&mut bytes, input).unwrap();
    bytes
}

fn sim_bytes(input: &CuttingSimulationInput) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_cutting_simulation_input_v1(&mut bytes, input).unwrap();
    bytes
}

#[test]
fn cam_solver_input_round_trips() {
    let mut no_length = Tool::flat_end_mill("EM8".to_string(), 8.0, 30.0);
    no_length.cutting_length = None;
    let inputs = [
        cam_input(mesh(), Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0)),
        cam_input(nurbs(), no_length),
        CamSolverInput {
            boundary: None,
            ..cam_input(
                mesh(),
                Tool::radius_end_mill("R".to_string(), 10.0, 2.0, 50.0),
            )
        },
    ];
    for input in inputs {
        let bytes = cam_bytes(&input);
        let read = read_cam_solver_input_v1(&mut bytes.as_slice()).unwrap();
        assert_eq!(read.operation_id, input.operation_id);
        assert_eq!(read.tool, input.tool);
        assert_eq!(read.operation, input.operation);
        assert_eq!(read.boundary, input.boundary);
        assert_eq!(read.chord_tolerance, input.chord_tolerance);
        assert_eq!(read.tessellation_limits, input.tessellation_limits);
        assert_eq!(read.geometry.kind(), input.geometry.kind());
        // 形状を含め、書き直すと同じバイト列になる
        assert_eq!(cam_bytes(&read), bytes);
    }
}

#[test]
fn cutting_simulation_input_round_trips() {
    let intervals = [
        SnapshotInterval::ByToolAndGrid,
        SnapshotInterval::ByDistance {
            interval_mm: 2.5,
            include_segment_endpoints: true,
        },
        SnapshotInterval::ByAutoDistance {
            target_count: 300,
            min_interval_mm: 0.5,
            include_segment_endpoints: false,
        },
    ];
    for interval in intervals {
        let input = CuttingSimulationInput {
            product: nurbs(),
            ..sim_input(interval)
        };
        let bytes = sim_bytes(&input);
        let read = read_cutting_simulation_input_v1(&mut bytes.as_slice()).unwrap();
        assert_eq!(read.stock, input.stock);
        assert_eq!(read.tool, input.tool);
        assert_eq!(read.precision, input.precision);
        assert_eq!(read.tolerance, input.tolerance);
        assert_eq!(sim_bytes(&read), bytes);
    }
}

#[test]
fn reading_the_other_kind_of_input_is_a_kind_mismatch() {
    let cam = cam_bytes(&cam_input(
        mesh(),
        Tool::ball_end_mill("B".to_string(), 6.0, 30.0),
    ));
    let sim = sim_bytes(&sim_input(SnapshotInterval::ByToolAndGrid));
    assert!(matches!(
        read_cutting_simulation_input_v1(&mut cam.as_slice()),
        Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::CuttingSimulationInput,
            found: ArtifactKind::CamSolverInput,
        })
    ));
    assert!(matches!(
        read_cam_solver_input_v1(&mut sim.as_slice()),
        Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::CamSolverInput,
            found: ArtifactKind::CuttingSimulationInput,
        })
    ));
}

#[test]
fn older_minor_versions_and_trailing_bytes_are_rejected() {
    let mut bytes = sim_bytes(&sim_input(SnapshotInterval::ByToolAndGrid));
    // 共通のヘッダの minor（magic・major の後ろ）を前の版にする
    let mut older = bytes.clone();
    older[6..8].copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        read_cutting_simulation_input_v1(&mut older.as_slice()),
        Err(BinaryFormatError::UnsupportedMinorVersion { found_minor: 2, .. })
    ));

    // 本体の長さ（ヘッダの最後の u64）を 1 増やし、余分な 1 バイトを足す
    let length_at = ArtifactHeaderV1::FIXED_HEADER_LEN - 8;
    let length = u64::from_le_bytes(bytes[length_at..length_at + 8].try_into().unwrap());
    bytes[length_at..length_at + 8].copy_from_slice(&(length + 1).to_le_bytes());
    bytes.push(0);
    assert!(matches!(
        read_cutting_simulation_input_v1(&mut bytes.as_slice()),
        Err(BinaryFormatError::InvalidPayload("trailing bytes"))
    ));
}

#[test]
fn counts_larger_than_the_remaining_bytes_are_rejected_before_allocating() {
    let input = cam_input(mesh(), Tool::ball_end_mill("B".to_string(), 6.0, 30.0));
    let bytes = cam_bytes(&input);
    // 三角形メッシュの頂点の数（形状の種類の直後）に大きな値を書く
    let geometry_at = bytes.len() - (1 + 4 + 4 * 24 + 4 + 2 * 12);
    assert_eq!(bytes[geometry_at], GEOMETRY_TRIANGLE_MESH);
    let mut corrupted = bytes.clone();
    corrupted[geometry_at + 1..geometry_at + 5].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(
        read_cam_solver_input_v1(&mut corrupted.as_slice()),
        Err(BinaryFormatError::InvalidPayload("mesh.vertices"))
    ));
}

#[test]
fn inputs_cut_off_in_the_geometry_are_an_unexpected_end() {
    // 最後の曲面は重みを持たず、最後の 1 バイトは重みの有無
    let input = cam_input(nurbs(), Tool::ball_end_mill("B".to_string(), 6.0, 30.0));
    let mut bytes = cam_bytes(&input);
    // 重みの有無の 1 バイトを落とし、ヘッダの本体の長さも 1 つ減らす
    bytes.pop();
    let length_at = ArtifactHeaderV1::FIXED_HEADER_LEN - 8;
    let length = u64::from_le_bytes(bytes[length_at..length_at + 8].try_into().unwrap());
    bytes[length_at..length_at + 8].copy_from_slice(&(length - 1).to_le_bytes());
    let Err(BinaryFormatError::Io(error)) = read_cam_solver_input_v1(&mut bytes.as_slice()) else {
        panic!("a cut-off geometry must be an I/O error");
    };
    assert_eq!(error.kind(), std::io::ErrorKind::UnexpectedEof);
}

/// 正しいファイルのバイトを壊したもの（書き換え・大きな値・切り詰め）を読んでも落ちない
fn assert_corruptions_do_not_panic(bytes: &[u8], read: impl Fn(&[u8])) {
    for position in 0..bytes.len() {
        for value in [0x00, 0x01, 0x7f, 0x80, 0xff] {
            let mut corrupted = bytes.to_vec();
            corrupted[position] = value;
            read(&corrupted);
        }
        if position + 4 <= bytes.len() {
            let mut corrupted = bytes.to_vec();
            corrupted[position..position + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            read(&corrupted);
        }
        read(&bytes[..position]);
    }
}

#[test]
fn corrupted_cam_solver_inputs_do_not_panic() {
    for geometry in [mesh(), nurbs()] {
        let bytes = cam_bytes(&cam_input(
            geometry,
            Tool::ball_end_mill("B".to_string(), 6.0, 30.0),
        ));
        assert_corruptions_do_not_panic(&bytes, |corrupted| {
            let _ = read_cam_solver_input_v1(&mut &corrupted[..]);
        });
    }
}

#[test]
fn corrupted_cutting_simulation_inputs_do_not_panic() {
    let input = CuttingSimulationInput {
        product: nurbs(),
        ..sim_input(SnapshotInterval::ByAutoDistance {
            target_count: 10,
            min_interval_mm: 1.0,
            include_segment_endpoints: true,
        })
    };
    let bytes = sim_bytes(&input);
    assert_corruptions_do_not_panic(&bytes, |corrupted| {
        let _ = read_cutting_simulation_input_v1(&mut &corrupted[..]);
    });
}

#[test]
fn tessellation_limits_above_the_defaults_are_rejected() {
    let mut input = cam_input(nurbs(), Tool::ball_end_mill("B".to_string(), 6.0, 30.0));
    input.tessellation_limits = TessellationLimits::default();
    let bytes = cam_bytes(&input);
    assert!(read_cam_solver_input_v1(&mut bytes.as_slice()).is_ok());
    for limits in [
        TessellationLimits {
            max_subdivisions: u32::MAX,
            ..TessellationLimits::default()
        },
        TessellationLimits {
            max_refinement_iterations: 7,
            ..TessellationLimits::default()
        },
        TessellationLimits {
            max_vertices_per_surface: 1_000_001,
            ..TessellationLimits::default()
        },
    ] {
        input.tessellation_limits = limits;
        let bytes = cam_bytes(&input);
        assert!(matches!(
            read_cam_solver_input_v1(&mut bytes.as_slice()),
            Err(BinaryFormatError::InvalidPayload("tessellation_limits"))
        ));
    }
}
