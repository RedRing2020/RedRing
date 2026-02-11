//! Octree空間分割データ構造
//!
//! 3D空間を再帰的に8つの子領域に分割する空間データ構造を提供します。
//!
//! ## 主要な機能
//!
//! - **自動分割**: データ数が閾値を超えると自動的にノードを8分割
//! - **範囲検索**: O(log n)の効率的な空間検索
//! - **最近傍探索**: 枝刈り最適化による高速検索
//! - **衝突判定高速化**: O(n²) → O(n log n)（粗判定フェーズ）
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use geo_algorithms::octree::{Octree, HasBoundingBox, HasPosition};
//! use geo_core::Aabb3D;
//!
//! // Octreeの作成（境界、最大深さ、ノードあたり最大アイテム数）
//! let scene_bbox = Aabb3D::new(/* min */, /* max */);
//! let mut octree = Octree::new(scene_bbox, 8, 10);
//!
//! // データ挿入（自動分割対応）
//! octree.insert(shape);
//!
//! // 範囲検索
//! let candidates = octree.query_region(&search_region);
//!
//! // 最近傍検索
//! if let Some((nearest, distance)) = octree.nearest(&query_point) {
//!     println!("Found nearest at distance: {}", distance);
//! }
//! ```
//!
//! ## 実装状況
//!
//! ✅ **Phase 1 完了**:
//! - 基本データ構造（Octree, OctreeNode）
//! - 完全な再帰挿入と自動分割
//! - 範囲検索（query_region）
//! - 最適化された最近傍探索（nearest）
//! - ノード走査（traverse）
//!
//! 🔄 **Phase 2 予定**:
//! - k近傍探索（k-nearest neighbors）
//! - ✅ ボクセルOctree（切削シミュレーション用） ← Phase 2 完了
//! - デバッグ可視化対応
//! - バルク挿入最適化

use geo_core::{Aabb3D, Point3D};
use geo_foundation::Scalar;

pub mod node;
pub mod voxel;

pub use node::OctreeNode;
pub use voxel::{VoxelNode, VoxelOctree, VoxelState};

/// データが境界ボックスを持つことを示すトレイト
pub trait HasBoundingBox<T: Scalar> {
    /// 境界ボックスを取得
    fn bounding_box(&self) -> Aabb3D<T>;
}

/// データが位置を持つことを示すトレイト
pub trait HasPosition<T: Scalar> {
    /// 位置を取得
    fn position(&self) -> Point3D<T>;
}

/// Octree空間分割データ構造
///
/// 3D空間を再帰的に8つの子領域に分割し、効率的な検索を実現します。
///
/// # Type Parameters
///
/// * `T` - 座標値の型（Scalarトレイト境界）
/// * `D` - 格納するデータの型
///
/// # Examples
///
/// ```rust,ignore
/// use geo_algorithms::octree::Octree;
/// use geo_core::Aabb3D;
///
/// let scene_bbox = Aabb3D::new(/* ... */);
/// let mut octree = Octree::new(scene_bbox, 8, 10);
///
/// octree.insert(shape);
/// let results = octree.query_region(&search_region);
/// ```
#[derive(Debug)]
pub struct Octree<T: Scalar, D: Clone> {
    /// ルートノード
    root: OctreeNode<T, D>,

    /// 最大分割深さ
    max_depth: usize,

    /// 分割閾値（ノードあたりの最大要素数）
    max_items: usize,

    /// 総ノード数（統計情報）
    total_nodes: usize,

