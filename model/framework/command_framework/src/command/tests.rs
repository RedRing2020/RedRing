use super::{
    Access, AccessKind, Admission, AllowedInterrupt, CommandDeclaration, CommandId,
    CommandScheduler, Execution, Started, Submission, TargetSet, WhenBlocked,
};

/// 試験で使う対象の区分
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    View,
    SpaceModel,
    ModelFile,
    ProductShape,
    MachiningSequence,
}

type Declaration = CommandDeclaration<Target>;
type Scheduler = CommandScheduler<Target, &'static str>;

fn access(kind: AccessKind, target: Target) -> Access<Target> {
    Access { kind, target }
}

fn any(kind: AccessKind) -> AllowedInterrupt<Target> {
    AllowedInterrupt {
        kind,
        targets: TargetSet::Any,
    }
}

fn any_except(kind: AccessKind, target: Target) -> AllowedInterrupt<Target> {
    AllowedInterrupt {
        kind,
        targets: TargetSet::AnyExcept(vec![target]),
    }
}

fn synchronous(touches: Access<Target>, when_blocked: WhenBlocked) -> Declaration {
    CommandDeclaration {
        execution: Execution::Synchronous,
        touches: vec![touches],
        allowed_interrupts: Vec::new(),
        when_blocked,
        cancellable: false,
    }
}

/// 表示の操作（回転など）
fn rotate() -> Declaration {
    synchronous(
        access(AccessKind::ViewOnly, Target::View),
        WhenBlocked::Reject,
    )
}

/// 読むだけの操作（工程を選ぶなど）
fn select() -> Declaration {
    synchronous(
        access(AccessKind::ReadOnly, Target::MachiningSequence),
        WhenBlocked::Reject,
    )
}

/// 条件の適用
fn apply_conditions() -> Declaration {
    synchronous(
        access(AccessKind::Modify, Target::MachiningSequence),
        WhenBlocked::Reject,
    )
}

/// 終了
fn quit() -> Declaration {
    synchronous(
        access(AccessKind::Replace, Target::SpaceModel),
        WhenBlocked::CancelThenRun,
    )
}

/// 保存
fn save() -> Declaration {
    CommandDeclaration {
        execution: Execution::Asynchronous,
        touches: vec![
            access(AccessKind::ReadOnly, Target::SpaceModel),
            access(AccessKind::Modify, Target::ModelFile),
        ],
        allowed_interrupts: vec![
            any(AccessKind::ViewOnly),
            any(AccessKind::ReadOnly),
            any_except(AccessKind::Modify, Target::ModelFile),
        ],
        when_blocked: WhenBlocked::Wait,
        cancellable: false,
    }
}

/// 開く
fn open() -> Declaration {
    CommandDeclaration {
        execution: Execution::Asynchronous,
        touches: vec![access(AccessKind::Replace, Target::SpaceModel)],
        allowed_interrupts: vec![any(AccessKind::ViewOnly), any(AccessKind::ReadOnly)],
        when_blocked: WhenBlocked::Wait,
        cancellable: true,
    }
}

/// 形状の読み込み
fn import_shape() -> Declaration {
    CommandDeclaration {
        execution: Execution::Asynchronous,
        touches: vec![access(AccessKind::Modify, Target::ProductShape)],
        allowed_interrupts: vec![
            any(AccessKind::ViewOnly),
            any(AccessKind::ReadOnly),
            any_except(AccessKind::Modify, Target::ProductShape),
        ],
        when_blocked: WhenBlocked::Wait,
        cancellable: true,
    }
}

/// コマンドを出し、実行を始めたことを確かめて、識別子を返す
fn start(scheduler: &mut Scheduler, declaration: Declaration, command: &'static str) -> CommandId {
    match scheduler.submit(declaration, command) {
        Submission::Started(started) => {
            assert_eq!(started.command, command);
            started.id
        }
        other => panic!("{command} did not start: {other:?}"),
    }
}

