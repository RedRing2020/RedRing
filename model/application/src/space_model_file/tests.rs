use std::num::NonZeroU32;

use cam_core::{
    MachiningBoundary, MachiningStage, OperationSpec, RectangleBoundary, ScanlineCutDirection,
    ScanlineParams, ScanlineToolRange, StockMargins, ToleranceProfile, Tool,
};
use chrono::DateTime;
use geo_algorithms::{
    NurbsSurface3D, NurbsSurface3DConstructor, Point3D, TriangleMesh3D, TriangleMesh3DConstructor,
    TriangleMesh3DProperties,
};
use geo_entity::{
    AttributeCode, AttributeValue, Attributes, DisplayAttributes, EntityId, FaceId, Metadata,
    StrokePattern,
};

use super::*;
use crate::machining_sequence::{
    MachiningOperationAttributes, MachiningOperationConditions, MachiningOperationContent,
    MachiningOperationId, MachiningOperationRecord, MachiningRevisionKey, MachiningSequenceId,
    MachiningSequenceRecord, MachiningStock,
};
use crate::product_shape::{ProductFaceGeometry, ProductFaceRecord};

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn triangle() -> TriangleMesh3D<f64> {
    TriangleMesh3D::new(
        vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.5),
        ],
        vec![[0, 1, 2]],
    )
    .unwrap()
}

fn plane() -> NurbsSurface3D<f64> {
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

fn user_code() -> AttributeCode {
    AttributeCode::try_new(10_01_02_02).unwrap()
}

/// ID・時刻を決めた面の記録（メッシュの面は表示の属性・属性・メタデータを変える）
fn face_records() -> Vec<ProductFaceRecord> {
    let time = DateTime::from_timestamp(1_700_000_000, 123_456_789).unwrap();
    let metadata = Metadata {
        name: None,
        description: None,
        tags: Vec::new(),
        created_at: time,
        modified_at: time,
        custom: Default::default(),
    };
    let mut attributes = Attributes::new();
    attributes
        .set(user_code(), AttributeValue::Float(180.0))
        .unwrap();
    let mut named = metadata.clone();
    named.name = Some("面 1".to_string());
    named.tags.push("見切り".to_string());
    named
        .custom
        .insert("取り込み元".to_string(), "a.stl".to_string());
    vec![
        ProductFaceRecord {
            entity_id: EntityId::from_uuid(uuid(1)),
            face_id: FaceId::from_uuid(uuid(2)),
            geometry: ProductFaceGeometry::Mesh(triangle()),
            display: DisplayAttributes {
                color: [0.2, 0.4, 0.6, 1.0],
                line_style: StrokePattern::Dashed,
                line_width: 2.0,
                layer: "補助".to_string(),
                visible: false,
            },
            attributes,
            metadata: named,
        },
        ProductFaceRecord {
            entity_id: EntityId::from_uuid(uuid(3)),
            face_id: FaceId::from_uuid(uuid(4)),
            geometry: ProductFaceGeometry::Surface(plane()),
            display: DisplayAttributes::default(),
            attributes: Attributes::new(),
            metadata,
        },
    ]
}

fn content(name: &str) -> MachiningOperationContent {
    let mut tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    tool.cutting_length = None;
    MachiningOperationContent {
        attributes: MachiningOperationAttributes {
            name: name.to_string(),
            stage: MachiningStage::SemiFinish,
        },
        conditions: MachiningOperationConditions {
            spec: OperationSpec::Scanline(ScanlineParams {
                stepover: 0.5,
                feed_rate: 1200.0,
                tool_range: ScanlineToolRange::GeometryExtent,
                cut_direction: ScanlineCutDirection::ZigZag,
                angle_degrees: 45.0,
            }),
            tool,
            boundary: Some(MachiningBoundary::Rectangle(RectangleBoundary {
                rect_min: [1.0, 2.0],
                rect_max: [3.0, 4.0],
            })),
            tolerance_profile: ToleranceProfile::MoldFinish,
        },
    }
}

/// ID を決めた加工手順の記録（加工の対象の 1 つは製品形状にない面）
fn sequence_records() -> Vec<MachiningSequenceRecord> {
    let mut rough = content("荒");
    rough.attributes.stage = MachiningStage::Rough;
    rough.conditions.boundary = None;
    rough.conditions.tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    rough.conditions.tolerance_profile = ToleranceProfile::PressRough;
    vec![MachiningSequenceRecord {
        id: MachiningSequenceId::from_uuid(uuid(10)),
        name: "手順".to_string(),
        target_faces: vec![FaceId::from_uuid(uuid(2)), FaceId::from_uuid(uuid(99))],
        stock: MachiningStock {
            margins: StockMargins {
                minus_x: 1.0,
                plus_x: 2.0,
                minus_y: 3.0,
                plus_y: 4.0,
                bottom: 0.0,
                top: 5.0,
            },
            precision: 0.05,
        },
        operations: vec![
            MachiningOperationRecord {
                id: MachiningOperationId::from_uuid(uuid(11)),
                content: rough,
                revision: NonZeroU32::new(1).unwrap(),
                revision_key: MachiningRevisionKey::from_uuid(uuid(12)),
                conditions_set: true,
            },
            MachiningOperationRecord {
                id: MachiningOperationId::from_uuid(uuid(13)),
                content: content("中"),
                revision: NonZeroU32::new(3).unwrap(),
                revision_key: MachiningRevisionKey::from_uuid(uuid(14)),
                conditions_set: true,
            },
        ],
    }]
}

fn sample_model() -> SpaceModel {
    SpaceModel {
        id: SpaceModelId(uuid(100)),
        shapes: ProductShapeStore::from_records(face_records(), Vec::new()).unwrap(),
        sequences: MachiningSequenceStore::from_records(sequence_records()).unwrap(),
        toolpaths: ImportedToolpathStore::new(),
        preserved: Vec::new(),
        ext_attributes: Vec::new(),
    }
}

fn info() -> SpaceModelSaveInfo {
    SpaceModelSaveInfo {
        program_version: "0.1.3".to_string(),
        saved_at_ms: 1_700_000_000_123,
    }
}

fn bytes(model: &SpaceModel) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_space_model(&mut bytes, model, &info()).unwrap();
    bytes
}

