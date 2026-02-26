use super::*;

impl<T: Scalar> VoxelOctree<T> {
    /// 新しいボクセルOctreeを作成
    ///
    /// 初期状態では全体が Solid（占有あり）です。
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

    /// AABB形状による領域除去
    ///
    /// 指定された境界ボックスと交差する占有領域を除去します。
    ///
    /// # Arguments
    ///
    /// * `tool_aabb` - 除去対象形状の境界ボックス
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    ///
    /// // 掃引形状が通過した領域を除去
    /// let tool_region = Aabb3D::new(
    ///     Point3D::new(10.0, 10.0, 0.0),
    ///     Point3D::new(20.0, 20.0, 50.0)
    /// );
    /// voxel_tree.remove_material_box(&tool_region);
    /// ```
    pub fn remove_material_box(&mut self, tool_aabb: &Aabb3D<T>) {
        self.root.remove_material_box(tool_aabb, self.max_depth);
    }

    /// 線分を中心軸とした円柱（カプセル）領域で占有除去
    ///
    /// 線分に沿って指定した半径の円柱領域内の占有を除去します。
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
    /// // 線分経路に沿って占有除去
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

    /// 線分を中心軸とした平端掃引円柱領域で占有除去
    ///
    /// `remove_material_capsule` と異なり、線分端点の半球キャップを含みません。
    /// 平端キャップを持つ掃引円柱に対応する除去モデルです。
    pub fn remove_material_swept_cylinder(&mut self, segment: &LineSegment3D<T>, radius: T) {
        self.root
            .remove_material_swept_cylinder(segment, radius, self.max_depth);
    }

    /// Z軸方向の円柱領域で占有除去（高速版）
    ///
    /// 形状軸がZ軸に平行な場合の最適化実装。
    /// 汎用版`remove_material_capsule`より3-5倍高速。
    ///
    /// # Arguments
    ///
    /// * `center_x` - 掃引円の中心X座標
    /// * `center_y` - 掃引円の中心Y座標
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
    /// // Z軸方向に占有除去
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

    /// 残存体積を計算
    ///
    /// # Returns
    ///
    /// 残存領域の総体積（単位: mm³）
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

    /// 未除去領域検出
    ///
    /// 目的形状の境界ボックスの外側に残っているSolidボクセルを検出します。
    /// これは、除去形状が到達できなかった領域や、意図しない占有の残存を示します。
    ///
    /// # Arguments
    ///
    /// * `target_region` - 目的形状の境界ボックス（この内側にあるべき領域）
    ///
    /// # Returns
    ///
    /// 未除去領域として検出されたボクセルの境界ボックスリスト。
    /// 空のリストは、未除去領域が存在しないことを示します。
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
    /// // 領域除去シミュレーション実行
    /// // ... remove_material_* 呼び出し ...
    ///
    /// // 目的形状（この範囲外の占有は未除去領域）
    /// let target = Aabb3D::new(
    ///     Point3D::new(10.0, 10.0, 10.0),
    ///     Point3D::new(90.0, 90.0, 90.0)
    /// );
    ///
    /// // 未除去領域検出
    /// let undercuts = voxel_tree.detect_undercut(&target);
    /// if !undercuts.is_empty() {
    ///     eprintln!("警告: {} 箇所の未除去領域を検出", undercuts.len());
    /// }
    /// ```
    pub fn detect_undercut(&self, target_region: &Aabb3D<T>) -> Vec<Aabb3D<T>> {
        let mut undercut_voxels = Vec::new();
        self.root
            .collect_solid_voxels_outside(target_region, &mut undercut_voxels);
        undercut_voxels
    }

    /// 全てのSolidボクセルの境界ボックスを収集（可視化用）
    ///
    /// VoxelOctree内の全てのSolid状態のリーフノード境界ボックスを取得します。
    /// ワイヤーフレーム描画など、可視化目的で使用されます。
    ///
    /// # Returns
    ///
    /// Solid状態のボクセル境界ボックスのベクタ
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// let voxel_tree = VoxelOctree::new(work_bounds, 6);
    /// voxel_tree.remove_material_box(&tool_region);
    ///
    /// let solid_boxes = voxel_tree.collect_solid_voxel_bounds();
    /// println!("Solid voxels: {}", solid_boxes.len());
    /// ```
    pub fn collect_solid_voxel_bounds(&self) -> Vec<Aabb3D<T>> {
        let mut result = Vec::new();
        self.root.collect_solid_voxels(&mut result);
        result
    }