/// 実行中のコマンド `running` の最中に、それぞれのコマンドを出したときの扱い
fn admissions_during(running: Option<Declaration>) -> [(&'static str, Admission); 7] {
    let mut scheduler = Scheduler::default();
    if let Some(running) = running {
        start(&mut scheduler, running, "running");
    }
    [
        ("rotate", scheduler.admission(&rotate())),
        ("select", scheduler.admission(&select())),
        ("apply", scheduler.admission(&apply_conditions())),
        ("save", scheduler.admission(&save())),
        ("open", scheduler.admission(&open())),
        ("import", scheduler.admission(&import_shape())),
        ("quit", scheduler.admission(&quit())),
    ]
}

#[test]
fn every_command_runs_when_nothing_is_running() {
    assert_eq!(
        admissions_during(None),
        [
            ("rotate", Admission::Run),
            ("select", Admission::Run),
            ("apply", Admission::Run),
            ("save", Admission::Run),
            ("open", Admission::Run),
            ("import", Admission::Run),
            ("quit", Admission::Run),
        ]
    );
}

#[test]
fn saving_allows_changes_except_to_the_model_file() {
    assert_eq!(
        admissions_during(Some(save())),
        [
            ("rotate", Admission::Run),
            ("select", Admission::Run),
            ("apply", Admission::Run),
            ("save", Admission::Wait),
            ("open", Admission::Wait),
            ("import", Admission::Run),
            ("quit", Admission::CancelThenRun),
        ]
    );
}

#[test]
fn opening_allows_only_viewing_and_reading() {
    assert_eq!(
        admissions_during(Some(open())),
        [
            ("rotate", Admission::Run),
            ("select", Admission::Run),
            ("apply", Admission::Reject),
            ("save", Admission::Wait),
            ("open", Admission::Wait),
            ("import", Admission::Wait),
            ("quit", Admission::CancelThenRun),
        ]
    );
}

#[test]
fn importing_a_shape_allows_changes_except_to_the_product_shapes() {
    assert_eq!(
        admissions_during(Some(import_shape())),
        [
            ("rotate", Admission::Run),
            ("select", Admission::Run),
            ("apply", Admission::Run),
            ("save", Admission::Run),
            ("open", Admission::Wait),
            ("import", Admission::Wait),
            ("quit", Admission::CancelThenRun),
        ]
    );
}

#[test]
fn a_synchronous_command_leaves_the_stack_when_it_finishes() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");

    let apply_id = start(&mut scheduler, apply_conditions(), "apply");
    // 同期のコマンドは、実行中の間は何も許さない
    assert_eq!(scheduler.admission(&rotate()), Admission::Reject);
    scheduler.finish(apply_id);

    assert_eq!(scheduler.admission(&rotate()), Admission::Run);
    assert_eq!(scheduler.admission(&save()), Admission::Wait);
    scheduler.finish(save_id);
    assert_eq!(scheduler.admission(&save()), Admission::Run);
}

#[test]
fn a_rejected_command_is_returned_and_not_queued() {
    let mut scheduler = Scheduler::default();
    let open_id = start(&mut scheduler, open(), "open");

    assert_eq!(
        scheduler.submit(apply_conditions(), "apply"),
        Submission::Rejected("apply")
    );

    scheduler.finish(open_id);
    assert_eq!(scheduler.take_ready(), None);
}

#[test]
fn an_incoming_command_must_be_allowed_by_every_running_command() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");
    let import_id = start(&mut scheduler, import_shape(), "import");

    // 保存も形状の読み込みも許す
    assert_eq!(scheduler.admission(&apply_conditions()), Admission::Run);
    // 保存は許すが、形状の読み込みが許さない
    assert_eq!(scheduler.admission(&import_shape()), Admission::Wait);
    // 形状の読み込みは許すが、保存が許さない
    assert_eq!(scheduler.admission(&save()), Admission::Wait);

    // 先に始めたものが先に終わってもよい
    scheduler.finish(save_id);
    assert_eq!(scheduler.admission(&save()), Admission::Run);
    assert_eq!(scheduler.admission(&import_shape()), Admission::Wait);
    scheduler.finish(import_id);
    assert_eq!(scheduler.admission(&import_shape()), Admission::Run);
}

