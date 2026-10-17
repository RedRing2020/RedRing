//! 形状をまたぐ演算（距離・衝突判定・交差）の trait定義
//!
//! 形状の組み合わせごとに trait を実装する。代表の向き（A-B）を実装し、逆の向き（B-A）は代表の向きを呼ぶ。
//! 実装する組み合わせは `dev/architecture/CROSS_OPERATION_COVERAGE_MATRIX_SPEC.md` の網羅表で管理する。
//!
//! 形状の種類（`geo_primitives` / `geo_nurbs`）をまたいで実装できるよう、trait は `geo_algorithms` に置く。
//! 1 つの形状と点の距離は、各形状の trait定義（`*Distance::distance_to_point`）で扱う。
//!
//! 曲線（線分・円・楕円等）は曲線、領域・立体（三角形・球等）は領域として扱う
//! （`GEOMETRY_SHAPE_SEMANTICS_DESIGN.md` の点との距離の規約と同じ）。

mod impls_2d;
mod impls_3d;

use crate::IntersectionResult;
use geo_contracts::Scalar;

/// 相手の形状との最短距離
pub trait CrossDistance<T: Scalar, Other> {
    /// 相手の形状との最短距離（重なる場合は 0）
    fn distance_to(&self, other: &Other) -> T;
}

/// 相手の形状との衝突判定
pub trait CrossCollision<T: Scalar, Other> {
    /// 相手の形状との距離が `tolerance` 以内か
    fn collides(&self, other: &Other, tolerance: T) -> bool;
}

/// 相手の形状との交差
pub trait CrossIntersection<T: Scalar, Other> {
    /// 相手の形状との交差（交点・重なり）
    fn intersection(&self, other: &Other, tolerance: T) -> IntersectionResult<T>;
}
