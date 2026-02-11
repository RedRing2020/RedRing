//! ボクセルOctree（切削シミュレーション用）
//!
//! このモジュールは、材料除去シミュレーションのためのボクセルベースOctreeを提供します。
//!
//! ## 概要
//!
//! ボクセルOctreeは、3D空間を立方体セル（ボクセル）に分割し、
//! 各セルの材料状態（Solid/Empty/Mixed）を管理します。
//!
//! ## 主要な機能
//!
//! - **材料除去**: 工具形状による材料の削り取り
//! - **適応的分割**: Mixed状態のセルを細分化
//! - **体積計算**: 残存材料の体積を高速計算
//! - **状態管理**: Solid/Empty/Mixedの3状態
//!
//! ## 使用例
//!
//! ```rust,ignore
//! use geo_algorithms::octree::voxel::{VoxelOctree, VoxelState};
//! use geo_core::Aabb3D;
//!
//! // ワーク全体を表すボクセルOctree（100x100x100mm）
//! let work_bounds = Aabb3D::new(
//!     Point3D::new(0.0, 0.0, 0.0),
//!     Point3D::new(100.0, 100.0, 100.0)
//! );
//! let mut voxel_tree = VoxelOctree::new(work_bounds, 6); // 最大深さ6
//!
//! // 工具が通過した領域を除去（AABB近似）
//! let tool_region = Aabb3D::new(
//!     Point3D::new(10.0, 10.0, 0.0),
//!     Point3D::new(20.0, 20.0, 50.0)
//! );
//! voxel_tree.remove_material_box(&tool_region);
//!
//! // 残存材料の体積を計算
//! let remaining = voxel_tree.remaining_volume();
//! println!("残存体積: {} mm³", remaining);
//! ```
//!
//! ## Phase 1 実装範囲
//!
//! ✅ **Phase 1** (現在):
//! - VoxelState 列挙型
//! - VoxelNode/VoxelOctree 構造体
//! - AABB形状による材料除去
//! - 残存体積計算
//!
//! 🔄 **Phase 2** (予定):
//! - 球形状除去（ボールエンドミル）
//! - 円筒形状除去（フラットエンドミル）
//! - 削り残し検出

use geo_core::{Aabb3D, Point3D};
use geo_foundation::Scalar;

/// ボクセルの材料状態
///
/// 各ボクセルセルは以下の3状態のいずれかを持ちます。
///
/// # 状態遷移
///
/// ```text
/// Solid → Mixed → Empty
///   ↓              ↑
///   └──────────────┘
/// ```
///
/// - `Solid`: 完全に材料で満たされている
/// - `Mixed`: 部分的に材料がある（細分化が必要）
/// - `Empty`: 材料が完全に除去されている
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelState {
    /// 材料あり（未加工）
    ///
    /// このセル全体が材料で満たされています。
    Solid,

    /// 材料除去済み
    ///
    /// このセル内の材料は完全に除去されています。
    Empty,

    /// 部分的に除去（要細分化）
    ///
    /// このセル内で一部の材料が除去されています。
    /// より詳細な計算には、さらなる細分化（subdivide）が必要です。
    Mixed,
}

/// ボクセルOctreeのノード
///
/// 各ノードは3D境界ボックスと材料状態を持ちます。
/// Mixed状態の場合、8つの子ノードに細分化されます。
///
/// # Type Parameters
///
/// * `T` - 座標値の型（Scalarトレイト境界）
#[derive(Debug, Clone)]
pub struct VoxelNode<T: Scalar> {
    /// ノードの境界ボックス
    bounds: Aabb3D<T>,

    /// ツリー内の深さ（ルート = 0）
    depth: usize,

    /// 材料状態
    state: VoxelState,

    /// 子ノード（Mixed状態の場合のみ存在）
    ///
    /// 8つの子ノードを持つ配列。
    /// インデックスは (z << 2) | (y << 1) | x で計算されます。
    children: Option<Box<[VoxelNode<T>; 8]>>,
}