fn read(bytes: &[u8]) -> Result<(SpaceModel, SpaceModelSaveInfo), SpaceModelFileError> {
    read_space_model(&mut &bytes[..])
}

/// 面の記録が同じか（形状は頂点・制御点で比べる）
fn assert_same_faces(actual: &[ProductFaceRecord], expected: &[ProductFaceRecord]) {
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected) {
        assert_eq!((a.entity_id, a.face_id), (e.entity_id, e.face_id));
        assert_eq!(a.display, e.display);
        assert_eq!(a.attributes.len(), e.attributes.len());
        assert_eq!(a.attributes.get(user_code()), e.attributes.get(user_code()));
        let (am, em) = (&a.metadata, &e.metadata);
        assert_eq!(
            (&am.name, &am.description, &am.tags, &am.custom),
            (&em.name, &em.description, &em.tags, &em.custom)
        );
        assert_eq!(
            (am.created_at, am.modified_at),
            (em.created_at, em.modified_at)
        );
        match (&a.geometry, &e.geometry) {
            (ProductFaceGeometry::Mesh(a), ProductFaceGeometry::Mesh(e)) => {
                assert_eq!(a.vertices(), e.vertices());
                assert_eq!(a.indices(), e.indices());
            }
            (ProductFaceGeometry::Surface(a), ProductFaceGeometry::Surface(e)) => {
                assert_eq!(format!("{a:?}"), format!("{e:?}"));
            }
            _ => panic!("the face kinds differ"),
        }
    }
}

#[test]
fn space_models_round_trip_with_the_same_ids_and_contents() {
    let model = sample_model();
    let (read_model, read_info) = read(&bytes(&model)).unwrap();
    assert_eq!(read_model.id, model.id);
    assert_eq!(read_info, info());
    assert_same_faces(&read_model.shapes.records(), &face_records());
    assert_eq!(read_model.sequences.records(), sequence_records());
}

