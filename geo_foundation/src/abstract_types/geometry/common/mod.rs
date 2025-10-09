//! 共通インターフェイスとヘルパー
//!
//! 幾何要素全般に適用される共通のトレイトと計算ロジック

pub mod curve_analysis;
pub mod curve_operations; // 曲線の共通操作トレイト
pub mod vector_operations; // ベクトルの共通操作トレイト
                           // pub mod curve_analysis_factory_pattern; // 実装例（コンパイルエラーのため一時コメントアウト）

// 主要なトレイトを再エクスポート
pub use curve_analysis::{
    AnalyticalCurve, CurveAnalysis2D, CurveAnalysis3D, CurveAnalysisHelper, CurveType,
    DifferentialGeometry, NumericalCurveAnalysis,
};

pub use curve_operations::{
    AngularCurve, CenteredCurve, CurveContainment, CurveMetrics, CurvePoints, CurveTransformation,
    CurveTypes, RadialCurve,
};

pub use vector_operations::{
    Normalizable, Vector2DOperations, Vector3DOperations, VectorOperations,
};

// ファクトリパターンの例（参考実装）
// pub use curve_analysis_factory_pattern::{CurveAnalysisFactory, CurveGeometryData};
