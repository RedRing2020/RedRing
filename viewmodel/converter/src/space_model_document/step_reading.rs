//! STEP の形状を読む処理（空間モデルに加える前まで）
//!
//! ファイルを読み、表示用の三角形を作るところまでを行う。文書を触らず、区切りごとに進み具合を知らせ、取り消しを
//! 頼まれていれば止める。

use std::convert::Infallible;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use application::background_task::{BackgroundTask, TaskContext, TaskFailure, TaskOutcome};
use application::product_shape::{
    DisplayMeshWork, ProductShapeError, StepImportProgress, StepShapes, read_step_file,
};

use crate::product_shape_converter::ProductShapeDisplaySettings;

/// 表示用の三角形を作るスレッドが終わったかを確かめる間隔
const WORKER_POLL_INTERVAL: Duration = Duration::from_millis(5);

/// STEP ファイル `path` を読み、表示の設定 `display` の表示用の三角形を作った形状にする
///
/// # Errors
///
/// STEP の読み込みの失敗。取り消しを頼まれて止めた場合（[`TaskFailure::Cancelled`]）。
pub(super) fn read_step_shapes(
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
pub(super) fn triangulate_display_meshes<E>(
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
