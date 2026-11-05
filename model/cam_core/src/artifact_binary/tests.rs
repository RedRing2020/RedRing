//! `artifact_binary` のテスト

use super::*;
use crate::toolpath::TOOLPATH_SCHEMA_VERSION;

/// ゴールデンバイト: toolpath 最小ペイロード (tool_id="t", Down, 1 contour, 1 line segment)
///
/// Wire layout を一度確定させるための回帰テスト用定数。
/// この定数を変更する場合は必ず `FORMAT_VERSION_MINOR_V1` を +1 してください。
const GOLDEN_TOOLPATH_MINIMAL: &[u8] = &[
    // tool_id: len=1
    0x01, 0x00, // tool_id: "t"
    0x74, // cutting_direction: Down=0
    0x00, // ext_attributes: count=0
    0x00, 0x00, 0x00, 0x00, // approach_segments: count=0
    0x00, 0x00, 0x00, 0x00, // contour_levels: count=1
    0x01, 0x00, 0x00, 0x00, // contour level_index=0
    0x00, 0x00, 0x00, 0x00, // z_level=0.0
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // segments: count=1
    0x01, 0x00, 0x00, 0x00, // segment_type=Cutting=0
    0x00, // feed_rate=1.0 (f64 LE)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // ext_attributes: count=0
    0x00, 0x00, 0x00, 0x00, // geometry_type=Line=0
    0x00, // start: (0.0, 0.0, 0.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // end: (1.0, 0.0, 0.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // retract_segments: count=0
    0x00, 0x00, 0x00, 0x00,
];

/// ゴールデンバイト: interference 最小ペイロード (1 event, Tool kind)
///
/// Wire layout を一度確定させるための回帰テスト用定数。
/// この定数を変更する場合は必ず `FORMAT_VERSION_MINOR_V1` を +1 してください。
const GOLDEN_INTERFERENCE_MINIMAL: &[u8] = &[
    // event_count=1
    0x01, 0x00, 0x00, 0x00, // sample_index=0
    0x00, 0x00, 0x00, 0x00, // tool_id: len=1
    0x01, 0x00, // tool_id: "t"
    0x74, // position: (0.0, 0.0, 0.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // normal: (0.0, 0.0, 1.0)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F, // penetration_depth=0.5 (f64 LE)
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x3F, // kind=Tool=1
    0x01,
];

#[test]
fn header_round_trip_toolpath() {
    let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 4096);

    let mut bytes = Vec::new();
    header.write_to(&mut bytes).unwrap();

    assert_eq!(bytes.len(), ArtifactHeaderV1::FIXED_HEADER_LEN);

    let decoded = ArtifactHeaderV1::read_from(&mut bytes.as_slice()).unwrap();

    assert_eq!(decoded.kind, ArtifactKind::ToolPath);
    assert_eq!(decoded.version_major, FORMAT_VERSION_MAJOR_V1);
    assert_eq!(decoded.version_minor, FORMAT_VERSION_MINOR_V1);
    assert_eq!(decoded.payload_len, 4096);
    assert_eq!(decoded.unit, LengthUnit::Millimeter);
    assert_eq!(decoded.frame, CoordinateFrame::WorldRightHandedZUp);
}

#[test]
fn ext_attribute_round_trip() {
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
    header
        .ext_attributes
        .push(ExtAttribute::new(-1_i16, b"sys-note".to_vec()));
    header
        .ext_attributes
        .push(ExtAttribute::new(2_i16, vec![0xDE, 0xAD]));

    let mut bytes = Vec::new();
    header.write_to(&mut bytes).unwrap();

    // fixed(22) + attr1(tag2+len2+data8=12) + attr2(tag2+len2+data2=6)
    assert_eq!(bytes.len(), ArtifactHeaderV1::FIXED_HEADER_LEN + 12 + 6);

    let decoded = ArtifactHeaderV1::read_from(&mut bytes.as_slice()).unwrap();
    assert_eq!(decoded.ext_attributes.len(), 2);
    assert_eq!(decoded.ext_attributes[0].tag, -1);
    assert_eq!(decoded.ext_attributes[0].data, b"sys-note");
    assert_eq!(decoded.ext_attributes[1].tag, 2);
    assert_eq!(decoded.ext_attributes[1].data, vec![0xDE, 0xAD]);
}

