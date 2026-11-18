use super::*;

fn uuid(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

/// ID を決めたツールライブラリ（工具長のあるボールエンドミルと、工具長のないフラットエンドミル）
fn sample_library() -> ToolLibrary {
    let mut flat = Tool::flat_end_mill("flat_endmill_10mm".to_string(), 10.0, 0.0);
    flat.cutting_length = None;
    ToolLibrary {
        id: ToolLibraryId::from_uuid(uuid(1)),
        name: "MC 1 号機".to_string(),
        toolsets: vec![
            ToolSetEntry {
                id: ToolSetId::from_uuid(uuid(2)),
                name: "R3 ボール 突き出し 30".to_string(),
                tool: Tool::ball_end_mill("ball_endmill_6mm".to_string(), 6.0, 30.0),
            },
            ToolSetEntry {
                id: ToolSetId::from_uuid(uuid(3)),
                name: "φ10 フラット".to_string(),
                tool: flat,
            },
        ],
        preserved: Vec::new(),
        ext_attributes: Vec::new(),
    }
}

fn info() -> ToolLibrarySaveInfo {
    ToolLibrarySaveInfo {
        program_version: "0.2.0".to_string(),
        saved_at_ms: 1_700_000_000_123,
    }
}

fn bytes(library: &ToolLibrary) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_tool_library(&mut bytes, library, &info()).unwrap();
    bytes
}

fn read(bytes: &[u8]) -> Result<(ToolLibrary, ToolLibrarySaveInfo), ToolLibraryFileError> {
    read_tool_library(&mut &bytes[..])
}

/// 区画 `kind` を、ツールライブラリのバイト列の最後に足す
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
    ArtifactHeaderV1::new(ArtifactKind::ToolLibrary, payload.len() as u64)
        .write_to(&mut out)
        .unwrap();
    out.extend_from_slice(&payload);
    out
}

#[test]
fn tool_libraries_round_trip_with_the_same_ids_and_contents() {
    let (library, saved) = read(&bytes(&sample_library())).unwrap();
    assert_eq!(library, sample_library());
    assert_eq!(saved, info());
}

/// 版 1 のツールライブラリのファイルのゴールデンのバイト列（`sample_library` と `info` を書いたもの。16 進数）
///
/// 書き方・読み方の片方だけを変えて形式が壊れないようにする。区画の中身を変えるときは、区画の版を上げて、この
/// バイト列を前の版として読めることを確かめる。
const GOLDEN_V1: &str = concat!(
    "5252544c000003000101000000000201000000000000020000004c494e46010001003000000000000000000000000000",
    "000000000000000000010b0000004d43203120e58fb7e6a99f05000000302e322e307b68e5cf8b010000545345540100",
    "0100ae0000000000000002000000000000000000000000000000000000021c000000523320e3839ce383bce383ab20e7",
    "aa81e3818de587bae38197203330031000000062616c6c5f656e646d696c6c5f366d6d00000000000018400000000000",
    "000840010000000000003e400000000000000000000000000000000311000000cf86313020e38395e383a9e38383e383",
    "880111000000666c61745f656e646d696c6c5f31306d6d0000000000002440000000000000000000",
);

fn golden() -> Vec<u8> {
    (0..GOLDEN_V1.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&GOLDEN_V1[at..at + 2], 16).unwrap())
        .collect()
}

#[test]
fn tool_libraries_are_written_as_the_golden_bytes() {
    assert_eq!(bytes(&sample_library()), golden());
}

#[test]
fn the_golden_bytes_are_read() {
    let (library, saved) = read(&golden()).unwrap();
    assert_eq!(library, sample_library());
    assert_eq!(saved, info());
}

#[test]
fn unknown_optional_sections_and_header_attributes_are_written_back_unchanged() {
    let original = with_extra_section(&bytes(&sample_library()), *b"HOLD", 3, 0x0002, b"later");
    let (library, _) = read(&original).unwrap();
    assert_eq!(bytes(&library), original);

    let mut library = sample_library();
    library.ext_attributes = vec![ExtAttribute::new(7, vec![1, 2, 3])];
    let written = bytes(&library);
    let (read_library, _) = read(&written).unwrap();
    assert_eq!(read_library.ext_attributes, library.ext_attributes);
    assert_eq!(bytes(&read_library), written);
}

