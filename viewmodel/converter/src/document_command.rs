//! 空間モデルの文書に対するコマンドの宣言
//!
//! コマンドの種類ごとに、触るもの・自分の最中に許す割り込み・許されないときの扱いを宣言する。実行中のコマンドと
//! 待たせたコマンドは、[`DocumentCommandScheduler`] が持ち、来たコマンドをすぐ実行するか・待たせるか・受け付けないかを、
//! 宣言から決める。

use application::command::{
    Access, AccessKind, AllowedInterrupt, CommandDeclaration, CommandScheduler, Execution,
    TargetSet, WhenBlocked,
};
/// 実行中のコマンドの識別子と、コマンドを出した結果
pub use application::command::{CommandId, Started, Submission};

/// コマンドが触るものの対象
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentTarget {
    /// 製品形状
    ProductShapes,
    /// 加工手順（加工工程・計算の結果を含む）
    MachiningSequences,
    /// モデルファイル
    ModelFile,
}

/// 空間モデルの文書に対するコマンドの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentCommandKind {
    /// 空間モデルを入れ替える（新規・開く）
    ReplaceModel,
    /// モデルファイルに保存する
    SaveModel,
    /// 形状ファイルを、その場で読み込んで製品形状に加える
    ImportShape,
    /// 形状ファイルを、別のスレッドで読み込み、終わったら製品形状に加える
    ImportShapeInBackground,
    /// 空間モデルに触らない（ファイルを選ぶ画面を開くなど）
    NoAccess,
    /// アプリを終える
    Exit,
}

impl DocumentCommandKind {
    /// この種類のコマンドの宣言
    pub fn declaration(self) -> CommandDeclaration<DocumentTarget> {
        let access = |kind, target| Access { kind, target };
        let replace_model = || {
            vec![
                access(AccessKind::Replace, DocumentTarget::ProductShapes),
                access(AccessKind::Replace, DocumentTarget::MachiningSequences),
            ]
        };
        let synchronous = |touches, when_blocked| CommandDeclaration {
            execution: Execution::Synchronous,
            touches,
            allowed_interrupts: Vec::new(),
            when_blocked,
            cancellable: false,
        };
        let import_shape = vec![access(AccessKind::Modify, DocumentTarget::ProductShapes)];
        match self {
            Self::ReplaceModel => synchronous(replace_model(), WhenBlocked::Wait),
            Self::SaveModel => synchronous(
                vec![
                    access(AccessKind::ReadOnly, DocumentTarget::ProductShapes),
                    access(AccessKind::ReadOnly, DocumentTarget::MachiningSequences),
                    access(AccessKind::Modify, DocumentTarget::ModelFile),
                ],
                WhenBlocked::Wait,
            ),
            Self::ImportShape => synchronous(import_shape, WhenBlocked::Wait),
            Self::ImportShapeInBackground => CommandDeclaration {
                execution: Execution::Asynchronous,
                touches: import_shape,
                // 読み終えるまで製品形状に触らないので、製品形状を変える・空間モデルを入れ替えるコマンドのほかは許す
                allowed_interrupts: vec![
                    AllowedInterrupt {
                        kind: AccessKind::ViewOnly,
                        targets: TargetSet::Any,
                    },
                    AllowedInterrupt {
                        kind: AccessKind::ReadOnly,
                        targets: TargetSet::Any,
                    },
                    AllowedInterrupt {
                        kind: AccessKind::Modify,
                        targets: TargetSet::AnyExcept(vec![DocumentTarget::ProductShapes]),
                    },
                ],
                when_blocked: WhenBlocked::Wait,
                cancellable: true,
            },
            Self::NoAccess => synchronous(Vec::new(), WhenBlocked::Wait),
            // 読んだものはまだ保存していないので、読み込みを取り消してから終える
            Self::Exit => synchronous(replace_model(), WhenBlocked::CancelThenRun),
        }
    }
}

/// 空間モデルの文書に対する、実行中のコマンドのスタックと、待たせたコマンドの待ち行列（`C` はコマンドの値の型）
pub type DocumentCommandScheduler<C> = CommandScheduler<DocumentTarget, C>;