    /// 総要素数（統計情報）
    total_items: usize,
}

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
        Self {
            root: OctreeNode::new(bounds, 0),
            max_depth,
            max_items,
            total_nodes: 1,
            total_items: 0,
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

    /// すべての要素をクリア
    pub fn clear(&mut self) {
        let bounds = *self.root.bounds();
        self.root = OctreeNode::new(bounds, 0);
        self.total_nodes = 1;
        self.total_items = 0;
    }
}

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
        // 1. このノードの範囲外なら挿入失敗
        if !node.bounds().intersects(&item.bounding_box()) {
            return false;
        }

        // 2. 分割済みなら子ノードに委譲
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

        // 3. 未分割 - このノードにデータ追加
        node.add_data(item.clone());

        // 4. 分割条件チェック
        if node.data().len() > max_items && depth < max_depth {
            node.subdivide();
            *total_nodes += 8; // 8つの子ノード追加

            // 既存データを子ノードに再配置
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
        let mut results = Vec::new();
        self.query_region_recursive(&self.root, region, &mut results);
        results
    }

    /// 再帰的に範囲検索（内部実装）
    #[allow(clippy::only_used_in_recursion)]
    fn query_region_recursive<'a>(
        &'a self,
        node: &'a OctreeNode<T, D>,
        region: &Aabb3D<T>,
        results: &mut Vec<&'a D>,
    ) {
        // 1. ノードと範囲が交差しないなら終了
        if !node.bounds().intersects(region) {
            return;
        }

        // 2. このノードのデータを結果に追加
        results.extend(node.data().iter());

        // 3. 子ノードがあれば再帰的に検索
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

    /// 再帰的にノードを走査（内部実装）
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

    /// 再帰的に最近傍を探索（内部実装、枝刈り最適化）
    #[allow(clippy::only_used_in_recursion)]
    fn nearest_recursive<'a>(
        &'a self,
        node: &'a OctreeNode<T, D>,
        point: &Point3D<T>,
        mut best: Option<(&'a D, T)>,
    ) -> Option<(&'a D, T)> {
        // 1. このノードの最小距離が現在の最良距離より大きい → 枝刈り
        let min_dist_sq = self.min_distance_squared_to_node(node, point);
        if let Some((_, best_dist)) = best {
            if min_dist_sq > best_dist * best_dist {
                return best;
            }
        }

        // 2. このノードのデータから最近傍候補を探す
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

        // 3. 子ノードがあれば探索（距離順にソート）
        if let Some(children) = node.children() {
            // 各子ノードへの最小距離を計算
            let mut child_dists: Vec<(usize, T)> = children
                .iter()
                .enumerate()
                .map(|(idx, child)| {
                    let min_dist_sq = self.min_distance_squared_to_node(child, point);
                    (idx, min_dist_sq.sqrt())
                })
                .collect();

            // 距離でソート（近い順）
            child_dists.sort_by(|(_, d1), (_, d2)| {
                d1.partial_cmp(d2).unwrap_or(std::cmp::Ordering::Equal)
            });

            // 近い子ノードから順に探索
            for (idx, _) in child_dists {
                best = self.nearest_recursive(&children[idx], point, best);
            }
        }

        best
    }

    /// ノードの境界ボックスから点までの最小距離の二乗を計算
    fn min_distance_squared_to_node(&self, node: &OctreeNode<T, D>, point: &Point3D<T>) -> T {
        let bounds = node.bounds();
        let min = bounds.min();
        let max = bounds.max();

        let dx = if point.x() < min.x() {
            min.x() - point.x()
        } else if point.x() > max.x() {
            point.x() - max.x()
        } else {
            // point is inside the bounds on x-axis
            point.x() - point.x() // returns 0
        };

        let dy = if point.y() < min.y() {
            min.y() - point.y()
        } else if point.y() > max.y() {
            point.y() - max.y()
        } else {
            // point is inside the bounds on y-axis
            point.y() - point.y() // returns 0
        };

        let dz = if point.z() < min.z() {
            min.z() - point.z()
        } else if point.z() > max.z() {
            point.z() - max.z()
        } else {
            // point is inside the bounds on z-axis
            point.z() - point.z() // returns 0
        };

        dx * dx + dy * dy + dz * dz
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // テスト用の簡単なデータ型
    #[derive(Debug, Clone)]
    struct TestPoint {
        pos: Point3D<f64>,
    }

    impl HasBoundingBox<f64> for TestPoint {
        fn bounding_box(&self) -> Aabb3D<f64> {
            let epsilon = 0.001;
            Aabb3D::new(
                Point3D::new(
                    self.pos.x() - epsilon,
                    self.pos.y() - epsilon,
                    self.pos.z() - epsilon,
                ),
                Point3D::new(
                    self.pos.x() + epsilon,
                    self.pos.y() + epsilon,
                    self.pos.z() + epsilon,
                ),
            )
        }
    }

    impl HasPosition<f64> for TestPoint {
        fn position(&self) -> Point3D<f64> {
            self.pos
        }
    }

    #[test]
    fn test_octree_creation() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let octree: Octree<f64, TestPoint> = Octree::new(bbox, 8, 10);

        assert_eq!(octree.total_nodes(), 1);
        assert_eq!(octree.total_items(), 0);
        assert!(octree.is_empty());
    }

    #[test]
    fn test_insert_and_query() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut octree = Octree::new(bbox, 8, 10);

        // データ挿入
        let point1 = TestPoint {
            pos: Point3D::new(10.0, 10.0, 10.0),
        };
        let point2 = TestPoint {
            pos: Point3D::new(50.0, 50.0, 50.0),
        };

        octree.insert(point1.clone());
        octree.insert(point2.clone());

        assert_eq!(octree.total_items(), 2);

        // 範囲検索
        let query_bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(20.0, 20.0, 20.0));
        let results = octree.query_region(&query_bbox);

        // 簡易実装では全データが返される
        assert!(results.len() >= 1);
    }

    #[test]
    fn test_nearest_search() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut octree = Octree::new(bbox, 8, 10);

        // データ挿入
        octree.insert(TestPoint {
            pos: Point3D::new(10.0, 10.0, 10.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(50.0, 50.0, 50.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(90.0, 90.0, 90.0),
        });

        // 最近傍検索
        let query_point = Point3D::new(12.0, 12.0, 12.0);
        let result = octree.nearest(&query_point);

        assert!(result.is_some());
        let (_nearest, distance) = result.unwrap();
        assert!(distance < 5.0); // (10, 10, 10)が最近傍
    }

    #[test]
    fn test_clear() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut octree = Octree::new(bbox, 8, 10);

        octree.insert(TestPoint {
            pos: Point3D::new(10.0, 10.0, 10.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(50.0, 50.0, 50.0),
        });

        assert_eq!(octree.total_items(), 2);

        octree.clear();

        assert_eq!(octree.total_items(), 0);
        assert_eq!(octree.total_nodes(), 1);
        assert!(octree.is_empty());
    }

    #[test]
    fn test_traverse() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let octree: Octree<f64, TestPoint> = Octree::new(bbox, 8, 10);

        let mut visited_count = 0;
        octree.traverse(|_node, _depth| {
            visited_count += 1;
        });

        assert_eq!(visited_count, 1); // ルートノードのみ
    }

    #[test]
    fn test_recursive_insert_with_subdivision() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        // max_items=2 で分割を容易に発生させる
        let mut octree = Octree::new(bbox, 8, 2);

        // 同じ領域に3つのポイントを挿入（分割が発生するはず）
        octree.insert(TestPoint {
            pos: Point3D::new(10.0, 10.0, 10.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(11.0, 11.0, 11.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(12.0, 12.0, 12.0),
        });

        assert_eq!(octree.total_items(), 3);
        // 分割が発生したため、ノード数が増加しているはず
        assert!(octree.total_nodes() > 1);
    }

    #[test]
    fn test_optimized_nearest_search() {
        let bbox = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut octree = Octree::new(bbox, 8, 5);

        // 複数のポイントを挿入
        octree.insert(TestPoint {
            pos: Point3D::new(10.0, 10.0, 10.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(50.0, 50.0, 50.0),
        });
        octree.insert(TestPoint {
            pos: Point3D::new(90.0, 90.0, 90.0),
        });

        // (11, 11, 11)に最も近い点は(10, 10, 10)のはず
        let query_point = Point3D::new(11.0, 11.0, 11.0);
        let result = octree.nearest(&query_point);

        assert!(result.is_some());
        let (nearest, distance) = result.unwrap();
        assert!(distance < 2.0); // sqrt(3) ≈ 1.732

        // 実際に最も近い点かを確認
        let expected_pos = Point3D::new(10.0, 10.0, 10.0);
        let nearest_pos = nearest.position();
        assert!((nearest_pos.x() - expected_pos.x()).abs() < 0.001);
        assert!((nearest_pos.y() - expected_pos.y()).abs() < 0.001);
        assert!((nearest_pos.z() - expected_pos.z()).abs() < 0.001);
    }
}