#[test]
fn an_asynchronous_incoming_command_must_also_allow_the_running_commands() {
    // 製品形状を読むだけだが、自分の最中には表示と読むことだけを許す、非同期のコマンド
    let read_shapes = |execution| CommandDeclaration {
        execution,
        touches: vec![access(AccessKind::ReadOnly, Target::ProductShape)],
        allowed_interrupts: vec![any(AccessKind::ViewOnly), any(AccessKind::ReadOnly)],
        when_blocked: WhenBlocked::Wait,
        cancellable: true,
    };
    let mut scheduler = Scheduler::default();
    start(&mut scheduler, import_shape(), "import");

    // 形状の読み込みは読むことを許すが、来たコマンドが、形状の読み込み（製品形状を変える）を許さない
    assert_eq!(
        scheduler.admission(&read_shapes(Execution::Asynchronous)),
        Admission::Wait
    );
    // 同期のコマンドは、その場で終わるため、実行中のコマンドが許せば実行する
    assert_eq!(
        scheduler.admission(&read_shapes(Execution::Synchronous)),
        Admission::Run
    );
}

#[test]
fn a_waiting_command_starts_when_the_running_command_finishes() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");

    assert_eq!(scheduler.submit(open(), "open"), Submission::Queued);
    assert_eq!(scheduler.take_ready(), None);

    // 待っているコマンドがあっても、実行できるコマンドは実行する
    let apply_id = start(&mut scheduler, apply_conditions(), "apply");
    scheduler.finish(apply_id);
    assert_eq!(scheduler.take_ready(), None);

    scheduler.finish(save_id);
    let started = scheduler.take_ready().expect("open starts");
    assert_eq!(started.command, "open");
    assert_eq!(scheduler.take_ready(), None);

    // 開くは、実行中のコマンドとして残る
    assert_eq!(scheduler.admission(&apply_conditions()), Admission::Reject);
    scheduler.finish(started.id);
    assert_eq!(scheduler.admission(&apply_conditions()), Admission::Run);
}

#[test]
fn waiting_commands_start_one_at_a_time_in_the_order_they_were_submitted() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");
    assert_eq!(scheduler.submit(save(), "second save"), Submission::Queued);
    assert_eq!(scheduler.submit(open(), "open"), Submission::Queued);
    assert_eq!(scheduler.submit(save(), "third save"), Submission::Queued);

    scheduler.finish(save_id);
    let second = scheduler.take_ready().expect("the second save starts");
    assert_eq!(second.command, "second save");
    // 始めた保存が、残りのコマンドを待たせる
    assert_eq!(scheduler.take_ready(), None);

    scheduler.finish(second.id);
    let opened = scheduler.take_ready().expect("open starts");
    assert_eq!(opened.command, "open");
    assert_eq!(scheduler.take_ready(), None);

    scheduler.finish(opened.id);
    let third = scheduler.take_ready().expect("the third save starts");
    assert_eq!(third.command, "third save");
}

#[test]
fn a_waiting_command_that_can_run_starts_before_earlier_ones_that_cannot() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");
    let import_id = start(&mut scheduler, import_shape(), "import");
    assert_eq!(scheduler.submit(open(), "open"), Submission::Queued);
    assert_eq!(scheduler.submit(save(), "second save"), Submission::Queued);

    // 開くは形状の読み込みを待ち続け、後から出した保存が先に始まる
    scheduler.finish(save_id);
    let second = scheduler.take_ready().expect("the second save starts");
    assert_eq!(second.command, "second save");
    assert_eq!(scheduler.take_ready(), None);

    scheduler.finish(import_id);
    assert_eq!(scheduler.take_ready(), None);
    scheduler.finish(second.id);
    assert_eq!(
        scheduler.take_ready().map(|started| started.command),
        Some("open")
    );
}

#[test]
fn cancel_then_run_cancels_the_cancellable_commands_and_waits_for_the_others() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");
    let import_id = start(&mut scheduler, import_shape(), "import");

    // 取り消してから実行するコマンドが待っていなければ、何も取り消さない
    assert_eq!(scheduler.take_cancel_requests(), Vec::new());

    assert_eq!(scheduler.admission(&quit()), Admission::CancelThenRun);
    assert_eq!(scheduler.submit(quit(), "quit"), Submission::Queued);

    // 取り消せる形状の読み込みだけを、1 回だけ返す（保存は取り消せない）
    assert_eq!(scheduler.take_cancel_requests(), vec![import_id]);
    assert_eq!(scheduler.take_cancel_requests(), Vec::new());

    // 取り消した形状の読み込みが止まっても、保存が終わるまで待つ
    scheduler.finish(import_id);
    assert_eq!(scheduler.take_ready(), None);
    scheduler.finish(save_id);
    assert_eq!(
        scheduler.take_ready().map(|started| started.command),
        Some("quit")
    );
}

