//! STEP ファイルを、画面を止めずに読む処理
//!
//! ファイルを読み、表示用の三角形を作るところまでを、別のスレッドで行う。始めた側は、進み具合と結果を待たずに
//! 問い合わせ、取り消しを頼む。読んだ形状は、空間モデルの文書に加えるまで、文書を変えない。

#[cfg(test)]
mod tests;

use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use application::background_task::{BackgroundTask, TaskContext, TaskFailure, TaskOutcome};
use application::product_shape::{DisplayMeshWork, ProductShapeError, read_step_file};
/// 読み込みの進み具合と、読んだ形状
pub use application::product_shape::{StepImportProgress, StepShapes};

use crate::product_shape_converter::ProductShapeDisplaySettings;

/// 表示用の三角形を作るスレッドが終わったかを確かめる間隔
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// 別のスレッドで行っている、STEP ファイルの読み込み
///
/// 結果を受け取らずに手放すと、取り消しを頼む。
pub struct StepReading {
    task: BackgroundTask<StepShapes, ProductShapeError, StepImportProgress>,
}

/// STEP ファイルの読み込みの結果
pub enum StepReadingOutcome {
    /// 読めた（表示用の三角形を作り終えた形状）
    Read(StepShapes),
    /// 読めなかった
    Failed(ProductShapeError),
    /// 取り消しを頼まれて止めた
    Cancelled,
}

impl StepReading {
    /// STEP ファイル `path` の読み込みを、表示の設定 `display` で、別のスレッドで始める
    ///
    /// スレッドを作れなければ、今のスレッドで終わりまで行う（結果の受け取り方は同じ）。
    pub(crate) fn start(path: PathBuf, display: ProductShapeDisplaySettings) -> Self {
        let work = move |context: &TaskContext<StepImportProgress>| {
            read_step_shapes(&path, display, context)
        };
        let task = BackgroundTask::spawn("step-reading", work).unwrap_or_else(|error| {
            tracing::warn!("reading the STEP file on the current thread: {error}");
            BackgroundTask::run_on_current_thread(error.into_work())
        });
        Self { task }
    }

    /// 最も新しい進み具合（まだ知らせていなければ `None`）
    pub fn progress(&self) -> Option<StepImportProgress> {
        self.task.progress()
    }

    /// 取り消しを頼む（止まるのを待たない。止まったら、結果が [`StepReadingOutcome::Cancelled`] になる）
    pub fn request_cancel(&self) {
        self.task.request_cancel();
    }

    /// 取り消しを頼んであるか
    pub fn is_cancel_requested(&self) -> bool {
        self.task.is_cancel_requested()
    }

    /// 終わっていれば、結果を受け取る（終わっていないとき・受け取った後は `None`）
    ///
    /// # Panics
    ///
    /// 読む処理の中で panic していた場合（その panic の文で panic する）。
    pub fn take_outcome(&mut self) -> Option<StepReadingOutcome> {
        Some(match self.task.take_outcome()? {
            TaskOutcome::Succeeded(shapes) => StepReadingOutcome::Read(shapes),
            TaskOutcome::Failed(error) => StepReadingOutcome::Failed(error),
            TaskOutcome::Cancelled => StepReadingOutcome::Cancelled,
            TaskOutcome::Panicked(message) => panic!("{message}"),
        })
    }
}

/// STEP ファイル `path` を読み、表示の設定 `display` の表示用の三角形を作った形状にする
///
/// # Errors
///
/// STEP の読み込みの失敗。取り消しを頼まれて止めた場合（[`TaskFailure::Cancelled`]）。
fn read_step_shapes(
    path: &Path,
    display: ProductShapeDisplaySettings,
    context: &TaskContext<StepImportProgress>,
) -> Result<StepShapes, TaskFailure<ProductShapeError>> {
    let shapes = read_step_file(path, context)?;
    let work = Arc::new(shapes.display_mesh_work(display.chord_tolerance));
    triangulate_display_meshes(&work, context)?;
    Ok(shapes)
}

/// 表示用の三角形を作る仕事 `work` を、B-rep ごとに、別のスレッドに並べて行う
///
/// 作るスレッドの数は、使えるスレッドの数から 1 つ引いた数（画面のスレッドの分を残す。1 つは作る）と、B-rep の数の
/// 小さいほうにする。このスレッドは、作るスレッドが終わるのを待ちながら、進み具合を知らせ、取り消しを確かめる
/// （1 つの B-rep に時間がかかっていても、取り消しをすぐに返せる）。取り消しで返ると、作るスレッドは、作りかけの
/// B-rep を作り終えてから止まる。
fn triangulate_display_meshes<E>(
    work: &Arc<DisplayMeshWork>,
    context: &TaskContext<StepImportProgress>,
) -> Result<(), TaskFailure<E>> {
    context.report_progress(work.progress());
    let threads = std::thread::available_parallelism()
        .map_or(1, NonZeroUsize::get)
        .saturating_sub(1)
        .max(1)
        .min(work.brep_count());
    let mut workers: Vec<BackgroundTask<(), Infallible, ()>> = (0..threads)
        .filter_map(|index| {
            let work = Arc::clone(work);
            BackgroundTask::spawn(&format!("display-mesh-{index}"), move |worker| {
                while !worker.is_cancel_requested() && work.triangulate_next() {}
                Ok(())
            })
            .inspect_err(|error| {
                tracing::warn!("cannot start a thread to build display meshes: {error}");
            })
            .ok()
        })
        .collect();
    if workers.is_empty() {
        // スレッドを 1 つも作れなければ、このスレッドで作る
        while !context.is_cancel_requested() && work.triangulate_next() {
            context.report_progress(work.progress());
        }
    }
    while !workers.is_empty() {
        if context.is_cancel_requested() {
            return Err(TaskFailure::Cancelled);
        }
        context.report_progress(work.progress());
        std::thread::sleep(WORKER_POLL_INTERVAL);
        let mut running = Vec::with_capacity(workers.len());
        for mut worker in workers {
            match worker.take_outcome() {
                None => running.push(worker),
                // 作るスレッドの panic は、この処理の panic にする
                Some(TaskOutcome::Panicked(message)) => panic!("{message}"),
                Some(_) => {}
            }
        }
        workers = running;
    }
    if context.is_cancel_requested() {
        return Err(TaskFailure::Cancelled);
    }
    context.report_progress(work.progress());
    Ok(())
}
