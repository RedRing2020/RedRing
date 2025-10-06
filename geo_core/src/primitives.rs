/// 幾何プリミティブモジュール
///
/// 2D/3D幾何要素の統合インターフェース

// 主要な型の再エクスポート
pub use crate::primitives2d::{
    Point2D, LineSegment2D, Arc2D, Polygon2D,
    ParametricCurve2D,
};
pub use crate::point3d::Point3D; // always-on 3D point

// 3D primitives removed - use geo_primitives instead

// Point3D is always available (needed by downstream)
// Other 3D primitives have been moved to geo_primitives