#[test]
fn cancel_then_run_also_cancels_commands_that_start_while_it_waits() {
    let mut scheduler = Scheduler::default();
    let save_id = start(&mut scheduler, save(), "save");
    assert_eq!(scheduler.submit(quit(), "quit"), Submission::Queued);
    assert_eq!(scheduler.take_cancel_requests(), Vec::new());

    let import_id = start(&mut scheduler, import_shape(), "import");
    assert_eq!(scheduler.take_cancel_requests(), vec![import_id]);

    scheduler.finish(import_id);
    scheduler.finish(save_id);
    assert_eq!(
        scheduler.take_ready().map(|started| started.command),
        Some("quit")
    );
}

#[test]
fn a_waiting_command_that_only_waits_does_not_cancel_anything() {
    let mut scheduler = Scheduler::default();
    start(&mut scheduler, import_shape(), "import");

    assert_eq!(scheduler.submit(open(), "open"), Submission::Queued);

    assert_eq!(scheduler.take_cancel_requests(), Vec::new());
}

#[test]
fn a_child_command_is_not_blocked_by_its_parent() {
    let mut scheduler = Scheduler::default();
    let open_id = start(&mut scheduler, open(), "open");

    // 開くは、モデルファイルを変えるコマンドを許さない
    assert_eq!(scheduler.admission(&save()), Admission::Wait);

    // 開くの中の「保存して開く」の保存は、子として実行する
    let Submission::Started(Started { id: save_id, .. }) =
        scheduler.submit_child(open_id, save(), "save before open")
    else {
        panic!("the child save did not start");
    };

    // 外から来たコマンドは、開くと保存のどちらにも照らす
    assert_eq!(scheduler.admission(&rotate()), Admission::Run);
    assert_eq!(scheduler.admission(&apply_conditions()), Admission::Reject);

    // 子が終わっても、親は実行中のまま残る
    scheduler.finish(save_id);
    assert_eq!(scheduler.admission(&apply_conditions()), Admission::Reject);
    scheduler.finish(open_id);
    assert_eq!(scheduler.admission(&apply_conditions()), Admission::Run);
}

#[test]
fn a_child_command_is_not_blocked_by_commands_below_its_parent() {
    let mut scheduler = Scheduler::default();
    start(&mut scheduler, save(), "save");
    let import_id = start(&mut scheduler, import_shape(), "import");

    // 外から来た保存は、先に始めた保存に待たされる
    assert_eq!(scheduler.admission(&save()), Admission::Wait);

    assert!(matches!(
        scheduler.submit_child(import_id, save(), "child save"),
        Submission::Started(_)
    ));
}

#[test]
fn a_child_command_is_checked_against_commands_above_its_parent() {
    let mut scheduler = Scheduler::default();
    let import_id = start(&mut scheduler, import_shape(), "import");
    start(&mut scheduler, save(), "save");

    // 親より後に始めた保存が、モデルファイルを変える子を許さない（子は待たせずに返す）
    assert_eq!(
        scheduler.submit_child(import_id, save(), "child save"),
        Submission::Rejected("child save")
    );
    assert_eq!(scheduler.take_ready(), None);

    // 親より後に始めた保存が許す子は、実行する
    assert!(matches!(
        scheduler.submit_child(import_id, apply_conditions(), "child apply"),
        Submission::Started(_)
    ));
}

#[test]
fn a_child_of_a_command_that_is_not_running_is_rejected() {
    let mut scheduler = Scheduler::default();
    let open_id = start(&mut scheduler, open(), "open");
    scheduler.finish(open_id);

    assert_eq!(
        scheduler.submit_child(open_id, save(), "save before open"),
        Submission::Rejected("save before open")
    );
}