#[test]
fn files_that_cannot_be_opened_safely_are_rejected_with_the_reason() {
    let valid = bytes(&sample_library());
    let required = with_extra_section(&valid, *b"HOLD", 1, SECTION_REQUIRED, b"");
    assert!(matches!(
        read(&required),
        Err(ToolLibraryFileError::UnknownRequiredSection(kind)) if kind == *b"HOLD"
    ));

    let duplicate = with_extra_section(&valid, LINF, 1, SECTION_REQUIRED, b"");
    assert!(matches!(
        read(&duplicate),
        Err(ToolLibraryFileError::DuplicateSection(kind)) if kind == LINF
    ));

    // 先頭の区画（LINF）の版を上げる（区画の数の 4 バイトと種類の 4 バイトの後ろ）
    let mut newer = valid.clone();
    let at = ArtifactHeaderV1::FIXED_HEADER_LEN + 4 + 4;
    newer[at..at + 2].copy_from_slice(&2_u16.to_le_bytes());
    assert!(matches!(
        read(&newer),
        Err(ToolLibraryFileError::UnsupportedSectionVersion { kind, version: 2 }) if kind == LINF
    ));

    // 先頭の区画（LINF）の種類を、知らない必須でない区画に変えると、LINF がない
    let mut missing = valid.clone();
    let at = ArtifactHeaderV1::FIXED_HEADER_LEN + 4;
    missing[at..at + 4].copy_from_slice(b"XXXX");
    missing[at + 6..at + 8].copy_from_slice(&0_u16.to_le_bytes());
    assert!(matches!(
        read(&missing),
        Err(ToolLibraryFileError::MissingSection(kind)) if kind == LINF
    ));

    // ほかの種類のファイル（モデルファイルのヘッダ）
    let mut other = valid.clone();
    other[0..4].copy_from_slice(b"RRMF");
    assert!(matches!(
        read(&other),
        Err(ToolLibraryFileError::Format(
            BinaryFormatError::ArtifactKindMismatch { .. }
        ))
    ));

    let mut trailing = valid;
    trailing.push(0);
    assert!(matches!(
        read(&trailing),
        Err(ToolLibraryFileError::InvalidSectionTable("trailing bytes"))
    ));
}

/// ライブラリ `library` を書いて読んだときの、区画の中身の誤りの項目の名前
fn invalid_field(library: &ToolLibrary) -> Option<([u8; 4], &'static str)> {
    match read(&bytes(library)) {
        Err(ToolLibraryFileError::InvalidSection { kind, field }) => Some((kind, field)),
        _ => None,
    }
}

#[test]
fn empty_names_duplicate_ids_and_invalid_tools_are_rejected() {
    let mut library = sample_library();
    library.name.clear();
    assert_eq!(invalid_field(&library), Some((LINF, "library.name")));

    let mut library = sample_library();
    library.toolsets[1].name.clear();
    assert_eq!(invalid_field(&library), Some((TSET, "toolset.name")));

    let mut library = sample_library();
    library.toolsets[1].id = library.toolsets[0].id;
    assert_eq!(invalid_field(&library), Some((TSET, "toolset.id")));

    let invalid_tools = [
        // 径が NaN・0・負
        Tool::flat_end_mill("t".to_string(), f64::NAN, 30.0),
        Tool::flat_end_mill("t".to_string(), 0.0, 30.0),
        Tool::flat_end_mill("t".to_string(), -6.0, 30.0),
        // 工具長が 0・無限
        Tool::ball_end_mill("t".to_string(), 6.0, 0.0),
        Tool::ball_end_mill("t".to_string(), 6.0, f64::INFINITY),
        // 種類と角の半径が合わない
        Tool::new("t".to_string(), ToolType::FlatEndMill, 6.0, 1.0, 30.0),
    ];
    for tool in invalid_tools {
        let mut library = sample_library();
        library.toolsets[0].tool = tool;
        assert_eq!(invalid_field(&library), Some((TSET, "toolset.tool")));
    }
}

