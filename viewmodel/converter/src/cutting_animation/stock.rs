//! 切削アニメーションの素材の面
//!
//! 時点の素材の状態の作り直しと、変わったブロックの面の作成は時間がかかる（遠くの時点へ移る場合）ため、
//! 別のスレッドで行う。画面の側は、表示したい時点を頼み、できた面を受け取って描くだけにする。

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use cam_core::{Tool, ToolPath};
use cam_sim::{CuttingSimulationArtifact, CuttingSimulationReplay, DexelSurface, SurfaceMesh};

use super::{CuttingAnimationError, StockBlockMesh};
use crate::mesh_converter::VertexData;

/// 素材の面（今の時点の素材の状態と面を持ち、時点を移すと変わったブロックの面を作る）
#[derive(Debug)]
pub(super) struct StockAnimation {
    /// 成果物の再生（今の時点の素材の状態を持つ）
    replay: CuttingSimulationReplay,
    surface: DexelSurface,
}

impl StockAnimation {
    /// 成果物とツールパスと工具と素材の精度から、最初の時点の素材の面を作る
    pub(super) fn new(
        artifact: CuttingSimulationArtifact,
        toolpath: &ToolPath<f64>,
        tool: &Tool<f64>,
        precision: f64,
    ) -> Result<Self, CuttingAnimationError> {
        let replay = CuttingSimulationReplay::new(artifact, toolpath, tool)
            .map_err(CuttingAnimationError::Replay)?;
        let surface = DexelSurface::new(
            replay.artifact().grids,
            replay.states(),
            precision,
            tool.radius(),
        );
        Ok(Self { replay, surface })
    }

    /// 時点の数
    fn frame_count(&self) -> usize {
        self.replay.artifact().snapshots.len()
    }

    /// 今の時点の、すべてのブロックの素材の面（三角形のないブロックは含めない）
    pub(super) fn stock_blocks(&self) -> Vec<StockBlockMesh> {
        let [bx, by, bz] = self.surface.block_counts();
        let blocks: Vec<[usize; 3]> = (0..bx)
            .flat_map(|x| (0..by).flat_map(move |y| (0..bz).map(move |z| [x, y, z])))
            .collect();
        self.block_meshes(&blocks)
            .into_iter()
            .filter(|mesh| !mesh.indices.is_empty())
            .collect()
    }

    /// 時点 `frame` へ移し、面が変わりうるブロックの、移った後の素材の面を返す（範囲外は最後の時点にする）
    ///
    /// 再生の入口で時点の素材の状態を作り直し、区間が変わった部分のブロックだけを作り直す。
    /// 三角形がなくなったブロックは、三角形のない面として返す。
    fn seek(&mut self, frame: usize) -> Vec<StockBlockMesh> {
        let target = frame.min(self.frame_count() - 1);
        let blocks = self.surface.apply_changes(self.replay.seek(target));
        self.block_meshes(&blocks)
    }

    /// ブロックの列 `blocks` の素材の面（複数のコアで並べて作る）
    fn block_meshes(&self, blocks: &[[usize; 3]]) -> Vec<StockBlockMesh> {
        let to_f32 = |value: [f64; 3]| value.map(|component| component as f32);
        blocks
            .iter()
            .zip(self.surface.mesh_blocks(blocks))
            .map(
                |(
                    &block,
                    SurfaceMesh {
                        positions,
                        normals,
                        indices,
                    },
                )| StockBlockMesh {
                    block,
                    vertices: positions
                        .into_iter()
                        .zip(normals)
                        .map(|(position, normal)| VertexData::new(to_f32(position), to_f32(normal)))
                        .collect(),
                    indices,
                },
            )
            .collect()
    }
}

/// 素材の面を別のスレッドで作る係
///
/// 頼まれた時点へ素材を移し、変わったブロックの面を、画面の側が受け取るまでブロックごとに最も新しい面として貯める。
/// 受け取らなかった途中の時点の面も、後の時点の面に含めて渡すため、変わったブロックが抜けることはない。
/// 作り直しのあいだに次の時点を頼まれた場合は、作り直しが終わった後に、最後に頼まれた時点へ移る（間に頼まれた時点は飛ばす）。
#[derive(Debug)]
pub(super) struct StockWorker {
    shared: Arc<Shared>,
}

/// 画面の側と別のスレッドが共有する状態
#[derive(Debug, Default)]
struct Shared {
    state: Mutex<SharedState>,
    wake: Condvar,
}

