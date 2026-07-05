use super::*;
use analysis::linalg::vector::Vector2;
use geo_contracts::CrossDistance;

impl<T: Scalar> VoxelNode<T> {
    /// 新しいソリッドノードを作成
    ///
    /// # Arguments
    ///
    /// * `bounds` - ノードの境界ボックス
    /// * `depth` - ツリー内の深さ
    pub(super) fn new_solid(bounds: Aabb3D<T>, depth: usize) -> Self {
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

    /// AABB形状による占有除去（再帰的）
    ///
    /// # Arguments
    ///
    /// * `tool_aabb` - 除去対象形状の境界ボックス
    /// * `max_depth` - 最大深さ（分割上限）
    pub(super) fn remove_material_box(&mut self, tool_aabb: &Aabb3D<T>, max_depth: usize) {
        match self.state {
            VoxelState::Empty => (),

            VoxelState::Solid => {
                if !self.bounds.intersects(tool_aabb) {
                    return;
                }

                if tool_aabb.contains_aabb(&self.bounds) {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }

                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_box(tool_aabb, max_depth);
                    }
                } else {
                    if self.should_remove_leaf_for_box(tool_aabb) {
                        self.state = VoxelState::Empty;
                    }
                }
            }

            VoxelState::Mixed => {
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_box(tool_aabb, max_depth);
                    }

                    if children.iter().all(|c| c.state == VoxelState::Empty) {
                        self.state = VoxelState::Empty;
                        self.children = None;
                    } else if children.iter().all(|c| c.state == VoxelState::Solid) {
                        self.state = VoxelState::Solid;
                        self.children = None;
                    }
                }
            }
        }
    }

    /// 線分+半径（カプセル）領域との交差に基づいて材料を除去する。
    pub(super) fn remove_material_capsule(
        &mut self,
        segment: &LineSegment3D<T>,
        radius: T,
        max_depth: usize,
    ) {
        match self.state {
            VoxelState::Empty => (),
            VoxelState::Solid => {
                let min = self.bounds.min();
                let max = self.bounds.max();
                let distance = segment
                    .distance_to(&((min.x(), min.y(), min.z()), (max.x(), max.y(), max.z())));

                if distance > radius {
                    return;
                }

                if self.is_aabb_inside_capsule(segment, radius) {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }

                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_capsule(segment, radius, max_depth);
                    }
                } else {
                    self.state = VoxelState::Empty;
                }
            }
            VoxelState::Mixed => {
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_capsule(segment, radius, max_depth);
                    }

                    if children.iter().all(|c| c.state == VoxelState::Empty) {
                        self.state = VoxelState::Empty;
                        self.children = None;
                    } else if children.iter().all(|c| c.state == VoxelState::Solid) {
                        self.state = VoxelState::Solid;
                        self.children = None;
                    }
                }
            }
        }
    }

    /// 線分端面を平端として扱う掃引円柱領域で材料を除去する。
    pub(super) fn remove_material_swept_cylinder(
        &mut self,
        segment: &LineSegment3D<T>,
        radius: T,
        max_depth: usize,
    ) {
        match self.state {
            VoxelState::Empty => (),
            VoxelState::Solid => {
                // 掃引区間の端面（2つのキャップ平面）から外れるAABBは早期除外。
                if self.is_aabb_outside_swept_cylinder_cap_range(segment) {
                    return;
                }

                let min = self.bounds.min();
                let max = self.bounds.max();
                let distance = segment
                    .distance_to(&((min.x(), min.y(), min.z()), (max.x(), max.y(), max.z())));

                // 軸方向へ射影可能でも半径外なら非交差。
                if distance > radius {
                    return;
                }

                // AABB全体が掃引体内ならノードごとEmpty化して再帰を打ち切る。
                if self.is_aabb_inside_swept_cylinder(segment, radius) {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }

                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_swept_cylinder(segment, radius, max_depth);
                    }
                } else {
                    self.state = VoxelState::Empty;
                }
            }
            VoxelState::Mixed => {
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_swept_cylinder(segment, radius, max_depth);
                    }

                    if children.iter().all(|c| c.state == VoxelState::Empty) {
                        self.state = VoxelState::Empty;
                        self.children = None;
                    } else if children.iter().all(|c| c.state == VoxelState::Solid) {
                        self.state = VoxelState::Solid;
                        self.children = None;
                    }
                }
            }
        }
    }

    /// AABBの8頂点がすべてカプセル内部にあるかを判定する。
    fn is_aabb_inside_capsule(&self, segment: &LineSegment3D<T>, radius: T) -> bool {
        let min = self.bounds.min();
        let max = self.bounds.max();

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

        for &vertex in &vertices {
            let distance = self.point_to_segment_distance(vertex, segment);
            if distance > radius {
                return false;
            }
        }

        true
    }

    /// AABBの軸方向射影が掃引区間端面の外側かを判定する。
    fn is_aabb_outside_swept_cylinder_cap_range(&self, segment: &LineSegment3D<T>) -> bool {
        let sx = segment.start().x();
        let sy = segment.start().y();
        let sz = segment.start().z();
        let ex = segment.end().x();
        let ey = segment.end().y();
        let ez = segment.end().z();

        let axis_x = ex - sx;
        let axis_y = ey - sy;
        let axis_z = ez - sz;
        let len_sq = axis_x * axis_x + axis_y * axis_y + axis_z * axis_z;
        if len_sq <= T::EPSILON {
            return false;
        }

        let min = self.bounds.min();
        let max = self.bounds.max();
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

        let (first_x, first_y, first_z) = vertices[0];
        let first_rel_x = first_x - sx;
        let first_rel_y = first_y - sy;
        let first_rel_z = first_z - sz;
        let first_proj = first_rel_x * axis_x + first_rel_y * axis_y + first_rel_z * axis_z;

        let mut min_proj = first_proj;
        let mut max_proj = first_proj;
        for &(vx, vy, vz) in vertices.iter().skip(1) {
            let rel_x = vx - sx;
            let rel_y = vy - sy;
            let rel_z = vz - sz;
            let proj = rel_x * axis_x + rel_y * axis_y + rel_z * axis_z;
            min_proj = min_proj.min(proj);
            max_proj = max_proj.max(proj);
        }

        // AABBの射影区間が [0, |axis|^2] と重ならない場合、平端掃引体とは交差しない。
        max_proj < T::ZERO || min_proj > len_sq
    }

    /// AABBの8頂点がすべて平端掃引円柱内部にあるかを判定する。
    fn is_aabb_inside_swept_cylinder(&self, segment: &LineSegment3D<T>, radius: T) -> bool {
        let min = self.bounds.min();
        let max = self.bounds.max();

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

        // AABBの8頂点がすべて掃引体内にある場合のみ「完全内包」とみなす。
        for &vertex in &vertices {
            if !self.is_point_inside_swept_cylinder(vertex, segment, radius) {
                return false;
            }
        }

        true
    }

    /// 1点が平端掃引円柱内部にあるかを判定する。
    fn is_point_inside_swept_cylinder(
        &self,
        point: (T, T, T),
        segment: &LineSegment3D<T>,
        radius: T,
    ) -> bool {
        let (px, py, pz) = point;
        let sx = segment.start().x();
        let sy = segment.start().y();
        let sz = segment.start().z();
        let ex = segment.end().x();
        let ey = segment.end().y();
        let ez = segment.end().z();

        let axis_x = ex - sx;
        let axis_y = ey - sy;
        let axis_z = ez - sz;
        let len_sq = axis_x * axis_x + axis_y * axis_y + axis_z * axis_z;
        if len_sq <= T::EPSILON {
            return false;
        }

        let rel_x = px - sx;
        let rel_y = py - sy;
        let rel_z = pz - sz;
        let proj = rel_x * axis_x + rel_y * axis_y + rel_z * axis_z;

        // 軸方向射影が端面範囲外なら、平端円柱の体積外。
        if proj < T::ZERO || proj > len_sq {
            return false;
        }

        let t = proj / len_sq;
        let closest_x = sx + axis_x * t;
        let closest_y = sy + axis_y * t;
        let closest_z = sz + axis_z * t;

        let point = Point3D::new(px, py, pz);
        let closest = Point3D::new(closest_x, closest_y, closest_z);
        let distance = point.distance_to(&closest);
        // 半径方向距離で円柱断面内判定。
        distance <= radius
    }

    /// 点と線分の最短距離を返す。
    fn point_to_segment_distance(&self, point: (T, T, T), segment: &LineSegment3D<T>) -> T {
        let (px, py, pz) = point;
        let sx = segment.start().x();
        let sy = segment.start().y();
        let sz = segment.start().z();
        let ex = segment.end().x();
        let ey = segment.end().y();
        let ez = segment.end().z();

        let dx = ex - sx;
        let dy = ey - sy;
        let dz = ez - sz;

        let to_px = px - sx;
        let to_py = py - sy;
        let to_pz = pz - sz;

        let start_point = Point3D::new(sx, sy, sz);
        let end_point = Point3D::new(ex, ey, ez);
        let len_sq = start_point.distance_squared_to(&end_point);
        let t = if len_sq <= T::EPSILON {
            T::ZERO
        } else {
            let dot = to_px * dx + to_py * dy + to_pz * dz;
            (dot / len_sq).clamp(T::ZERO, T::ONE)
        };

        let closest_x = sx + dx * t;
        let closest_y = sy + dy * t;
        let closest_z = sz + dz * t;

        let point = Point3D::new(px, py, pz);
        let closest = Point3D::new(closest_x, closest_y, closest_z);
        point.distance_to(&closest)
    }

    /// 最大深さ葉ノードでのAABB除去判定（中心点サンプリング）。
    fn should_remove_leaf_for_box(&self, tool_aabb: &Aabb3D<T>) -> bool {
        tool_aabb.contains_point(&self.bounds.center())
    }

    /// Z軸平行の軸付き形状に特化した高速除去を行う。
    pub(super) fn remove_material_capsule_z_axis(
        &mut self,
        center_x: T,
        center_y: T,
        z_start: T,
        z_end: T,
        radius: T,
        max_depth: usize,
    ) {
        match self.state {
            VoxelState::Empty => (),
            VoxelState::Solid => {
                let min = self.bounds.min();
                let max = self.bounds.max();

                let z_min_seg = z_start.min(z_end);
                let z_max_seg = z_start.max(z_end);

                if z_max_seg < min.z() || z_min_seg > max.z() {
                    return;
                }

                let distance_2d = self.circle_to_aabb_2d_distance(center_x, center_y, &min, &max);

                if distance_2d > radius {
                    return;
                }

                if self
                    .is_aabb_inside_z_axis_capsule(center_x, center_y, z_min_seg, z_max_seg, radius)
                {
                    self.state = VoxelState::Empty;
                    self.children = None;
                    return;
                }

                if self.depth < max_depth {
                    self.subdivide();
                    for child in self.children.as_mut().unwrap().iter_mut() {
                        child.remove_material_capsule_z_axis(
                            center_x, center_y, z_start, z_end, radius, max_depth,
                        );
                    }
                } else {
                    self.state = VoxelState::Empty;
                }
            }

            VoxelState::Mixed => {
                if let Some(ref mut children) = self.children {
                    for child in children.iter_mut() {
                        child.remove_material_capsule_z_axis(
                            center_x, center_y, z_start, z_end, radius, max_depth,
                        );
                    }

                    if children.iter().all(|c| c.state == VoxelState::Empty) {
                        self.state = VoxelState::Empty;
                        self.children = None;
                    } else if children.iter().all(|c| c.state == VoxelState::Solid) {
                        self.state = VoxelState::Solid;
                        self.children = None;
                    }
                }
            }
        }
    }

    /// XY平面で円中心とAABBの最短距離を返す。
    fn circle_to_aabb_2d_distance(
        &self,
        center_x: T,
        center_y: T,
        aabb_min: &Point3D<T>,
        aabb_max: &Point3D<T>,
    ) -> T {
        let closest_x = center_x.clamp(aabb_min.x(), aabb_max.x());
        let closest_y = center_y.clamp(aabb_min.y(), aabb_max.y());

        let dx = center_x - closest_x;
        let dy = center_y - closest_y;
        Vector2::new(dx, dy).norm()
    }

    /// Z軸平行カプセルにAABB全体が内包されるかを判定する。
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

        if min.z() < z_min || max.z() > z_max {
            return false;
        }

        let corners_2d = [
            (min.x(), min.y()),
            (max.x(), min.y()),
            (min.x(), max.y()),
            (max.x(), max.y()),
        ];

        for &(x, y) in &corners_2d {
            let distance = Vector2::new(x - center_x, y - center_y).norm();
            if distance > radius {
                return false;
            }
        }

        true
    }

    /// このノード配下の残存体積を再帰集計する。
    pub(super) fn volume(&self) -> T {
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

    /// このノード配下のSolidリーフ数を再帰集計する。
    pub(super) fn solid_voxel_count(&self) -> usize {
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

    /// 指定領域外にあるSolidボクセル境界を収集する。
    pub(super) fn collect_solid_voxels_outside(
        &self,
        target_region: &Aabb3D<T>,
        undercut_voxels: &mut Vec<Aabb3D<T>>,
    ) {
        match self.state {
            VoxelState::Empty => {}
            VoxelState::Solid => {
                if !target_region.contains_aabb(&self.bounds) {
                    undercut_voxels.push(self.bounds);
                }
            }
            VoxelState::Mixed => {
                if let Some(ref children) = self.children {
                    for child in children.iter() {
                        child.collect_solid_voxels_outside(target_region, undercut_voxels);
                    }
                }
            }
        }
    }

    /// Solid状態のボクセル境界を収集する。
    pub(super) fn collect_solid_voxels(&self, solid_voxels: &mut Vec<Aabb3D<T>>) {
        match self.state {
            VoxelState::Empty => {}
            VoxelState::Solid => {
                solid_voxels.push(self.bounds);
            }
            VoxelState::Mixed => {
                if let Some(ref children) = self.children {
                    for child in children.iter() {
                        child.collect_solid_voxels(solid_voxels);
                    }
                }
            }
        }
    }

    /// 指定深さまでの非Emptyボクセル境界を収集する。
    pub(super) fn collect_non_empty_voxels_up_to_depth(
        &self,
        visible_depth: usize,
        voxels: &mut Vec<Aabb3D<T>>,
    ) {
        if self.state == VoxelState::Empty {
            return;
        }

        if self.depth >= visible_depth {
            voxels.push(self.bounds);
            return;
        }

        match self.state {
            VoxelState::Solid => {
                voxels.push(self.bounds);
            }
            VoxelState::Mixed => {
                if let Some(ref children) = self.children {
                    for child in children.iter() {
                        child.collect_non_empty_voxels_up_to_depth(visible_depth, voxels);
                    }
                } else {
                    voxels.push(self.bounds);
                }
            }
            VoxelState::Empty => {}
        }
    }
}
