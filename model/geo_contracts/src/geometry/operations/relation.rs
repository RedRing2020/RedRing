//! 幾何 relation のtrait定義
//!
//! 平行・垂直・同一直線・向き一致のような、相手との関係を返す契約を集約する。

use analysis::abstract_types::Scalar;

/// 最近点対を返す relation
pub trait ClosestPointPair<Other> {
    /// 最近点対の型
    type PointPair;

    /// 最近点対を返す
    fn closest_points(&self, other: &Other) -> Option<Self::PointPair>;
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

/// ねじれ関係
pub trait SkewRelation<Other> {
    /// 相手とスキュー関係かを返す
    fn is_skew_to(&self, other: &Other) -> bool;
}

/// 角度 relation
pub trait AngularRelation<T: Scalar, Other> {
    /// 相手との角度を返す
    fn angle_to(&self, other: &Other) -> T;
}

/// 向き relation
pub trait DirectionalRelation<Other> {
    /// 相手と同方向かを返す
    fn is_same_direction(&self, other: &Other) -> bool;

    /// 相手と逆方向かを返す
    fn is_opposite_direction(&self, other: &Other) -> bool;
}

/// 指定方向を向くかを判定する relation
pub trait PointsTowards<Target> {
    /// 指定対象を向くかを返す
    fn points_towards(&self, target: Target) -> bool;
}

/// 平面上に存在するかを判定する relation
pub trait OnPlaneRelation<Other> {
    /// 相手の平面上にあるかを返す
    fn is_on_plane(&self, other: &Other) -> bool;
}

/// 相手との角度を返す relation
pub trait AngleBetween<T: Scalar, Other> {
    /// 相手との角度を返す
    fn angle_between(&self, other: &Other) -> T;
}
