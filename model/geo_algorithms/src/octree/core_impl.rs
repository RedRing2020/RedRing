use super::{Octree, OctreeNode, OctreeTolerance};
use geo_contracts::Scalar;
use geo_core::Aabb3D;

impl<T: Scalar, D: Clone> Octree<T, D> {
    /// 新しいOctreeを作成
    ///
    /// # Arguments
    ///
    /// * `bounds` - 全体の境界ボックス
    /// * `max_depth` - 最大分割深さ（推奨: 6-10）
    /// * `max_items` - ノードあたりの最大要素数（推奨: 8-16）
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(100.0, 100.0, 100.0));
    /// let octree = Octree::new(bbox, 8, 10);
    /// ```
    pub fn new(bounds: Aabb3D<T>, max_depth: usize, max_items: usize) -> Self {
        Self::with_tolerance(bounds, max_depth, max_items, OctreeTolerance::default())
    }

    /// トレランス設定付きで新しいOctreeを作成
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use geo_algorithms::octree::{Octree, OctreeTolerance};
    /// use geo_core::{Aabb3D, Point3D};
    ///
    /// let bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(100.0, 100.0, 100.0));
    /// let tolerance = OctreeTolerance::new(1.0e-3, 1.0e-3, 5.0e-4);
    /// let octree = Octree::<f64, usize>::with_tolerance(bbox, 8, 10, tolerance);
    /// ```
    pub fn with_tolerance(
        bounds: Aabb3D<T>,
        max_depth: usize,
        max_items: usize,
        tolerance: OctreeTolerance<T>,
    ) -> Self {
        Self {
            root: OctreeNode::new(bounds, 0),
            max_depth,
            max_items,
            total_nodes: 1,
            total_items: 0,
            tolerance,
        }
    }

    /// 全体の境界ボックスを取得
    pub fn bounds(&self) -> &Aabb3D<T> {
        self.root.bounds()
    }

    /// 最大分割深さを取得
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// 分割閾値を取得
    pub fn max_items(&self) -> usize {
        self.max_items
    }

    /// 総ノード数を取得
    pub fn total_nodes(&self) -> usize {
        self.total_nodes
    }

    /// 総要素数を取得
    pub fn total_items(&self) -> usize {
        self.total_items
    }

    /// Octreeが空かを判定
    pub fn is_empty(&self) -> bool {
        self.total_items == 0
    }

    /// Octree専用トレランス設定を取得
    pub fn tolerance(&self) -> OctreeTolerance<T> {
        self.tolerance
    }

    /// すべての要素をクリア
    pub fn clear(&mut self) {
        let bounds = *self.root.bounds();
        *self = Self::with_tolerance(bounds, self.max_depth, self.max_items, self.tolerance);
    }
}