/// 必須でない・知らない区画 `kind` を、空間モデルのバイト列の最後に足す
fn with_extra_section(
    bytes: &[u8],
    kind: [u8; 4],
    version: u16,
    flags: u16,
    body: &[u8],
) -> Vec<u8> {
    let header_len = ArtifactHeaderV1::FIXED_HEADER_LEN;
    let mut payload = bytes[header_len..].to_vec();
    let count = u32::from_le_bytes(payload[0..4].try_into().unwrap());
    payload[0..4].copy_from_slice(&(count + 1).to_le_bytes());
    let mut section = ByteWriter::default();
    write_section(&mut section, kind, version, flags, body);
    payload.extend_from_slice(&section.into_bytes());
    let mut out = Vec::new();
    ArtifactHeaderV1::new(ArtifactKind::SpaceModel, payload.len() as u64)
        .write_to(&mut out)
        .unwrap();
    out.extend_from_slice(&payload);
    out
}

#[test]
fn operations_whose_conditions_are_not_set_round_trip() {
    let mut model = sample_model();
    let mut records = sequence_records();
    records[0].operations[1].conditions_set = false;
    model.sequences = MachiningSequenceStore::from_records(records.clone()).unwrap();
    let written = bytes(&model);
    // 設定していない工程があるときだけ、区画を書く
    assert!(written.windows(4).any(|kind| kind == b"NSET"));
    assert!(
        !bytes(&sample_model())
            .windows(4)
            .any(|kind| kind == b"NSET")
    );
    let (read_model, _) = read(&written).unwrap();
    assert_eq!(read_model.sequences.records(), records);
}

#[test]
fn operations_not_in_the_sequences_are_ignored_when_reading_the_conditions_not_set() {
    // 前の版が工程を消して書き戻した区画（手順にない工程の ID）
    let body = super::codec::write_operations_not_set(&[MachiningOperationId::from_uuid(uuid(99))])
        .unwrap();
    let original = with_extra_section(&bytes(&sample_model()), *b"NSET", 1, 0, &body);
    let (model, _) = read(&original).unwrap();
    assert_eq!(model.sequences.records(), sequence_records());
}

#[test]
fn unknown_optional_sections_are_written_back_unchanged() {
    let original = with_extra_section(&bytes(&sample_model()), *b"XTRA", 7, 0x0002, b"later");
    // 読めない版の CL の区画も、知らない区画と同じに残す
    let original = with_extra_section(&original, *b"CLDT", 2, 0, b"newer toolpaths");
    let (model, _) = read(&original).unwrap();
    // 読んで書き直しても、知らない区画は同じ種類・版・印・中身のまま残る
    let rewritten = bytes(&model);
    assert_eq!(rewritten, original);
}

#[test]
fn files_that_cannot_be_opened_safely_are_rejected_with_the_reason() {
    let valid = bytes(&sample_model());
    let required = with_extra_section(&valid, *b"FEAT", 1, SECTION_REQUIRED, b"");
    assert!(matches!(
        read(&required),
        Err(SpaceModelFileError::UnknownRequiredSection(kind)) if kind == *b"FEAT"
    ));

    let duplicate = with_extra_section(&valid, INFO, 1, SECTION_REQUIRED, b"");
    assert!(matches!(
        read(&duplicate),
        Err(SpaceModelFileError::DuplicateSection(kind)) if kind == INFO
    ));

    // 先頭の区画（INFO）の版を上げる（区画の数の 4 バイトと種類の 4 バイトの後ろ）
    let mut newer = valid.clone();
    let at = ArtifactHeaderV1::FIXED_HEADER_LEN + 4 + 4;
    newer[at..at + 2].copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        read(&newer),
        Err(SpaceModelFileError::UnsupportedSectionVersion { kind, version: 2 }) if kind == INFO
    ));

    // 先頭の区画（INFO）の種類を、知らない必須でない区画に変えると、INFO がない
    let mut missing = valid.clone();
    let at = ArtifactHeaderV1::FIXED_HEADER_LEN + 4;
    missing[at..at + 4].copy_from_slice(b"XXXX");
    missing[at + 6..at + 8].copy_from_slice(&0_u16.to_le_bytes());
    assert!(matches!(
        read(&missing),
        Err(SpaceModelFileError::MissingSection(kind)) if kind == INFO
    ));

    // ほかの種類のファイル（ジョブの入力のヘッダ）
    let mut other = valid.clone();
    other[0..4].copy_from_slice(b"RRCI");
    assert!(matches!(
        read(&other),
        Err(SpaceModelFileError::Format(
            cam_core::BinaryFormatError::ArtifactKindMismatch { .. }
        ))
    ));
}