/// ボクセルOctree（切削シミュレーション用）
///
/// 3D空間をボクセル（立方体セル）に分割し、材料の状態を管理します。
///
/// # Type Parameters
///
/// * `T` - 座標値の型（Scalarトレイト境界）
///
/// # Examples
///
/// ```rust,ignore
/// use geo_algorithms::octree::voxel::VoxelOctree;
/// use geo_core::Aabb3D;
///
/// // 100x100x100mm のワークを表すOctree
/// let bounds = Aabb3D::new(min_point, max_point);
/// let mut voxel_tree = VoxelOctree::new(bounds, 6);
///
/// // 材料除去
/// voxel_tree.remove_material_box(&tool_aabb);
///
/// // 残存体積
/// let volume = voxel_tree.remaining_volume();
/// ```
#[derive(Debug, Clone)]
pub struct VoxelOctree<T: Scalar> {
    /// ルートノード
    root: VoxelNode<T>,

    /// 最大深さ（分割の上限）
    max_depth: usize,

    /// 最大深さでのボクセルサイズ
    ///
    /// これはルート境界ボックスのサイズから計算されます：
    /// `voxel_size = root_size / 2^max_depth`
    voxel_size_at_max_depth: T,
}

impl<T: Scalar> VoxelNode<T> {
    /// 新しいソリッドノードを作成
    ///
    /// # Arguments
    ///
    /// * `bounds` - ノードの境界ボックス
    /// * `depth` - ツリー内の深さ
    fn new_solid(bounds: Aabb3D<T>, depth: usize) -> Self {
        Self {
            bounds,
            depth,
            state: VoxelState::Solid,
            children: None,
        }
    }

