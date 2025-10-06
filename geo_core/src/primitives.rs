/// 幾何プリミティブモジュール
///
/// 2D/3D幾何要素の統合インターフェース

// 主要な型の再エクスポート
pub use crate::primitives2d::{
    Point2D, LineSegment2D, Arc2D, Polygon2D,
    ParametricCurve2D,
};
pub use crate::point3d::Point3D; // always-on 3D point

// 3D primitives are only available with legacy feature
#[cfg(feature = "legacy-primitives3d")]
pub use crate::primitives3d::{
    LineSegment3D, Plane, Sphere,
    ParametricCurve3D, ParametricSurface,
};

// Provide Point3D always (needed by downstream), duplicating minimal struct when legacy feature off.
// (Point3D is always exported from point3d module now)

// 後方互換性のために、3Dトレイトを再エクスポート
#[cfg(feature = "legacy-primitives3d")]
pub use ParametricCurve3D as ParametricCurve;
