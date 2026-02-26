//! AppState の基盤設定（単位系・トレランス）に関する処理を扱うモジュール。

use super::AppState;

impl AppState {
    /// 現在の単位でのトレランス値を取得
    ///
    /// # Examples
    ///
    /// 単位系がミリメートル、トレランスが0.01mmの場合 → 0.01
    /// 単位系がメートル、トレランスが0.01mmの場合 → 0.00001
    pub fn tolerance_in_current_unit(&self) -> f64 {
        self.display_tolerance.in_unit(self.unit_system)
    }
}
