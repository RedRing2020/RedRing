//! 3次元軸平行境界ボックス（Aabb3D）
//!
//! Foundation Pattern に基づく Aabb3D の実装。
//! geo_primitives, geo_nurbs など全クレートから共通利用されます。

use analysis::abstract_types::Scalar;
use geo_foundation::commons::Aabb3DTrait;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3D<T: Scalar> {
    min: analysis::Point3<T>,
    max: analysis::Point3<T>,
}

impl<T: Scalar> Aabb3D<T> {
    /// 新しいAABBを作成
    pub fn new(min: analysis::Point3<T>, max: analysis::Point3<T>) -> Self {
        Self { min, max }
    }

    /// 複数の点からAABBを作成
    pub fn from_points(points: &[analysis::Point3<T>]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let mut min_x = points[0].x;
        let mut max_x = points[0].x;
        let mut min_y = points[0].y;
        let mut max_y = points[0].y;
        let mut min_z = points[0].z;
        let mut max_z = points[0].z;

        for p in points.iter().skip(1) {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            min_y = min_y.min(p.y);
            max_y = max_y.max(p.y);
            min_z = min_z.min(p.z);
            max_z = max_z.max(p.z);
        }

        Some(Self {
            min: analysis::Point3::new(min_x, min_y, min_z),
            max: analysis::Point3::new(max_x, max_y, max_z),
        })
    }

    /// 点がAABB内に含まれるか
    pub fn contains(&self, p: &analysis::Point3<T>) -> bool {
        (self.min.x <= p.x && p.x <= self.max.x)
            && (self.min.y <= p.y && p.y <= self.max.y)
            && (self.min.z <= p.z && p.z <= self.max.z)
    }

    /// 最小点を取得
    pub fn min(&self) -> analysis::Point3<T> {
        self.min
    }

    /// 最大点を取得
    pub fn max(&self) -> analysis::Point3<T> {
        self.max
    }

    /// 幅（X軸方向のサイズ）
    pub fn width(&self) -> T {
        self.max.x - self.min.x
    }

    /// 高さ（Y軸方向のサイズ）
    pub fn height(&self) -> T {
        self.max.y - self.min.y
    }

    /// 深さ（Z軸方向のサイズ）
    pub fn depth(&self) -> T {
        self.max.z - self.min.z
    }

    /// 体積
    pub fn volume(&self) -> T {
        self.width() * self.height() * self.depth()
    }

    /// 中心点
    pub fn center(&self) -> analysis::Point3<T> {
        let two = T::ONE + T::ONE;
        analysis::Point3::new(
            (self.min.x + self.max.x) / two,
            (self.min.y + self.max.y) / two,
            (self.min.z + self.max.z) / two,
        )
    }

    /// 空のAABBか判定
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// 他のAABBを完全に含むか
    pub fn contains_aabb(&self, other: &Self) -> bool {
        self.min.x <= other.min.x
            && self.max.x >= other.max.x
            && self.min.y <= other.min.y
            && self.max.y >= other.max.y
            && self.min.z <= other.min.z
            && self.max.z >= other.max.z
    }

    /// 他のAABBと交差するか
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

// ============================================================================
// Foundation Trait Implementation
// ============================================================================

impl<T: Scalar> Aabb3DTrait<T> for Aabb3D<T> {
    type Point3D = analysis::Point3<T>;

    fn min(&self) -> Self::Point3D {
        self.min
    }

    fn max(&self) -> Self::Point3D {
        self.max
    }

    fn width(&self) -> T {
        self.width()
    }

    fn height(&self) -> T {
        self.height()
    }

    fn depth(&self) -> T {
        self.depth()
    }

    fn volume(&self) -> T {
        self.volume()
    }

    fn center(&self) -> Self::Point3D {
        self.center()
    }

    fn contains_point(&self, point: &Self::Point3D) -> bool {
        self.contains(point)
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        self.contains_aabb(other)
    }

    fn intersects(&self, other: &Self) -> bool {
        self.intersects(other)
    }

    fn is_valid(&self) -> bool {
        !self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb3d_creation() {
        let min = analysis::Point3::new(0.0, 0.0, 0.0);
        let max = analysis::Point3::new(1.0, 2.0, 3.0);
        let aabb = Aabb3D::new(min, max);
        assert_eq!(aabb.min(), min);
        assert_eq!(aabb.max(), max);
    }

    #[test]
    fn test_aabb3d_dimensions() {
        let min = analysis::Point3::new(1.0, 2.0, 3.0);
        let max = analysis::Point3::new(4.0, 7.0, 9.0);
        let aabb = Aabb3D::new(min, max);
        assert_eq!(aabb.width(), 3.0);
        assert_eq!(aabb.height(), 5.0);
        assert_eq!(aabb.depth(), 6.0);
        assert_eq!(aabb.volume(), 90.0);
    }

    #[test]
    fn test_aabb3d_center() {
        let min = analysis::Point3::new(0.0, 0.0, 0.0);
        let max = analysis::Point3::new(2.0, 4.0, 6.0);
        let aabb = Aabb3D::new(min, max);
        let center = aabb.center();
        assert_eq!(center.x, 1.0);
        assert_eq!(center.y, 2.0);
        assert_eq!(center.z, 3.0);
    }

    #[test]
    fn test_contains() {
        let min = analysis::Point3::new(0.0, 0.0, 0.0);
        let max = analysis::Point3::new(2.0, 2.0, 2.0);
        let aabb = Aabb3D::new(min, max);
        assert!(aabb.contains(&analysis::Point3::new(1.0, 1.0, 1.0)));
        assert!(aabb.contains(&analysis::Point3::new(0.0, 0.0, 0.0)));
        assert!(aabb.contains(&analysis::Point3::new(2.0, 2.0, 2.0)));
        assert!(!aabb.contains(&analysis::Point3::new(3.0, 1.0, 1.0)));
    }

    #[test]
    fn test_from_points() {
        let points = vec![
            analysis::Point3::new(1.0, 2.0, 3.0),
            analysis::Point3::new(4.0, 5.0, 6.0),
            analysis::Point3::new(0.0, 1.0, 2.0),
        ];
        let aabb = Aabb3D::from_points(&points).unwrap();
        assert_eq!(aabb.min(), analysis::Point3::new(0.0, 1.0, 2.0));
        assert_eq!(aabb.max(), analysis::Point3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn test_is_empty() {
        let valid = Aabb3D::new(
            analysis::Point3::new(0.0, 0.0, 0.0),
            analysis::Point3::new(1.0, 1.0, 1.0),
        );
        let invalid = Aabb3D::new(
            analysis::Point3::new(1.0, 0.0, 0.0),
            analysis::Point3::new(0.0, 1.0, 1.0),
        );
        assert!(!valid.is_empty());
        assert!(invalid.is_empty());
    }
}