#[test]
fn corrupted_files_do_not_panic() {
    let bytes = bytes(&sample_library());
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
fn tool_libraries_are_saved_without_leaving_a_partial_file_and_loaded_again() {
    let dir =
        std::env::temp_dir().join(format!("redring-tool-library-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("mc1.rrtl");
    fs::write(&path, b"old").unwrap();

    save_tool_library(&path, &sample_library(), &info()).unwrap();
    // 一時のファイルを残さない
    let names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, [std::ffi::OsString::from("mc1.rrtl")]);
    let (library, saved) = load_tool_library(&path).unwrap();
    assert_eq!(library, sample_library());
    assert_eq!(saved, info());

    // 読めないファイルは、読み込みの誤りになる（ファイルは残る）
    fs::write(&path, b"broken").unwrap();
    assert!(load_tool_library(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"broken");

    let missing_dir = dir.join("missing").join("mc1.rrtl");
    assert!(matches!(
        save_tool_library(&missing_dir, &sample_library(), &info()),
        Err(ToolLibraryFileError::Io(_))
    ));
    assert!(matches!(
        load_tool_library(&missing_dir),
        Err(ToolLibraryFileError::Io(_))
    ));
    let _ = fs::remove_dir_all(&dir);
}

// ---- 共有の書き込みと、使うライブラリの一覧 ----

/// 試験ごとのフォルダ（前の回の残りを消して作る）
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "redring-tool-library-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// `dir` に `sample_library` を保存したファイル
fn saved_library(dir: &Path, name: &str) -> PathBuf {
    let path = dir.join(name);
    save_tool_library(&path, &sample_library(), &info()).unwrap();
    path
}

fn entry(id: u128, name: &str) -> ToolSetEntry {
    ToolSetEntry {
        id: ToolSetId::from_uuid(uuid(id)),
        name: name.to_string(),
        tool: Tool::ball_end_mill("ball_endmill_2mm".to_string(), 2.0, 10.0),
    }
}

fn names(library: &ToolLibrary) -> Vec<&str> {
    library
        .toolsets
        .iter()
        .map(|entry| entry.name.as_str())
        .collect()
}

#[test]
fn edits_are_applied_to_the_library() {
    let mut library = sample_library();
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::AddToolSet(entry(10, "足した")),
    )
    .unwrap();
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::MoveToolSet {
            id: ToolSetId::from_uuid(uuid(10)),
            index: 0,
        },
    )
    .unwrap();
    let mut renamed = entry(3, "替えた");
    renamed.tool = Tool::ball_end_mill("ball_endmill_4mm".to_string(), 4.0, 20.0);
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::UpdateToolSet(renamed.clone()),
    )
    .unwrap();
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::RemoveToolSet(ToolSetId::from_uuid(uuid(2))),
    )
    .unwrap();
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::Rename("MC 2 号機".to_string()),
    )
    .unwrap();
    assert_eq!(library.name, "MC 2 号機");
    assert_eq!(names(&library), ["足した", "替えた"]);
    assert_eq!(library.toolsets[1], renamed);

    // 範囲を超える位置は最後、同じ ID の追加は項目を替える
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::MoveToolSet {
            id: ToolSetId::from_uuid(uuid(10)),
            index: 99,
        },
    )
    .unwrap();
    shared::apply_edit(
        &mut library,
        ToolLibraryEdit::AddToolSet(entry(10, "もう一度")),
    )
    .unwrap();
    assert_eq!(names(&library), ["替えた", "もう一度"]);
}

