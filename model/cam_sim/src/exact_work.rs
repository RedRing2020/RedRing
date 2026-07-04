use geo_algorithms::{Aabb3D, LineSegment3D, Point3D, Scalar};

/// SAT-Cut PoCで扱う工具掃引プリミティブ。
#[derive(Debug, Clone, Copy)]
pub enum ExactToolPrimitive<T: Scalar> {
    Flat {
        segment: LineSegment3D<T>,
        radius: T,
    },
    Ball {
        segment: LineSegment3D<T>,
        radius: T,
    },
}

/// Voxel/Octree非依存のワーク正本を扱うためのtrait定義。
pub trait ExactWorkModel<T: Scalar> {
    /// 掃引プリミティブを正本へ適用する。
    fn apply_primitive(&mut self, primitive: ExactToolPrimitive<T>);

    /// 点が材料内部かどうかを返す。
    fn contains_material_at(&self, point: &Point3D<T>) -> bool;

    /// 除去境界までの最近傍距離を返す。
    fn nearest_removed_surface_distance(&self, point: &Point3D<T>) -> T;

    /// サンプリングで残存体積を推定する。
    fn estimate_remaining_volume(&self, sample_pitch: T) -> T;

    /// 正本が保持している掃引プリミティブ数を返す。
    fn primitive_count(&self) -> usize;
}

/// 表示キャッシュ側の更新境界を分離するためのtrait定義。
pub trait ExactWorkProjectionCache<T: Scalar> {
    /// 指定領域のみを再投影して表示キャッシュを更新する。
    fn update_from_exact_work(&mut self, work: &dyn ExactWorkModel<T>, dirty_region: &Aabb3D<T>);

    /// キャッシュを破棄して初期状態へ戻す。
    fn clear(&mut self);
}

/// 掃引プリミティブ集合を正本として保持する最小PoC実装。
#[derive(Debug, Clone)]
pub struct PrimitiveSetExactWork {
    bounds: Aabb3D<f64>,
    removed_primitives: Vec<ExactToolPrimitive<f64>>,
}

impl PrimitiveSetExactWork {
    pub fn new(bounds: Aabb3D<f64>) -> Self {
        Self {
            bounds,
            removed_primitives: Vec::new(),
        }
    }

    pub fn estimate_memory_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.removed_primitives.capacity() * std::mem::size_of::<ExactToolPrimitive<f64>>()
    }

    fn contains_material(&self, point: &Point3D<f64>) -> bool {
        if !point_in_aabb(point, &self.bounds) {
            return false;
        }
        !self
            .removed_primitives
            .iter()
            .any(|primitive| primitive_contains(primitive, point))
    }
}

impl ExactWorkModel<f64> for PrimitiveSetExactWork {
    fn apply_primitive(&mut self, primitive: ExactToolPrimitive<f64>) {
        self.removed_primitives.push(primitive);
    }

    fn contains_material_at(&self, point: &Point3D<f64>) -> bool {
        self.contains_material(point)
    }

    fn nearest_removed_surface_distance(&self, point: &Point3D<f64>) -> f64 {
        let mut best = f64::INFINITY;
        for primitive in &self.removed_primitives {
            let surface_distance = match primitive {
                ExactToolPrimitive::Ball { segment, radius } => {
                    (point_to_segment_distance(point, segment) - *radius).abs()
                }
                ExactToolPrimitive::Flat { segment, radius } => {
                    let distance = point_to_flat_swept_distance(point, segment);
                    if !distance.is_finite() {
                        continue;
                    }
                    (distance - *radius).abs()
                }
            };
            best = best.min(surface_distance);
        }
        best
    }

    fn estimate_remaining_volume(&self, sample_pitch: f64) -> f64 {
        if sample_pitch <= f64::EPSILON {
            return 0.0;
        }

        let min = self.bounds.min();
        let max = self.bounds.max();
        let half = sample_pitch * 0.5;

        let mut solid_count = 0usize;
        let mut x = min.x() + half;
        while x < max.x() {
            let mut y = min.y() + half;
            while y < max.y() {
                let mut z = min.z() + half;
                while z < max.z() {
                    let point = Point3D::new(x, y, z);
                    if self.contains_material(&point) {
                        solid_count += 1;
                    }
                    z += sample_pitch;
                }
                y += sample_pitch;
            }
            x += sample_pitch;
        }

        (solid_count as f64) * sample_pitch.powi(3)
    }

    fn primitive_count(&self) -> usize {
        self.removed_primitives.len()
    }
}

fn point_in_aabb(point: &Point3D<f64>, aabb: &Aabb3D<f64>) -> bool {
    let min = aabb.min();
    let max = aabb.max();
    point.x() >= min.x()
        && point.x() <= max.x()
        && point.y() >= min.y()
        && point.y() <= max.y()
        && point.z() >= min.z()
        && point.z() <= max.z()
}

fn primitive_contains(primitive: &ExactToolPrimitive<f64>, point: &Point3D<f64>) -> bool {
    match primitive {
        ExactToolPrimitive::Ball { segment, radius } => {
            point_to_segment_distance(point, segment) <= *radius
        }
        ExactToolPrimitive::Flat { segment, radius } => {
            point_to_flat_swept_distance(point, segment) <= *radius
        }
    }
}

