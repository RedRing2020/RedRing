//! geo_foundation commons - 共通計算トレイト
//!
//! geo_commons クレートで使用される共通計算トレイトを定義します。
//! Foundation Pattern において、具体的な計算実装とトレイト定義を分離する役割を担います。

pub mod ellipse_calculation_traits;

// 便利な再エクスポート
pub use ellipse_calculation_traits::{
    EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
};
