//! 切削アニメーションの素材の面
//!
//! 時点の素材の状態の作り直しと、変わったブロックの面の作成は時間がかかる（遠くの時点へ移る場合）ため、
//! 別のスレッドで行う。画面の側は、表示したい時点を頼み、できた面を受け取って描くだけにする。

use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use application::cutting_simulation_playback::{StockPlayback, SurfaceMesh};

use super::StockBlockMesh;
use crate::mesh_converter::VertexData;

/// ブロック `block` の素材の面 `mesh` を、描画用の頂点に詰め替える
pub(super) fn block_mesh(block: [usize; 3], mesh: SurfaceMesh) -> StockBlockMesh {
    let to_f32 = |value: [f64; 3]| value.map(|component| component as f32);
    let SurfaceMesh {
        positions,
        normals,
        indices,
        classes,
    } = mesh;
    StockBlockMesh {
        block,
        vertices: positions
            .into_iter()
            .zip(normals)
            .map(|(position, normal)| VertexData::new(to_f32(position), to_f32(normal)))
            .collect(),
        indices,
        classes,
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
    /// まだ取りかかっていない、頼まれた時点と、最後の時点の面を製品形状との比較で分けるか
    request: Option<(usize, bool)>,
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
    pub(super) fn spawn(stock: StockPlayback) -> Self {
        let shared = Arc::new(Shared::default());
        let worker = Arc::clone(&shared);
        std::thread::spawn(move || run(&worker, stock));
        Self { shared }
    }

    /// 時点 `frame` の素材の面を頼む（最後の時点なら、`compare` の場合に製品形状との比較で分ける）
    pub(super) fn request(&self, frame: usize, compare: bool) {
        self.shared.lock().request = Some((frame, compare));
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
fn run(shared: &Shared, mut stock: StockPlayback) {
    loop {
        let (frame, compare) = {
            let mut state = shared.lock();
            loop {
                if state.stopped {
                    return;
                }
                if let Some(request) = state.request.take() {
                    break request;
                }
                state = shared
                    .wake
                    .wait(state)
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            }
        };
        let (target, meshes) = stock.seek(frame, compare);
        let blocks: Vec<StockBlockMesh> = meshes
            .into_iter()
            .map(|(block, mesh)| block_mesh(block, mesh))
            .collect();
        let mut state = shared.lock();
        let (shown, update) = state.update.get_or_insert_with(|| (target, HashMap::new()));
        *shown = target;
        update.extend(blocks.into_iter().map(|mesh| (mesh.block, mesh)));
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::super::tests::{animation, positions};
    use super::*;

    /// 時点 `frame` の面ができるまで、受け取った面を `shown` に当てる
    fn wait_for(
        animation: &mut super::super::CuttingAnimation,
        frame: usize,
        shown: &mut HashMap<[usize; 3], StockBlockMesh>,
    ) {
        let started = Instant::now();
        loop {
            if let Some(blocks) = animation.take_stock_update() {
                shown.extend(blocks.into_iter().map(|mesh| (mesh.block, mesh)));
                if animation.frame() == frame {
                    return;
                }
            }
            assert!(started.elapsed() < Duration::from_secs(60), "frame {frame}");
            std::thread::sleep(Duration::from_millis(1));
        }
    }

    /// 別の切削アニメーションで、最初の時点から時点 `frame` へ直に移した素材の面
    fn expected_at(frame: usize) -> HashMap<[usize; 3], Vec<[f32; 3]>> {
        let mut direct = animation();
        let mut shown: HashMap<_, _> = direct
            .take_initial_stock_blocks()
            .into_iter()
            .map(|mesh| (mesh.block, mesh))
            .collect();
        direct.request_frame(frame);
        wait_for(&mut direct, frame, &mut shown);
        positions(&shown)
    }

    #[test]
    fn surfaces_made_on_the_other_thread_follow_the_requested_frames() {
        let mut animation = animation();
        let last = animation.frame_count() - 1;
        let mut shown: HashMap<_, _> = animation
            .take_initial_stock_blocks()
            .into_iter()
            .map(|mesh| (mesh.block, mesh))
            .collect();
        // 1 つずつ進む・遠くへ飛ぶ・戻ると、直に移した面と同じになる
        for frame in [1, 2, last, last / 2, 0] {
            animation.request_frame(frame);
            wait_for(&mut animation, frame, &mut shown);
            assert_eq!(positions(&shown), expected_at(frame), "frame {frame}");
        }
        // 受け取る前に次の時点を頼むと、最後に頼んだ時点の面になる（途中の時点で変わったブロックも含む）
        for frame in [last / 3, last / 2, 2 * last / 3] {
            animation.request_frame(frame);
        }
        wait_for(&mut animation, 2 * last / 3, &mut shown);
        assert_eq!(positions(&shown), expected_at(2 * last / 3));
    }
}
