//! 切削シミュレーションの再生の問い合わせ
//!
//! 取り込んだツールパスと切削シミュレーションの成果物を読み、時点の位置・ツールパスの直線の区間と、時点の素材の面を
//! 返す。成果物のバイナリ形式は外に見せない。
//!
//! 時点を移すと、区間が変わった部分のブロックの面だけを作り直す。最後の時点では、素材の面の三角形を、製品形状との
//! 比較（削り込み・トレランスの中・削り残し・比べていない）で分けて作れる。

use std::fmt;

use cam_core::{Tool, read_toolpath_artifact_v1};
use cam_sim::{
    CuttingSimulationArtifact, CuttingSimulationReplay, DexelSurface, SurfaceDeviations,
    collect_toolpath_line_segments, read_cutting_simulation_artifact_v1,
};
pub use cam_sim::{DeviationClass, SimulationSnapshotExport, SurfaceMesh};
use geo_algorithms::{DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM, LineSegment3D};

use crate::cam_simulation_workflow::CamSimulationWorkflowResult;
use crate::toolpath::ToolPath;

/// 切削シミュレーションを再生できない理由
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CuttingSimulationPlaybackError {
    /// 成果物を読めない（理由）
    Artifact(String),
    /// スナップショットがない
    NoSnapshot,
    /// 成果物をツールパスと工具で再生できない（理由）
    Replay(String),
}

impl fmt::Display for CuttingSimulationPlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(reason) => write!(f, "failed to read the artifact: {reason}"),
            Self::NoSnapshot => write!(f, "the cutting simulation has no snapshot"),
            Self::Replay(reason) => write!(f, "failed to replay the cutting simulation: {reason}"),
        }
    }
}

impl std::error::Error for CuttingSimulationPlaybackError {}

/// 取り込んだ切削シミュレーションの再生
#[derive(Debug)]
pub struct CuttingSimulationPlayback {
    /// ツールパス
    pub toolpath: ToolPath<f64>,
    /// 切削シミュレーションと同じ分け方の、ツールパスの直線の区間と、切削の区間か（時点の区間の番号の対象）
    pub segments: Vec<(LineSegment3D<f64>, bool)>,
    /// 各時点の位置と統計（経路の上の位置の昇順）
    pub snapshots: Vec<SimulationSnapshotExport>,
    /// 最初の素材の上面の高さ
    pub stock_top: f64,
    /// 時点の素材の面
    pub stock: StockPlayback,
}

impl CuttingSimulationPlayback {
    /// 取り込んだ結果 `result` の成果物を読み、最初の時点の再生にする
    ///
    /// # Errors
    ///
    /// 成果物を読めない場合・スナップショットがない場合・成果物をツールパスと工具で再生できない場合。
    pub fn open(
        result: &CamSimulationWorkflowResult,
    ) -> Result<Self, CuttingSimulationPlaybackError> {
        Self::from_artifacts(
            &result.toolpath_artifact,
            &result.simulation_artifact,
            &result.simulation_input.tool,
            result.simulation_input.precision,
        )
    }

    /// ツールパスと切削シミュレーションの成果物と工具と素材の精度から、最初の時点の再生にする
    fn from_artifacts(
        toolpath_artifact: &[u8],
        simulation_artifact: &[u8],
        tool: &Tool<f64>,
        precision: f64,
    ) -> Result<Self, CuttingSimulationPlaybackError> {
        let artifact_error = |error: cam_core::BinaryFormatError| {
            CuttingSimulationPlaybackError::Artifact(error.to_string())
        };
        let (_, toolpath) =
            read_toolpath_artifact_v1(&mut &toolpath_artifact[..]).map_err(artifact_error)?;
        let (_, artifact) = read_cutting_simulation_artifact_v1(&mut &simulation_artifact[..])
            .map_err(artifact_error)?;
        if artifact.snapshots.is_empty() {
            return Err(CuttingSimulationPlaybackError::NoSnapshot);
        }
        let segments =
            collect_toolpath_line_segments(&toolpath, DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM);
        let snapshots = artifact.snapshots.clone();
        let stock_top = artifact.grids[2].initial.1;
        let stock = StockPlayback::new(artifact, &toolpath, tool, precision)?;
        Ok(Self {
            toolpath,
            segments,
            snapshots,
            stock_top,
            stock,
        })
    }
}

