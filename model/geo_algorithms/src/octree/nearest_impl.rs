use super::{HasPosition, Octree, OctreeNode};
use geo_contracts::Scalar;
use geo_core::Point3D;

impl<T: Scalar, D: Clone> Octree<T, D>
where
    D: HasPosition<T>,
{
    /// 点から最も近いデータを探索（最適化版）
    ///
    /// 空間分割を利用して効率的に検索します。
    ///
    /// # Arguments
    ///
    /// * `point` - 検索の基準点
    ///
    /// # Returns
    ///
    /// Some((データ参照, 距離)) または None（Octreeが空の場合）
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// if let Some((nearest, distance)) = octree.nearest(&query_point) {
    ///     println!("Nearest: distance = {}", distance);
    /// }
    /// ```
    pub fn nearest(&self, point: &Point3D<T>) -> Option<(&D, T)> {
        if self.is_empty() {
            return None;
        }
        self.nearest_recursive(&self.root, point, None)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn nearest_recursive<'a>(
        &'a self,
        node: &'a OctreeNode<T, D>,
        point: &Point3D<T>,
        mut best: Option<(&'a D, T)>,
    ) -> Option<(&'a D, T)> {
        let min_dist_sq = self.min_distance_squared_to_node(node, point);
        if let Some((_, best_dist)) = best {
            let prune_limit = best_dist + self.tolerance.nearest_prune_margin;
            if min_dist_sq > prune_limit * prune_limit {
                return best;
            }
        }

        for item in node.data().iter() {
            let pos = item.position();
            let dx = point.x() - pos.x();
            let dy = point.y() - pos.y();
            let dz = point.z() - pos.z();
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();

            if best.is_none() || dist < best.unwrap().1 {
                best = Some((item, dist));
            }
        }

        if let Some(children) = node.children() {
            let mut child_dists: Vec<(usize, T)> = children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    let min_dist_sq = self.min_distance_squared_to_node(child, point);
                    (idx, min_dist_sq.sqrt())
                })
                .collect();

            child_dists.sort_by(|(_, d1), (_, d2)| {
                d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal)
            });

            for (idx, _) in child_dists {
                best = self.nearest_recursive(&children[idx], point, best);
            }
        }

        best
    }

    fn min_distance_squared_to_node(&self, node: &OctreeNode<T, D>, point: &Point3D<T>) -> T {
        let bounds = node.bounds();
        let min = bounds.min();
        let max = bounds.max();

        let dx = if point.x() < min.x() {
            min.x() - point.x()
        } else if point.x() > max.x() {
            point.x() - max.x()
        } else {
            point.x() - point.x()
        };

        let dy = if point.y() < min.y() {
            min.y() - point.y()
        } else if point.y() > max.y() {
            point.y() - max.y()
        } else {
            point.y() - point.y()
        };

        let dz = if point.z() < min.z() {
            min.z() - point.z()
        } else if point.z() > max.z() {
            point.z() - max.z()
        } else {
            point.z() - point.z()
        };

        dx * dx + dy * dy + dz * dz
    }
}
