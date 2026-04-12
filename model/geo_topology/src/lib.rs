//! geo_topology - 幾何トポロジー基盤クレート
//!
//! 交差判定・モデリングで利用する最小のトポロジー構造を提供する。
//! 現在の提供:
//! - CompositeCurve3D: 拘束端点で連結された複合曲線
//! - CurveSegment3D: 個別セグメント（Line / Arc / Nurbs、ideal/constraint endpoint を区別）
//!
//! 将来拡張（#205 連携）:
//! - Vertex: 点トポロジー
//! - Edge: 曲線トポロジー
//! - Face: 面トポロジー
//! - HalfEdge: 隣接情報を持つ有向エッジ

pub mod composite_curve;
pub mod tolerance;
pub mod topology_core;
pub mod topology_validator;
pub mod wire;

pub use composite_curve::{CompositeCurve3D, CurveSegment3D};
pub use geo_core::{Point3D, Vector3D};
pub use tolerance::TopologyToleranceSettings;
pub use topology_core::{CurveRef, Edge, TopoId, Vertex};
pub use topology_validator::{EdgeValidationReport, TopologyValidator, WireValidationReport};
pub use wire::Wire;

pub(crate) const TOPO_NURBS_LENGTH_SUBDIVISIONS: usize =
    geo_nurbs::constants::CURVE_LENGTH_SUBDIVISIONS;

pub type TopoArc3D<T> = geo_primitives::Arc3D<T>;
pub type TopoEllipseArc3D<T> = geo_primitives::EllipseArc3D<T>;
pub type TopoInfiniteLine3D<T> = geo_primitives::InfiniteLine3D<T>;
pub type TopoLineSegment3D<T> = geo_primitives::LineSegment3D<T>;
pub type TopoNurbsCurve3D<T> = geo_nurbs::NurbsCurve3D<T>;
