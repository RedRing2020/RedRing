//! CuttingSimulation ジョブの `InputRef` → 切削シミュレーションの入力の解決境界
//!
//! ツールパスは親の CamProcess ジョブの成果物から読み、素材・工具・精度の設定は
//! 本境界で `input://sim/<name>` の参照から解決する。

use std::collections::HashMap;
use std::sync::RwLock;

use cam_core::Tool;
use geo_algorithms::Aabb3D;
use job_runtime::RefValidationError;

use crate::SnapshotInterval;
use crate::solver_input::validate_input_ref;

/// CuttingSimulation の入力参照ドメイン
pub(crate) const SIM_INPUT_DOMAIN: &str = "sim";

/// 切削シミュレーションの入力
///
/// 長さはすべてツールパスと同じ座標系・長さの単位で表す。
#[derive(Debug, Clone)]
pub struct CuttingSimulationInput {
    /// 素材（直方体）
    pub stock: Aabb3D<f64>,
    /// 工具（ツールパスの `tool_id` と同じ ID を持つ）
    pub tool: Tool<f64>,
    /// 素材の形状の許容誤差
    pub precision: f64,
    /// スナップショットの記録の間隔
    pub snapshot_interval: SnapshotInterval,
}

/// `InputRef` から切削シミュレーションの入力を解決する境界
pub trait CuttingSimulationInputProvider: Send + Sync {
    /// 参照先の入力を返す。未登録の場合は `None`。
    fn resolve(&self, input_ref: &str) -> Option<CuttingSimulationInput>;
}

/// プロセス内メモリで切削シミュレーションの入力を保持する provider
#[derive(Debug, Default)]
pub struct InMemoryCuttingSimulationInputStore {
    inputs: RwLock<HashMap<String, CuttingSimulationInput>>,
}

impl InMemoryCuttingSimulationInputStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// `input://sim/...` 形式の参照に入力を登録する。同一参照は上書きする。
    pub fn register(
        &self,
        input_ref: &str,
        input: CuttingSimulationInput,
    ) -> Result<(), RefValidationError> {
        validate_sim_input_ref(input_ref)?;
        self.inputs
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(input_ref.trim().to_string(), input);
        Ok(())
    }
}

impl CuttingSimulationInputProvider for InMemoryCuttingSimulationInputStore {
    fn resolve(&self, input_ref: &str) -> Option<CuttingSimulationInput> {
        self.inputs
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(input_ref.trim())
            .cloned()
    }
}

/// `input://sim/<name>...` 形式であることを検証する。
pub(crate) fn validate_sim_input_ref(input_ref: &str) -> Result<(), RefValidationError> {
    validate_input_ref(input_ref, SIM_INPUT_DOMAIN)
}