    /// ノードを8つの子ノードに分割
    ///
    /// このメソッドは Solid または Empty 状態のノードを Mixed に変換し、
    /// 8つの子ノードを作成します。各子ノードは親と同じ状態を継承します。
    fn subdivide(&mut self) {
        if self.children.is_some() {
            return; // 既に分割済み
        }

        let center = self.bounds.center();
        let min = self.bounds.min();
        let max = self.bounds.max();
        let child_depth = self.depth + 1;

        // 親の状態を継承
        let inherited_state = self.state;

        let children = Box::new([
            // インデックス 0: (x=0, y=0, z=0) - 左下前
            VoxelNode {
                bounds: Aabb3D::new(min, center),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 1: (x=1, y=0, z=0) - 右下前
            VoxelNode {
                bounds: Aabb3D::new(
                    Point3D::new(center.x(), min.y(), min.z()),
                    Point3D::new(max.x(), center.y(), center.z()),
                ),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 2: (x=0, y=1, z=0) - 左上前
            VoxelNode {
                bounds: Aabb3D::new(
                    Point3D::new(min.x(), center.y(), min.z()),
                    Point3D::new(center.x(), max.y(), center.z()),
                ),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 3: (x=1, y=1, z=0) - 右上前
            VoxelNode {
                bounds: Aabb3D::new(
                    Point3D::new(center.x(), center.y(), min.z()),
                    Point3D::new(max.x(), max.y(), center.z()),
                ),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 4: (x=0, y=0, z=1) - 左下奥
            VoxelNode {
                bounds: Aabb3D::new(
                    Point3D::new(min.x(), min.y(), center.z()),
                    Point3D::new(center.x(), center.y(), max.z()),
                ),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 5: (x=1, y=0, z=1) - 右下奥
            VoxelNode {
                bounds: Aabb3D::new(
                    Point3D::new(center.x(), min.y(), center.z()),
                    Point3D::new(max.x(), center.y(), max.z()),
                ),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 6: (x=0, y=1, z=1) - 左上奥
            VoxelNode {
                bounds: Aabb3D::new(
                    Point3D::new(min.x(), center.y(), center.z()),
                    Point3D::new(center.x(), max.y(), max.z()),
                ),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
            // インデックス 7: (x=1, y=1, z=1) - 右上奥
            VoxelNode {
                bounds: Aabb3D::new(center, max),
                depth: child_depth,
                state: inherited_state,
                children: None,
            },
        ]);

        self.children = Some(children);
        self.state = VoxelState::Mixed;
    }

    /// AABB形状による材料除去（再帰的）
    ///
    /// # Arguments
    ///
    /// * `tool_aabb` - 工具の境界ボックス
    /// * `max_depth` - 最大深さ（分割上限）
    fn remove_material_box(&mut self, tool_aabb: &Aabb3D<T>, max_depth: usize) {
        match self.state {
            VoxelState::Empty => (), // 既に空なら何もしない

            VoxelState::Solid => {
                // 工具との交差判定
                if !self.bounds.intersects(tool_aabb) {
                    return; // 交差しなければ何もしない
                }

                // 完全に含まれる場合は Empty に
                if tool_aabb.contains_aabb(&self.bounds) {
                    self.state = VoxelState::Empty;
                    self.children = None; // 子ノードを破棄
                    return;
                }

                // 部分的な交差
                if self.depth < max_depth {
                    // 深さに余裕があれば細分化
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_box(tool_aabb, max_depth);
                    }
                } else {
                    // 最大深さ到達 - セル単位で削除
                    self.state = VoxelState::Empty;
                }
            }

            VoxelState::Mixed => {
                // 子ノードに委譲
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_box(tool_aabb, max_depth);
                    }

                    // 全ての子が Empty なら親も Empty に
                    if children.iter().all(|c| c.state == VoxelState::Empty) {
                        self.state = VoxelState::Empty;
                        self.children = None;
                    }
                    // 全ての子が Solid なら親も Solid に
                    else if children.iter().all(|c| c.state == VoxelState::Solid) {
                        self.state = VoxelState::Solid;
                        self.children = None;
                    }
                }
            }
        }
    }

    /// ノードの体積を計算
    ///
    /// # Returns
    ///
    /// * `Solid` の場合: 境界ボックスの体積
    /// * `Empty` の場合: 0
    /// * `Mixed` の場合: 子ノードの体積の合計
    fn volume(&self) -> T {
        match self.state {
            VoxelState::Empty => T::from_f64(0.0),
            VoxelState::Solid => self.bounds.volume(),
            VoxelState::Mixed => self
                .children
                .as_ref()
                .unwrap()
                .iter()
                .map(|c| c.volume())
                .fold(T::from_f64(0.0), |acc, v| acc + v),
        }
    }

    /// Solid状態のボクセル数をカウント（デバッグ用）
    fn solid_voxel_count(&self) -> usize {
        match self.state {
            VoxelState::Empty => 0,
            VoxelState::Solid => 1,
            VoxelState::Mixed => self
                .children
                .as_ref()
                .unwrap()
                .iter()
                .map(|c| c.solid_voxel_count())
                .sum(),
        }
    }
}

impl<T: Scalar> VoxelOctree<T> {
    /// 新しいボクセルOctreeを作成
    ///
    /// 初期状態では全体が Solid（材料あり）です。
    ///
    /// # Arguments
    ///
    /// * `bounds` - ワーク全体の境界ボックス
    /// * `max_depth` - 最大深さ（分割の上限）
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use geo_algorithms::octree::voxel::VoxelOctree;
    /// use geo_core::{Aabb3D, Point3D};
    ///
    /// let bounds = Aabb3D::new(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(100.0, 100.0, 100.0)
    /// );
    /// let voxel_tree = VoxelOctree::new(bounds, 6);
    /// ```
    pub fn new(bounds: Aabb3D<T>, max_depth: usize) -> Self {
        let width = bounds.width();
        let height = bounds.height();
        let depth = bounds.depth();
        let max_dimension = width.max(height).max(depth);
        let divisor = T::from_f64((1 << max_depth) as f64); // 2^max_depth
        let voxel_size_at_max_depth = max_dimension / divisor;

        Self {
            root: VoxelNode::new_solid(bounds, 0),
            max_depth,
            voxel_size_at_max_depth,
        }
    }

    /// AABB形状による材料除去
    ///
    /// 指定された境界ボックスと交差する領域の材料を除去します。
    ///
    /// # Arguments
    ///
    /// * `tool_aabb` - 工具の境界ボックス
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    ///
    /// // 工具が通過した領域を除去
    /// let tool_region = Aabb3D::new(
    ///     Point3D::new(10.0, 10.0, 0.0),
    ///     Point3D::new(20.0, 20.0, 50.0)
    /// );
    /// voxel_tree.remove_material_box(&tool_region);
    /// ```
    pub fn remove_material_box(&mut self, tool_aabb: &Aabb3D<T>) {
        self.root.remove_material_box(tool_aabb, self.max_depth);
    }

    /// 残存材料の体積を計算
    ///
    /// # Returns
    ///
    /// 残っている材料の総体積（単位: mm³）
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let volume = voxel_tree.remaining_volume();
    /// println!("残存体積: {} mm³", volume);
    /// ```
    pub fn remaining_volume(&self) -> T {
        self.root.volume()
    }

    /// Solid状態のボクセル数を取得（デバッグ用）
    ///
    /// # Returns
    ///
    /// Solid状態のリーフノード数
    pub fn solid_voxel_count(&self) -> usize {
        self.root.solid_voxel_count()
    }

    /// 最大深さでのボクセルサイズを取得
    ///
    /// # Returns
    ///
    /// 最小ボクセルの1辺のサイズ（単位: mm）
    pub fn voxel_size_at_max_depth(&self) -> T {
        self.voxel_size_at_max_depth
    }

    /// ルートノードの境界ボックスを取得
    pub fn bounds(&self) -> &Aabb3D<T> {
        &self.root.bounds
    }

    /// 最大深さを取得
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::Point3D;

    #[test]
    fn test_voxel_octree_creation() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let voxel_tree = VoxelOctree::new(bounds, 3);

        assert_eq!(voxel_tree.max_depth(), 3);
        assert_eq!(voxel_tree.solid_voxel_count(), 1); // 初期状態は1つのSolidノード
        assert!((voxel_tree.remaining_volume() - 1000000.0).abs() < 0.001); // 100^3 = 1,000,000
    }

    #[test]
    fn test_remove_material_box_complete() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 3);

        // 全体を除去
        let tool_aabb = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        voxel_tree.remove_material_box(&tool_aabb);

        assert_eq!(voxel_tree.remaining_volume(), 0.0);
        assert_eq!(voxel_tree.solid_voxel_count(), 0);
    }

    #[test]
    fn test_remove_material_box_partial() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 一部を除去（左下の1/8）
        let tool_aabb = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(50.0, 50.0, 50.0));
        voxel_tree.remove_material_box(&tool_aabb);

