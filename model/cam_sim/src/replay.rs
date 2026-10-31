//! 切削シミュレーションの再生の入口（設計: CUTTING_SIMULATION_STOCK_MODEL_DESIGN.md §5.2）
//!
//! 成果物の差分のキーフレームと正本（ツールパス・工具から作る掃引体の列）から、任意の時点の素材の状態を作り直す。
//! 先へ進むときは今の状態から続けて掃引体を差し引き、遠くへ飛ぶときは、移る先の手前のキーフレームの状態に戻してから
//! 差し引く。先へ進むあいだは、時点ごとに変わったデクセルの変わる前の区間を、経路の距離でキーフレームの間隔の分だけ
//! 残し、近くへ戻るときはそれを当てる。それより前へ戻るときは、キーフレームから作り直す。
//! 掃引体の差し引きはシミュレーションと同じ計算のため、元の実行と同じ状態になる。

use std::collections::VecDeque;

use cam_core::{Tool, ToolPath};
use geo_algorithms::{Aabb3D, Point3D};

use crate::constants::KEYFRAME_INTERVAL_MM;
use crate::error::SimulationError;
use crate::simulation_artifact::{
    CuttingSimulationArtifact, DexelGridStates, DexelState, dexel_count,
};
use crate::stock_model::StockModel;
use crate::tri_dexel::{DexelGrid, PreviousIntervals, Scratch, initial_normals};

/// 同じデクセルの前の変化がないことを表す番号
const NO_PREVIOUS_CHANGE: u32 = u32::MAX;

/// 1 つの時点を進めたときに変わったデクセルの、変わる前の区間（戻るときに当てる）
#[derive(Debug, Clone)]
struct Undo {
    /// 進める前の時点
    snapshot: usize,
    /// 格子ごとの、変わる前の区間
    grids: [PreviousIntervals; 3],
}

/// 切削シミュレーションの再生
#[derive(Debug, Clone)]
pub struct CuttingSimulationReplay {
    artifact: CuttingSimulationArtifact,
    model: StockModel,
    /// 今の時点の素材の状態
    grids: [DexelGrid; 3],
    /// 今の時点（スナップショットの番号）と、差し引いた掃引体の数
    snapshot: usize,
    applied: usize,
    /// `grids` の状態の元にしたキーフレーム（`None` は初期の状態）
    base: Option<usize>,
    /// 元にしたキーフレームの後に変わったデクセル（格子ごとの印と、格子・番号の列）
    since_base: [Vec<bool>; 3],
    since_base_dexels: Vec<(u8, u32)>,
    /// キーフレームの変化ごとの、同じデクセルの 1 つ前の変化の番号
    previous_change: Vec<u32>,
    /// デクセルごとの、最後のキーフレームの変化の番号
    last_change: [Vec<u32>; 3],
    scratch: Scratch,
    /// 今の時点まで 1 つずつ進めたときの、戻るための記録（古い順。最後は今の時点の 1 つ前から進めた記録）
    undo: VecDeque<Undo>,
    /// 戻るための記録を残す、経路の距離（mm）
    undo_distance_mm: f64,
    /// 前の移動で変わったデクセル
    moved: Vec<(u8, u32)>,
}

