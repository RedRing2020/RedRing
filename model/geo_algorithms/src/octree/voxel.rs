//! ボクセルOctree（領域除去シミュレーション用）
//!
//! このモジュールは、領域除去シミュレーションのためのボクセルベースOctreeを提供します。
//!
//! ## 概要
//!
//! ボクセルOctreeは、3D空間を立方体セル（ボクセル）に分割し、
//! 各セルの占有状態（Solid/Empty/Mixed）を管理します。
//!
//! ## 主要な機能
//!
//! - **領域除去**: 除去形状による占有領域の更新
//! - **適応的分割**: Mixed状態のセルを細分化
//! - **体積計算**: 残存体積を高速計算
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
//! // 掃引形状が通過した領域を除去（AABB近似）
//! let tool_region = Aabb3D::new(
//!     Point3D::new(10.0, 10.0, 0.0),
//!     Point3D::new(20.0, 20.0, 50.0)
//! );
//! voxel_tree.remove_material_box(&tool_region);
//!
//! // 残存体積を計算
//! let remaining = voxel_tree.remaining_volume();
//! println!("残存体積: {} mm³", remaining);
//! ```

use crate::{Arc3D, LineSegment3D};
use geo_core::{Aabb3D, Point3D};
use geo_foundation::{LineSegment3DCollisionDetection, Scalar};

/// ボクセルの占有状態
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
/// - `Solid`: 完全に占有されている
/// - `Mixed`: 部分的に占有されている（細分化が必要）
/// - `Empty`: 占有が完全に除去されている
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelState {
    /// 占有あり
    ///
    /// このセル全体が占有されています。
    Solid,

    /// 占有除去済み
    ///
    /// このセル内の占有は完全に除去されています。
    Empty,

    /// 部分的に除去（要細分化）
    ///
    /// このセル内で一部の占有が除去されています。
    /// より詳細な計算には、さらなる細分化（subdivide）が必要です。
    Mixed,
}

/// ボクセルOctreeのノード
///
/// 各ノードは3D境界ボックスと占有状態を持ちます。
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

    /// 占有状態
    state: VoxelState,

    /// 子ノード（Mixed状態の場合のみ存在）
    ///
    /// 8つの子ノードを持つ配列。
    /// インデックスは (z << 2) | (y << 1) | x で計算されます。
    children: Option<Box<[VoxelNode<T>; 8]>>,
}

/// ボクセルOctree（領域除去シミュレーション用）
///
/// 3D空間をボクセル（立方体セル）に分割し、占有状態を管理します。
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
/// // 領域除去
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

mod node_impl;
mod tree_impl;

#[cfg(test)]
mod tests;
