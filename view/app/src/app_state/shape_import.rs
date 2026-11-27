//! AppState の、別のスレッドで行う形状の読み込み（STEP）を扱うモジュール。
//!
//! 読み込みは、終わるまで実行中のまま残るコマンドとして始める。毎フレーム、進み具合を画面の下の行に出し、終わって
//! いれば、結果を文書に反映してコマンドを終える。

use std::path::PathBuf;
use std::time::Instant;

use viewmodel::document_command::CommandId;
use viewmodel::step_reading::{StepReading, StepReadingOutcome};

use super::AppState;
use super::model_file::{file_bytes, file_name, millis_since};
use crate::help_messages::resolve_help_text;
use crate::help_ui::{Notice, ProgressLine, StepImportIssues};

/// 別のスレッドで行っている形状の読み込み
pub(super) struct RunningShapeImport {
    /// 実行中のコマンドの識別子
    command: CommandId,
    reading: StepReading,
    path: PathBuf,
    started: Instant,
}

impl AppState {
    /// 形状ファイル `path`（STEP）の読み込みを、コマンド `command` の処理として、別のスレッドで始める
    pub(super) fn start_shape_import(&mut self, command: CommandId, path: PathBuf) {
        let reading = self.document.start_step_reading(&path);
        self.shape_import = Some(RunningShapeImport {
            command,
            reading,
            path,
            started: Instant::now(),
        });
    }

    /// 別のスレッドの形状の読み込みが終わっていれば、結果を文書に反映して、コマンドを終える（取り消した・読めなかった
    /// ときは、何も加えない。取り消しを頼んだ後に読み終えていたときも、取り消したものとして、加えない）
    pub(super) fn finish_shape_import(&mut self) {
        let Some(outcome) = self
            .shape_import
            .as_mut()
            .and_then(|import| import.reading.take_outcome())
        else {
            return;
        };
        let Some(import) = self.shape_import.take() else {
            return;
        };
        self.commands.finish(import.command);
        let file = file_name(&import.path);
        let elapsed_ms = millis_since(import.started);
        let cancel_requested = import.reading.is_cancel_requested();
        match outcome {
            StepReadingOutcome::Read(shapes) if !cancel_requested => {
                // 読む時間と、反映して製品形状を表示する時間を分けて出す
                let shown = Instant::now();
                let report = self.document.add_step_shapes(shapes);
                self.show_imported_shapes(Notice::StepImported {
                    file,
                    shapes: report.shells,
                    issues: StepImportIssues {
                        skipped_elements: report.skipped_elements,
                        unplaced_parts: report.unplaced_occurrences,
                        skipped_parts: report.skipped_occurrences,
                    },
                });
                tracing::info!(
                    path = %import.path.display(),
                    elapsed_ms,
                    display_ms = millis_since(shown),
                    bytes = file_bytes(&import.path),
                    "product shape imported"
                );
            }
            StepReadingOutcome::Failed(error) => {
                tracing::warn!(
                    path = %import.path.display(),
                    elapsed_ms,
                    "product shape import failed: {error}"
                );
                self.set_notice(Notice::ShapeImportFailed {
                    file,
                    reason: viewmodel::failure_message_mapper::product_shape_failure_message(
                        &error,
                    ),
                });
            }
            StepReadingOutcome::Read(_) | StepReadingOutcome::Cancelled => {
                tracing::info!(
                    path = %import.path.display(),
                    elapsed_ms,
                    "product shape import cancelled"
                );
                self.set_notice(Notice::ShapeImportCancelled { file });
            }
        }
    }

    /// 別のスレッドの形状の読み込みに、取り消しを頼む（止まるのを待たない）
    pub(crate) fn cancel_shape_import(&self) {
        if let Some(import) = &self.shape_import {
            import.reading.request_cancel();
        }
    }

    /// コマンド `command` が別のスレッドの形状の読み込みなら、取り消しを頼む
    pub(super) fn cancel_command(&self, command: CommandId) {
        if self
            .shape_import
            .as_ref()
            .is_some_and(|import| import.command == command)
        {
            self.cancel_shape_import();
        }
    }

    /// 画面の下の行に出す、形状の読み込みの進み具合（読み込んでいなければ `None`）
    pub(super) fn shape_import_progress(&self) -> Option<ProgressLine> {
        let import = self.shape_import.as_ref()?;
        let fraction = import
            .reading
            .progress()
            .map(|progress| progress.done as f32 / progress.total.max(1) as f32);
        Some(ProgressLine {
            text: resolve_help_text(
                self.ui_locale,
                "progress.shape_import",
                &[("file", file_name(&import.path))],
            ),
            fraction,
            cancelling: import.reading.is_cancel_requested(),
        })
    }
}
