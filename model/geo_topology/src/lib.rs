//! geo_topology - 幾何トポロジー基盤クレート
//!
//! 交差判定・モデリング・形状の読み込みで利用するトポロジー構造を提供する。
//! 現在の提供:
//! - Brep: 頂点・稜線・ループ・面・シェル・立体の表（設計: BREP_AND_STEP_IMPORT_DESIGN.md 5 章）
//! - CurveRef: 稜線が参照する母曲線
//! - SurfaceRef: 面が載る曲面
//! - ParameterCurve: 稜線の使用が持つ、面の曲面の上のパラメータ曲線
//! - CompositeCurve3D: 拘束端点で連結された複合曲線
//! - CurveSegment3D: 個別セグメント（Line / Arc / Nurbs、ideal/constraint endpoint を区別）

pub mod brep;
pub mod composite_curve;
pub mod curve_ref;
pub mod parameter_curve;
pub mod surface_ref;
pub mod tolerance;
pub mod topology_validator;

pub use brep::{
    Brep, BrepEdge, BrepError, BrepFace, BrepLoop, BrepShell, BrepSolid, BrepVertex, EdgeIndex,
    EdgeUse, EdgeUseLocation, FaceIndex, LoopIndex, ShellIndex, SolidIndex, VertexIndex,
};
pub use composite_curve::{CompositeCurve3D, CurveSegment3D};
pub use curve_ref::CurveRef;
pub use geo_core::{Point2D, Point3D, Vector2D, Vector3D};
pub use parameter_curve::ParameterCurve;
pub use surface_ref::SurfaceRef;
pub use tolerance::TopologyToleranceSettings;
pub use topology_validator::BrepValidationReport;

pub type TopoArc3D<T> = geo_primitives::Arc3D<T>;
pub type TopoCircle3D<T> = geo_primitives::Circle3D<T>;
pub type TopoConicalSurface3D<T> = geo_primitives::ConicalSurface3D<T>;
pub type TopoCylindricalSurface3D<T> = geo_primitives::CylindricalSurface3D<T>;
pub type TopoEllipse3D<T> = geo_primitives::Ellipse3D<T>;
pub type TopoInfiniteLine3D<T> = geo_primitives::InfiniteLine3D<T>;
pub type TopoLineSegment3D<T> = geo_primitives::LineSegment3D<T>;
pub type TopoNurbsCurve2D<T> = geo_nurbs::NurbsCurve2D<T>;
pub type TopoNurbsCurve3D<T> = geo_nurbs::NurbsCurve3D<T>;
pub type TopoNurbsSurface3D<T> = geo_nurbs::NurbsSurface3D<T>;
pub type TopoPlane3D<T> = geo_primitives::Plane3D<T>;
pub type TopoSphericalSurface3D<T> = geo_primitives::SphericalSurface3D<T>;
pub type TopoTorusSurface3D<T> = geo_primitives::TorusSurface3D<T>;