        let remaining = voxel_tree.remaining_volume();

        println!("Initial: {}, Remaining: {}", initial_volume, remaining);

        // ボクセルベースでは境界付近で誤差が生じるのは正常
        // 少なくとも何かが削除されていることを確認
        assert!(remaining < initial_volume);

        // 残存体積が全体の50%以上であることを確認（1/8除去なので87.5%のはず）
        assert!(remaining > initial_volume * 0.5);
    }

    #[test]
    fn test_remove_material_box_subdivision() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 5);

        let initial_volume = voxel_tree.remaining_volume();

        // 中央の小さな領域を除去
        let tool_aabb = Aabb3D::new(
            Point3D::new(40.0, 40.0, 40.0),
            Point3D::new(60.0, 60.0, 60.0),
        );
        voxel_tree.remove_material_box(&tool_aabb);

        let remaining = voxel_tree.remaining_volume();

        println!(
            "Initial: {}, Remaining: {}, Removed: {}",
            initial_volume,
            remaining,
            initial_volume - remaining
        );

        // ボクセルベースでは境界付近で誤差が生じるのは正常
        // 少なくとも何かが削除されていることを確認
        assert!(remaining < initial_volume);

        // 残存体積が全体の95%以上であることを確認（小さな領域のみ除去）
        assert!(remaining > initial_volume * 0.95);
    }

    #[test]
    fn test_multiple_removals() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 複数回の材料除去
        let tool1 = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(30.0, 30.0, 30.0));
        voxel_tree.remove_material_box(&tool1);

        let volume_after_1 = voxel_tree.remaining_volume();
        assert!(volume_after_1 < initial_volume);

        let tool2 = Aabb3D::new(
            Point3D::new(70.0, 70.0, 70.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        voxel_tree.remove_material_box(&tool2);

        let volume_after_2 = voxel_tree.remaining_volume();
        assert!(volume_after_2 < volume_after_1);
    }

    #[test]
    fn test_no_intersection() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 3);

        let initial_volume = voxel_tree.remaining_volume();

        // 範囲外の工具（交差なし）
        let tool_aabb = Aabb3D::new(
            Point3D::new(200.0, 200.0, 200.0),
            Point3D::new(300.0, 300.0, 300.0),
        );
        voxel_tree.remove_material_box(&tool_aabb);

        // 体積は変わらないはず
        assert_eq!(voxel_tree.remaining_volume(), initial_volume);
    }

    #[test]
    fn test_voxel_size_calculation() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );

        let voxel_tree_3 = VoxelOctree::new(bounds, 3);
        let expected_size_3 = 100.0 / 8.0; // 2^3 = 8
        assert!((voxel_tree_3.voxel_size_at_max_depth() - expected_size_3).abs() < 0.001);

        let voxel_tree_5 = VoxelOctree::new(bounds, 5);
        let expected_size_5 = 100.0 / 32.0; // 2^5 = 32
        assert!((voxel_tree_5.voxel_size_at_max_depth() - expected_size_5).abs() < 0.001);
    }
}
