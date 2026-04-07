//! RedRing の幾何アルゴリズム層
//!
//! このクレートは、衝突判定・交点計算・空間分割などの幾何アルゴリズムを提供する
//! 設計方針と全体構成は `dev/architecture/ARCHITECTURE.md` を参照
//!
//! endpoint semantics に関する前提:
//! - `LineSegment2D` / `LineSegment3D` は bounded geometry の ideal endpoint 基準
//! - `CompositeCurve3D` / `CurveSegment3D` は `geo_topology` の語彙をそのまま再公開する
//!   ため、whole-curve の public endpoint は拘束端点基準

pub mod angle_utils;
pub mod collision;
pub mod constraint_validation;
pub mod curve_discretization;
pub mod distance;
pub mod intersection;
pub mod nurbs_fixtures;
pub mod octree;
pub mod result;

// Octree関連の公開API
pub use octree::{Octree, OctreeNode, OctreeTolerance};

pub use angle_utils::{
    are_angles_equivalent_deg, is_equivalent_0_360, normalize_angle_deg,
    normalize_angle_signed_deg, normalize_to_0_360, normalize_to_minus180_180, rewound_target_deg,
    shortest_angle, shortest_angular_delta_deg, unwind_angles_deg, AngularPosition, RewindPolicy,
};
pub use constraint_validation::{
    validate_acceleration, validate_feed_rate, validate_linear_acceleration_mm_per_s2,
    validate_linear_speed_mm_per_min, validate_linear_travel, validate_linear_travel_mm,
    validate_rotary_acceleration_deg_per_s2, validate_rotary_angle, validate_rotary_angle_deg,
    validate_rotary_speed_deg_per_min, ConstraintViolation, ValidationResult,
};
pub use curve_discretization::{
    circular_arc_to_polyline, CircularArcDirection, CircularArcPolylineOptions,
    DEFAULT_CIRCULAR_ARC_CHORD_TOLERANCE_MM,
};

// geo_algorithms が提供する交差結果型
pub use result::{IntersectionGeometry, IntersectionResult, IntersectionTopology};

// NURBS型の再エクスポート（ViewModel層からのアクセス用）
pub use geo_contracts::Scalar;
pub use geo_nurbs::adaptive_tessellation;
pub use geo_nurbs::{NurbsCurve3D, NurbsSurface3D};

// Topology types from geo_topology
// `CompositeCurve3D` の public endpoint は拘束端点基準であることに注意。
pub use geo_topology::{CompositeCurve3D, CurveSegment3D};

// geo_primitives の基本型を再エクスポート（ViewModel層がgeo_primitivesに直接依存しないように）
// 基本ポイント・ベクトル型（geo_coreから）
pub use geo_core::{Aabb3D, Point2D, Point3D, Vector2D, Vector3D};
// 基本型（geo_primitivesから）
pub use geo_primitives::{Angle, Direction3D};

// 基本形状
pub use geo_primitives::{
    Arc2D, Arc3D, Circle2D, Circle3D, ConicalSolid3D, ConicalSurface3D, CylindricalSolid3D,
    CylindricalSurface3D, Ellipse2D, Ellipse3D, EllipseArc2D, EllipseArc3D, EllipsoidalSolid3D,
    EllipsoidalSurface3D, InfiniteLine2D, InfiniteLine3D, LineSegment2D, LineSegment3D, Plane3D,
    Ray2D, Ray3D, SphericalSolid3D, SphericalSurface3D, TorusSolid3D, TorusSurface3D, Triangle2D,
    Triangle3D, TriangleMesh3D,
};
