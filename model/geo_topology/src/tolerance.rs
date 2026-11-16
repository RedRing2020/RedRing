use geo_contracts::Scalar;

/// topology 判定で外部から与える公開設定。
///
/// 現段階では距離系だけを公開し、稜線の局所の `bind` / `ideal` / `eval` 配分は
/// カーネル内部の内部解決 budget で扱う。頂点はループの中で番号で共有するため、
/// 頂点どうしの離れを測る距離は持たない。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologyToleranceSettings<T: Scalar> {
    /// topology 判定全体の代表距離トレランス。
    pub distance_tolerance: T,
}

impl<T: Scalar> TopologyToleranceSettings<T> {
    /// 代表距離トレランスから公開設定を生成する。
    pub fn new(distance_tolerance: T) -> Self {
        Self { distance_tolerance }
    }

    /// 代表距離トレランスから稜線の局所整合用の内部解決 budget を導出する。
    pub(crate) fn resolve_edge_tolerances(&self) -> ResolvedEdgeToleranceSettings<T> {
        ResolvedEdgeToleranceSettings::symmetric(self.distance_tolerance)
    }
}

/// 稜線の局所整合でのみ使う内部解決 budget。
///
/// これは公開 API ではなく、`TopologyToleranceSettings` から導出される内部表現とする。
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ResolvedEdgeToleranceSettings<T: Scalar> {
    /// 拘束点と頂点の整合に使う budget。
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

#[cfg(test)]
mod tests {
    use super::{ResolvedEdgeToleranceSettings, TopologyToleranceSettings};

    #[test]
    fn representative_distance_tolerance_resolves_local_budget_evenly() {
        let settings = TopologyToleranceSettings::new(0.3_f64);
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
}