impl CuttingSimulationReplay {
    /// 成果物 `artifact` を、ツールパス `toolpath` と工具 `tool` から作った掃引体の列で再生する（最初の時点から始める）
    ///
    /// # Errors
    ///
    /// 工具がフラット・ボールエンドミルでない場合、成果物の時点が差し引く掃引体の数が、ツールパスと工具から作った
    /// 掃引体の数を超える場合。
    ///
    /// # Panics
    ///
    /// 成果物にスナップショットがない場合。
    pub fn new(
        artifact: CuttingSimulationArtifact,
        toolpath: &ToolPath<f64>,
        tool: &Tool<f64>,
    ) -> Result<Self, SimulationError> {
        assert!(
            !artifact.snapshots.is_empty(),
            "the artifact must have snapshots"
        );
        let [x, y, z] = artifact.grids.map(|grid| grid.initial);
        let stock = Aabb3D::new(Point3D::new(x.0, y.0, z.0), Point3D::new(x.1, y.1, z.1));
        let model = StockModel::from_toolpath(stock, toolpath, tool)?;
        let needed = artifact.snapshot_applied.last().copied().unwrap_or(0);
        if needed > model.primitives().len() {
            return Err(SimulationError::PrimitiveCountMismatch {
                artifact: needed,
                toolpath: model.primitives().len(),
            });
        }

        let changes = &artifact.keyframe_changes;
        let mut last_change = artifact
            .grids
            .map(|grid| vec![NO_PREVIOUS_CHANGE; dexel_count(&grid)]);
        let previous_change = (0..changes.len())
            .map(|number| {
                let (grid, index) = changes.dexel(number);
                let number = u32::try_from(number).expect("dexel change count fits in u32");
                std::mem::replace(&mut last_change[grid as usize][index as usize], number)
            })
            .collect();

        let mut replay = Self {
            grids: artifact.grids.each_ref().map(DexelGrid::from_layout),
            since_base: artifact.grids.map(|grid| vec![false; dexel_count(&grid)]),
            artifact,
            model,
            snapshot: 0,
            applied: 0,
            base: None,
            since_base_dexels: Vec::new(),
            previous_change,
            last_change,
            scratch: Scratch::default(),
            undo: VecDeque::new(),
            undo_distance_mm: KEYFRAME_INTERVAL_MM,
            moved: Vec::new(),
        };
        replay.subtract_until(replay.artifact.snapshot_applied[0]);
        replay.take_moved();
        Ok(replay)
    }

    /// 成果物
    pub fn artifact(&self) -> &CuttingSimulationArtifact {
        &self.artifact
    }

    /// 今の時点（スナップショットの番号）
    pub fn snapshot(&self) -> usize {
        self.snapshot
    }

    /// 今の時点の、各格子のデクセルの区間と区間の端の法線
    pub fn states(&self) -> DexelGridStates {
        let counts = self.artifact.grids.map(|grid| dexel_count(&grid));
        DexelGridStates {
            intervals: std::array::from_fn(|grid| {
                (0..counts[grid])
                    .map(|index| self.grids[grid].current(index).to_vec())
                    .collect()
            }),
            normals: std::array::from_fn(|grid| {
                (0..counts[grid])
                    .map(|index| self.grids[grid].current_normals(index).to_vec())
                    .collect()
            }),
        }
    }

