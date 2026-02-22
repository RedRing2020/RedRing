use super::*;

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

    /// AABB形状による材料除去（再帰的）
    ///
    /// # Arguments
    ///
    /// * `tool_aabb` - 工具の境界ボックス
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
                    self.state = VoxelState::Empty;
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
                    .distance_to_aabb((min.x(), min.y(), min.z()), (max.x(), max.y(), max.z()));

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

        let len_sq = dx * dx + dy * dy + dz * dz;
        let t = if len_sq <= T::EPSILON {
            T::ZERO
        } else {
            let dot = to_px * dx + to_py * dy + to_pz * dz;
            (dot / len_sq).clamp(T::ZERO, T::ONE)
        };

        let closest_x = sx + dx * t;
        let closest_y = sy + dy * t;
        let closest_z = sz + dz * t;

        let diff_x = px - closest_x;
        let diff_y = py - closest_y;
        let diff_z = pz - closest_z;
        (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt()
    }

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

        (dx * dx + dy * dy).sqrt()
    }

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
            let dx = x - center_x;
            let dy = y - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > radius {
                return false;
            }
        }

        true
    }

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
