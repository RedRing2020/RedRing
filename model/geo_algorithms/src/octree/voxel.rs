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

use geo_commons::metrics::distance::line_segment_to_aabb_distance;
use geo_core::{Aabb3D, Point3D};
use geo_foundation::Scalar;
use geo_primitives::LineSegment3D;

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

    /// 線分を中心軸とした円柱（カプセル）領域で材料除去
    ///
    /// # Arguments
    ///
    /// * `segment` - 中心軸となる線分
    /// * `radius` - 円柱の半径
    /// * `max_depth` - 最大深さ（分割上限）
    fn remove_material_capsule(&mut self, segment: &LineSegment3D<T>, radius: T, max_depth: usize) {
        match self.state {
            VoxelState::Empty => (), // 既に空なら何もしない

            VoxelState::Solid => {
                // 線分とAABBの距離を計算
                let min = self.bounds.min();
                let max = self.bounds.max();
                let distance = line_segment_to_aabb_distance(
                    (
                        segment.start().x(),
                        segment.start().y(),
                        segment.start().z(),
                    ),
                    (segment.end().x(), segment.end().y(), segment.end().z()),
                    (min.x(), min.y(), min.z()),
                    (max.x(), max.y(), max.z()),
                );

                // カプセル範囲外なら何もしない（枝刈り）
                if distance > radius {
                    return;
                }

                // AABBが完全にカプセル内部にあるか判定
                // AABBの8頂点すべてがカプセル内部にあれば完全包含
                if self.is_aabb_inside_capsule(segment, radius) {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }

                // 部分的な交差
                if self.depth < max_depth {
                    // 深さに余裕があれば細分化
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_capsule(segment, radius, max_depth);
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
                        child.remove_material_capsule(segment, radius, max_depth);
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

    /// AABBがカプセル内部に完全に含まれるか判定
    ///
    /// # Arguments
    ///
    /// * `segment` - カプセルの中心軸
    /// * `radius` - カプセルの半径
    ///
    /// # Returns
    ///
    /// AABBの8頂点すべてがカプセル内部にあれば `true`
    fn is_aabb_inside_capsule(&self, segment: &LineSegment3D<T>, radius: T) -> bool {
        let min = self.bounds.min();
        let max = self.bounds.max();

        // AABBの8頂点を生成
        let vertices = [
            (min.x(), min.y(), min.z()),
            (max.x(), min.y(), min.z()),
            (min.x(), max.y(), min.z()),
            (max.x(), max.y(), min.z()),
            (min.x(), min.y(), max.z()),
            (max.x(), min.y(), max.z()),
            (min.x(), max.y(), max.z()),
            (max.x(), max.y(), max.z()),
        ];

        // 全頂点が半径内にあるかチェック
        for &vertex in &vertices {
            let distance = self.point_to_segment_distance(vertex, segment);
            if distance > radius {
                return false;
            }
        }

        true
    }

    /// 点から線分への最短距離を計算
    ///
    /// # Arguments
    ///
    /// * `point` - 計算対象の点 (x, y, z)
    /// * `segment` - 線分
    ///
    /// # Returns
    ///
    /// 点から線分への最短距離
    fn point_to_segment_distance(&self, point: (T, T, T), segment: &LineSegment3D<T>) -> T {
        let (px, py, pz) = point;
        let sx = segment.start().x();
        let sy = segment.start().y();
        let sz = segment.start().z();
        let ex = segment.end().x();
        let ey = segment.end().y();
        let ez = segment.end().z();

        // 線分の方向ベクトル
        let dx = ex - sx;
        let dy = ey - sy;
        let dz = ez - sz;

        // 点から始点へのベクトル
        let to_px = px - sx;
        let to_py = py - sy;
        let to_pz = pz - sz;

        // パラメータ t を計算
        let len_sq = dx * dx + dy * dy + dz * dz;
        let t = if len_sq <= T::EPSILON {
            T::ZERO
        } else {
            let dot = to_px * dx + to_py * dy + to_pz * dz;
            (dot / len_sq).clamp(T::ZERO, T::ONE)
        };

        // 線分上の最近点
        let closest_x = sx + dx * t;
        let closest_y = sy + dy * t;
        let closest_z = sz + dz * t;

        // 距離を計算
        let diff_x = px - closest_x;
        let diff_y = py - closest_y;
        let diff_z = pz - closest_z;
        (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt()
    }

    /// Z軸方向の円柱領域で材料除去（高速版）
    ///
    /// 工具軸がZ軸に平行な場合の最適化実装。
    /// XY平面での2D円-矩形距離計算に簡略化することで高速化。
    ///
    /// # Arguments
    ///
    /// * `center_x` - 工具中心のX座標
    /// * `center_y` - 工具中心のY座標
    /// * `z_start` - Z方向の開始座標
    /// * `z_end` - Z方向の終了座標
    /// * `radius` - 円柱の半径
    /// * `max_depth` - 最大深さ（分割上限）
    ///
    /// # Performance
    ///
    /// 汎用版の`remove_material_capsule`と比較して：
    /// - 線分サンプリング不要
    /// - 8頂点チェック不要
    /// - XY平面での2D計算のみ
    /// - 約3-5倍高速
    fn remove_material_capsule_z_axis(
        &mut self,
        center_x: T,
        center_y: T,
        z_start: T,
        z_end: T,
        radius: T,
        max_depth: usize,
    ) {
        match self.state {
            VoxelState::Empty => (), // 既に空なら何もしない

            VoxelState::Solid => {
                let min = self.bounds.min();
                let max = self.bounds.max();

                // 1. Z方向の範囲チェック（高速枝刈り）
                let z_min_seg = z_start.min(z_end);
                let z_max_seg = z_start.max(z_end);

                if z_max_seg < min.z() || z_min_seg > max.z() {
                    return; // Z方向で交差なし
                }

                // 2. XY平面での円-矩形距離計算
                let distance_2d = self.circle_to_aabb_2d_distance(center_x, center_y, &min, &max);

                // カプセル範囲外なら何もしない（枝刈り）
                if distance_2d > radius {
                    return;
                }

                // 3. AABBが完全にカプセル内部にあるか判定
                if self
                    .is_aabb_inside_z_axis_capsule(center_x, center_y, z_min_seg, z_max_seg, radius)
                {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }

                // 4. 部分的な交差
                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_capsule_z_axis(
                            center_x, center_y, z_start, z_end, radius, max_depth,
                        );
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
                        child.remove_material_capsule_z_axis(
                            center_x, center_y, z_start, z_end, radius, max_depth,
                        );
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

    /// XY平面での円-矩形(AABB)間の2D距離を計算
    ///
    /// # Arguments
    ///
    /// * `center_x` - 円の中心X座標
    /// * `center_y` - 円の中心Y座標
    /// * `aabb_min` - AABBの最小座標
    /// * `aabb_max` - AABBの最大座標
    ///
    /// # Returns
    ///
    /// XY平面での円中心から矩形までの最短距離
    fn circle_to_aabb_2d_distance(
        &self,
        center_x: T,
        center_y: T,
        aabb_min: &Point3D<T>,
        aabb_max: &Point3D<T>,
    ) -> T {
        // AABBの最近点をXY平面で計算
        let closest_x = center_x.clamp(aabb_min.x(), aabb_max.x());
        let closest_y = center_y.clamp(aabb_min.y(), aabb_max.y());

        let dx = center_x - closest_x;
        let dy = center_y - closest_y;

        (dx * dx + dy * dy).sqrt()
    }

    /// AABBがZ軸カプセル内部に完全に含まれるか判定
    ///
    /// # Arguments
    ///
    /// * `center_x` - 円柱中心のX座標
    /// * `center_y` - 円柱中心のY座標
    /// * `z_min` - Z方向の最小座標
    /// * `z_max` - Z方向の最大座標
    /// * `radius` - 円柱の半径
    ///
    /// # Returns
    ///
    /// AABBが完全に円柱内部にあれば `true`
    fn is_aabb_inside_z_axis_capsule(
        &self,
        center_x: T,
        center_y: T,
        z_min: T,
        z_max: T,
        radius: T,
    ) -> bool {
        let min = self.bounds.min();
        let max = self.bounds.max();

        // 1. Z方向の完全包含チェック
        if min.z() < z_min || max.z() > z_max {
            return false;
        }

        // 2. XY平面での4頂点が全て半径内にあるかチェック
        let corners_2d = [
            (min.x(), min.y()),
            (max.x(), min.y()),
            (min.x(), max.y()),
            (max.x(), max.y()),
        ];

        for &(x, y) in &corners_2d {
            let dx = x - center_x;
            let dy = y - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > radius {
                return false;
            }
        }

        true
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

    /// 指定された領域の外側にあるSolidボクセルを収集（削り残し検出用）
    ///
    /// # Arguments
    ///
    /// * `target_region` - 目的形状の境界ボックス
    /// * `undercut_voxels` - 検出された削り残しボクセルを格納するVec
    ///
    /// # Note
    ///
    /// このノードの境界ボックスが `target_region` と交差しない場合、
    /// 全てのSolidボクセルが削り残しとして収集されます。
    fn collect_solid_voxels_outside(&self, target_region: &Aabb3D<T>, undercut_voxels: &mut Vec<Aabb3D<T>>) {
        match self.state {
            VoxelState::Empty => {
                // 空のボクセルは削り残しではない
            }
            VoxelState::Solid => {
                // Solidボクセルが目的領域の外側にあるかチェック
                if !self.bounds.intersects(target_region) {
                    // 完全に領域外 = 削り残し
                    undercut_voxels.push(self.bounds.clone());
                } else if !target_region.contains_aabb(&self.bounds) {
                    // 部分的に外側にある可能性がある
                    // リーフノードなのでこのボクセル全体を削り残しとして扱う
                    undercut_voxels.push(self.bounds.clone());
                }
                // target_region に完全に含まれる場合は削り残しではない
            }
            VoxelState::Mixed => {
                // 子ノードを再帰的に探索
                if let Some(ref children) = self.children {
                    for child in children.iter() {
                        child.collect_solid_voxels_outside(target_region, undercut_voxels);
                    }
                }
            }
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

    /// 線分を中心軸とした円柱（カプセル）領域で材料除去
    ///
    /// 線分に沿って指定した半径の円柱領域内の材料を除去します。
    ///
    /// # Arguments
    ///
    /// * `segment` - 中心軸となる線分
    /// * `radius` - 円柱の半径
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use geo_primitives::LineSegment3D;
    /// use geo_core::Point3D;
    ///
    /// let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    ///
    /// // 線分経路に沿って材料除去
    /// let segment = LineSegment3D::new(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(50.0, 50.0, 50.0)
    /// );
    /// voxel_tree.remove_material_capsule(&segment, 5.0); // 半径5mm
    /// ```
    pub fn remove_material_capsule(&mut self, segment: &LineSegment3D<T>, radius: T) {
        self.root
            .remove_material_capsule(segment, radius, self.max_depth);
    }

    /// Z軸方向の円柱領域で材料除去（高速版）
    ///
    /// 工具軸がZ軸に平行な場合の最適化実装。
    /// 汎用版`remove_material_capsule`より3-5倍高速。
    ///
    /// # Arguments
    ///
    /// * `center_x` - 工具中心のX座標
    /// * `center_y` - 工具中心のY座標
    /// * `z_start` - Z方向の開始座標
    /// * `z_end` - Z方向の終了座標
    /// * `radius` - 円柱の半径
    ///
    /// # Performance
    ///
    /// XY平面での2D円-矩形距離計算に簡略化：
    /// - サンプリング不要
    /// - 8頂点チェックが4頂点に削減
    /// - Z方向は単純な範囲チェックのみ
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use geo_core::Point3D;
    ///
    /// let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    ///
    /// // Z軸方向に材料除去
    /// voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 5.0);
    /// ```
    pub fn remove_material_z_axis(
        &mut self,
        center_x: T,
        center_y: T,
        z_start: T,
        z_end: T,
        radius: T,
    ) {
        self.root.remove_material_capsule_z_axis(
            center_x,
            center_y,
            z_start,
            z_end,
            radius,
            self.max_depth,
        );
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

    /// 削り残し検出
    ///
    /// 目的形状の境界ボックスの外側に残っているSolidボクセルを検出します。
    /// これは、工具が到達できなかった領域や、意図しない材料の残存を示します。
    ///
    /// # Arguments
    ///
    /// * `target_region` - 目的形状の境界ボックス（この内側にあるべき領域）
    ///
    /// # Returns
    ///
    /// 削り残しとして検出されたボクセルの境界ボックスリスト。
    /// 空のリストは、削り残しが存在しないことを示します。
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use geo_algorithms::octree::voxel::VoxelOctree;
    /// use geo_core::{Aabb3D, Point3D};
    ///
    /// // ワークピース全体
    /// let work_bounds = Aabb3D::new(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(100.0, 100.0, 100.0)
    /// );
    /// let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    ///
    /// // 材料除去シミュレーション実行
    /// // ... remove_material_* 呼び出し ...
    ///
    /// // 目的形状（この範囲外の材料は削り残し）
    /// let target = Aabb3D::new(
    ///     Point3D::new(10.0, 10.0, 10.0),
    ///     Point3D::new(90.0, 90.0, 90.0)
    /// );
    ///
    /// // 削り残し検出
    /// let undercuts = voxel_tree.detect_undercut(&target);
    /// if !undercuts.is_empty() {
    ///     eprintln!("警告: {} 箇所の削り残しを検出", undercuts.len());
    /// }
    /// ```
    pub fn detect_undercut(&self, target_region: &Aabb3D<T>) -> Vec<Aabb3D<T>> {
        let mut undercut_voxels = Vec::new();
        self.root.collect_solid_voxels_outside(target_region, &mut undercut_voxels);
        undercut_voxels
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

    // ========================================================================
    // カプセル（線分+半径）材料除去テスト
    // ========================================================================

    #[test]
    fn test_capsule_removal_basic() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 中心を貫通する線分
        let segment = LineSegment3D::new(
            Point3D::new(50.0, 50.0, 0.0),
            Point3D::new(50.0, 50.0, 100.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment, 10.0);

        let remaining = voxel_tree.remaining_volume();

        // 材料が除去されたことを確認
        assert!(remaining < initial_volume);

        // 概算チェック: 円柱の体積 = π * r² * h = π * 10² * 100 ≈ 31415
        // 除去量が少なくとも20000以上であることを確認
        assert!(initial_volume - remaining > 20000.0);
    }

    #[test]
    fn test_capsule_removal_diagonal() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 対角線を通る線分
        let segment = LineSegment3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment, 5.0);

        let remaining = voxel_tree.remaining_volume();

        // 材料が除去されたことを確認
        assert!(remaining < initial_volume);
    }

    #[test]
    fn test_capsule_removal_no_intersection() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // ワークから遠く離れた線分
        let segment = LineSegment3D::new(
            Point3D::new(200.0, 200.0, 200.0),
            Point3D::new(300.0, 300.0, 300.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment, 10.0);

        // 体積は変わらないはず
        assert_eq!(voxel_tree.remaining_volume(), initial_volume);
    }

    #[test]
    fn test_capsule_removal_small_radius() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 5); // より高解像度

        let initial_volume = voxel_tree.remaining_volume();

        // 細い円柱（半径1mm）
        let segment = LineSegment3D::new(
            Point3D::new(50.0, 50.0, 0.0),
            Point3D::new(50.0, 50.0, 100.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment, 1.0);

        let remaining = voxel_tree.remaining_volume();

        // わずかに材料が除去されたことを確認
        assert!(remaining < initial_volume);

        // 円柱の体積 = π * 1² * 100 ≈ 314
        // ボクセル誤差を考慮して100以上の除去を確認
        assert!(initial_volume - remaining > 100.0);
    }

    #[test]
    fn test_capsule_removal_multiple_segments() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 複数の線分で材料除去
        let segment1 = LineSegment3D::new(
            Point3D::new(25.0, 25.0, 0.0),
            Point3D::new(25.0, 25.0, 100.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment1, 8.0);

        let volume_after_1 = voxel_tree.remaining_volume();
        assert!(volume_after_1 < initial_volume);

        let segment2 = LineSegment3D::new(
            Point3D::new(75.0, 75.0, 0.0),
            Point3D::new(75.0, 75.0, 100.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment2, 8.0);

        let volume_after_2 = voxel_tree.remaining_volume();
        assert!(volume_after_2 < volume_after_1);
    }

    #[test]
    fn test_capsule_removal_horizontal() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 水平方向の線分
        let segment = LineSegment3D::new(
            Point3D::new(0.0, 50.0, 50.0),
            Point3D::new(100.0, 50.0, 50.0),
        )
        .unwrap();
        voxel_tree.remove_material_capsule(&segment, 10.0);

        let remaining = voxel_tree.remaining_volume();

        // 材料が除去されたことを確認
        assert!(remaining < initial_volume);
    }

    // ========================================================================
    // Z軸特化版材料除去テスト（高速版）
    // ========================================================================

    #[test]
    fn test_z_axis_removal_basic() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // Z軸方向の円柱除去
        voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);

        let remaining = voxel_tree.remaining_volume();

        // 材料が除去されたことを確認
        assert!(remaining < initial_volume);

        // 概算チェック: 円柱の体積 = π * 10² * 100 ≈ 31415
        assert!(initial_volume - remaining > 20000.0);
    }

    #[test]
    fn test_z_axis_removal_comparison_with_capsule() {
        use geo_primitives::LineSegment3D;

        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );

        // 汎用版
        let mut tree1 = VoxelOctree::new(bounds, 4);
        let segment = LineSegment3D::new(
            Point3D::new(50.0, 50.0, 0.0),
            Point3D::new(50.0, 50.0, 100.0),
        )
        .unwrap();
        tree1.remove_material_capsule(&segment, 10.0);
        let volume1 = tree1.remaining_volume();

        // Z軸特化版
        let mut tree2 = VoxelOctree::new(bounds, 4);
        tree2.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 10.0);
        let volume2 = tree2.remaining_volume();

        // 結果が同じであることを確認（ボクセル誤差を考慮）
        let diff = (volume1 - volume2).abs();
        assert!(diff < 1000.0); // 1%以下の誤差
    }

    #[test]
    fn test_z_axis_removal_offset_center() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // オフセットした位置での除去
        voxel_tree.remove_material_z_axis(25.0, 75.0, 10.0, 90.0, 8.0);

        let remaining = voxel_tree.remaining_volume();
        assert!(remaining < initial_volume);
    }

    #[test]
    fn test_z_axis_removal_no_intersection() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // ワークから遠く離れた位置
        voxel_tree.remove_material_z_axis(200.0, 200.0, 0.0, 100.0, 10.0);

        // 体積は変わらないはず
        assert_eq!(voxel_tree.remaining_volume(), initial_volume);
    }

    #[test]
    fn test_z_axis_removal_partial_z_range() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // Z方向の一部のみ除去
        voxel_tree.remove_material_z_axis(50.0, 50.0, 20.0, 60.0, 15.0);

        let remaining = voxel_tree.remaining_volume();

        // 材料が除去されたことを確認
        assert!(remaining < initial_volume);

        // 一部除去なので除去量は少ないはず
        let removed_ratio = (initial_volume - remaining) / initial_volume;
        assert!(removed_ratio < 0.5); // 50%以下の除去
    }

    #[test]
    fn test_z_axis_removal_multiple_operations() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        let initial_volume = voxel_tree.remaining_volume();

        // 複数回の除去
        voxel_tree.remove_material_z_axis(25.0, 25.0, 0.0, 100.0, 8.0);
        let volume_after_1 = voxel_tree.remaining_volume();
        assert!(volume_after_1 < initial_volume);

        voxel_tree.remove_material_z_axis(75.0, 75.0, 0.0, 100.0, 8.0);
        let volume_after_2 = voxel_tree.remaining_volume();
        assert!(volume_after_2 < volume_after_1);
    }

    #[test]
    fn test_z_axis_removal_small_radius() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 5); // 高解像度

        let initial_volume = voxel_tree.remaining_volume();

        // 細い円柱
        voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 1.0);

        let remaining = voxel_tree.remaining_volume();

        // わずかに材料が除去されたことを確認
        assert!(remaining < initial_volume);
        assert!(initial_volume - remaining > 100.0);
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

    #[test]
    fn test_detect_undercut_no_removal() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let voxel_tree = VoxelOctree::new(bounds, 4);

        // 完全にワーク内の目的領域
        let target = Aabb3D::new(
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(90.0, 90.0, 90.0),
        );

        // 何も除去していないので、全てが削り残し
        let undercuts = voxel_tree.detect_undercut(&target);
        assert!(!undercuts.is_empty());
    }

    #[test]
    fn test_detect_undercut_complete_removal() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        // 全体を除去
        voxel_tree.remove_material_box(&bounds);

        // 目的領域（何でもいい）
        let target = Aabb3D::new(
            Point3D::new(10.0, 10.0, 10.0),
            Point3D::new(90.0, 90.0, 90.0),
        );

        // 全て除去したので削り残しなし
        let undercuts = voxel_tree.detect_undercut(&target);
        assert!(undercuts.is_empty());
    }

    #[test]
    fn test_detect_undercut_partial_removal() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 4);

        // 中央部分のみ除去
        let removed_region = Aabb3D::new(
            Point3D::new(20.0, 20.0, 20.0),
            Point3D::new(80.0, 80.0, 80.0),
        );
        voxel_tree.remove_material_box(&removed_region);

        // 目的領域は除去した領域と同じ
        let target = removed_region;

        // 周辺部分が削り残しとして検出されるはず
        let undercuts = voxel_tree.detect_undercut(&target);
        assert!(!undercuts.is_empty());

        // 削り残しボクセルは目的領域と交差しない
        for undercut_bbox in &undercuts {
            assert!(!undercut_bbox.intersects(&target));
        }
    }

    #[test]
    fn test_detect_undercut_z_axis_tool() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let mut voxel_tree = VoxelOctree::new(bounds, 5);

        // Z軸方向の工具で中央を除去
        voxel_tree.remove_material_z_axis(50.0, 50.0, 0.0, 100.0, 20.0);

        // 目的領域: 円柱内部を近似した直方体
        let target = Aabb3D::new(
            Point3D::new(30.0, 30.0, 0.0),
            Point3D::new(70.0, 70.0, 100.0),
        );

        // 外側の角部分が削り残しとして検出されるはず
        let undercuts = voxel_tree.detect_undercut(&target);
        assert!(!undercuts.is_empty());
    }

    #[test]
    fn test_detect_undercut_target_larger_than_work() {
        let bounds = Aabb3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(100.0, 100.0, 100.0),
        );
        let voxel_tree = VoxelOctree::new(bounds, 4);

        // 目的領域がワークより大きい場合
        let target = Aabb3D::new(
            Point3D::new(-50.0, -50.0, -50.0),
            Point3D::new(150.0, 150.0, 150.0),
        );

        // ワーク全体が目的領域内なので削り残しなし
        let undercuts = voxel_tree.detect_undercut(&target);
        assert!(undercuts.is_empty());
    }
}

