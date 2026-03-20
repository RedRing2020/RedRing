/// geo_algorithms 内の互換トレランスコンテキスト。
///
/// 旧アルゴリズム実装が参照していた `ToleranceContext` を
/// 外部基盤クレート非依存で維持するための最小定義。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToleranceContext {
    pub linear: f64,
    pub angular: f64,
    pub parametric: f64,
    pub curvature: f64,
}

impl ToleranceContext {
    pub fn new(linear: f64, angular: f64, parametric: f64, curvature: f64) -> Self {
        Self {
            linear,
            angular,
            parametric,
            curvature,
        }
    }

    pub fn precision() -> Self {
        Self::new(1e-12, 1e-10, 1e-10, 1e-10)
    }

    pub fn standard() -> Self {
        Self::new(1e-6, 1e-4, 1e-6, 1e-3)
    }

    pub fn relaxed() -> Self {
        Self::new(1e-3, 1e-2, 1e-3, 1e-3)
    }
}

impl Default for ToleranceContext {
    fn default() -> Self {
        Self::standard()
    }
}