    /// 指定深さまでの「非Empty」ボクセル境界ボックスを収集（可視化用）
    ///
    /// # Arguments
    ///
    /// * `visible_depth` - 表示したい最大深さ（0はルートのみ）
    pub fn collect_non_empty_voxel_bounds_up_to_depth(
        &self,
        visible_depth: usize,
    ) -> Vec<Aabb3D<T>> {
        let mut result = Vec::new();
        let clamped_depth = visible_depth.min(self.max_depth);
        self.root
            .collect_non_empty_voxels_up_to_depth(clamped_depth, &mut result);
        result
    }

    /// 円弧経路による占有除去（線分近似版）
    ///
    /// 円弧経路を線分列に近似して占有を除去します。
    /// 円弧を等間隔でサンプリングし、隣接点間を線分として既存の`remove_material_capsule()`を適用します。
    ///
    /// # Arguments
    ///
    /// * `arc` - 掃引経路の円弧（Arc3D）
    /// * `radius` - 掃引半径
    /// * `num_segments` - 近似に使用する線分数（推奨: 8-32）
    ///
    /// # Performance
    ///
    /// 線分数に比例して計算時間が増加します。
    ///
    /// **推奨パラメータ**:
    /// - 低分解能: 8線分（速度重視）
    /// - 中分解能: 16-32線分（精度重視）
    /// - 高分解能: 64線分以上（特殊用途）
    ///
    /// **精度とパフォーマンスのトレードオフ**:
    ///
    /// | 線分数 | 誤差（半径10mm, 90度円弧） | 計算量 |
    /// |--------|---------------------------|--------|
    /// | 8      | ~0.19mm (1.9%)            | 8回    |
    /// | 16     | ~0.05mm (0.5%)            | 16回   |
    /// | 32     | ~0.01mm (0.1%)            | 32回   |
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use geo_algorithms::octree::voxel::VoxelOctree;
    /// use geo_primitives::{Arc3D, Angle};
    /// use geo_core::{Aabb3D, Point3D};
    ///
    /// let work_bounds = Aabb3D::new(
    ///     Point3D::new(0.0, 0.0, 0.0),
    ///     Point3D::new(100.0, 100.0, 100.0)
    /// );
    /// let mut voxel_tree = VoxelOctree::new(work_bounds, 6);
    ///
    /// // XY平面上の90度円弧（G02/G03相当）
    /// let arc = Arc3D::xy_arc(
    ///     Point3D::new(50.0, 50.0, 0.0),  // 中心
    ///     20.0,                            // 半径
    ///     Angle::degrees(0.0),             // 開始角度
    ///     Angle::degrees(90.0)             // 終了角度
    /// ).unwrap();
    ///
    /// // 掃引半径5mm、16線分で近似（誤差0.5%）
    /// voxel_tree.remove_material_arc_polyline(&arc, 5.0, 16);
    /// ```
    ///
    /// # 円弧パラメータ表現との対応
    ///
    /// ```text
    /// center/radius/start_angle/end_angle
    /// ↓
    /// Arc3D::xy_arc(center, radius, start_angle, end_angle)
    /// → remove_material_arc_polyline(&arc, radius, 16)
    /// ```
    ///
    /// # Notes
    ///
    /// - 線分近似により若干の過剰除去が生じる場合があります
    /// - より正確な円弧処理が必要な場合は、将来実装予定の正確版を検討してください
    /// - NURBS曲線など他の曲線型も同様の手法で対応可能です
    pub fn remove_material_arc_polyline(&mut self, arc: &Arc3D<T>, radius: T, num_segments: usize) {
        if num_segments == 0 {
            return; // 線分数0は何もしない
        }

        // 円弧を等間隔でサンプリング
        let points = arc.sample_points(num_segments + 1);

        // 隣接点間を線分で除去
        for i in 0..num_segments {
            if let Some(segment) = LineSegment3D::new(points[i], points[i + 1]) {
                self.remove_material_capsule(&segment, radius);
            }
        }
    }
}
