/// ジェネリックなバウンディングボックストレイト
/// 
/// 次元に依存しない境界ボックス操作の共通インターフェース

/// バウンディングボックスの基本操作を定義するトレイト
pub trait BoundingBox<const D: usize> {
    /// 座標型（通常はf64）
    type Coord;
    
    /// 最小座標を取得
    fn min(&self) -> [Self::Coord; D];
    
    /// 最大座標を取得
    fn max(&self) -> [Self::Coord; D];
    
    /// 新しいバウンディングボックスを作成
    fn new(min: [Self::Coord; D], max: [Self::Coord; D]) -> Self;
    
    /// 指定次元の幅を取得
    fn extent(&self, dim: usize) -> Self::Coord
    where
        Self::Coord: std::ops::Sub<Output = Self::Coord> + Copy;
    
    /// 体積/面積を計算
    fn volume(&self) -> Self::Coord
    where
        Self::Coord: std::ops::Mul<Output = Self::Coord> + Copy + From<f64>;
    
    /// 中心点を取得
    fn center(&self) -> [Self::Coord; D]
    where
        Self::Coord: std::ops::Add<Output = Self::Coord> + std::ops::Div<f64, Output = Self::Coord> + Copy;
}

/// バウンディングボックスの高度な操作を定義するトレイト
pub trait BoundingBoxOps<const D: usize>: BoundingBox<D> {
    /// 点が境界ボックス内にあるかチェック
    fn contains_point(&self, point: [Self::Coord; D]) -> bool
    where
        Self::Coord: PartialOrd + Copy;
    
    /// 他の境界ボックスと交差するかチェック
    fn intersects(&self, other: &Self) -> bool
    where
        Self::Coord: PartialOrd + Copy;
    
    /// 他の境界ボックスとの結合（和集合）
    fn union(&self, other: &Self) -> Self
    where
        Self::Coord: PartialOrd + Copy;
    
    /// 境界ボックスを指定量だけ拡張
    fn expand(&self, amount: Self::Coord) -> Self
    where
        Self::Coord: std::ops::Add<Output = Self::Coord> + std::ops::Sub<Output = Self::Coord> + Copy;
    
    /// 境界ボックスが有効かチェック（min <= max）
    fn is_valid(&self) -> bool
    where
        Self::Coord: PartialOrd + Copy;
}

/// 衝突判定用の特殊操作
pub trait CollisionBounds<const D: usize>: BoundingBoxOps<D> {
    /// 高速な重複テスト（軸平行境界ボックス特化）
    fn fast_overlaps(&self, other: &Self) -> bool
    where
        Self::Coord: PartialOrd + Copy;
    
    /// 境界ボックス間の距離（分離している場合のみ）
    fn separation_distance(&self, other: &Self) -> Option<Self::Coord>
    where
        Self::Coord: PartialOrd + std::ops::Sub<Output = Self::Coord> + Copy + From<f64>;
    
    /// 最も近い点を境界ボックス表面で取得
    fn closest_point_on_surface(&self, point: [Self::Coord; D]) -> [Self::Coord; D]
    where
        Self::Coord: PartialOrd + Copy;
}

/// 次元別のヘルパー型エイリアス
pub type BBox2D = crate::geometry2d::BBox2D;
pub type BBox3D = crate::geometry3d::BBox3D;

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用のモック実装（テスト専用）
    #[derive(Debug, Clone, PartialEq)]
    struct MockBBox<const D: usize> {
        min: [f64; D],
        max: [f64; D],
    }

    impl<const D: usize> BoundingBox<D> for MockBBox<D> {
        type Coord = f64;

        fn min(&self) -> [Self::Coord; D] {
            self.min
        }

        fn max(&self) -> [Self::Coord; D] {
            self.max
        }

        fn new(min: [Self::Coord; D], max: [Self::Coord; D]) -> Self {
            Self { min, max }
        }

        fn extent(&self, dim: usize) -> Self::Coord {
            if dim < D {
                self.max[dim] - self.min[dim]
            } else {
                0.0
            }
        }

        fn volume(&self) -> Self::Coord {
            let mut vol = 1.0;
            for i in 0..D {
                vol *= self.extent(i);
            }
            vol
        }

        fn center(&self) -> [Self::Coord; D] {
            let mut center = [0.0; D];
            for i in 0..D {
                center[i] = (self.min[i] + self.max[i]) / 2.0;
            }
            center
        }
    }

    impl<const D: usize> BoundingBoxOps<D> for MockBBox<D> {
        fn contains_point(&self, point: [Self::Coord; D]) -> bool {
            for i in 0..D {
                if point[i] < self.min[i] || point[i] > self.max[i] {
                    return false;
                }
            }
            true
        }

        fn intersects(&self, other: &Self) -> bool {
            for i in 0..D {
                if self.max[i] < other.min[i] || self.min[i] > other.max[i] {
                    return false;
                }
            }
            true
        }

        fn union(&self, other: &Self) -> Self {
            let mut min = [0.0; D];
            let mut max = [0.0; D];
            for i in 0..D {
                min[i] = self.min[i].min(other.min[i]);
                max[i] = self.max[i].max(other.max[i]);
            }
            Self::new(min, max)
        }

        fn expand(&self, amount: Self::Coord) -> Self {
            let mut min = self.min;
            let mut max = self.max;
            for i in 0..D {
                min[i] -= amount;
                max[i] += amount;
            }
            Self::new(min, max)
        }

        fn is_valid(&self) -> bool {
            for i in 0..D {
                if self.min[i] > self.max[i] {
                    return false;
                }
            }
            true
        }
    }

    #[test]
    fn test_generic_bbox_2d() {
        let bbox = MockBBox::<2>::new([0.0, 0.0], [2.0, 3.0]);
        
        assert_eq!(bbox.min(), [0.0, 0.0]);
        assert_eq!(bbox.max(), [2.0, 3.0]);
        assert_eq!(bbox.extent(0), 2.0);
        assert_eq!(bbox.extent(1), 3.0);
        assert_eq!(bbox.volume(), 6.0);
        assert_eq!(bbox.center(), [1.0, 1.5]);
        assert!(bbox.is_valid());
    }

    #[test]
    fn test_generic_bbox_3d() {
        let bbox = MockBBox::<3>::new([0.0, 0.0, 0.0], [2.0, 3.0, 4.0]);
        
        assert_eq!(bbox.volume(), 24.0);
        assert_eq!(bbox.center(), [1.0, 1.5, 2.0]);
        assert!(bbox.contains_point([1.0, 1.0, 1.0]));
        assert!(!bbox.contains_point([3.0, 1.0, 1.0]));
    }

    #[test]
    fn test_bbox_operations() {
        let bbox1 = MockBBox::<3>::new([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]);
        let bbox2 = MockBBox::<3>::new([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]);
        let bbox3 = MockBBox::<3>::new([3.0, 3.0, 3.0], [4.0, 4.0, 4.0]);
        
        assert!(bbox1.intersects(&bbox2));
        assert!(!bbox1.intersects(&bbox3));
        
        let union = bbox1.union(&bbox2);
        assert_eq!(union.min(), [0.0, 0.0, 0.0]);
        assert_eq!(union.max(), [3.0, 3.0, 3.0]);
        
        let expanded = bbox1.expand(0.5);
        assert_eq!(expanded.min(), [-0.5, -0.5, -0.5]);
        assert_eq!(expanded.max(), [2.5, 2.5, 2.5]);
    }
}