#[test]
fn unsupported_major_version_is_rejected() {
    let mut header = ArtifactHeaderV1::new(ArtifactKind::Interference, 10);
    header.version_major = 1;

    let result = header.ensure_supported_major();
    assert!(matches!(
        result,
        Err(BinaryFormatError::UnsupportedMajorVersion {
            found: 1,
            supported: 0
        })
    ));
}

#[test]
fn version_sync_with_toolpath_schema_constant() {
    assert_eq!(
        TOOLPATH_SCHEMA_VERSION,
        (FORMAT_VERSION_MAJOR_V1, FORMAT_VERSION_MINOR_V1)
    );
}

/// wire layout バージョンのリテラルピンテスト。
///
/// wire layout を変更して GOLDEN_* を更新する際は以下の 3 か所をセットで更新してください:
///   1. `FORMAT_VERSION_MINOR_V1` を +1
///   2. `GOLDEN_TOOLPATH_MINIMAL` / `GOLDEN_INTERFERENCE_MINIMAL` を新 layout に合わせて更新
///   3. このテストの `PINNED_MINOR` を `FORMAT_VERSION_MINOR_V1` の新しい値に更新
///      (= `KNOWN_MINOR_VERSIONS_V0` への追記も同時に必要)
#[test]
fn version_pin_matches_golden_generation() {
    /// golden 定数が生成された時点の FORMAT_VERSION_MINOR_V1。
    /// wire layout を変更して golden を更新する場合はここも同時に変更してください。
    /// v0.2 は切削方向の値を足しただけ、v0.3 は切削シミュレーションの中身だけを変えたもので、
    /// ToolPath・Interference の layout は v0.1 と同じため、golden は変えていない。
    const PINNED_MINOR: u16 = 3;
    assert_eq!(
        FORMAT_VERSION_MINOR_V1, PINNED_MINOR,
        "FORMAT_VERSION_MINOR_V1 が変わりました。\
         GOLDEN_TOOLPATH_MINIMAL / GOLDEN_INTERFERENCE_MINIMAL を新 layout に合わせ、\
         KNOWN_MINOR_VERSIONS_V0 に新しい minor を追加し、\
         この PINNED_MINOR も更新してください。"
    );
    assert!(
        KNOWN_MINOR_VERSIONS_V0.contains(&PINNED_MINOR),
        "PINNED_MINOR={PINNED_MINOR} が KNOWN_MINOR_VERSIONS_V0 に含まれていません。\
         バージョンを +1 した場合は KNOWN_MINOR_VERSIONS_V0 に追記してください。"
    );
}

#[test]
fn known_minor_is_accepted() {
    let header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
    assert_eq!(
        header.compatibility_decision(),
        CompatibilityDecision::Accept
    );
    assert!(header.ensure_acceptable_version().is_ok());
}

#[test]
fn unknown_minor_is_rejected() {
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
    header.version_minor = 99;

    assert_eq!(
        header.compatibility_decision(),
        CompatibilityDecision::Reject
    );
    assert!(matches!(
        header.ensure_acceptable_version(),
        Err(BinaryFormatError::UnsupportedMinorVersion {
            major: 0,
            found_minor: 99,
            ..
        })
    ));
}

#[test]
fn different_major_is_convert_decision() {
    let mut header = ArtifactHeaderV1::new(ArtifactKind::Interference, 0);
    header.version_major = 2;
    header.version_minor = 0;

    assert_eq!(
        header.compatibility_decision(),
        CompatibilityDecision::Convert
    );
    assert!(matches!(
        header.ensure_acceptable_version(),
        Err(BinaryFormatError::ConverterRequired {
            found_major: 2,
            found_minor: 0
        })
    ));
}

#[test]
fn unknown_magic_is_rejected() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"ABCD");
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&0_u16.to_le_bytes());
    bytes.push(1_u8);
    bytes.push(1_u8);
    bytes.extend_from_slice(&[0_u8; 4]);
    bytes.extend_from_slice(&0_u64.to_le_bytes());

    let result = ArtifactHeaderV1::read_from(&mut bytes.as_slice());
    assert!(matches!(result, Err(BinaryFormatError::UnknownMagic(_))));
}

