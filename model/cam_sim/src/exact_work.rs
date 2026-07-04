use geo_algorithms::distance::line_segment3d_point3d_distance;
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
    /// 除去プリミティブが未適用の場合は `T::INFINITY` を返す。
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
        assert!(aabb_is_finite(&bounds), "bounds coordinates must be finite");
        assert!(
            !bounds.is_empty(),
            "PrimitiveSetExactWork::new received invalid bounds (min > max)"
        );
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
        if !self.bounds.contains(point) {
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
        let (segment, radius) = match primitive {
            ExactToolPrimitive::Flat { segment, radius }
            | ExactToolPrimitive::Ball { segment, radius } => (segment, radius),
        };
        assert!(
            segment_is_finite(&segment),
            "segment endpoints must be finite"
        );
        assert!(radius.is_finite(), "radius must be finite");
        assert!(radius >= 0.0, "radius must be non-negative");
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
                    point_to_flat_swept_surface_distance(point, segment, *radius)
                }
            };
            best = best.min(surface_distance);
        }
        best
    }

    fn estimate_remaining_volume(&self, sample_pitch: f64) -> f64 {
        if self.bounds.is_empty() {
            return 0.0;
        }

        if self.removed_primitives.is_empty() {
            return self.bounds.volume();
        }

        if !sample_pitch.is_finite() || sample_pitch <= 0.0 {
            return 0.0;
        }

        let min = self.bounds.min();
        let max = self.bounds.max();
        let width = max.x() - min.x();
        let height = max.y() - min.y();
        let depth = max.z() - min.z();

        let x_samples = axis_sample_count(width, sample_pitch);
        let y_samples = axis_sample_count(height, sample_pitch);
        let z_samples = axis_sample_count(depth, sample_pitch);

        let dx = width / (x_samples as f64);
        let dy = height / (y_samples as f64);
        let dz = depth / (z_samples as f64);

        let mut solid_count = 0usize;
        for ix in 0..x_samples {
            let x = min.x() + ((ix as f64) + 0.5) * dx;
            for iy in 0..y_samples {
                let y = min.y() + ((iy as f64) + 0.5) * dy;
                for iz in 0..z_samples {
                    let z = min.z() + ((iz as f64) + 0.5) * dz;
                    let point = Point3D::new(x, y, z);
                    if self.contains_material(&point) {
                        solid_count += 1;
                    }
                }
            }
        }

        let total_samples = (x_samples * y_samples * z_samples) as f64;
        self.bounds.volume() * ((solid_count as f64) / total_samples)
    }

    fn primitive_count(&self) -> usize {
        self.removed_primitives.len()
    }
}

const MAX_AXIS_SAMPLES: usize = 128;

fn point_is_finite(point: &Point3D<f64>) -> bool {
    point.x().is_finite() && point.y().is_finite() && point.z().is_finite()
}

fn segment_is_finite(segment: &LineSegment3D<f64>) -> bool {
    point_is_finite(&segment.start()) && point_is_finite(&segment.end())
}

fn aabb_is_finite(aabb: &Aabb3D<f64>) -> bool {
    point_is_finite(&aabb.min()) && point_is_finite(&aabb.max())
}

fn axis_sample_count(span: f64, sample_pitch: f64) -> usize {
    if !span.is_finite() || !sample_pitch.is_finite() || sample_pitch <= 0.0 {
        return 1;
    }

    let raw = (span / sample_pitch).ceil();
    if !raw.is_finite() {
        return MAX_AXIS_SAMPLES;
    }

    raw.clamp(1.0, MAX_AXIS_SAMPLES as f64) as usize
}

fn primitive_contains(primitive: &ExactToolPrimitive<f64>, point: &Point3D<f64>) -> bool {
    match primitive {
        ExactToolPrimitive::Ball { segment, radius } => {
            point_to_segment_distance(point, segment) <= *radius
        }
        ExactToolPrimitive::Flat { segment, radius } => {
            flat_swept_contains(point, segment, *radius)
        }
    }
}

