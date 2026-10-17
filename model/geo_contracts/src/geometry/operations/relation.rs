//! 幾何 relation のtrait定義
//!
//! 平行・垂直・角度のような、同じ種類の形状どうしの関係を返す契約を集約する。

use crate::{Angle, Scalar};

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

/// 角度 relation
pub trait AngularRelation<T: Scalar, Other> {
    /// 相手との角度を返す
    fn angle_to(&self, other: &Other) -> Angle<T>;
}
