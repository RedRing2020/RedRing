//! 幾何 relation のtrait定義
//!
//! 平行・垂直・同一直線・向き一致のような、相手との関係を返す契約を集約する。

use crate::Scalar;

/// 対象を包含するかを返す汎用 relation
///
/// 命名規約:
/// - 統一名の `contains(...)` は、呼び出し側が対象型を抽象化したい場面で使用する
/// - 明示名の `contains_point` / `contains_bbox` / `contains_aabb` は、対象を明示して
///   意味を固定したい場面で使用する
pub trait Contains<Target> {
    /// 対象を包含するかを返す
    fn contains(&self, target: &Target) -> bool;
}

/// 領域を囲む形状に対する点の位置の分類
///
/// 境界までの距離が許容誤差以内なら `OnBoundary` とし、それ以外の点を境界の内側（`Inside`）と
/// 外側（`Outside`）に分ける。閉曲線では、曲線が平面上に囲む領域に対して分類する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointClassification {
    /// 境界の内側（境界までの距離が許容誤差を超える）
    Inside,
    /// 境界上（境界までの距離が許容誤差以内）
    OnBoundary,
    /// 境界の外側（境界までの距離が許容誤差を超える）
    Outside,
}

/// AABB 2D relation
pub trait Aabb2DRelation<T: Scalar> {
    /// 点型
    type Point2D;

    /// 点を包含するかを返す（境界を含む厳密な判定）
    fn contains_point(&self, point: &Self::Point2D) -> bool;

    /// AABB の領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: &Self::Point2D) -> PointClassification;

    /// 他方の AABB と交差するかを返す
    fn intersects(&self, other: &Self) -> bool;
}

/// AABB 3D relation
pub trait Aabb3DRelation<T: Scalar> {
    /// 点型
    type Point3D;

    /// 点を包含するかを返す（境界を含む厳密な判定）
    fn contains_point(&self, point: &Self::Point3D) -> bool;

    /// AABB の領域に対する点の位置を分類する（既定の距離トレランス）
    fn classify_point(&self, point: &Self::Point3D) -> PointClassification;

    /// 他方の AABB と交差するかを返す
    fn intersects(&self, other: &Self) -> bool;
}

/// 平行関係
pub trait ParallelRelation<Other> {
    /// 相手と平行かを返す
    fn is_parallel_to(&self, other: &Other) -> bool;
}

/// 垂直関係
pub trait PerpendicularRelation<Other> {
    /// 相手と垂直かを返す
    fn is_perpendicular_to(&self, other: &Other) -> bool;
}

/// 同一直線関係
pub trait SameLineRelation<Other> {
    /// 相手と同一直線かを返す
    fn is_same_line(&self, other: &Other) -> bool;
}

/// 交差関係
pub trait IntersectsRelation<Other> {
    /// 相手と交差するかを返す
    fn intersects(&self, other: &Other) -> bool;
}

/// 角度 relation
pub trait AngularRelation<T: Scalar, Other> {
    /// 相手との角度を返す
    fn angle_to(&self, other: &Other) -> T;
}

/// 相手との角度を返す relation
pub trait AngleBetween<T: Scalar, Other> {
    /// 相手との角度を返す
    fn angle_between(&self, other: &Other) -> T;
}
