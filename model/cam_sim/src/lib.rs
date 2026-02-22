//! CAMシミュレーション実行クレート
//!
//! CAMドメインのシミュレーション実行責務を提供します。
//! 幾何計算カーネルは `geo_algorithms` に委譲します。

pub mod cutting_simulator;

pub use cutting_simulator::{CuttingSimulator, PathPosition, SimulationSnapshot, SnapshotInterval};