/// 時点の素材の面（今の時点の素材の状態と面を持ち、時点を移すと変わったブロックの面を作る）
#[derive(Debug)]
pub struct StockPlayback {
    /// 成果物の再生（今の時点の素材の状態を持つ）
    replay: CuttingSimulationReplay,
    surface: DexelSurface,
    /// 今の面を、製品形状との比較で分けて作ったか
    compared: bool,
}

impl StockPlayback {
    fn new(
        artifact: CuttingSimulationArtifact,
        toolpath: &ToolPath<f64>,
        tool: &Tool<f64>,
        precision: f64,
    ) -> Result<Self, CuttingSimulationPlaybackError> {
        let replay = CuttingSimulationReplay::new(artifact, toolpath, tool)
            .map_err(|error| CuttingSimulationPlaybackError::Replay(error.to_string()))?;
        let surface = DexelSurface::new(
            replay.artifact().grids,
            replay.states(),
            precision,
            tool.radius(),
        );
        Ok(Self {
            replay,
            surface,
            compared: false,
        })
    }

    /// 時点の数
    pub fn frame_count(&self) -> usize {
        self.replay.artifact().snapshots.len()
    }

    /// 今の時点の、すべてのブロックの素材の面（ブロックの番号と面。三角形のないブロックは含めない）
    pub fn stock_blocks(&self) -> Vec<([usize; 3], SurfaceMesh)> {
        let deviations = self.deviations();
        self.block_surfaces(&self.all_blocks(), deviations.as_ref())
            .into_iter()
            .filter(|(_, mesh)| !mesh.indices.is_empty())
            .collect()
    }

    /// 時点 `frame` へ移し、移った時点と、面が変わりうるブロックの移った後の素材の面を返す（範囲外は最後の時点にする）
    ///
    /// 区間が変わった部分のブロックだけを作り直す。最後の時点で `compare` の場合は、成果物に製品形状との比較があれば
    /// 三角形を分ける。分けるかが前の面と変わった場合は、すべてのブロックを作り直す。三角形がなくなったブロックは、
    /// 三角形のない面として返す。
    pub fn seek(&mut self, frame: usize, compare: bool) -> (usize, Vec<([usize; 3], SurfaceMesh)>) {
        let last = self.frame_count() - 1;
        let target = frame.min(last);
        let mut blocks = self.surface.apply_changes(self.replay.seek(target));
        let deviations = (compare && target == last)
            .then(|| self.comparable_deviations())
            .flatten();
        if deviations.is_some() != self.compared {
            blocks = self.all_blocks();
        }
        let meshes = self.block_surfaces(&blocks, deviations.as_ref());
        self.compared = deviations.is_some();
        (target, meshes)
    }

    /// すべてのブロック
    fn all_blocks(&self) -> Vec<[usize; 3]> {
        let [bx, by, bz] = self.surface.block_counts();
        (0..bx)
            .flat_map(|x| (0..by).flat_map(move |y| (0..bz).map(move |z| [x, y, z])))
            .collect()
    }

    /// 今の面を分ける場合の、今の区間の端点ごとの製品形状までの距離
    fn deviations(&self) -> Option<SurfaceDeviations<'_>> {
        if !self.compared {
            return None;
        }
        self.comparable_deviations()
    }

    /// 成果物の製品形状との比較を、今の区間の端点ごとの距離にする（比較がないか、端点の数が違えば `None`）
    fn comparable_deviations(&self) -> Option<SurfaceDeviations<'_>> {
        let deviations = self.replay.artifact().product_deviations.as_ref()?;
        let comparable = self.surface.deviations(deviations);
        if comparable.is_none() {
            tracing::warn!(
                "The stock faces are not colored because the number of comparison endpoints differs from the stock segments"
            );
        }
        comparable
    }

    /// ブロックの列 `blocks` の素材の面（複数のコアで並べて作る。`deviations` があれば三角形を分ける）
    fn block_surfaces(
        &self,
        blocks: &[[usize; 3]],
        deviations: Option<&SurfaceDeviations>,
    ) -> Vec<([usize; 3], SurfaceMesh)> {
        let meshes = match deviations {
            Some(deviations) => self.surface.compared_mesh_blocks(blocks, deviations),
            None => self.surface.mesh_blocks(blocks),
        };
        blocks.iter().copied().zip(meshes).collect()
    }
}

#[cfg(test)]
mod tests;
