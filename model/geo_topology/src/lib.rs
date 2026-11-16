//! geo_topology - 幾何トポロジー基盤クレート
//!
//! 交差判定・モデリング・形状の読み込みで利用するトポロジー構造を提供する。
//! 現在の提供:
//! - Brep: 頂点・稜線・ループ・面の表（設計: BREP_AND_STEP_IMPORT_DESIGN.md 5 章）
//! - CurveRef: 稜線が参照する母曲線
//! - CompositeCurve3D: 拘束端点で連結された複合曲線
//! - CurveSegment3D: 個別セグメント（Line / Arc / Nurbs、ideal/constraint endpoint を区別）

pub mod brep;
pub mod composite_curve;
pub mod curve_ref;
pub mod tolerance;
pub mod topology_validator;

pub use brep::{
    Brep, BrepEdge, BrepError, BrepFace, BrepLoop, BrepVertex, EdgeIndex, EdgeUse, EdgeUseLocation,
    FaceIndex, LoopIndex, SurfaceRef, VertexIndex,
};
pub use composite_curve::{CompositeCurve3D, CurveSegment3D};
pub use curve_ref::CurveRef;
pub use geo_core::{Point3D, Vector3D};
pub use tolerance::TopologyToleranceSettings;
pub use topology_validator::{BrepValidationReport, EdgeValidationReport, TopologyValidator};

pub type TopoArc3D<T> = geo_primitives::Arc3D<T>;
pub type TopoEllipseArc3D<T> = geo_primitives::EllipseArc3D<T>;
pub type TopoInfiniteLine3D<T> = geo_primitives::InfiniteLine3D<T>;
pub type TopoLineSegment3D<T> = geo_primitives::LineSegment3D<T>;
pub type TopoNurbsCurve3D<T> = geo_nurbs::NurbsCurve3D<T>;
pub type TopoPlane3D<T> = geo_primitives::Plane3D<T>;
