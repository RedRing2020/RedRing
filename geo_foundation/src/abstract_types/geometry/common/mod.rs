//! 共通インターフェイスとヘルパー
//!
//! 幾何要素全般に適用される共通のトレイトと計算ロジック

// pub mod bbox_operations;    // 境界ボックス操作トレイト -> bbox.rsに移動
pub mod curve_analysis;
pub mod curve_operations; // 曲線の共通操作トレイト
pub mod direction_operations; // 方向ベクトル操作トレイト
pub mod line_operations; // 直線操作トレイト
pub mod point_operations; // 点操作トレイト
pub mod ray_operations; // レイ操作トレイト
pub mod vector_operations; // ベクトルの共通操作トレイト
                           // pub mod curve_analysis_factory_pattern; // 実装例（コンパイルエラーのため一時コメントアウト）

// 主要なトレイトを再エクスポート
// BBox関連トレイトは bbox.rs に移動しました
// pub use bbox_operations::{
//     BBoxOps, BBoxContainment, BBoxCollision, BBoxTransform,
// };

pub use curve_analysis::{
    AnalyticalCurve, CurveAnalysis2D, CurveAnalysis3D, CurveAnalysisHelper, CurveType,
    DifferentialGeometry, NumericalCurveAnalysis,
};

pub use curve_operations::{
    AngularCurve, CenteredCurve, CurveContainment, CurveMetrics, CurvePoints, CurveTransformation,
    CurveTypes, RadialCurve,
};

pub use direction_operations::{
    Direction2DOps, Direction3DConstants, Direction3DOps, DirectionConstants, DirectionOps,
};

pub use line_operations::{Line3DOps, LineIntersection, LineOps, LineTransform, SegmentOps};

pub use point_operations::{Point2DOps, Point3DOps, PointGeometry, PointOps, PointTransform};

pub use ray_operations::{RayIntersection, RayOps, RayPlaneIntersection, RaySphereIntersection};

pub use vector_operations::{
    Normalizable, Vector2DOperations, Vector3DOperations, VectorOperations,
};

// ファクトリパターンの例（参考実装）
// pub use curve_analysis_factory_pattern::{CurveAnalysisFactory, CurveGeometryData};