fn point_to_segment_distance(point: &Point3D<f64>, segment: &LineSegment3D<f64>) -> f64 {
    let start_x = segment.start().x();
    let start_y = segment.start().y();
    let start_z = segment.start().z();
    let end_x = segment.end().x();
    let end_y = segment.end().y();
    let end_z = segment.end().z();

    let axis_x = end_x - start_x;
    let axis_y = end_y - start_y;
    let axis_z = end_z - start_z;
    let axis_len_sq = axis_x * axis_x + axis_y * axis_y + axis_z * axis_z;

    if axis_len_sq <= f64::EPSILON {
        let dx = point.x() - start_x;
        let dy = point.y() - start_y;
        let dz = point.z() - start_z;
        return (dx * dx + dy * dy + dz * dz).sqrt();
    }

    let point_x = point.x() - start_x;
    let point_y = point.y() - start_y;
    let point_z = point.z() - start_z;
    let t =
        ((point_x * axis_x + point_y * axis_y + point_z * axis_z) / axis_len_sq).clamp(0.0, 1.0);

    let closest_x = start_x + axis_x * t;
    let closest_y = start_y + axis_y * t;
    let closest_z = start_z + axis_z * t;
    let dx = point.x() - closest_x;
    let dy = point.y() - closest_y;
    let dz = point.z() - closest_z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn point_to_flat_swept_distance(point: &Point3D<f64>, segment: &LineSegment3D<f64>) -> f64 {
    let start_x = segment.start().x();
    let start_y = segment.start().y();
    let start_z = segment.start().z();
    let end_x = segment.end().x();
    let end_y = segment.end().y();
    let end_z = segment.end().z();

    let axis_x = end_x - start_x;
    let axis_y = end_y - start_y;
    let axis_z = end_z - start_z;
    let axis_len_sq = axis_x * axis_x + axis_y * axis_y + axis_z * axis_z;
    if axis_len_sq <= f64::EPSILON {
        let dx = point.x() - start_x;
        let dy = point.y() - start_y;
        let dz = point.z() - start_z;
        return (dx * dx + dy * dy + dz * dz).sqrt();
    }

    let point_x = point.x() - start_x;
    let point_y = point.y() - start_y;
    let point_z = point.z() - start_z;
    let t = (point_x * axis_x + point_y * axis_y + point_z * axis_z) / axis_len_sq;
    if !(0.0..=1.0).contains(&t) {
        return f64::INFINITY;
    }

    let closest_x = start_x + axis_x * t;
    let closest_y = start_y + axis_y * t;
    let closest_z = start_z + axis_z * t;
    let dx = point.x() - closest_x;
    let dy = point.y() - closest_y;
    let dz = point.z() - closest_z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

#[cfg(test)]
mod tests {
    use super::{
        ExactToolPrimitive, ExactWorkModel, ExactWorkProjectionCache, PrimitiveSetExactWork,
    };
    use geo_algorithms::{Aabb3D, LineSegment3D, Point3D};

    #[derive(Default)]
    struct DummyProjectionCache {
        updated_count: usize,
    }

    impl ExactWorkProjectionCache<f64> for DummyProjectionCache {
        fn update_from_exact_work(
            &mut self,
            _work: &dyn ExactWorkModel<f64>,
            _dirty_region: &Aabb3D<f64>,
        ) {
            self.updated_count += 1;
        }

        fn clear(&mut self) {
            self.updated_count = 0;
        }
    }

    #[test]
    fn primitive_set_exact_work_implements_trait_contract() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let mut cache = DummyProjectionCache::default();

        let segment = LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .expect("segment must be valid");
        work.apply_primitive(ExactToolPrimitive::Flat {
            segment,
            radius: 1.0,
        });

        assert_eq!(work.primitive_count(), 1);

        let dirty = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(2.0, 2.0, 2.0));
        cache.update_from_exact_work(&work, &dirty);
        assert_eq!(cache.updated_count, 1);
    }

    #[test]
    fn primitive_set_exact_work_removes_material_for_flat_primitive() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let segment = LineSegment3D::new(Point3D::new(1.0, 5.0, 5.0), Point3D::new(9.0, 5.0, 5.0))
            .expect("segment must be valid");
        work.apply_primitive(ExactToolPrimitive::Flat {
            segment,
            radius: 1.0,
        });

        assert!(!work.contains_material_at(&Point3D::new(5.0, 5.0, 5.0)));
        assert!(work.contains_material_at(&Point3D::new(5.0, 8.0, 8.0)));
        assert!(work.nearest_removed_surface_distance(&Point3D::new(5.0, 5.0, 5.0)) <= 1.0);
    }

    #[test]
    fn primitive_set_exact_work_estimate_volume_decreases_after_cutting() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let before = work.estimate_remaining_volume(2.0);

        let segment = LineSegment3D::new(Point3D::new(1.0, 5.0, 5.0), Point3D::new(9.0, 5.0, 5.0))
            .expect("segment must be valid");
        work.apply_primitive(ExactToolPrimitive::Ball {
            segment,
            radius: 1.5,
        });
        let after = work.estimate_remaining_volume(2.0);

        assert!(before > after, "before={} after={}", before, after);
    }
}
