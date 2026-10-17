//! Octreeノード実装
//!
//! 個々のOctreeノードのデータ構造と操作を提供します。

use geo_contracts::Scalar;
use geo_core::{Aabb3D, Point3D};

/// Octreeノード
///
/// 3D空間の特定領域を表し、最大8つの子ノードを持つことができます。
///
/// ## 子ノードの配置
///
/// ```text
///      Y
///      ↑
///     6───7
///    /│  /│
///   2─┼─3 │  Z
///   │ 4─┼─5  ↗
///   │/  │/
///   0───1  → X
///
/// ビット演算: (z_bit << 2) | (y_bit << 1) | x_bit
/// 0: (x_min, y_min, z_min)  1: (x_max, y_min, z_min)
/// 2: (x_min, y_max, z_min)  3: (x_max, y_max, z_min)
/// 4: (x_min, y_min, z_max)  5: (x_max, y_min, z_max)
/// 6: (x_min, y_max, z_max)  7: (x_max, y_max, z_max)
/// ```
#[derive(Debug, Clone)]
pub struct OctreeNode<T: Scalar, D: Clone> {
    /// ノードの境界ボックス
    bounds: Aabb3D<T>,

    /// 深さレベル（ルート=0）
    depth: usize,

    /// このノードに含まれるデータ
    data: Vec<D>,

    /// 子ノード（8個 or None）
    children: Option<Box<[OctreeNode<T, D>; 8]>>,
}

impl<T: Scalar, D: Clone> OctreeNode<T, D> {
    /// 新しいOctreeノードを作成
    ///
    /// # Arguments
    ///
    /// * `bounds` - ノードの境界ボックス
    /// * `depth` - 深さレベル（ルート=0）
    pub fn new(bounds: Aabb3D<T>, depth: usize) -> Self {
        Self {
            bounds,
            depth,
            data: Vec::new(),
            children: None,
        }
    }

    /// 境界ボックスを取得
    pub fn bounds(&self) -> &Aabb3D<T> {
        &self.bounds
    }

    /// 深さレベルを取得
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// データへの参照を取得
    pub fn data(&self) -> &[D] {
        &self.data
    }

    /// 子ノードがあるかを判定
    pub fn has_children(&self) -> bool {
        self.children.is_some()
    }

    /// ノードを8つの子ノードに分割
    ///
    /// 既に分割済みの場合は何もしません。
    pub fn subdivide(&mut self) {
        if self.children.is_some() {
            return; // 既に分割済み
        }

        let center = self.bounds.center();
        let min = self.bounds.min();
        let max = self.bounds.max();

        // 8つの子ノードの境界ボックスを作成
        // ビット演算に基づく配置: (z_bit << 2) | (y_bit << 1) | x_bit
        let child_bounds = [
            // 0: (x_min, y_min, z_min) ~ (x_mid, y_mid, z_mid)
            Aabb3D::new(min, center),
            // 1: (x_max, y_min, z_min) ~ (x_max, y_mid, z_mid)
            Aabb3D::new(
                Point3D::new(center.x(), min.y(), min.z()),
                Point3D::new(max.x(), center.y(), center.z()),
            ),
            // 2: (x_min, y_max, z_min) ~ (x_mid, y_max, z_mid)
            Aabb3D::new(
                Point3D::new(min.x(), center.y(), min.z()),
                Point3D::new(center.x(), max.y(), center.z()),
            ),
            // 3: (x_max, y_max, z_min) ~ (x_max, y_max, z_mid)
            Aabb3D::new(
                Point3D::new(center.x(), center.y(), min.z()),
                Point3D::new(max.x(), max.y(), center.z()),
            ),
            // 4: (x_min, y_min, z_max) ~ (x_mid, y_mid, z_max)
            Aabb3D::new(
                Point3D::new(min.x(), min.y(), center.z()),
                Point3D::new(center.x(), center.y(), max.z()),
            ),
            // 5: (x_max, y_min, z_max) ~ (x_max, y_mid, z_max)
            Aabb3D::new(
                Point3D::new(center.x(), min.y(), center.z()),
                Point3D::new(max.x(), center.y(), max.z()),
            ),
            // 6: (x_min, y_max, z_max) ~ (x_mid, y_max, z_max)
            Aabb3D::new(
                Point3D::new(min.x(), center.y(), center.z()),
                Point3D::new(center.x(), max.y(), max.z()),
            ),
            // 7: (x_max, y_max, z_max) ~ (x_max, y_max, z_max)
            Aabb3D::new(center, max),
        ];

        // 子ノードを作成
        let children = child_bounds.map(|bbox| OctreeNode::new(bbox, self.depth + 1));

        self.children = Some(Box::new(children));
    }

    /// データを追加（内部使用）
    pub(crate) fn add_data(&mut self, item: D) {
        self.data.push(item);
    }

    /// データをクリア（内部使用）
    #[allow(dead_code)]
    pub(crate) fn clear_data(&mut self) {
        self.data.clear();
    }

    /// 子ノードへの可変参照を取得（内部使用）
    #[allow(dead_code)]
    pub(crate) fn children_mut(&mut self) -> Option<&mut [OctreeNode<T, D>; 8]> {
        self.children.as_deref_mut()
    }

    /// 子ノードへの参照を取得
    pub fn children(&self) -> Option<&[OctreeNode<T, D>; 8]> {
        self.children.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let node: OctreeNode<f64, i32> = OctreeNode::new(bbox, 0);

        assert_eq!(node.depth(), 0);
        assert_eq!(node.data().len(), 0);
        assert!(!node.has_children());
    }

    #[test]
    fn test_subdivide() {
        let bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut node: OctreeNode<f64, i32> = OctreeNode::new(bbox, 0);

        assert!(!node.has_children());

        node.subdivide();

        assert!(node.has_children());
        let children = node.children().unwrap();
        assert_eq!(children.len(), 8);

        // 各子ノードの深さをチェック
        for child in children.iter() {
            assert_eq!(child.depth(), 1);
        }

        // 子ノード0の境界をチェック（(0,0,0) ~ (5,5,5)）
        let child0_bounds = children[0].bounds();
        assert_eq!(child0_bounds.min().x(), 0.0);
        assert_eq!(child0_bounds.min().y(), 0.0);
        assert_eq!(child0_bounds.min().z(), 0.0);
        assert_eq!(child0_bounds.max().x(), 5.0);
        assert_eq!(child0_bounds.max().y(), 5.0);
        assert_eq!(child0_bounds.max().z(), 5.0);
    }

    #[test]
    fn test_subdivide_idempotent() {
        let bbox = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut node: OctreeNode<f64, i32> = OctreeNode::new(bbox, 0);

        node.subdivide();
        let children_count = node.children().unwrap().len();

        // 再度分割を試みても変化なし
        node.subdivide();
        assert_eq!(node.children().unwrap().len(), children_count);
    }
}