#[test]
fn bytes_after_the_body_are_rejected() {
    let mut bytes = bytes(&sample_model());
    bytes.push(0);
    assert!(matches!(
        read(&bytes),
        Err(SpaceModelFileError::InvalidSectionTable("trailing bytes"))
    ));
}

#[test]
fn header_extension_attributes_are_written_back_unchanged() {
    let valid = bytes(&sample_model());
    let payload = &valid[ArtifactHeaderV1::FIXED_HEADER_LEN..];
    let mut header = ArtifactHeaderV1::new(ArtifactKind::SpaceModel, payload.len() as u64);
    header.ext_attributes = vec![ExtAttribute::new(7, b"later".to_vec())];
    let mut original = Vec::new();
    header.write_to(&mut original).unwrap();
    original.extend_from_slice(payload);
    let (model, _) = read(&original).unwrap();
    assert_eq!(bytes(&model), original);
}

#[test]
fn all_known_header_minor_versions_are_accepted() {
    let mut older = bytes(&sample_model());
    for minor in cam_core::KNOWN_MINOR_VERSIONS_V0 {
        older[6..8].copy_from_slice(&minor.to_le_bytes());
        assert!(read(&older).is_ok(), "minor {minor}");
    }
}

#[test]
fn duplicate_ids_are_rejected_by_the_stores() {
    let mut faces = face_records();
    faces[1].face_id = faces[0].face_id;
    let model = SpaceModel {
        id: SpaceModelId(uuid(100)),
        // 保管先を通さずに書くため、重なった面をそれぞれ別の保管先に入れてから記録を並べる
        shapes: ProductShapeStore::from_records(vec![faces[0].clone()], Vec::new()).unwrap(),
        sequences: MachiningSequenceStore::new(),
        toolpaths: ImportedToolpathStore::new(),
        preserved: Vec::new(),
        ext_attributes: Vec::new(),
    };
    let mut payload = ByteWriter::default();
    payload.count(3, "sections").unwrap();
    write_section(
        &mut payload,
        INFO,
        1,
        SECTION_REQUIRED,
        &codec::write_info(model.id.0, &info()).unwrap(),
    );
    write_section(
        &mut payload,
        FACE,
        1,
        SECTION_REQUIRED,
        &codec::write_faces(&faces).unwrap(),
    );
    write_section(
        &mut payload,
        MSEQ,
        1,
        SECTION_REQUIRED,
        &codec::write_sequences(&[]).unwrap(),
    );
    let payload = payload.into_bytes();
    let mut file = Vec::new();
    ArtifactHeaderV1::new(ArtifactKind::SpaceModel, payload.len() as u64)
        .write_to(&mut file)
        .unwrap();
    file.extend_from_slice(&payload);
    assert!(matches!(
        read(&file),
        Err(SpaceModelFileError::ProductShape(
            ProductShapeRestoreError::DuplicateFace(_)
        ))
    ));
}

