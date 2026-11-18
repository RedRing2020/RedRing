use super::*;
use crate::failure_message_catalog::resolve_failure_message;
use i18n_foundation::UiLocale;

/// 試験ごとのフォルダ（前の回の残りを消して作る）
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "redring-tool-library-session-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn session(paths: Vec<PathBuf>) -> ToolLibrarySession {
    ToolLibrarySession::open(paths, "pc-a / user".to_string(), "0.2.0".to_string())
}

fn draft(name: &str, diameter: f64) -> ToolSetDraft {
    ToolSetDraft {
        name: name.to_string(),
        kind: CuttingToolKind::BallEndMill,
        diameter,
        cutting_length: Some(30.0),
    }
}

fn names(session: &ToolLibrarySession, index: usize) -> Vec<String> {
    session
        .toolsets(index)
        .into_iter()
        .map(|row| row.name)
        .collect()
}

#[test]
fn libraries_are_created_and_their_tool_sets_are_edited() {
    let dir = test_dir("edit");
    let path = dir.join("mc1.rrtl");
    let mut session = session(Vec::new());
    let index = session.create_library(path.clone(), " MC 1 号機 ").unwrap();
    assert_eq!(index, 0);
    assert_eq!(session.paths(), std::slice::from_ref(&path));
    let rows = session.libraries();
    assert_eq!(rows[0].name.as_deref(), Some("MC 1 号機"));
    assert!(rows[0].editable());

    let r3 = session.add_toolset(0, &draft("R3 ボール", 6.0)).unwrap();
    let r1 = session.add_toolset(0, &draft("R1 ボール", 2.0)).unwrap();
    session.move_toolset(0, r1, 0).unwrap();
    assert_eq!(names(&session, 0), ["R1 ボール", "R3 ボール"]);

    let mut changed = ToolSetDraft::of(&session.toolsets(0)[1]);
    changed.name = "φ8 フラット".to_string();
    changed.kind = CuttingToolKind::FlatEndMill;
    changed.diameter = 8.0;
    changed.cutting_length = None;
    session.update_toolset(0, r3, &changed).unwrap();
    let row = &session.toolsets(0)[1];
    assert_eq!(row.kind, Some(CuttingToolKind::FlatEndMill));
    assert_eq!(row.diameter, 8.0);
    assert_eq!(row.corner_radius, 0.0);
    assert_eq!(row.cutting_length, None);
    assert_eq!(ToolSetDraft::of(row), changed);

    session.remove_toolset(0, r1).unwrap();
    session.rename_library(0, "MC 2 号機").unwrap();

    // 書いた内容は、開き直しても残る
    let reopened = self::session(vec![path]);
    assert_eq!(reopened.libraries()[0].name.as_deref(), Some("MC 2 号機"));
    assert_eq!(names(&reopened, 0), ["φ8 フラット"]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn libraries_are_added_to_and_removed_from_the_list_without_deleting_files() {
    let dir = test_dir("list");
    let path = dir.join("mc1.rrtl");
    let mut first = session(Vec::new());
    first.create_library(path.clone(), "MC 1 号機").unwrap();

    let mut session = session(Vec::new());
    assert_eq!(session.add_library(path.clone()), 0);
    // 同じ場所は 1 回だけ
    assert_eq!(session.add_library(path.clone()), 0);
    // 写したファイルは、同じライブラリとして使わない
    let copy = dir.join("mc1 copy.rrtl");
    std::fs::copy(&path, &copy).unwrap();
    assert_eq!(session.add_library(copy), 1);
    let rows = session.libraries();
    assert_eq!(
        rows[1].state,
        ToolLibraryState::Duplicate {
            first: path.clone()
        }
    );
    assert!(!rows[1].editable());
    assert!(session.toolsets(1).is_empty());

    session.remove_library(0);
    assert_eq!(session.libraries().len(), 1);
    assert!(path.exists());
    // 前のファイルを外すと、写したファイルを使う
    assert!(session.libraries()[0].editable());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn failures_are_returned_with_their_reasons() {
    let dir = test_dir("failures");
    let path = dir.join("mc1.rrtl");
    let mut session = session(Vec::new());
    session.create_library(path.clone(), "MC 1 号機").unwrap();

    let reason = |message: &UiMessage| message.key.clone();
    // 同じ場所に作り直さない・空の名前
    assert_eq!(
        reason(&session.create_library(path.clone(), "別").unwrap_err()),
        "failure.file.already_exists"
    );
    assert_eq!(
        reason(&session.create_library(dir.join("x.rrtl"), " ").unwrap_err()),
        "failure.tool_library.empty_name"
    );
    let failed = |result: Result<(), ToolLibraryFailure>| match result {
        Err(ToolLibraryFailure::Failed(message)) => message.key,
        other => panic!("expected a failure: {other:?}"),
    };
    assert_eq!(
        failed(session.rename_library(0, "")),
        "failure.tool_library.empty_name"
    );
    assert_eq!(
        failed(session.remove_toolset(0, ToolSetId::new_random())),
        "failure.tool_library.toolset_not_found"
    );
    assert_eq!(
        failed(session.rename_library(5, "x")),
        "failure.tool_library.unavailable"
    );

    // 読めないファイル
    std::fs::write(&path, b"broken").unwrap();
    session.refresh();
    let ToolLibraryState::Unreadable(message) = &session.libraries()[0].state else {
        panic!("expected an unreadable library");
    };
    assert_eq!(message.key, "failure.model_file.invalid");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn locked_libraries_report_the_writer_and_stale_locks_can_be_removed() {
    let dir = test_dir("locked");
    let path = dir.join("mc1.rrtl");
    let mut session = session(Vec::new());
    session.create_library(path.clone(), "MC 1 号機").unwrap();
    let lock = dir.join("mc1.rrtl.lock");

    // 新しいロックは消さない
    std::fs::write(&lock, format!("pc-b / user\n{}\n", now_ms())).unwrap();
    assert_eq!(
        session.add_toolset(0, &draft("R3 ボール", 6.0)),
        Err(ToolLibraryFailure::Locked {
            owner: "pc-b / user".to_string(),
            stale: false,
        })
    );
    session.remove_stale_lock(0).unwrap();
    assert!(lock.exists());

    // 古いロックは、確かめてから消すと書ける
    std::fs::write(&lock, "pc-b / user\n1700000000000\n").unwrap();
    assert!(matches!(
        session.add_toolset(0, &draft("R3 ボール", 6.0)),
        Err(ToolLibraryFailure::Locked { stale: true, .. })
    ));
    session.remove_stale_lock(0).unwrap();
    assert!(!lock.exists());
    session.add_toolset(0, &draft("R3 ボール", 6.0)).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn draft_issues_are_found() {
    assert!(draft("R3 ボール", 6.0).issues().is_empty());
    let mut wrong = draft(" ", 0.0);
    wrong.cutting_length = Some(-1.0);
    assert_eq!(
        wrong.issues(),
        [
            ToolSetDraftIssue::EmptyName,
            ToolSetDraftIssue::DiameterNotPositive,
            ToolSetDraftIssue::CuttingLengthNotPositive,
        ]
    );
    wrong = draft("x", f64::NAN);
    wrong.cutting_length = None;
    assert_eq!(wrong.issues(), [ToolSetDraftIssue::DiameterNotPositive]);
}

#[test]
fn every_tool_library_failure_has_a_text_in_both_languages() {
    let keys = [
        "failure.file.already_exists",
        "failure.tool_library.read_only",
        "failure.tool_library.unavailable",
        "failure.tool_library.toolset_not_found",
        "failure.tool_library.empty_name",
        "failure.tool_library.empty_toolset_name",
        "failure.tool_library.invalid_tool",
    ];
    let messages = keys
        .iter()
        .map(|key| UiMessage {
            key: key.to_string(),
            args: Vec::new(),
        })
        .chain([locked_message("pc-b / user")]);
    for message in messages {
        for locale in [UiLocale::Ja, UiLocale::En] {
            let text = resolve_failure_message(locale, &message);
            assert!(
                !text.contains('{')
                    && !text.contains("unexpected reason")
                    && !text.contains("思わぬ"),
                "{message:?}: {text}"
            );
        }
    }
    assert!(resolve_failure_message(UiLocale::Ja, &locked_message("pc-b")).starts_with("pc-b "));
}

#[test]
fn file_labels_are_the_file_names() {
    assert_eq!(file_label(Path::new("C:/tools/mc1.rrtl")), "mc1.rrtl");
}

#[test]
fn tool_sets_with_the_same_name_are_told_apart_by_their_ids() {
    let dir = test_dir("same-name");
    let path = dir.join("mc1.rrtl");
    let mut session = session(Vec::new());
    session.create_library(path.clone(), "MC 1 号機").unwrap();
    let first = session.add_toolset(0, &draft("R3 ボール", 6.0)).unwrap();
    let second = session.add_toolset(0, &draft("R3 ボール", 6.0)).unwrap();
    assert_ne!(first, second);

    // 後の 1 本だけを替え・動かし・消しても、同じ名前の前の 1 本は変わらない
    session
        .update_toolset(0, second, &draft("R3 ボール", 8.0))
        .unwrap();
    session.move_toolset(0, second, 0).unwrap();
    let rows = session.toolsets(0);
    assert_eq!((rows[0].id, rows[0].diameter), (second, 8.0));
    assert_eq!((rows[1].id, rows[1].diameter), (first, 6.0));
    session.remove_toolset(0, second).unwrap();
    let rows = self::session(vec![path]).toolsets(0);
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].id, rows[0].diameter), (first, 6.0));
    let _ = std::fs::remove_dir_all(&dir);
}

// ---- 加工工程の工具とライブラリ ----

use crate::space_model_document::{DefaultNames, SpaceModelDocument};

fn document() -> SpaceModelDocument {
    SpaceModelDocument::new(DefaultNames {
        sequence: "手順",
        operation: "スキャン加工",
    })
}

fn status_of(
    session: &ToolLibrarySession,
    content: &MachiningOperationContent,
) -> OperationToolStatus {
    session.operation_tool_status(
        content.attributes.toolset.as_ref(),
        &content.conditions.tool,
    )
}

#[test]
fn operations_select_tool_sets_and_are_aligned_after_the_library_changes() {
    let dir = test_dir("operation-tools");
    let mut session = session(Vec::new());
    session
        .create_library(dir.join("mc1.rrtl"), "MC 1 号機")
        .unwrap();
    let r3 = session.add_toolset(0, &draft("R3 ボール", 6.0)).unwrap();

    let mut document = document();
    let content = document.operation_content().clone();
    assert_eq!(
        status_of(&session, &content),
        OperationToolStatus::NotFromLibrary
    );
    let selected = session.content_with_toolset(&content, 0, r3).unwrap();
    assert_eq!(
        selected.attributes.toolset.as_ref().unwrap().name,
        "R3 ボール"
    );
    assert_eq!(
        session.library_index_of(selected.attributes.toolset.as_ref().unwrap()),
        Some(0)
    );
    assert!(
        session
            .content_with_toolset(&content, 0, ToolSetId::new_random())
            .is_none()
    );
    document.set_operation_content(selected);
    assert_eq!(
        status_of(&session, document.operation_content()),
        OperationToolStatus::SameAsLibrary
    );

    // 新しい工程は、選んでいた工程の工具を引き継ぐ
    document.add_operation("仕上げ".to_string());
    assert_eq!(
        document
            .operation_content()
            .attributes
            .toolset
            .as_ref()
            .unwrap()
            .toolset,
        r3
    );

    // ライブラリを直すと「ライブラリと違う」になり、手順のすべての工程をそろえる
    session
        .update_toolset(0, r3, &draft("R4 ボール", 8.0))
        .unwrap();
    let rows = document.operations();
    for row in &rows {
        assert_eq!(
            session.operation_tool_status(row.toolset.as_ref(), &row.tool),
            OperationToolStatus::DiffersFromLibrary
        );
    }
    assert_eq!(document.align_sequence_tools(&session), 2);
    for row in document.operations() {
        assert_eq!(row.tool.diameter(), 8.0);
        assert_eq!(row.toolset.unwrap().name, "R4 ボール");
    }
    // もう一度そろえても、何も変わらない
    assert_eq!(document.align_sequence_tools(&session), 0);

    // 1 つの工程の中身だけをそろえる
    let mut changed = document.operation_content().clone();
    changed.conditions.tool = CuttingToolKind::BallEndMill.tool(6.0, Some(30.0));
    let aligned = session.aligned_content(&changed).unwrap();
    assert_eq!(aligned.conditions.tool.diameter(), 8.0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn operation_tools_are_registered_in_a_library() {
    let dir = test_dir("operation-register");
    let mut session = session(Vec::new());
    session
        .create_library(dir.join("mc1.rrtl"), "MC 1 号機")
        .unwrap();
    let content = document().operation_content().clone();
    let registered = session
        .register_operation_tool(&content, 0, " ボール φ10 ")
        .unwrap();
    assert_eq!(
        status_of(&session, &registered),
        OperationToolStatus::SameAsLibrary
    );
    assert_eq!(registered.conditions, content.conditions);
    assert_eq!(names(&session, 0), ["ボール φ10"]);

    // すでにライブラリにある工具は、登録しない
    assert_eq!(
        session.register_operation_tool(&registered, 0, "もう一度"),
        Err(ToolLibraryFailure::Failed(UiMessage {
            key: "failure.tool_library.already_registered".to_string(),
            args: Vec::new(),
        }))
    );
    let _ = std::fs::remove_dir_all(&dir);
}
