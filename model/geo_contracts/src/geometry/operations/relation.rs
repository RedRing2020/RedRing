//! 幾何 relation のtrait定義
//!
//! 平行・垂直・同一直線・向き一致のような、相手との関係を返す契約を集約する。

use crate::Scalar;

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
