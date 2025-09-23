use super::{point::Point2D, geometry_kind::GeometryKind2D};

/// 2D形状間の交差判定トレイト
pub trait Intersect2D {
    /// 他の形状と交差するか（接触含むかは実装側で定義）
    fn intersects_with(&self, other: &GeometryKind2D, epsilon: f64) -> bool;

    /// 他の形状との交点（存在しない場合は空）
    fn intersection_points(&self, other: &GeometryKind2D, epsilon: f64) -> Vec<Point2D>;
}