use super::{Octree, OctreeNode};
use geo_contracts::Scalar;
use geo_core::{Aabb3D, Point3D};

impl<T: Scalar, D: Clone> Octree<T, D> {
    /// 境界ボックスと交差するすべてのデータを取得
    ///
    /// # Arguments
    ///
    /// * `region` - 検索範囲の境界ボックス
    ///
    /// # Returns
    ///
    /// 交差するデータへの参照のベクタ
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let results = octree.query_region(&search_bbox);
    /// ```
    pub fn query_region(&self, region: &Aabb3D<T>) -> Vec<&D> {
        let expanded_region = if self.tolerance.query_expand > T::ZERO {
            let min = region.min();
            let max = region.max();
            Aabb3D::new(
                Point3D::new(
                    min.x() - self.tolerance.query_expand,
                    min.y() - self.tolerance.query_expand,
                    min.z() - self.tolerance.query_expand,
                ),
                Point3D::new(
                    max.x() + self.tolerance.query_expand,
                    max.y() + self.tolerance.query_expand,
                    max.z() + self.tolerance.query_expand,
                ),
            )
        } else {
            *region
        };

        let mut results = Vec::new();
        self.query_region_recursive(&self.root, &expanded_region, &mut results);
        results
    }

    #[allow(clippy::only_used_in_recursion)]
    fn query_region_recursive<'a>(
        &'a self,
        node: &'a OctreeNode<T, D>,
        region: &Aabb3D<T>,
        results: &mut Vec<&'a D>,
    ) {
        if !node.bounds().intersects(region) {
            return;
        }

        results.extend(node.data().iter());

        if let Some(children) = node.children() {
            for child in children.iter() {
                self.query_region_recursive(child, region, results);
            }
        }
    }

    /// ノードを走査するイテレータパターン
    ///
    /// 深さ優先でノードを訪問し、各ノードでコールバックを実行します。
    ///
    /// # Arguments
    ///
    /// * `callback` - 各ノードで実行される関数（引数: ノード参照, 深さ）
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// octree.traverse(|node, depth| {
    ///     println!("Depth {}: {} items", depth, node.data().len());
    /// });
    /// ```
    pub fn traverse<F>(&self, mut callback: F)
    where
        F: FnMut(&OctreeNode<T, D>, usize),
    {
        self.traverse_recursive(&self.root, &mut callback);
    }

    #[allow(clippy::only_used_in_recursion)]
    fn traverse_recursive<F>(&self, node: &OctreeNode<T, D>, callback: &mut F)
    where
        F: FnMut(&OctreeNode<T, D>, usize),
    {
        callback(node, node.depth());

        if let Some(children) = node.children() {
            for child in children.iter() {
                self.traverse_recursive(child, callback);
            }
        }
    }
}
