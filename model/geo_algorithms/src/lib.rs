//! RedRing の幾何アルゴリズム層
//!
//! このクレートは、衝突判定・交点計算・空間分割などの幾何アルゴリズムを提供する
//! 設計方針と全体構成は `dev/architecture/ARCHITECTURE.md` を参照

pub mod collision;
pub mod intersection;
pub mod octree;
pub mod tolerance;

// Octree関連の公開API
pub use octree::{Octree, OctreeNode, OctreeTolerance};

// NURBS型の再エクスポート（ViewModel層からのアクセス用）
pub use geo_contracts::Scalar;
pub use geo_nurbs::adaptive_tessellation;
pub use geo_nurbs::{NurbsCurve3D, NurbsSurface3D};

// geo_primitives の基本型を再エクスポート（ViewModel層がgeo_primitivesに直接依存しないように）
// 基本ポイント・ベクトル型（geo_coreから）
pub use geo_core::{Aabb3D, Point2D, Point3D, Vector2D, Vector3D};
// 基本型（geo_primitivesから）
pub use geo_primitives::{Angle, Direction3D};

// 基本形状
pub use geo_primitives::{
    Arc2D, Arc3D, Circle2D, Circle3D, ConicalSolid3D, ConicalSurface3D, CylindricalSolid3D,
    CylindricalSurface3D, Ellipse2D, Ellipse3D, EllipseArc3D, EllipsoidalSolid3D,
    EllipsoidalSurface3D, InfiniteLine3D, LineSegment2D, LineSegment3D, Plane3D, Ray2D, Ray3D,
    SphericalSolid3D, SphericalSurface3D, TorusSolid3D, TorusSurface3D, Triangle2D, Triangle3D,
    TriangleMesh3D,
};