#[derive(Debug, Default)]
struct SharedState {
    /// まだ取りかかっていない、頼まれた時点
    request: Option<usize>,
    /// 画面の側がまだ受け取っていない面（面が表す時点と、ブロックごとの最も新しい面）
    update: Option<(usize, HashMap<[usize; 3], StockBlockMesh>)>,
    /// 画面の側が係を手放した
    stopped: bool,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, SharedState> {
        // 別のスレッドは状態を書き換える途中で止まらない（作り直しは鍵の外で行う）ため、毒された鍵も使える
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl StockWorker {
    /// 素材の面 `stock` を別のスレッドに渡して、係を始める
    pub(super) fn spawn(stock: StockAnimation) -> Self {
        let shared = Arc::new(Shared::default());
        let worker = Arc::clone(&shared);
        std::thread::spawn(move || run(&worker, stock));
        Self { shared }
    }

    /// 時点 `frame` の素材の面を頼む
    pub(super) fn request(&self, frame: usize) {
        self.shared.lock().request = Some(frame);
        self.shared.wake.notify_one();
    }

    /// できた面を受け取る（面が表す時点と、前に受け取ってから変わったブロックの面。まだなければ `None`）
    pub(super) fn take_update(&self) -> Option<(usize, Vec<StockBlockMesh>)> {
        let (frame, blocks) = self.shared.lock().update.take()?;
        Some((frame, blocks.into_values().collect()))
    }
}

impl Drop for StockWorker {
    fn drop(&mut self) {
        // 作り直しの途中なら、別のスレッドは作り直しを終えてから止まる（画面の側は待たない）
        self.shared.lock().stopped = true;
        self.shared.wake.notify_one();
    }
}

/// 別のスレッドの処理
fn run(shared: &Shared, mut stock: StockAnimation) {
    loop {
        let frame = {
            let mut state = shared.lock();
            loop {
                if state.stopped {
                    return;
                }
                if let Some(frame) = state.request.take() {
                    break frame;
                }
                state = shared
                    .wake
                    .wait(state)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };
        let target = frame.min(stock.frame_count() - 1);
        let blocks = stock.seek(target);
        let mut state = shared.lock();
        let (shown, update) = state.update.get_or_insert_with(|| (target, HashMap::new()));
        *shown = target;
        update.extend(blocks.into_iter().map(|mesh| (mesh.block, mesh)));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::super::tests::{artifacts, positions, tool, PRECISION};
    use super::*;
    use cam_core::read_toolpath_artifact_v1;
    use cam_sim::read_cutting_simulation_artifact_v1;

    fn stock() -> StockAnimation {
        let (toolpath, simulation) = artifacts();
        let (_, toolpath) = read_toolpath_artifact_v1(&mut &toolpath[..]).unwrap();
        let (_, artifact) = read_cutting_simulation_artifact_v1(&mut &simulation[..]).unwrap();
        StockAnimation::new(artifact, &toolpath, &tool(), PRECISION).unwrap()
    }

    fn by_block(blocks: Vec<StockBlockMesh>) -> HashMap<[usize; 3], StockBlockMesh> {
        blocks.into_iter().map(|mesh| (mesh.block, mesh)).collect()
    }

    /// 時点 `frame` の面ができるまで、受け取った面を `shown` に当てる
    fn wait_for(
        worker: &StockWorker,
        frame: usize,
        shown: &mut HashMap<[usize; 3], StockBlockMesh>,
    ) {
        let started = Instant::now();
        loop {
            if let Some((shown_frame, blocks)) = worker.take_update() {
                shown.extend(blocks.into_iter().map(|mesh| (mesh.block, mesh)));
                if shown_frame == frame {
                    return;
                }
            }
            assert!(started.elapsed() < Duration::from_secs(60), "frame {frame}");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// 同じスレッドで時点 `frame` へ移した素材の面
    fn expected_at(frame: usize) -> HashMap<[usize; 3], Vec<[f32; 3]>> {
        let mut stock = stock();
        stock.seek(frame);
        positions(&by_block(stock.stock_blocks()))
    }

    #[test]
    fn updating_changed_blocks_forward_and_backward_matches_the_state_at_each_frame() {
        let mut stock = stock();
        let mut shown = by_block(stock.stock_blocks());
        let last = stock.frame_count() - 1;
        for frame in [1, last / 2, last, last - 1, last / 2, 1, 0, last] {
            for mesh in stock.seek(frame) {
                shown.insert(mesh.block, mesh);
            }
            assert_eq!(stock.replay.snapshot(), frame);
            assert_eq!(
                positions(&shown),
                positions(&by_block(stock.stock_blocks())),
                "frame {frame}"
            );
        }
    }

    #[test]
    fn surfaces_made_on_the_other_thread_match_the_same_frames_made_on_this_thread() {
        let stock = stock();
        let last = stock.frame_count() - 1;
        let mut shown = by_block(stock.stock_blocks());
        let worker = StockWorker::spawn(stock);
        // 1 つずつ進む・遠くへ飛ぶ・戻る
        for frame in [1, 2, 3, last, last / 2, 1, 0] {
            worker.request(frame);
            wait_for(&worker, frame, &mut shown);
            assert_eq!(positions(&shown), expected_at(frame), "frame {frame}");
        }
        // 受け取る前に次の時点を頼むと、最後に頼んだ時点の面になる（途中の時点で変わったブロックも含む）
        for frame in [last / 3, last / 2, 2 * last / 3] {
            worker.request(frame);
        }
        wait_for(&worker, 2 * last / 3, &mut shown);
        assert_eq!(positions(&shown), expected_at(2 * last / 3));
    }
}
