//! CuttingSimulation ジョブの `InputRef` → 切削シミュレーションの入力の解決境界
//!
//! ツールパスは親の CamProcess ジョブの成果物から読み、素材・工具・精度の設定は
//! 本境界で `input://sim/<name>` の参照から解決する。

use std::collections::HashMap;
use std::sync::RwLock;

use cam_core::Tool;
use geo_algorithms::{Aabb3D, Point3D};
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

/// 製品形状を囲む箱から素材の箱の各面までの余裕（既定はすべて 0。製品形状を囲む箱そのもの）
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct StockMargins {
    /// −X 側
    pub minus_x: f64,
    /// +X 側
    pub plus_x: f64,
    /// −Y 側
    pub minus_y: f64,
    /// +Y 側
    pub plus_y: f64,
    /// 底（−Z 側）
    pub bottom: f64,
    /// 上（+Z 側）
    pub top: f64,
}

/// 製品形状を囲む境界ボックス `bounds` の各面の外側に余裕 `margins` を足して、素材（直方体）を作る
///
/// 余裕のいずれかが負または有限でない場合は `None`。
pub fn stock_around_product(bounds: Aabb3D<f64>, margins: StockMargins) -> Option<Aabb3D<f64>> {
    let StockMargins {
        minus_x,
        plus_x,
        minus_y,
        plus_y,
        bottom,
        top,
    } = margins;
    if ![minus_x, plus_x, minus_y, plus_y, bottom, top]
        .iter()
        .all(|margin| margin.is_finite() && *margin >= 0.0)
    {
        return None;
    }
    let (min, max) = (bounds.min(), bounds.max());
    Some(Aabb3D::new(
        Point3D::new(min.x() - minus_x, min.y() - minus_y, min.z() - bottom),
        Point3D::new(max.x() + plus_x, max.y() + plus_y, max.z() + top),
    ))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds() -> Aabb3D<f64> {
        Aabb3D::new(Point3D::new(-10.0, 0.0, 2.0), Point3D::new(10.0, 20.0, 8.0))
    }

    #[test]
    fn stock_adds_each_margin_outside_the_matching_face_of_the_product_bounds() {
        let margins = StockMargins {
            minus_x: 1.0,
            plus_x: 2.0,
            minus_y: 3.0,
            plus_y: 4.0,
            bottom: 5.0,
            top: 6.0,
        };
        let stock = stock_around_product(bounds(), margins).unwrap();
        assert_eq!(stock.min(), Point3D::new(-11.0, -3.0, -3.0));
        assert_eq!(stock.max(), Point3D::new(12.0, 24.0, 14.0));
    }

    #[test]
    fn stock_is_the_product_bounds_by_default() {
        let stock = stock_around_product(bounds(), StockMargins::default()).unwrap();
        assert_eq!(stock.min(), bounds().min());
        assert_eq!(stock.max(), bounds().max());
    }

    #[test]
    fn negative_or_non_finite_margins_make_no_stock() {
        for margins in [
            StockMargins {
                plus_y: -0.5,
                ..StockMargins::default()
            },
            StockMargins {
                top: f64::NAN,
                ..StockMargins::default()
            },
        ] {
            assert_eq!(stock_around_product(bounds(), margins), None);
        }
    }
}