#[test]
fn corrupted_files_do_not_panic() {
    let bytes = bytes(&sample_model());
    for position in 0..bytes.len() {
        for value in [0x00, 0x01, 0x7f, 0x80, 0xff] {
            let mut corrupted = bytes.clone();
            corrupted[position] = value;
            let _ = read(&corrupted);
        }
        if position + 4 <= bytes.len() {
            let mut corrupted = bytes.clone();
            corrupted[position..position + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            let _ = read(&corrupted);
        }
        let _ = read(&bytes[..position]);
    }
}

#[test]
fn files_are_saved_without_leaving_a_partial_file_and_loaded_again() {
    let dir = std::env::temp_dir().join(format!("redring-space-model-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("model.rrm");
    fs::write(&path, b"old").unwrap();

    save_space_model(&path, &sample_model(), &info()).unwrap();
    // 一時のファイルを残さない
    let names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, vec![std::ffi::OsString::from("model.rrm")]);
    let (model, saved) = load_space_model(&path).unwrap();
    assert_eq!(model.id, SpaceModelId(uuid(100)));
    assert_eq!(saved, info());

    // 書けないときは、前のファイルを残し、一時のファイルも残さない
    let missing_dir = dir.join("missing").join("model.rrm");
    assert!(matches!(
        save_space_model(&missing_dir, &sample_model(), &info()),
        Err(SpaceModelFileError::Io(_))
    ));
    assert!(matches!(
        load_space_model(&missing_dir),
        Err(SpaceModelFileError::Io(_))
    ));
    let _ = fs::remove_dir_all(&dir);
}

/// 版 1 のモデルファイルのゴールデンのバイト列（`sample_model` と `info` を書いたもの。16 進数）
///
/// 書き方・読み方の片方だけを変えて形式が壊れないようにする。区画の中身を変えるときは、区画の版を上げて、この
/// バイト列を前の版として読めることを確かめる。
const GOLDEN_V1: &str = concat!(
    "52524d4600000300010100000000020400000000000003000000494e464f010001002100000000000000000000000000",
    "0000000000000000006405000000302e312e337b68e5cf8b010000464143450100010041020000000000000200000000",
    "000000000000000000000000000001000000000000000000000000000000020103000000000000000000000000000000",
    "000000000000000000000000000000000000f03f00000000000000000000000000000000000000000000000000000000",
    "0000f03f000000000000e03f01000000000000000100000002000000cdcc4c3ecdcccc3e9a99193f0000803f02000000",
    "4006000000e8a39ce58aa900010000005abe98000300000000008066400105000000e99da22031000100000009000000",
    "e8a68be58887e3828a00f153650000000015cd5b0700f153650000000015cd5b07010000000f000000e58f96e3828ae8",
    "bebce381bfe5858305000000612e73746c00000000000000000000000000000003000000000000000000000000000000",
    "0402010000000100000002000000020000000400000000000000000000000000000000000000000000000000f03f0000",
    "00000000f03f0400000000000000000000000000000000000000000000000000f03f000000000000f03f000000000000",
    "00000000000000000000000000000000000000000000000000000000000000001440000000000000f03f000000000000",
    "144000000000000000000000000000000040000000000000144000000000000014400000000000000840010000000000",
    "00f03f000000000000e03f000000000000e83f000000000000f03f0000803f0000803f0000803f0000803f010000803f",
    "0100000030010000000000000000000000f153650000000015cd5b0700f153650000000015cd5b07000000004d534551",
    "010001006c01000000000000010000000000000000000000000000000000000a06000000e6898be9a086020000000000",
    "000000000000000000000000000200000000000000000000000000000063000000000000f03f00000000000000400000",
    "0000000008400000000000001040000000000000000000000000000014409a9999999999a93f02000000000000000000",
    "0000000000000000000b010000000000000000000000000000000000000c03000000e88d920101000000000000e03f00",
    "00000000c09240020300000000008046400104000000454d313000000000000024400000000000000000010000000000",
    "00494000010000000000000000000000000000000d030000000000000000000000000000000000000e03000000e4b8ad",
    "0201000000000000e03f0000000000c0924002030000000000804640030400000042454d360000000000001840000000",
    "00000008400001000000000000f03f00000000000000400000000000000840000000000000104002",
);

fn golden() -> Vec<u8> {
    (0..GOLDEN_V1.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&GOLDEN_V1[at..at + 2], 16).unwrap())
        .collect()
}

#[test]
fn space_models_are_written_as_the_golden_bytes() {
    assert_eq!(bytes(&sample_model()), golden());
}

#[test]
fn the_golden_bytes_are_read() {
    let (model, saved) = read(&golden()).unwrap();
    assert_eq!(model.id, SpaceModelId(uuid(100)));
    assert_eq!(saved, info());
    assert_same_faces(&model.shapes.records(), &face_records());
    assert_eq!(model.sequences.records(), sequence_records());
}

#[test]
fn imported_toolpaths_round_trip_in_an_optional_section() {
    use crate::imported_toolpath::{ImportedToolpathRecord, tests::toolpath_artifact};
    let record = ImportedToolpathRecord {
        operation: MachiningOperationId::from_uuid(uuid(13)),
        revision: NonZeroU32::new(3).unwrap(),
        revision_key: MachiningRevisionKey::from_uuid(uuid(14)),
        edit_number: 2,
        artifact: toolpath_artifact(2.5),
    };
    let mut model = sample_model();
    model.toolpaths = ImportedToolpathStore::from_records(vec![record.clone()]).unwrap();
    let written = bytes(&model);
    let (read_model, _) = read(&written).unwrap();
    assert_eq!(read_model.toolpaths.records(), &[record]);
    // CL の区画は必須でない（古い版のアプリは、そのまま書き戻す）
    let body_at = written.len() - (4 + 48 + toolpath_artifact(2.5).len());
    let flags_at = body_at - 8 - 2;
    assert_eq!(&written[flags_at - 6..flags_at - 2], b"CLDT");
    assert_eq!(&written[flags_at..flags_at + 2], &0_u16.to_le_bytes());
    // CL を取り込んでいなければ、区画を書かない（版 1 のゴールデンのバイト列と同じ）
    assert_eq!(bytes(&sample_model()), golden());

    // 読めない CL は、開かない
    let mut broken = written.clone();
    let artifact_at = written.len() - toolpath_artifact(2.5).len();
    broken[artifact_at] = b'X';
    assert!(matches!(
        read(&broken),
        Err(SpaceModelFileError::ImportedToolpath(_))
    ));
}

/// 面単体と、同じ四面体の B-rep を共有する 2 つのシェル（1 つは配置と元の要素を持つ）の空間モデル
fn model_with_shells() -> SpaceModel {
    use crate::product_shape::test_fixtures::tetrahedron_brep;
    use crate::product_shape::{ShellSource, SimilarityTransform3D};
    use std::sync::Arc;
    let mut shapes = ProductShapeStore::from_records(face_records(), Vec::new()).unwrap();
    let brep = Arc::new(tetrahedron_brep());
    let placement = SimilarityTransform3D::rotation_about_axis(
        geo_algorithms::Point3D::new(1.0, 2.0, 3.0),
        geo_algorithms::Direction3D::new(0.0, 0.6, 0.8).unwrap(),
        geo_algorithms::Angle::from_radians(0.7),
    );
    let first = shapes
        .add_shell(
            Arc::clone(&brep),
            placement,
            Some(ShellSource {
                file_name: "assembly.stp".to_string(),
                occurrences: vec![12, 34],
                faces: vec![101, 102, 103, 104],
            }),
        )
        .unwrap();
    shapes
        .add_shell(brep, SimilarityTransform3D::identity(), None)
        .unwrap();
    shapes.set_visible(first, false).unwrap();
    SpaceModel {
        id: SpaceModelId(uuid(100)),
        shapes,
        sequences: MachiningSequenceStore::from_records(sequence_records()).unwrap(),
        toolpaths: ImportedToolpathStore::new(),
        preserved: Vec::new(),
        ext_attributes: Vec::new(),
    }
}

#[test]
fn shells_round_trip_with_the_same_breps_and_face_ids() {
    let model = model_with_shells();
    let written = bytes(&model);
    let (loaded, _) = read(&written).unwrap();

    let (before, after) = (model.shapes.shell_records(), loaded.shapes.shell_records());
    assert_eq!(after.len(), 2);
    for (a, b) in after.iter().zip(&before) {
        assert_eq!(a.entity_id, b.entity_id);
        assert_eq!(a.face_ids, b.face_ids);
        assert_eq!(a.source, b.source);
        assert_eq!(a.placement.to_matrix(), b.placement.to_matrix());
        assert_eq!(a.display, b.display);
        let (ra, rb) = (a.brep.records(), b.brep.records());
        assert_eq!(ra.vertices, rb.vertices);
        assert_eq!(ra.shells, rb.shells);
        assert_eq!(ra.solids, rb.solids);
        for (fa, fb) in ra.faces.iter().zip(&rb.faces) {
            assert_eq!((fa.same_sense, &fa.loops), (fb.same_sense, &fb.loops));
        }
    }
    // 同じ部品の出現は、読み直しても同じ表を共有する
    assert!(std::sync::Arc::ptr_eq(&after[0].brep, &after[1].brep));
    assert_same_faces(&loaded.shapes.records(), &model.shapes.records());
    // 開いて何も変えずに保存し直すと、同じバイト列になる
    assert_eq!(bytes(&loaded), written);
}

#[test]
fn models_without_shells_do_not_write_the_shell_section() {
    // シェルのないモデルファイルは、区画 SHEL を知らない前の版でも開ける
    let written = bytes(&sample_model());
    assert!(!written.windows(4).any(|window| window == SHEL));
    assert!(
        bytes(&model_with_shells())
            .windows(4)
            .any(|window| window == SHEL)
    );
}

#[test]
fn corrupted_files_with_shells_do_not_panic() {
    let bytes = bytes(&model_with_shells());
    for position in 0..bytes.len() {
        for value in [0x00, 0x7f, 0xff] {
            let mut corrupted = bytes.clone();
            corrupted[position] = value;
            let _ = read(&corrupted);
        }
        if position + 4 <= bytes.len() && position % 4 == 0 {
            let mut corrupted = bytes.clone();
            corrupted[position..position + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            let _ = read(&corrupted);
        }
        if position % 16 == 0 {
            let _ = read(&bytes[..position]);
        }
    }
}

/// ID・時刻・形を決めたシェルの記録（三角形の平面の面 1 つの B-rep を、平行移動して置く）
fn shell_record() -> crate::product_shape::ProductShellRecord {
    use crate::product_shape::{Brep, ProductShellRecord, ShellSource, SimilarityTransform3D};
    use geo_contracts::{InfiniteLine3DConstructor, Plane3DConstructor};
    use geo_topology::{
        CurveRef, EdgeUse, SurfaceRef, TopoInfiniteLine3D, TopoPlane3D, TopologyToleranceSettings,
    };
    let mut brep = Brep::new(TopologyToleranceSettings::new(1.0e-6)).unwrap();
    let points = [
        geo_topology::Point3D::new(0.0, 0.0, 0.0),
        geo_topology::Point3D::new(2.0, 0.0, 0.0),
        geo_topology::Point3D::new(0.0, 2.0, 0.0),
    ];
    let vertices: Vec<_> = points
        .iter()
        .map(|&point| brep.add_vertex(point, 0.0).unwrap())
        .collect();
    let edges: Vec<_> = (0..3)
        .map(|index| {
            let (a, b) = (index, (index + 1) % 3);
            let line = TopoInfiniteLine3D::from_two_points(points[a], points[b]).unwrap();
            brep.add_edge(
                vertices[a],
                vertices[b],
                CurveRef::Line(line),
                (0.0, points[a].distance_to(points[b])),
                0.0,
            )
            .unwrap()
        })
        .collect();
    let plane = TopoPlane3D::from_three_points(points[0], points[1], points[2]).unwrap();
    brep.add_face(
        SurfaceRef::Plane(plane),
        true,
        vec![
            edges
                .iter()
                .map(|&edge| EdgeUse {
                    edge,
                    forward: true,
                })
                .collect(),
        ],
    )
    .unwrap();
    let face = face_records().remove(1);
    ProductShellRecord {
        entity_id: EntityId::from_uuid(uuid(20)),
        brep: std::sync::Arc::new(brep),
        placement: SimilarityTransform3D::translation(geo_algorithms::Vector3D::new(
            10.0, 0.0, 0.0,
        )),
        face_ids: vec![FaceId::from_uuid(uuid(21))],
        source: Some(ShellSource {
            file_name: "p.stp".to_string(),
            occurrences: vec![7],
            faces: vec![99],
        }),
        display: face.display,
        attributes: face.attributes,
        metadata: face.metadata,
    }
}

/// 版 1 の区画 `SHEL` のゴールデンのバイト列（`shell_record` を書いたもの。16 進数）
///
/// 書き方・読み方の片方だけを変えて形式が壊れないようにする。区画の中身を変えるときは、区画の版を上げて、この
/// バイト列を前の版として読めることを確かめる。
const SHELL_GOLDEN_V1: &str = concat!(
    "010000008dedb5a0f7c6b03e030000000000000000000000000000000000000000000000000000008dedb5a0f7c6b03e",
    "0000000000000040000000000000000000000000000000008dedb5a0f7c6b03e00000000000000000000000000000040",
    "00000000000000008dedb5a0f7c6b03e0300000001000000000000000000000000000000000000000000000000000000",
    "000000f03f000000000000000000000000000000000000000000000000000000000000004000000000010000008dedb5",
    "a0f7c6b03e01000000000000004000000000000000000000000000000000cc3b7f669ea0e6bfcc3b7f669ea0e63f0000",
    "0000000000000000000000000000cd3b7f669ea0064001000000020000008dedb5a0f7c6b03e01000000000000000000",
    "0000000000004000000000000000000000000000000000000000000000f0bf0000000000000000000000000000000000",
    "0000000000004002000000000000008dedb5a0f7c6b03e01000000010000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000f03f000000000000f03f000000000000000000000000",
    "000000000101000000030000000000000001010000000102000000010000000000000000010000000000000000000000",
    "000000000000001400000000000000000000f03f00000000000000000000000000000000000000000000244000000000",
    "00000000000000000000f03f000000000000000000000000000000000000000000000000000000000000000000000000",
    "0000f03f0000000000000000000000000000000000000000000000000000000000000000000000000000f03f01000000",
    "000000000000000000000000000000150105000000702e73747001000000070000000000000001000000630000000000",
    "00000000803f0000803f0000803f0000803f010000803f0100000030010000000000000000000000f153650000000015",
    "cd5b0700f153650000000015cd5b0700000000",
);

fn shell_golden() -> Vec<u8> {
    (0..SHELL_GOLDEN_V1.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&SHELL_GOLDEN_V1[at..at + 2], 16).unwrap())
        .collect()
}

#[test]
fn shells_are_written_as_the_golden_bytes() {
    assert_eq!(
        codec::write_shells(&[shell_record()]).unwrap(),
        shell_golden()
    );
}

#[test]
fn the_shell_golden_bytes_are_read() {
    let expected = shell_record();
    let [shell] = codec::read_shells(&shell_golden())
        .unwrap()
        .try_into()
        .unwrap();
    assert_eq!(shell.entity_id, expected.entity_id);
    assert_eq!(shell.face_ids, expected.face_ids);
    assert_eq!(shell.source, expected.source);
    assert_eq!(shell.placement.to_matrix(), expected.placement.to_matrix());
    assert_eq!(shell.display, expected.display);
    let (read, written) = (shell.brep.records(), expected.brep.records());
    assert_eq!(read.tolerance, written.tolerance);
    assert_eq!(read.vertices, written.vertices);
    assert_eq!(read.faces[0].loops, written.faces[0].loops);
}

#[test]
fn breps_that_no_shell_refers_to_are_rejected() {
    // 2 つ目の表を足し、どのシェルも指さないようにする（表の数を 2 にして、同じ表をもう 1 つ書く）
    let golden = shell_golden();
    let brep_end = {
        let mut out = ByteWriter::default();
        codec::write_brep(&mut out, &shell_record().brep.records()).unwrap();
        4 + out.into_bytes().len()
    };
    let mut broken = 2_u32.to_le_bytes().to_vec();
    broken.extend_from_slice(&golden[4..brep_end]);
    broken.extend_from_slice(&golden[4..]);
    assert_eq!(codec::read_shells(&broken).unwrap_err(), "breps");
}
