use super::{HasBoundingBox, Octree, OctreeNode};
use geo_contracts::Scalar;

impl<T: Scalar, D: Clone> Octree<T, D>
where
    D: HasBoundingBox<T>,
{
    /// データを挿入（完全版 - 再帰的分割対応）
    ///
    /// # Arguments
    ///
    /// * `item` - 挿入するデータ
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// octree.insert(shape);
    /// ```
    pub fn insert(&mut self, item: D) {
        let max_depth = self.max_depth;
        let max_items = self.max_items;
        let inserted = Self::insert_recursive_static(
            &mut self.root,
            item.clone(),
            0,
            max_depth,
            max_items,
            &mut self.total_nodes,
        );
        if inserted {
            self.total_items += 1;
        }
    }

    /// 再帰的にデータを挿入（内部実装・スタティックメソッド）
    fn insert_recursive_static(
        node: &mut OctreeNode<T, D>,
        item: D,
        depth: usize,
        max_depth: usize,
        max_items: usize,
        total_nodes: &mut usize,
    ) -> bool {
        if !node.bounds().intersects(&item.bounding_box()) {
            return false;
        }

        if node.has_children() {
            if let Some(children) = node.children_mut() {
                for child in children.iter_mut() {
                    if Self::insert_recursive_static(
                        child,
                        item.clone(),
                        depth + 1,
                        max_depth,
                        max_items,
                        total_nodes,
                    ) {
                        return true;
                    }
                }
            }
            return false;
        }

        node.add_data(item.clone());

        if node.data().len() > max_items && depth < max_depth {
            node.subdivide();
            *total_nodes += 8;

            let old_data: Vec<_> = node.data().to_vec();
            node.clear_data();

            for old_item in old_data {
                if let Some(children) = node.children_mut() {
                    for child in children.iter_mut() {
                        if child.bounds().intersects(&old_item.bounding_box()) {
                            child.add_data(old_item.clone());
                            break;
                        }
                    }
                }
            }
        }

        true
    }
}
