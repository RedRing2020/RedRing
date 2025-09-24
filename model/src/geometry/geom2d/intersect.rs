use super::point::Point;
use super::kind::CurveKind2;

/// 2D形状間の交差判定トレイト
pub trait Intersect {
    /// 他の形状と交差するか（接触含むかは実装側で定義）
    fn intersects_with(&self, other: &CurveKind2, epsilon: f64) -> bool;

    /// 他の形状との交点（存在しない場合は空）
    fn intersection_points(&self, other: &CurveKind2, epsilon: f64) -> Vec<Point>;
}