fn point_to_segment_distance(point: &Point3D<f64>, segment: &LineSegment3D<f64>) -> f64 {
    line_segment3d_point3d_distance(segment, point)
}

fn point_to_flat_swept_surface_distance(
    point: &Point3D<f64>,
    segment: &LineSegment3D<f64>,
    radius: f64,
) -> f64 {
    let start_x = segment.start().x();
    let start_y = segment.start().y();
    let start_z = segment.start().z();
    let end_x = segment.end().x();
    let end_y = segment.end().y();
    let end_z = segment.end().z();

    let axis_x = end_x - start_x;
    let axis_y = end_y - start_y;
    let axis_z = end_z - start_z;
    let axis_len = (axis_x * axis_x + axis_y * axis_y + axis_z * axis_z).sqrt();
    if axis_len <= f64::EPSILON {
        let dx = point.x() - start_x;
        let dy = point.y() - start_y;
        let dz = point.z() - start_z;
        return ((dx * dx + dy * dy + dz * dz).sqrt() - radius).abs();
    }

    let point_x = point.x() - start_x;
    let point_y = point.y() - start_y;
    let point_z = point.z() - start_z;
    let axis_dir_x = axis_x / axis_len;
    let axis_dir_y = axis_y / axis_len;
    let axis_dir_z = axis_z / axis_len;
    let axial = point_x * axis_dir_x + point_y * axis_dir_y + point_z * axis_dir_z;

    let proj_x = start_x + axis_dir_x * axial;
    let proj_y = start_y + axis_dir_y * axial;
    let proj_z = start_z + axis_dir_z * axial;
    let radial_x = point.x() - proj_x;
    let radial_y = point.y() - proj_y;
    let radial_z = point.z() - proj_z;
    let radial = (radial_x * radial_x + radial_y * radial_y + radial_z * radial_z).sqrt();

    if axial < 0.0 {
        let axial_excess = -axial;
        if radial <= radius {
            axial_excess
        } else {
            axial_excess.hypot(radial - radius)
        }
    } else if axial > axis_len {
        let axial_excess = axial - axis_len;
        if radial <= radius {
            axial_excess
        } else {
            axial_excess.hypot(radial - radius)
        }
    } else {
        (radial - radius).abs()
    }
}