#[test]
fn edits_of_missing_tool_sets_and_invalid_values_are_rejected() {
    let mut library = sample_library();
    let missing = ToolSetId::from_uuid(uuid(99));
    for edit in [
        ToolLibraryEdit::UpdateToolSet(entry(99, "ない")),
        ToolLibraryEdit::RemoveToolSet(missing),
        ToolLibraryEdit::MoveToolSet {
            id: missing,
            index: 0,
        },
    ] {
        assert!(matches!(
            shared::apply_edit(&mut library, edit),
            Err(ToolLibraryEditError::ToolSetNotFound(id)) if id == missing
        ));
    }
    let mut invalid = entry(10, "妥当でない");
    invalid.tool = Tool::flat_end_mill("t".to_string(), -1.0, 10.0);
    let edits = [
        (ToolLibraryEdit::Rename(String::new()), "library.name"),
        (ToolLibraryEdit::AddToolSet(entry(10, "")), "toolset.name"),
        (ToolLibraryEdit::AddToolSet(invalid), "toolset.tool"),
    ];
    for (edit, field) in edits {
        assert!(matches!(
            shared::apply_edit(&mut library, edit),
            Err(ToolLibraryEditError::InvalidEdit(f)) if f == field
        ));
    }
    assert_eq!(library, sample_library());
}

