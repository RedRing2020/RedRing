//! 読んだ STEP の形状（製品形状の保管先に加える前のもの）と、その表示用の三角形を作る仕事
//!
//! STEP の読み込みは、読む処理（ファイルを読み、保管先に加える前の形状にする）と、加える処理（保管先に加える）に
//! 分ける。読む処理は、区切りごとに進み具合を知らせ、取り消しを頼まれていれば止める。表示用の三角形は、保管先に
//! 加える前に、B-rep ごとに作って置き場に残せる。

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::ops::ControlFlow;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use geo_topology::Brep;

use super::step_import::{
    self, ImportedShell, StepImportError, StepImportProgress, StepImportReport, StepImportStage,
};
use super::{ProductShapeError, ShellMeshCache};
use crate::background_task::{TaskContext, TaskFailure};

/// 読んだ STEP の形状（アセンブリの出現ごとのシェル。同じ部品の出現は、B-rep の表と三角形の置き場を共有する）
pub struct StepShapes {
    /// シェルと、その B-rep の三角形の置き場
    shells: Vec<(ImportedShell, Arc<ShellMeshCache>)>,
    report: StepImportReport,
}

impl StepShapes {
    fn new(shells: Vec<ImportedShell>, report: StepImportReport) -> Self {
        // 読んだ B-rep は保管先のどのシェルとも共有しないので、同じ部品の出現の間でだけ置き場を共有する
        let mut shared: HashMap<*const Brep<f64>, Arc<ShellMeshCache>> = HashMap::new();
        let shells = shells
            .into_iter()
            .map(|shell| {
                let meshes = Arc::clone(shared.entry(Arc::as_ptr(&shell.brep)).or_default());
                (shell, meshes)
            })
            .collect();
        Self { shells, report }
    }

    /// シェル（三角形の置き場と一緒に）と、読み込みの結果に分ける
    pub(super) fn into_parts(
        self,
    ) -> (Vec<(ImportedShell, Arc<ShellMeshCache>)>, StepImportReport) {
        (self.shells, self.report)
    }

    /// 表示用の三角形（許容弦誤差 `chord_tolerance`）を、B-rep ごとに作る仕事（同じ部品の出現の B-rep は 1 回）
    pub fn display_mesh_work(&self, chord_tolerance: f64) -> DisplayMeshWork {
        let mut seen = HashSet::new();
        let mut items: Vec<MeshItem> = Vec::new();
        for (shell, meshes) in &self.shells {
            if seen.insert(Arc::as_ptr(&shell.brep)) {
                items.push(MeshItem {
                    brep: Arc::clone(&shell.brep),
                    meshes: Arc::clone(meshes),
                    file_name: shell.source.file_name.clone(),
                    faces: shell.source.faces.clone(),
                });
            }
        }
        // 面の多い B-rep から取る（最も時間のかかる B-rep を、最後に残さない）
        items.sort_by_key(|item| std::cmp::Reverse(item.faces.len()));
        DisplayMeshWork {
            total_faces: items.iter().map(|item| item.faces.len()).sum(),
            items,
            key: chord_tolerance.to_bits(),
            next: AtomicUsize::new(0),
            finished_faces: AtomicUsize::new(0),
        }
    }
}

/// STEP ファイル `path` を読み、製品形状の保管先に加える前の形状にする（外した要素・直したことは、要素ごとにログに
/// 出す）
///
/// 段の初めと、段の中の区切りごとに、`context` に進み具合を知らせ、取り消しを頼まれていれば止める。
///
/// # Errors
///
/// ファイルを読めない、構文の誤り、上限を超える、形状が 1 つも読めない場合。取り消しを頼まれて止めた場合
/// （[`TaskFailure::Cancelled`]）。
pub fn read_step_file(
    path: &Path,
    context: &TaskContext<StepImportProgress>,
) -> Result<StepShapes, TaskFailure<ProductShapeError>> {
    let file_error = |error: std::io::Error| {
        TaskFailure::Failed(ProductShapeError::StepFile {
            kind: error.kind(),
            reason: error.to_string(),
        })
    };
    let size = std::fs::metadata(path).map_err(file_error)?.len();
    let limits = geo_step::syntax::Limits::default();
    if size > limits.max_bytes as u64 {
        return Err(TaskFailure::Failed(ProductShapeError::Step(
            StepImportError::TooLarge,
        )));
    }
    let bytes = std::fs::read(path).map_err(file_error)?;
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (shells, report) = step_import::read_step(&bytes, &file_name, &mut |progress| {
        context.report_progress(progress);
        if context.is_cancel_requested() {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    })
    .map_err(|failure| match failure {
        TaskFailure::Cancelled => TaskFailure::Cancelled,
        TaskFailure::Failed(error) => TaskFailure::Failed(ProductShapeError::Step(error)),
    })?;
    Ok(StepShapes::new(shells, report))
}

/// 読んだ STEP の形状の、表示用の三角形を B-rep ごとに作る仕事
///
/// [`Self::triangulate_next`] は、複数のスレッドから同時に呼べる（B-rep は 1 つずつ取る）。作った三角形は、読んだ形状が
/// 持つ置き場に残り、形状を保管先に加えると、シェルのエンティティの置き場になる。
pub struct DisplayMeshWork {
    /// 三角形を作る B-rep（面の多い順）
    items: Vec<MeshItem>,
    /// 許容弦誤差のビット列
    key: u64,
    /// 次に取る B-rep の位置
    next: AtomicUsize,
    /// 三角形を作り終えた B-rep の面の数の合計
    finished_faces: AtomicUsize,
    /// すべての B-rep の面の数の合計
    total_faces: usize,
}

/// 三角形を作る B-rep と、その置き場・元のファイルの名前・面のエンティティの番号（B-rep の面の順）
struct MeshItem {
    brep: Arc<Brep<f64>>,
    meshes: Arc<ShellMeshCache>,
    file_name: String,
    faces: Vec<u64>,
}

impl DisplayMeshWork {
    /// 三角形を作る B-rep の数
    pub fn brep_count(&self) -> usize {
        self.items.len()
    }

    /// まだ取られていない B-rep を 1 つ取り、三角形を作って置き場に残す（三角形にできない面は、警告に出す）
    ///
    /// 取る B-rep が残っていなければ、何もせずに `false` を返す。
    pub fn triangulate_next(&self) -> bool {
        let position = self.next.fetch_add(1, Ordering::Relaxed);
        let Some(item) = self.items.get(position) else {
            return false;
        };
        let (mesh, built) = item.meshes.mesh_or_build(self.key, &item.brep);
        if built {
            for (face, result) in item.faces.iter().zip(&mesh.faces) {
                if let Err(error) = result {
                    tracing::warn!(
                        file = %item.file_name,
                        face,
                        "a shell face cannot be triangulated: {error}"
                    );
                }
            }
        }
        self.finished_faces
            .fetch_add(item.faces.len(), Ordering::Relaxed);
        true
    }

    /// 読み込みの全体の進み具合（三角形を作り終えた B-rep の面の数で測る）
    pub fn progress(&self) -> StepImportProgress {
        StepImportProgress::in_stage(
            StepImportStage::DisplayMeshes,
            self.finished_faces.load(Ordering::Relaxed),
            self.total_faces,
        )
    }
}
