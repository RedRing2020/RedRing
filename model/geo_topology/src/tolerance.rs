use geo_contracts::Scalar;

/// topology 判定で外部から与える公開設定。
///
/// 現段階では距離系を中心に公開し、局所の `bind` / `ideal` / `eval` 配分は
/// カーネル内部の内部解決 budget で扱う。将来 knit や Face / Shell 判定で責務が
/// 固まった場合に角度系設定を追加してよい。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyToleranceSettings<T: Scalar> {
    /// topology 判定全体の代表距離トレランス。
    pub distance_tolerance: T,

    /// Wire 以上で使う shared 連続性整合用の距離トレランス。
    pub shared_tolerance: T,
}

impl<T: Scalar> TopologyToleranceSettings<T> {
    /// 代表距離トレランスと shared 連続性整合トレランスから公開設定を生成する。
    pub fn new(distance_tolerance: T, shared_tolerance: T) -> Self {
        Self {
            distance_tolerance,
            shared_tolerance,
        }
    }

    /// 単一の代表距離トレランスから距離系公開設定を対称に初期化する。
    pub fn symmetric(distance_tolerance: T) -> Self {
        Self::new(distance_tolerance, distance_tolerance)
    }

    /// 代表距離トレランスから Edge 局所整合用の内部解決 budget を導出する。
    pub(crate) fn resolve_edge_tolerances(&self) -> ResolvedEdgeToleranceSettings<T> {
        ResolvedEdgeToleranceSettings::symmetric(self.distance_tolerance)
    }

    /// 代表トレランスから validator 用の内部解決 budget を導出する。
    pub(crate) fn resolve_validator_budget(&self) -> ResolvedTopologyToleranceBudget<T> {
        ResolvedTopologyToleranceBudget::new(self.resolve_edge_tolerances(), self.shared_tolerance)
    }
}

/// Edge 局所整合でのみ使う内部解決 budget。
///
/// これは公開 API ではなく、`TopologyToleranceSettings` から導出される内部表現とする。
/// 将来 validator や局所 override が必要になった場合は、この内部解決 budget 層を
/// 拡張して対応する。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedEdgeToleranceSettings<T: Scalar> {
    /// 拘束点と Vertex の整合に使う budget。
    pub bind_tolerance: T,

    /// ideal endpoint と拘束点の整合に使う budget。
    pub ideal_tolerance: T,

    /// evaluated endpoint と拘束点の整合に使う budget。
    pub eval_tolerance: T,
}

impl<T: Scalar> ResolvedEdgeToleranceSettings<T> {
    pub(crate) fn new(bind_tolerance: T, ideal_tolerance: T, eval_tolerance: T) -> Self {
        Self {
            bind_tolerance,
            ideal_tolerance,
            eval_tolerance,
        }
    }

    pub(crate) fn symmetric(distance_tolerance: T) -> Self {
        let third_tolerance = distance_tolerance / T::from_f64(3.0);
        Self::new(third_tolerance, third_tolerance, third_tolerance)
    }

    #[cfg(test)]
    pub(crate) fn local_budget(&self) -> T {
        self.bind_tolerance + self.ideal_tolerance + self.eval_tolerance
    }
}

/// validator で使う内部解決 budget。
///
/// 将来 `Face` / `Shell` / knit が追加された場合は、この型を拡張して扱う。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedTopologyToleranceBudget<T: Scalar> {
    pub edge: ResolvedEdgeToleranceSettings<T>,
    pub shared_tolerance: T,
}

impl<T: Scalar> ResolvedTopologyToleranceBudget<T> {
    pub(crate) fn new(edge: ResolvedEdgeToleranceSettings<T>, shared_tolerance: T) -> Self {
        Self {
            edge,
            shared_tolerance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ResolvedEdgeToleranceSettings, ResolvedTopologyToleranceBudget, TopologyToleranceSettings,
    };

    #[test]
    fn representative_distance_tolerance_resolves_local_budget_evenly() {
        let settings = TopologyToleranceSettings::new(0.3_f64, 1.0e-6);
        let resolved = settings.resolve_edge_tolerances();

        assert!((resolved.bind_tolerance - 0.1).abs() < 1.0e-12);
        assert!((resolved.ideal_tolerance - 0.1).abs() < 1.0e-12);
        assert!((resolved.eval_tolerance - 0.1).abs() < 1.0e-12);
        assert!((resolved.local_budget() - 0.3).abs() < 1.0e-12);
    }

    #[test]
    fn resolved_edge_tolerance_can_still_be_overridden_internally() {
        let resolved = ResolvedEdgeToleranceSettings::new(0.01_f64, 0.15, 0.14);

        assert!((resolved.local_budget() - 0.3).abs() < 1.0e-12);
    }

    #[test]
    fn topology_tolerance_keeps_shared_budget_separate() {
        let settings = TopologyToleranceSettings::new(0.3_f64, 1.0e-6);

        assert!((settings.distance_tolerance - 0.3).abs() < 1.0e-12);
        assert!((settings.shared_tolerance - 1.0e-6).abs() < 1.0e-12);
    }

    #[test]
    fn validator_budget_uses_edge_and_shared_components() {
        let settings = TopologyToleranceSettings::new(0.3_f64, 1.0e-6);
        let resolved = settings.resolve_validator_budget();

        assert_eq!(
            resolved,
            ResolvedTopologyToleranceBudget::new(
                ResolvedEdgeToleranceSettings::new(0.1, 0.1, 0.1),
                1.0e-6,
            )
        );
    }
}