#[test]
fn concurrent_writers_do_not_lose_each_other_edits() {
    let dir = test_dir("concurrent");
    let path = saved_library(&dir, "mc1.rrtl");
    let writers: Vec<_> = (0..4_u128)
        .map(|writer| {
            let path = path.clone();
            std::thread::spawn(move || {
                for step in 0..5_u128 {
                    let id = 100 + writer * 10 + step;
                    edit_tool_library(
                        &path,
                        ToolLibraryEdit::AddToolSet(entry(id, &format!("{writer}-{step}"))),
                        &format!("pc-{writer}"),
                        &info(),
                    )
                    .unwrap();
                }
            })
        })
        .collect();
    for writer in writers {
        writer.join().unwrap();
    }
    let (library, _) = load_tool_library(&path).unwrap();
    // 最初の 2 本と、4 つの書き手の 5 本ずつ
    assert_eq!(library.toolsets.len(), 2 + 4 * 5);
    // ロックのファイル・一時のファイルを残さない
    let files: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(files, [std::ffi::OsString::from("mc1.rrtl")]);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn unreadable_files_are_not_overwritten() {
    let dir = test_dir("unreadable");
    let path = dir.join("mc1.rrtl");
    fs::write(&path, b"broken").unwrap();
    assert!(matches!(
        edit_tool_library(
            &path,
            ToolLibraryEdit::AddToolSet(entry(10, "足す")),
            "pc-a",
            &info()
        ),
        Err(ToolLibraryEditError::File(_))
    ));
    assert_eq!(fs::read(&path).unwrap(), b"broken");
    assert!(tool_library_lock(&path).is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn read_only_files_are_used_only_for_reading() {
    let dir = test_dir("read-only");
    let path = saved_library(&dir, "mc1.rrtl");
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions.clone()).unwrap();

    assert!(matches!(
        edit_tool_library(
            &path,
            ToolLibraryEdit::AddToolSet(entry(10, "足す")),
            "pc-a",
            &info()
        ),
        Err(ToolLibraryEditError::ReadOnly(_))
    ));
    let mut catalog = ToolLibraryCatalog::open([path.clone()]);
    assert!(matches!(
        catalog.entries()[0].status(),
        ToolLibraryStatus::Available { writable: false }
    ));
    assert_eq!(catalog.entries()[0].library(), Some(&sample_library()));
    assert!(matches!(
        catalog.edit(0, ToolLibraryEdit::Rename("x".to_string()), "pc-a", &info()),
        Err(ToolLibraryEditError::ReadOnly(_))
    ));
    assert_eq!(load_tool_library(&path).unwrap().0, sample_library());

    #[allow(clippy::permissions_set_readonly_false)]
    permissions.set_readonly(false);
    fs::set_permissions(&path, permissions).unwrap();
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn edits_stop_with_the_reason_while_another_writer_holds_the_lock() {
    let dir = test_dir("locked");
    let path = saved_library(&dir, "mc1.rrtl");
    let lock = dir.join("mc1.rrtl.lock");
    fs::write(&lock, "pc-b\n1700000000000\n").unwrap();
    let result = edit_tool_library(
        &path,
        ToolLibraryEdit::AddToolSet(entry(10, "足す")),
        "pc-a",
        &info(),
    );
    let Err(ToolLibraryEditError::Locked(found)) = result else {
        panic!("expected a locked library: {result:?}");
    };
    assert_eq!(
        found,
        ToolLibraryLock {
            owner: "pc-b".to_string(),
            locked_at_ms: Some(1_700_000_000_000),
        }
    );
    // ほかの書き手のロックを消さず、ファイルも変えない
    assert!(lock.exists());
    assert_eq!(load_tool_library(&path).unwrap().0, sample_library());

    // 古いロックは、利用者に確かめてから消すと書ける
    assert!(found.is_stale(1_700_000_000_000 + 5 * 60 * 1000));
    assert!(!found.is_stale(1_700_000_000_000 + 1000));
    remove_tool_library_lock(&path).unwrap();
    edit_tool_library(
        &path,
        ToolLibraryEdit::AddToolSet(entry(10, "足す")),
        "pc-a",
        &info(),
    )
    .unwrap();
    assert!(tool_library_lock(&path).is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn locks_whose_time_cannot_be_read_are_stale() {
    let dir = test_dir("broken-lock");
    let path = saved_library(&dir, "mc1.rrtl");
    fs::write(dir.join("mc1.rrtl.lock"), "").unwrap();
    let lock = tool_library_lock(&path).unwrap();
    assert_eq!(lock.locked_at_ms, None);
    assert!(lock.is_stale(0));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn the_catalog_rereads_changed_files_and_keeps_the_last_contents_of_unreadable_ones() {
    let dir = test_dir("catalog");
    let path = saved_library(&dir, "mc1.rrtl");
    let mut catalog = ToolLibraryCatalog::open([path.clone()]);
    assert!(matches!(
        catalog.entries()[0].status(),
        ToolLibraryStatus::Available { writable: true }
    ));

    // 一覧から書くと、書いた内容を映す
    catalog
        .edit(
            0,
            ToolLibraryEdit::AddToolSet(entry(10, "足した")),
            "pc-a",
            &info(),
        )
        .unwrap();
    assert_eq!(catalog.entries()[0].library().unwrap().toolsets.len(), 3);

    // ほかの PC が書いた内容を、読み直して映す
    edit_tool_library(
        &path,
        ToolLibraryEdit::RemoveToolSet(ToolSetId::from_uuid(uuid(10))),
        "pc-b",
        &info(),
    )
    .unwrap();
    catalog.refresh();
    assert_eq!(catalog.entries()[0].library(), Some(&sample_library()));

    // 読めなくなっても、前に読めた内容を読むだけで使う
    fs::write(&path, b"broken").unwrap();
    catalog.refresh();
    assert!(matches!(
        catalog.entries()[0].status(),
        ToolLibraryStatus::Unreadable(_)
    ));
    assert_eq!(catalog.entries()[0].library(), Some(&sample_library()));
    assert!(matches!(
        catalog.edit(0, ToolLibraryEdit::Rename("x".to_string()), "pc-a", &info()),
        Err(ToolLibraryEditError::Unavailable)
    ));
    assert_eq!(fs::read(&path).unwrap(), b"broken");

    // ないファイルの一覧
    let missing = ToolLibraryCatalog::open([dir.join("missing.rrtl")]);
    assert!(missing.entries()[0].library().is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn files_with_the_same_library_id_as_an_earlier_one_are_not_used() {
    let dir = test_dir("duplicate");
    let first = saved_library(&dir, "mc1.rrtl");
    let copy = dir.join("mc1 copy.rrtl");
    fs::copy(&first, &copy).unwrap();
    let mut catalog = ToolLibraryCatalog::open([first.clone(), copy]);
    assert!(catalog.entries()[0].library().is_some());
    assert!(matches!(
        catalog.entries()[1].status(),
        ToolLibraryStatus::DuplicateId { first: path } if *path == first
    ));
    assert!(catalog.entries()[1].library().is_none());
    assert!(matches!(
        catalog.edit(1, ToolLibraryEdit::Rename("x".to_string()), "pc-a", &info()),
        Err(ToolLibraryEditError::Unavailable)
    ));
    assert!(matches!(
        catalog.edit(9, ToolLibraryEdit::Rename("x".to_string()), "pc-a", &info()),
        Err(ToolLibraryEditError::Unavailable)
    ));
    let _ = fs::remove_dir_all(&dir);
}
