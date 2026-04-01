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
//! ## パフォーマンス特性
//!
//! ### 計算量
//!
//! - **挿入**: O(log n) - 平均ケース
//! - **範囲検索**: O(log n + k) - k は結果数
//! - **最近傍探索**: O(log n) - 枝刈り最適化により
//! - **衝突判定**: O(n log n) - 総当たり O(n²) から約99%削減
//!
//! ### パラメータ選択ガイド
//!
//! #### max_depth（最大分割深さ）
//!
//! - **推奨値**: 6-10
//! - **小さすぎる場合**: 分割が不十分で検索効率が低下
//! - **大きすぎる場合**: メモリ使用量増大、オーバーヘッド増加
//! - **目安**: データ数1000で深さ8、10000で深さ10
//!
//! #### max_items（ノードあたり最大要素数）
//!
//! - **推奨値**: 8-16
//! - **小さすぎる場合**: 過度な分割でメモリ・処理オーバーヘッド
//! - **大きすぎる場合**: 線形探索コストの増大
//! - **目安**: 密集度が高い場合は小さめ、疎な場合は大きめ
//!
//! ## モジュール
//!
//! - [`Octree`] - 汎用空間分割データ構造（データ挿入・検索）
//! - [`voxel`] - ボリューム占有更新用ボクセルOctree（領域除去シミュレーション）
//!
//! ## 使用例
//!
//! ### 汎用Octree（衝突判定・検索）
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
//! ### 衝突判定の高速化
//!
//! ```rust,ignore
//! // 1000要素で総当たり判定 O(n²) → Octree O(n log n)
//! // 判定回数: 499,500回 → 約5,000回（99%削減）
//!
//! let mut octree = Octree::new(scene_bbox, 8, 10);
//!
//! // 全形状をOctreeに挿入
//! for shape in &shapes {
//!     octree.insert(shape.clone());
//! }
//!
//! // 各形状の周辺候補のみチェック
//! for shape in &shapes {
//!     let candidates = octree.query_region(&shape.bounding_box());
//!     for candidate in candidates {
//!         if shape.intersects(candidate) {
//!             // 衝突処理
//!         }
//!     }
//! }
//! ```
//!
//! ### ボクセルOctree（切削シミュレーション）
//!
//! ```rust,ignore
//! use geo_algorithms::octree::voxel::VoxelOctree;
//! use geo_core::{Aabb3D, Point3D};
//!
//! // ワーク全体を表すボクセルOctree（100x100x100mm）
//! let work_bounds = Aabb3D::new(
//!     Point3D::new(0.0, 0.0, 0.0),
//!     Point3D::new(100.0, 100.0, 100.0)
//! );
//! let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
//!
//! // 掃引形状が通過した領域を除去
//! let tool_region = Aabb3D::new(
//!     Point3D::new(10.0, 10.0, 0.0),
//!     Point3D::new(20.0, 20.0, 50.0)
//! );
//! voxel_tree.remove_material_box(&tool_region);
//!
//! // 残存材料の体積を計算
//! let remaining = voxel_tree.remaining_volume();
//! println!("残存体積: {} mm³", remaining);
//!
//! // 未除去領域検出
//! let target_shape = Aabb3D::new(
//!     Point3D::new(15.0, 15.0, 5.0),
//!     Point3D::new(85.0, 85.0, 95.0)
//! );
//! let undercuts = voxel_tree.detect_undercut(&target_shape);
//! if !undercuts.is_empty() {
//!     eprintln!("警告: {} 箇所の未除去領域を検出", undercuts.len());
//! }
//! ```
//!
//! ## ベストプラクティス
//!
//! ### データの前処理
//!
//! - 境界ボックスは事前計算してキャッシュする
//! - Octree境界は全データを含む最小境界に設定
//!
//! ### メモリ効率
//!
//! - 不要になったら `clear()` でメモリ解放
//! - Clone コストの高いデータは参照カウント（Rc/Arc）を検討
//!
//! ### パフォーマンス
//!
//! - 静的シーンは一度構築して再利用
//! - 動的シーンは増分更新を検討（または再構築）
//! - 並列処理時は Octree のクローンまたは Arc で共有

use geo_contracts::Scalar;
use geo_core::{Aabb3D, Point3D};

mod core_impl;
pub mod fixtures;
mod insert_impl;
mod nearest_impl;
pub mod node;
mod parallel_impl;
mod query_impl;
pub mod tolerance;
pub mod voxel;

pub use node::OctreeNode;
pub use tolerance::OctreeTolerance;
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

    /// Octree専用トレランス設定
    tolerance: OctreeTolerance<T>,
}

#[cfg(test)]
mod tests;