fn flat_swept_contains(point: &Point3D<f64>, segment: &LineSegment3D<f64>, radius: f64) -> bool {
    let start_x = segment.start().x();
    let start_y = segment.start().y();
    let start_z = segment.start().z();
    let end_x = segment.end().x();
    let end_y = segment.end().y();
    let end_z = segment.end().z();

    let axis_x = end_x - start_x;
    let axis_y = end_y - start_y;
    let axis_z = end_z - start_z;
    let axis_len = (axis_x * axis_x + axis_y * axis_y + axis_z * axis_z).sqrt();
    if axis_len <= f64::EPSILON {
        let dx = point.x() - start_x;
        let dy = point.y() - start_y;
        let dz = point.z() - start_z;
        return (dx * dx + dy * dy + dz * dz).sqrt() <= radius;
    }

    let point_x = point.x() - start_x;
    let point_y = point.y() - start_y;
    let point_z = point.z() - start_z;
    let axis_dir_x = axis_x / axis_len;
    let axis_dir_y = axis_y / axis_len;
    let axis_dir_z = axis_z / axis_len;
    let axial = point_x * axis_dir_x + point_y * axis_dir_y + point_z * axis_dir_z;

    if !(0.0..=axis_len).contains(&axial) {
        return false;
    }

    let proj_x = start_x + axis_dir_x * axial;
    let proj_y = start_y + axis_dir_y * axial;
    let proj_z = start_z + axis_dir_z * axial;
    let radial_x = point.x() - proj_x;
    let radial_y = point.y() - proj_y;
    let radial_z = point.z() - proj_z;
    (radial_x * radial_x + radial_y * radial_y + radial_z * radial_z).sqrt() <= radius
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

    #[test]
    fn primitive_set_exact_work_distinguishes_flat_from_ball_on_segment_endpoint_side() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let segment = LineSegment3D::new(Point3D::new(2.0, 5.0, 5.0), Point3D::new(8.0, 5.0, 5.0))
            .expect("segment must be valid");

        let mut flat_work = PrimitiveSetExactWork::new(bounds);
        flat_work.apply_primitive(ExactToolPrimitive::Flat {
            segment,
            radius: 1.0,
        });

        let mut ball_work = PrimitiveSetExactWork::new(bounds);
        ball_work.apply_primitive(ExactToolPrimitive::Ball {
            segment,
            radius: 1.0,
        });

        let endpoint_side = Point3D::new(1.5, 5.0, 5.0);
        assert!(flat_work.contains_material_at(&endpoint_side));
        assert!(!ball_work.contains_material_at(&endpoint_side));
    }

    #[test]
    fn primitive_set_exact_work_empty_returns_bounds_volume_even_with_tiny_pitch() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 20.0, 30.0));
        let work = PrimitiveSetExactWork::new(bounds);

        let volume = work.estimate_remaining_volume(f64::EPSILON);
        assert!((volume - bounds.volume()).abs() <= 1.0e-9);
    }

    #[test]
    fn primitive_set_exact_work_estimate_volume_never_exceeds_bounds_volume() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let work = PrimitiveSetExactWork::new(bounds);

        let volume = work.estimate_remaining_volume(6.0);
        assert!(volume <= bounds.volume());
    }

    #[test]
    fn primitive_set_exact_work_estimate_volume_rejects_non_finite_pitch() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let segment = LineSegment3D::new(Point3D::new(2.0, 5.0, 5.0), Point3D::new(8.0, 5.0, 5.0))
            .expect("segment must be valid");
        work.apply_primitive(ExactToolPrimitive::Ball {
            segment,
            radius: 1.0,
        });

        assert_eq!(work.estimate_remaining_volume(f64::NAN), 0.0);
        assert_eq!(work.estimate_remaining_volume(f64::INFINITY), 0.0);
    }

    #[test]
    fn primitive_set_exact_work_accepts_tiny_positive_pitch_with_primitives() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let segment = LineSegment3D::new(Point3D::new(2.0, 5.0, 5.0), Point3D::new(8.0, 5.0, 5.0))
            .expect("segment must be valid");
        work.apply_primitive(ExactToolPrimitive::Ball {
            segment,
            radius: 1.0,
        });

        let volume = work.estimate_remaining_volume(f64::EPSILON * 0.5);
        assert!(volume > 0.0);
        assert!(volume <= bounds.volume());
    }

    #[test]
    #[should_panic(expected = "PrimitiveSetExactWork::new received invalid bounds")]
    fn primitive_set_exact_work_new_rejects_invalid_bounds() {
        let invalid_bounds = Aabb3D::new(Point3D::new(1.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 1.0));
        let _ = PrimitiveSetExactWork::new(invalid_bounds);
    }

    #[test]
    #[should_panic(expected = "bounds coordinates must be finite")]
    fn primitive_set_exact_work_new_rejects_non_finite_bounds() {
        let invalid_bounds = Aabb3D::new(
            Point3D::new(f64::NAN, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 1.0),
        );
        let _ = PrimitiveSetExactWork::new(invalid_bounds);
    }

    #[test]
    #[should_panic(expected = "radius must be non-negative")]
    fn primitive_set_exact_work_apply_primitive_rejects_negative_radius() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let segment = LineSegment3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 0.0))
            .expect("segment must be valid");

        work.apply_primitive(ExactToolPrimitive::Flat {
            segment,
            radius: -1.0,
        });
    }

    #[test]
    #[should_panic(expected = "segment endpoints must be finite")]
    fn primitive_set_exact_work_apply_primitive_rejects_non_finite_segment() {
        let bounds = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 10.0, 10.0));
        let mut work = PrimitiveSetExactWork::new(bounds);
        let segment = LineSegment3D::new(
            Point3D::new(f64::INFINITY, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
        )
        .expect("segment must be valid");

        work.apply_primitive(ExactToolPrimitive::Flat {
            segment,
            radius: 1.0,
        });
    }
}