    /// 時点 `snapshot` へ移し、移る前と区間が変わりうるデクセルの、移った後の区間を返す（範囲外は最後の時点にする）
    pub fn seek(&mut self, snapshot: usize) -> impl ExactSizeIterator<Item = DexelState<'_>> {
        self.moved.clear();
        let target = snapshot.min(self.artifact.snapshots.len() - 1);
        // 戻るための記録が残っている範囲は、記録を当てて戻る
        while self.snapshot > target
            && let Some(undo) = self.undo.pop_back()
        {
            self.apply_undo(undo);
        }
        let keyframe = self.artifact.keyframe_before(target);
        // 今の状態から続けて差し引けるのは、移る先が今より後で、移る先の手前のキーフレームより後にいる場合
        if self.snapshot > target || self.applied < self.keyframe_applied(keyframe) {
            self.restore(keyframe);
        }
        for next in self.snapshot + 1..=target {
            self.step_forward(next);
        }
        self.moved.sort_unstable();
        self.moved.dedup();
        let grids = &self.grids;
        self.moved.iter().map(move |&(grid, index)| DexelState {
            grid,
            index,
            intervals: grids[grid as usize].current(index as usize),
            normals: grids[grid as usize].current_normals(index as usize),
        })
    }

    /// 次の時点 `next` まで掃引体を差し引き、戻るための記録を残す（記録は、経路の距離でキーフレームの間隔の分だけ残す）
    fn step_forward(&mut self, next: usize) {
        for grid in &mut self.grids {
            grid.record_previous(true);
        }
        self.subtract_until(self.artifact.snapshot_applied[next]);
        let grids = std::array::from_fn(|grid| {
            let previous = self.grids[grid].take_previous();
            self.grids[grid].record_previous(false);
            previous
        });
        self.undo.push_back(Undo {
            snapshot: self.snapshot,
            grids,
        });
        self.snapshot = next;
        self.take_moved();
        let distance = |snapshot: usize| self.artifact.snapshots[snapshot].accumulated_distance_mm;
        while self.undo.front().is_some_and(|oldest| {
            distance(self.snapshot) - distance(oldest.snapshot) > self.undo_distance_mm
        }) {
            self.undo.pop_front();
        }
    }

    /// 戻るための記録 `undo` を当て、1 つ前の時点に戻す
    fn apply_undo(&mut self, undo: Undo) {
        for (grid, previous) in self.grids.iter_mut().zip(&undo.grids) {
            for &(index, [start, end]) in &previous.dexels {
                grid.set(
                    index,
                    &previous.intervals[start..end],
                    &previous.normals[start..end],
                );
            }
        }
        self.snapshot = undo.snapshot;
        self.applied = self.artifact.snapshot_applied[undo.snapshot];
        self.take_moved();
    }

    /// キーフレーム `keyframe` の時点までに差し引いた掃引体の数（`None` は初期の状態で 0）
    fn keyframe_applied(&self, keyframe: Option<usize>) -> usize {
        keyframe.map_or(0, |keyframe| {
            self.artifact.snapshot_applied[self.artifact.keyframe_snapshots[keyframe]]
        })
    }

    /// 素材の状態を、キーフレーム `keyframe` の時点（`None` は最初の時点）の状態に戻す
    ///
    /// 戻すデクセルは、元にしたキーフレームの後に変わったデクセルと、元にしたキーフレームと `keyframe` の間の
    /// キーフレームで変わったデクセル。
    fn restore(&mut self, keyframe: Option<usize>) {
        let changes = &self.artifact.keyframe_changes;
        let (first, last) = if keyframe < self.base {
            (keyframe, self.base)
        } else {
            (self.base, keyframe)
        };
        let between = last.map_or(0..0, |last| {
            let start = first.map_or(0, |first| changes.numbers(first).end);
            start..changes.numbers(last).end
        });
        let mut dexels = std::mem::take(&mut self.since_base_dexels);
        dexels.extend(between.map(|number| changes.dexel(number)));
        for &(grid, index) in &dexels {
            self.since_base[grid as usize][index as usize] = false;
            // デクセルの、`keyframe` 以前の最も新しい変化（なければ初期の区間）
            let mut number = self.last_change[grid as usize][index as usize];
            while number != NO_PREVIOUS_CHANGE
                && keyframe.is_none_or(|keyframe| changes.keyframe_of(number as usize) > keyframe)
            {
                number = self.previous_change[number as usize];
            }
            let grid_state = &mut self.grids[grid as usize];
            if number == NO_PREVIOUS_CHANGE {
                let initial = self.artifact.grids[grid as usize].initial;
                grid_state.set(
                    index as usize,
                    &[initial],
                    &[initial_normals(grid as usize)],
                );
            } else {
                let state = changes.state_of(number as usize);
                grid_state.set(index as usize, state.intervals, state.normals);
            }
        }
        // 戻したデクセルは、移る前と区間が変わりうる
        for (grid, state) in self.grids.iter_mut().enumerate() {
            self.moved.extend(
                state
                    .take_changed()
                    .into_iter()
                    .map(|index| (grid as u8, dexel_number(index))),
            );
        }
        self.base = keyframe;
        self.applied = self.keyframe_applied(keyframe);
        self.undo.clear();
        match keyframe {
            Some(keyframe) => self.snapshot = self.artifact.keyframe_snapshots[keyframe],
            None => {
                self.subtract_until(self.artifact.snapshot_applied[0]);
                self.snapshot = 0;
                self.take_moved();
            }
        }
    }

    /// 掃引体を、差し引いた数が `applied` になるまで差し引く
    fn subtract_until(&mut self, applied: usize) {
        let primitives = &self.model.primitives()[self.applied..applied];
        for primitive in primitives {
            for grid in &mut self.grids {
                grid.subtract(primitive, &mut self.scratch);
            }
        }
        self.applied = applied;
    }

    /// 差し引きで変わったデクセルを、前の移動で変わったデクセルと、元にしたキーフレームの後に変わったデクセルに加える
    fn take_moved(&mut self) {
        for (grid, state) in self.grids.iter_mut().enumerate() {
            for index in state.take_changed() {
                let dexel = (grid as u8, dexel_number(index));
                self.moved.push(dexel);
                if !std::mem::replace(&mut self.since_base[grid][index], true) {
                    self.since_base_dexels.push(dexel);
                }
            }
        }
    }
}

