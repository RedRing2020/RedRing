//! CAMシミュレーション実行クレート
//!
//! CAMドメインのシミュレーション実行責務を提供します。
//! 幾何計算カーネルは `geo_algorithms` に委譲します。
//! Phase 1a では 3軸固定・フラットエンドミル限定の最小実装を提供します。

mod error;
mod simulator;

/// シミュレーション実行時のエラー型。
pub use error::SimulationError;
/// シミュレーション実行APIとスナップショット関連型。
pub use simulator::{
	CuttingSimulator, PathPosition, SimulationSnapshot, SimulationSnapshotExport, SnapshotInterval,
};

/// 旧参照名との互換公開（段階移行用）。
pub mod cutting_simulator {
	pub use crate::{
		CuttingSimulator, PathPosition, SimulationError, SimulationSnapshot, SimulationSnapshotExport,
		SnapshotInterval,
	};
}

#[cfg(test)]
mod tests;