#[cfg(test)]
mod tests {
    use super::DocumentCommandKind::{
        Exit, ImportShape, ImportShapeInBackground, NoAccess, ReplaceModel, SaveModel,
    };
    use super::*;

    /// 種類 `kind` のコマンドを出す（コマンドの値は、種類そのもの）
    fn submit(
        scheduler: &mut DocumentCommandScheduler<DocumentCommandKind>,
        kind: DocumentCommandKind,
    ) -> Submission<DocumentCommandKind> {
        scheduler.submit(kind.declaration(), kind)
    }

    /// 種類 `kind` のコマンドを出して、実行を始めた識別子（始まらなければ、試験を落とす）
    fn start(
        scheduler: &mut DocumentCommandScheduler<DocumentCommandKind>,
        kind: DocumentCommandKind,
    ) -> CommandId {
        match submit(scheduler, kind) {
            Submission::Started(started) => started.id,
            other => panic!("{kind:?} does not start: {other:?}"),
        }
    }

    #[test]
    fn every_command_starts_when_nothing_is_running() {
        for kind in [
            ReplaceModel,
            SaveModel,
            ImportShape,
            ImportShapeInBackground,
            NoAccess,
            Exit,
        ] {
            let mut scheduler = DocumentCommandScheduler::default();
            start(&mut scheduler, kind);
        }
    }

    #[test]
    fn commands_that_do_not_change_the_shapes_start_while_a_shape_is_imported_in_the_background() {
        let mut scheduler = DocumentCommandScheduler::default();
        start(&mut scheduler, ImportShapeInBackground);

        // 保存は、読み込む前の空間モデルを書く。ファイルを選ぶ画面は、空間モデルに触らない
        for kind in [SaveModel, NoAccess] {
            let id = start(&mut scheduler, kind);
            scheduler.finish(id);
        }
    }

    #[test]
    fn commands_that_change_or_replace_the_shapes_wait_for_the_background_import() {
        let mut scheduler = DocumentCommandScheduler::default();
        let import = start(&mut scheduler, ImportShapeInBackground);

        for kind in [ImportShape, ImportShapeInBackground, ReplaceModel] {
            assert_eq!(submit(&mut scheduler, kind), Submission::Queued, "{kind:?}");
        }
        // 待つだけのコマンドは、読み込みを取り消さない
        assert!(scheduler.take_cancel_requests().is_empty());

        // 読み込みが終わると、待たせた順に始まる（その場で終わるコマンドは、下ろしてから次を受け取る）
        scheduler.finish(import);
        let first = scheduler.take_ready().unwrap();
        assert_eq!(first.command, ImportShape);
        scheduler.finish(first.id);
        let second = scheduler.take_ready().unwrap();
        assert_eq!(second.command, ImportShapeInBackground);
        // 次の読み込みの最中は、入れ替えるコマンドは待ったまま
        assert_eq!(scheduler.take_ready(), None);
        scheduler.finish(second.id);
        assert_eq!(scheduler.take_ready().unwrap().command, ReplaceModel);
    }

    #[test]
    fn exiting_cancels_the_background_import_and_runs_after_it_stops() {
        let mut scheduler = DocumentCommandScheduler::default();
        let import = start(&mut scheduler, ImportShapeInBackground);

        assert_eq!(submit(&mut scheduler, Exit), Submission::Queued);
        // 取り消す読み込みは、1 回だけ受け取る
        assert_eq!(scheduler.take_cancel_requests(), vec![import]);
        assert!(scheduler.take_cancel_requests().is_empty());
        // 読み込みが止まるまでは、終了は待ったまま
        assert_eq!(scheduler.take_ready(), None);

        scheduler.finish(import);
        assert_eq!(scheduler.take_ready().unwrap().command, Exit);
    }

    #[test]
    fn the_background_import_is_the_only_command_that_stays_running() {
        for kind in [ReplaceModel, SaveModel, ImportShape, NoAccess, Exit] {
            let declaration = kind.declaration();
            assert_eq!(declaration.execution, Execution::Synchronous, "{kind:?}");
            assert!(!declaration.cancellable, "{kind:?}");
        }
        let background = ImportShapeInBackground.declaration();
        assert_eq!(background.execution, Execution::Asynchronous);
        assert!(background.cancellable);
    }
}