#[test]
fn reserved_tag_zero_is_rejected_on_write() {
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
    header
        .ext_attributes
        .push(ExtAttribute::new(0, b"x".to_vec()));

    let mut bytes = Vec::new();
    let result = header.write_to(&mut bytes);
    assert!(matches!(
        result,
        Err(BinaryFormatError::ReservedExtAttributeTag(0))
    ));
}

#[test]
fn reserved_tag_zero_is_rejected_on_read() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&TOOLPATH_MAGIC);
    bytes.extend_from_slice(&FORMAT_VERSION_MAJOR_V1.to_le_bytes());
    bytes.extend_from_slice(&FORMAT_VERSION_MINOR_V1.to_le_bytes());
    bytes.push(LengthUnit::Millimeter as u8);
    bytes.push(CoordinateFrame::WorldRightHandedZUp as u8);
    bytes.extend_from_slice(&(5_u32).to_le_bytes());
    bytes.extend_from_slice(&(0_u64).to_le_bytes());
    bytes.extend_from_slice(&(0_i16).to_le_bytes());
    bytes.extend_from_slice(&(1_u16).to_le_bytes());
    bytes.push(0xAA);

    let result = ArtifactHeaderV1::read_from(&mut bytes.as_slice());
    assert!(matches!(
        result,
        Err(BinaryFormatError::ReservedExtAttributeTag(0))
    ));
}

#[test]
fn every_cutting_direction_round_trips() {
    for direction in [
        CuttingDirection::Down,
        CuttingDirection::Up,
        CuttingDirection::Mixed,
    ] {
        let toolpath = ToolPath::new(
            "tool-1".to_string(),
            direction,
            vec![],
            vec![ContourLevelPath::new(
                0,
                0.0,
                vec![PathSegment::new_line(
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(10.0, 0.0, 0.0),
                    SegmentType::Cutting { feed_rate: 600.0 },
                )],
            )],
            vec![],
        );
        let mut bytes = Vec::new();
        write_toolpath_payload_v1(&mut bytes, &toolpath).unwrap();
        assert_eq!(
            read_toolpath_payload_v1(&mut bytes.as_slice()).unwrap(),
            toolpath
        );
    }
}

#[test]
fn v0_1_artifacts_are_still_accepted() {
    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, 0);
    header.version_minor = 1;
    assert_eq!(
        header.compatibility_decision(),
        CompatibilityDecision::Accept
    );
    assert!(header.ensure_acceptable_version().is_ok());
}

#[test]
fn toolpath_payload_round_trip_minimal_line() {
    let toolpath = ToolPath::new(
        "tool-1".to_string(),
        CuttingDirection::Down,
        vec![PathSegment::new_line(
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(0.0, 0.0, 0.0),
            SegmentType::Approach { feed_rate: 300.0 },
        )],
        vec![ContourLevelPath::new(
            0,
            -1.0,
            vec![PathSegment::new_line(
                Point3D::new(0.0, 0.0, -1.0),
                Point3D::new(10.0, 0.0, -1.0),
                SegmentType::Cutting { feed_rate: 600.0 },
            )],
        )],
        vec![PathSegment::new_line(
            Point3D::new(10.0, 0.0, -1.0),
            Point3D::new(10.0, 0.0, 5.0),
            SegmentType::Retract { feed_rate: 500.0 },
        )],
    );

    let mut bytes = Vec::new();
    write_toolpath_payload_v1(&mut bytes, &toolpath).unwrap();
    let decoded = read_toolpath_payload_v1(&mut bytes.as_slice()).unwrap();

    assert_eq!(decoded, toolpath);
}

#[test]
fn interference_payload_round_trip_minimal() {
    let payload = InterferencePayload {
        events: vec![InterferenceEvent {
            sample_index: 10,
            tool_id: "tool-1".to_string(),
            position: Point3D::new(1.0, 2.0, 3.0),
            normal: Point3D::new(0.0, 0.0, 1.0),
            penetration_depth: 0.25,
            kind: InterferenceKind::Holder,
        }],
    };

    let mut bytes = Vec::new();
    write_interference_payload_v1(&mut bytes, &payload).unwrap();
    let decoded = read_interference_payload_v1(&mut bytes.as_slice()).unwrap();

    assert_eq!(decoded, payload);
}

