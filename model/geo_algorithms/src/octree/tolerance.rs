use geo_foundation::{Scalar, ToleranceSettings};

/// Octree専用トレランス設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OctreeTolerance<T: Scalar> {
    /// 点データをAABBに変換する際の半径（最小厚み）
    pub point_aabb_half_extent: T,
    /// 範囲検索時にクエリ領域へ加える膨張量
    pub query_expand: T,
    /// 最近傍探索の枝刈り安全余白
    pub nearest_prune_margin: T,
}

impl<T: Scalar> OctreeTolerance<T> {
    /// カスタム設定
    pub fn new(point_aabb_half_extent: T, query_expand: T, nearest_prune_margin: T) -> Self {
        Self {
            point_aabb_half_extent,
            query_expand,
            nearest_prune_margin,
        }
    }
}

impl<T: Scalar> Default for OctreeTolerance<T> {
    fn default() -> Self {
        let base = ToleranceSettings::<T>::relaxed().distance_tolerance;
        Self {
            point_aabb_half_extent: base,
            query_expand: base,
            nearest_prune_margin: base / T::from_f64(2.0),
        }
    }
}