/// デクセルの番号
fn dexel_number(index: usize) -> u32 {
    u32::try_from(index).expect("dexel count is bounded by the line limit")
}

#[cfg(test)]
mod tests {
    use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType};

    use super::*;
    use crate::tri_dexel::TriDexelModel;
    use crate::{CuttingSimulator, SnapshotInterval};

    /// 10 × 10 × 4 の素材の上を、直径 2 のボールエンドミルで往復して掘り下げるツールパス
    fn toolpath() -> ToolPath<f64> {
        let points = [
            Point3D::new(-1.0, 2.0, -0.5),
            Point3D::new(11.0, 2.0, -0.5),
            Point3D::new(11.0, 3.0, -1.0),
            Point3D::new(-1.0, 3.0, -1.0),
            Point3D::new(-1.0, 4.0, -1.5),
            Point3D::new(11.0, 4.0, -1.5),
            Point3D::new(5.0, 8.0, -2.5),
        ];
        ToolPath::new(
            "BEM2".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(
                0,
                0.0,
                points
                    .windows(2)
                    .map(|pair| {
                        PathSegment::new_line(
                            pair[0],
                            pair[1],
                            SegmentType::Cutting { feed_rate: 100.0 },
                        )
                    })
                    .collect(),
            )],
            vec![],
        )
    }

    fn tool() -> Tool<f64> {
        Tool::ball_end_mill("BEM2".to_string(), 2.0, 6.0)
    }

    /// シミュレーションの成果物（キーフレームは最後の時点だけ）と、各時点の素材の状態（すべての時点にキーフレームを
    /// 置いた元の実行から求める）
    fn simulated() -> (CuttingSimulationArtifact, Vec<DexelGridStates>) {
        let stock = Aabb3D::new(Point3D::new(0.0, 0.0, -4.0), Point3D::new(10.0, 10.0, 0.0));
        let mut simulator = CuttingSimulator::new(
            stock,
            SnapshotInterval::ByDistance {
                interval_mm: 1.5,
                include_segment_endpoints: true,
            },
        );
        simulator.simulate(&toolpath(), &tool(), 0.005).unwrap();
        let artifact = simulator.artifact().unwrap();
        // 経路が短いため、キーフレームは最後の時点だけになる
        assert_eq!(
            artifact.keyframe_snapshots,
            vec![artifact.snapshots.len() - 1]
        );

        let model = StockModel::from_toolpath(stock, &toolpath(), &tool()).unwrap();
        let spacing = artifact.grids[2].pitch[0].max(artifact.grids[2].pitch[1]);
        let every = TriDexelModel::from_stock_model(&model, spacing, &artifact.snapshot_applied);
        let truth = CuttingSimulationArtifact {
            keyframe_snapshots: (0..artifact.snapshots.len()).collect(),
            keyframe_changes: every.keyframe_changes().clone(),
            ..artifact.clone()
        };
        let states = (0..artifact.snapshots.len())
            .map(|snapshot| truth.dexel_states_at_keyframe(snapshot))
            .collect();
        (artifact, states)
    }

    /// 成果物 `artifact` のキーフレームを、時点 `keyframes`（最後の時点を含む）に置き直したもの
    fn with_keyframes(
        artifact: &CuttingSimulationArtifact,
        keyframes: Vec<usize>,
    ) -> CuttingSimulationArtifact {
        let [x, y, z] = artifact.grids.map(|grid| grid.initial);
        let stock = Aabb3D::new(Point3D::new(x.0, y.0, z.0), Point3D::new(x.1, y.1, z.1));
        let model = StockModel::from_toolpath(stock, &toolpath(), &tool()).unwrap();
        let spacing = artifact.grids[2].pitch[0].max(artifact.grids[2].pitch[1]);
        let applied: Vec<usize> = keyframes
            .iter()
            .map(|&snapshot| artifact.snapshot_applied[snapshot])
            .collect();
        let dexels = TriDexelModel::from_stock_model(&model, spacing, &applied);
        CuttingSimulationArtifact {
            keyframe_snapshots: keyframes,
            keyframe_changes: dexels.keyframe_changes().clone(),
            ..artifact.clone()
        }
    }

    /// 戻るための記録を経路の距離 `undo_distance_mm` の分だけ残して時点を `order` の順に移り、各時点の状態と、
    /// 返した変化を当てた状態が元の実行と一致することを確かめる
    fn check_seeks(
        artifact: CuttingSimulationArtifact,
        states: &[DexelGridStates],
        order: &[usize],
        undo_distance_mm: f64,
    ) {
        let mut replay = CuttingSimulationReplay::new(artifact, &toolpath(), &tool()).unwrap();
        replay.undo_distance_mm = undo_distance_mm;
        assert_eq!(replay.states(), states[0]);
        // 返した変化だけを当てていく表示の側の状態
        let mut shown = states[0].clone();
        for &snapshot in order {
            for change in replay.seek(snapshot) {
                let (grid, index) = (change.grid as usize, change.index as usize);
                shown.intervals[grid][index] = change.intervals.to_vec();
                shown.normals[grid][index] = change.normals.to_vec();
            }
            assert_eq!(replay.snapshot(), snapshot);
            assert_eq!(replay.states(), states[snapshot], "snapshot {snapshot}");
            assert_eq!(shown, states[snapshot], "changes to snapshot {snapshot}");
        }
    }

    /// 先へ 1 つずつ進み、後ろへ 1 つずつ戻り、前後にばらばらに飛ぶ順
    fn seek_order(count: usize) -> Vec<usize> {
        let mut order: Vec<usize> = (1..count).collect();
        order.extend((0..count - 1).rev());
        order.extend([
            count - 1,
            0,
            count / 2,
            1,
            count - 2,
            count / 3,
            count / 3 + 1,
            2,
        ]);
        order
    }

    #[test]
    fn replayed_states_match_the_original_run_at_each_snapshot() {
        let (artifact, states) = simulated();
        let count = artifact.snapshots.len();
        assert!(count > 8, "{count}");
        let keyframes: Vec<usize> = (0..count)
            .filter(|s| s % 3 == 2 || *s == count - 1)
            .collect();
        let middle_keyframes = with_keyframes(&artifact, keyframes);
        // 戻るための記録が経路の全体に残る場合と、短い距離の場合（記録より前へ戻ると、キーフレームか初期の状態から作り直す）
        for undo_distance_mm in [f64::INFINITY, 3.0, 0.0] {
            // キーフレームが最後の時点だけの場合（初期の状態から作り直す）と、途中にもキーフレームがある場合
            for artifact in [&artifact, &middle_keyframes] {
                check_seeks(
                    artifact.clone(),
                    &states,
                    &seek_order(count),
                    undo_distance_mm,
                );
            }
        }
    }

    #[test]
    fn toolpaths_with_fewer_primitives_than_the_artifact_are_rejected() {
        let (artifact, _) = simulated();
        let short = ToolPath::new(
            "BEM2".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(
                0,
                0.0,
                vec![PathSegment::new_line(
                    Point3D::new(-1.0, 2.0, -0.5),
                    Point3D::new(11.0, 2.0, -0.5),
                    SegmentType::Cutting { feed_rate: 100.0 },
                )],
            )],
            vec![],
        );
        assert!(matches!(
            CuttingSimulationReplay::new(artifact, &short, &tool()),
            Err(SimulationError::PrimitiveCountMismatch { toolpath: 1, .. })
        ));
    }
}