#[test]
fn interference_payload_empty_events_round_trip() {
    let payload = InterferencePayload { events: vec![] };

    let mut bytes = Vec::new();
    write_interference_payload_v1(&mut bytes, &payload).unwrap();
    let decoded = read_interference_payload_v1(&mut bytes.as_slice()).unwrap();

    assert_eq!(decoded.events.len(), 0);
}

#[test]
fn unknown_interference_kind_is_rejected() {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&1_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&6_u16.to_le_bytes());
    bytes.extend_from_slice(b"tool-1");
    bytes.extend_from_slice(&1.0_f64.to_le_bytes());
    bytes.extend_from_slice(&2.0_f64.to_le_bytes());
    bytes.extend_from_slice(&3.0_f64.to_le_bytes());
    bytes.extend_from_slice(&0.0_f64.to_le_bytes());
    bytes.extend_from_slice(&0.0_f64.to_le_bytes());
    bytes.extend_from_slice(&1.0_f64.to_le_bytes());
    bytes.extend_from_slice(&0.1_f64.to_le_bytes());
    bytes.push(9_u8);

    let result = read_interference_payload_v1(&mut bytes.as_slice());
    assert!(matches!(
        result,
        Err(BinaryFormatError::UnknownInterferenceKind(9))
    ));
}

// ----- wire layout regression tests -----
// これらのテストは Write/Read どちらか一方だけが変更された場合に失敗します。
// - toolpath_wire_write_regression: Writer だけが変わると失敗
// - toolpath_wire_read_regression:  Reader だけが変わると失敗
// - interference_wire_write_regression / interference_wire_read_regression: 同上
// ゴールデン定数 (GOLDEN_TOOLPATH_MINIMAL / GOLDEN_INTERFERENCE_MINIMAL) を更新する場合は
// 必ず FORMAT_VERSION_MINOR_V1 を +1 してください。

fn make_minimal_toolpath() -> ToolPath<f64> {
    ToolPath::new(
        "t".to_string(),
        CuttingDirection::Down,
        vec![],
        vec![ContourLevelPath::new(
            0,
            0.0,
            vec![PathSegment::new_line(
                Point3D::new(0.0, 0.0, 0.0),
                Point3D::new(1.0, 0.0, 0.0),
                SegmentType::Cutting { feed_rate: 1.0 },
            )],
        )],
        vec![],
    )
}

fn make_minimal_interference() -> InterferencePayload {
    InterferencePayload {
        events: vec![InterferenceEvent {
            sample_index: 0,
            tool_id: "t".to_string(),
            position: Point3D::new(0.0, 0.0, 0.0),
            normal: Point3D::new(0.0, 0.0, 1.0),
            penetration_depth: 0.5,
            kind: InterferenceKind::Tool,
        }],
    }
}

#[test]
fn toolpath_wire_write_regression() {
    let mut bytes = Vec::new();
    write_toolpath_payload_v1(&mut bytes, &make_minimal_toolpath()).unwrap();
    assert_eq!(
        bytes, GOLDEN_TOOLPATH_MINIMAL,
        "toolpath の wire layout が変わっています。\
         FORMAT_VERSION_MINOR_V1 を +1 して GOLDEN_TOOLPATH_MINIMAL を更新してください。"
    );
}

#[test]
fn toolpath_wire_read_regression() {
    let mut bytes = GOLDEN_TOOLPATH_MINIMAL;
    let decoded = read_toolpath_payload_v1(&mut bytes).unwrap();
    assert_eq!(
        decoded,
        make_minimal_toolpath(),
        "toolpath Reader がゴールデンバイトを正しく解釈できません。\
         Reader の変更が Writer と整合しているか確認してください。"
    );
}

#[test]
fn interference_wire_write_regression() {
    let mut bytes = Vec::new();
    write_interference_payload_v1(&mut bytes, &make_minimal_interference()).unwrap();
    assert_eq!(
        bytes, GOLDEN_INTERFERENCE_MINIMAL,
        "interference の wire layout が変わっています。\
         FORMAT_VERSION_MINOR_V1 を +1 して GOLDEN_INTERFERENCE_MINIMAL を更新してください。"
    );
}

