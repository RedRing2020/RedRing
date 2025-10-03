/// 幾何プリミティブモジュール
///
/// 2D/3D幾何要素の統合インターフェース

// 主要な型の再エクスポート
pub use crate::primitives2d::{
    Point2D, LineSegment2D, Arc2D, Polygon2D,
    ParametricCurve2D,
};

pub use crate::primitives3d::{
    Point3D, LineSegment3D, Plane, Sphere,
    ParametricCurve3D, ParametricSurface,
};

// 後方互換性のために、3Dトレイトを再エクスポート
pub use ParametricCurve3D as ParametricCurve;