#[test]
fn interference_wire_read_regression() {
    let mut bytes = GOLDEN_INTERFERENCE_MINIMAL;
    let decoded = read_interference_payload_v1(&mut bytes).unwrap();
    assert_eq!(
        decoded,
        make_minimal_interference(),
        "interference Reader がゴールデンバイトを正しく解釈できません。\
         Reader の変更が Writer と整合しているか確認してください。"
    );
}

// ----- read_artifact_v1 統合エントリーポイントのテスト -----

fn make_toolpath_artifact_bytes(minor: u16) -> Vec<u8> {
    let toolpath = make_minimal_toolpath();
    let mut payload_bytes = Vec::new();
    write_toolpath_payload_v1(&mut payload_bytes, &toolpath).unwrap();

    let mut header = ArtifactHeaderV1::new(ArtifactKind::ToolPath, payload_bytes.len() as u64);
    header.version_minor = minor;
    let mut bytes = Vec::new();
    header.write_to(&mut bytes).unwrap();
    bytes.extend_from_slice(&payload_bytes);
    bytes
}

fn make_interference_artifact_bytes(minor: u16) -> Vec<u8> {
    let interference = make_minimal_interference();
    let mut payload_bytes = Vec::new();
    write_interference_payload_v1(&mut payload_bytes, &interference).unwrap();

    let mut header = ArtifactHeaderV1::new(ArtifactKind::Interference, payload_bytes.len() as u64);
    header.version_minor = minor;
    let mut bytes = Vec::new();
    header.write_to(&mut bytes).unwrap();
    bytes.extend_from_slice(&payload_bytes);
    bytes
}

#[test]
fn read_artifact_v1_returns_toolpath_payload() {
    let bytes = make_toolpath_artifact_bytes(FORMAT_VERSION_MINOR_V1);
    let (header, payload) = read_artifact_v1(&mut bytes.as_slice()).unwrap();

    assert_eq!(header.kind, ArtifactKind::ToolPath);
    assert!(matches!(payload, ArtifactPayload::ToolPath(_)));
    if let ArtifactPayload::ToolPath(tp) = payload {
        assert_eq!(tp, make_minimal_toolpath());
    }
}

#[test]
fn read_artifact_v1_rejects_unsupported_minor() {
    let bytes = make_toolpath_artifact_bytes(99);
    let result = read_artifact_v1(&mut bytes.as_slice());
    assert!(
        matches!(
            result,
            Err(BinaryFormatError::UnsupportedMinorVersion {
                found_minor: 99,
                ..
            })
        ),
        "バージョン不一致は payload に達する前に Err を返す必要があります"
    );
}

#[test]
fn read_toolpath_artifact_v1_returns_toolpath() {
    let bytes = make_toolpath_artifact_bytes(FORMAT_VERSION_MINOR_V1);
    let (header, toolpath) = read_toolpath_artifact_v1(&mut bytes.as_slice()).unwrap();

    assert_eq!(header.kind, ArtifactKind::ToolPath);
    assert_eq!(toolpath, make_minimal_toolpath());
}

#[test]
fn read_interference_artifact_v1_returns_interference() {
    let bytes = make_interference_artifact_bytes(FORMAT_VERSION_MINOR_V1);
    let (header, interference) = read_interference_artifact_v1(&mut bytes.as_slice()).unwrap();

    assert_eq!(header.kind, ArtifactKind::Interference);
    assert_eq!(interference, make_minimal_interference());
}

#[test]
fn read_toolpath_artifact_v1_rejects_interference_payload() {
    let bytes = make_interference_artifact_bytes(FORMAT_VERSION_MINOR_V1);
    let result = read_toolpath_artifact_v1(&mut bytes.as_slice());

    assert!(matches!(
        result,
        Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::ToolPath,
            found: ArtifactKind::Interference,
        })
    ));
}

#[test]
fn read_interference_artifact_v1_rejects_toolpath_payload() {
    let bytes = make_toolpath_artifact_bytes(FORMAT_VERSION_MINOR_V1);
    let result = read_interference_artifact_v1(&mut bytes.as_slice());

    assert!(matches!(
        result,
        Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::Interference,
            found: ArtifactKind::ToolPath,
        })
    ));
}
