//! BBox3D Foundation トレイト実装
//!
//! BBox3D の Foundation Pattern 実装
//! Core Traits (Constructor/Properties/Measure) を実装

use crate::{BBox3D, Point3D};
use geo_foundation::{
    extension_foundation::ExtensionFoundation, BBox3DConstructor, BBox3DMeasure, BBox3DProperties,
    PrimitiveKind, Scalar, TolerantEq,
};

// ============================================================================
// Constructor トレイト実装
// ============================================================================

impl<T: Scalar> BBox3DConstructor<T> for BBox3D<T> {
    fn new(min: (T, T, T), max: (T, T, T)) -> Self {
        // 座標を正規化（min <= max）
        let actual_min = Point3D::new(min.0.min(max.0), min.1.min(max.1), min.2.min(max.2));
        let actual_max = Point3D::new(min.0.max(max.0), min.1.max(max.1), min.2.max(max.2));
        BBox3D::new(actual_min, actual_max)
    }

    fn from_point(point: (T, T, T)) -> Self {
        let p = Point3D::new(point.0, point.1, point.2);
        Self::from_point(p)
    }

    fn from_points(points: &[(T, T, T)]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let point_vec: Vec<Point3D<T>> = points
            .iter()
            .map(|&(x, y, z)| Point3D::new(x, y, z))
            .collect();

        Self::from_points(&point_vec)
    }

    fn from_center_size(center: (T, T, T), width: T, height: T, depth: T) -> Self {
        let half_width = width / (T::ONE + T::ONE);
        let half_height = height / (T::ONE + T::ONE);
        let half_depth = depth / (T::ONE + T::ONE);

        let min_point = Point3D::new(
            center.0 - half_width,
            center.1 - half_height,
            center.2 - half_depth,
        );
        let max_point = Point3D::new(
            center.0 + half_width,
            center.1 + half_height,
            center.2 + half_depth,
        );
        BBox3D::new(min_point, max_point)
    }

    fn unit_box() -> Self {
        let half = T::ONE / (T::ONE + T::ONE);
        let min_point = Point3D::new(-half, -half, -half);
        let max_point = Point3D::new(half, half, half);
        BBox3D::new(min_point, max_point)
    }

    fn empty() -> Self {
        // 無効な境界ボックス（min > max）
        let invalid_min = Point3D::new(T::ONE, T::ONE, T::ONE);
        let invalid_max = Point3D::new(T::ZERO, T::ZERO, T::ZERO);
        BBox3D::new(invalid_min, invalid_max)
    }

    fn from_2d_with_z_range(min_2d: (T, T), max_2d: (T, T), z_min: T, z_max: T) -> Self {
        let min_point = Point3D::new(min_2d.0, min_2d.1, z_min);
        let max_point = Point3D::new(max_2d.0, max_2d.1, z_max);
        BBox3D::new(min_point, max_point)
    }
}

// ============================================================================
// Properties トレイト実装
// ============================================================================

impl<T: Scalar> BBox3DProperties<T> for BBox3D<T> {
    fn min(&self) -> (T, T, T) {
        let min_point = BBox3D::min(self);
        (min_point.x(), min_point.y(), min_point.z())
    }

    fn max(&self) -> (T, T, T) {
        let max_point = BBox3D::max(self);
        (max_point.x(), max_point.y(), max_point.z())
    }

    fn center(&self) -> (T, T, T) {
        let center_point = BBox3D::center(self);
        (center_point.x(), center_point.y(), center_point.z())
    }

    fn width(&self) -> T {
        BBox3D::width(self)
    }

    fn height(&self) -> T {
        BBox3D::height(self)
    }

    fn depth(&self) -> T {
        BBox3D::depth(self)
    }

    fn size(&self) -> (T, T, T) {
        (
            BBox3D::width(self),
            BBox3D::height(self),
            BBox3D::depth(self),
        )
    }

    fn is_empty(&self) -> bool {
        BBox3D::is_empty(self)
    }

    fn is_valid(&self) -> bool {
        !BBox3D::is_empty(self)
    }

    fn is_point(&self) -> bool {
        let tolerance = T::EPSILON;
        BBox3D::width(self) <= tolerance
            && BBox3D::height(self) <= tolerance
            && BBox3D::depth(self) <= tolerance
    }

    fn vertices(&self) -> [(T, T, T); 8] {
        let min_point = BBox3D::min(self);
        let max_point = BBox3D::max(self);
        [
            (min_point.x(), min_point.y(), min_point.z()), // 0: 左下奥
            (max_point.x(), min_point.y(), min_point.z()), // 1: 右下奥
            (max_point.x(), max_point.y(), min_point.z()), // 2: 右上奥
            (min_point.x(), max_point.y(), min_point.z()), // 3: 左上奥
            (min_point.x(), min_point.y(), max_point.z()), // 4: 左下手前
            (max_point.x(), min_point.y(), max_point.z()), // 5: 右下手前
            (max_point.x(), max_point.y(), max_point.z()), // 6: 右上手前
            (min_point.x(), max_point.y(), max_point.z()), // 7: 左上手前
        ]
    }

    fn xy_projection(&self) -> ((T, T), (T, T)) {
        let min_point = BBox3D::min(self);
        let max_point = BBox3D::max(self);
        (
            (min_point.x(), min_point.y()),
            (max_point.x(), max_point.y()),
        )
    }

    fn xz_projection(&self) -> ((T, T), (T, T)) {
        let min_point = BBox3D::min(self);
        let max_point = BBox3D::max(self);
        (
            (min_point.x(), min_point.z()),
            (max_point.x(), max_point.z()),
        )
    }

    fn yz_projection(&self) -> ((T, T), (T, T)) {
        let min_point = BBox3D::min(self);
        let max_point = BBox3D::max(self);
        (
            (min_point.y(), min_point.z()),
            (max_point.y(), max_point.z()),
        )
    }

    fn dimension(&self) -> u32 {
        3
    }
}

// ============================================================================
// Measure トレイト実装
// ============================================================================

impl<T: Scalar> BBox3DMeasure<T> for BBox3D<T> {
    fn volume(&self) -> T {
        BBox3D::volume(self)
    }

    fn surface_area(&self) -> T {
        let width = BBox3D::width(self);
        let height = BBox3D::height(self);
        let depth = BBox3D::depth(self);
        let two = T::ONE + T::ONE;
        two * (width * height + height * depth + depth * width)
    }

    fn diagonal_length(&self) -> T {
        let width = BBox3D::width(self);
        let height = BBox3D::height(self);
        let depth = BBox3D::depth(self);
        (width * width + height * height + depth * depth).sqrt()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        let p = Point3D::new(point.0, point.1, point.2);
        BBox3D::contains_point(self, &p)
    }

    fn point_on_surface(&self, point: (T, T, T)) -> bool {
        if !<Self as BBox3DMeasure<T>>::contains_point(self, point) {
            return false;
        }

        let min = <Self as BBox3DProperties<T>>::min(self);
        let max = <Self as BBox3DProperties<T>>::max(self);
        let tolerance = T::EPSILON;

        // 各面からの距離をチェック
        let on_x_face =
            (point.0 - min.0).abs() <= tolerance || (point.0 - max.0).abs() <= tolerance;
        let on_y_face =
            (point.1 - min.1).abs() <= tolerance || (point.1 - max.1).abs() <= tolerance;
        let on_z_face =
            (point.2 - min.2).abs() <= tolerance || (point.2 - max.2).abs() <= tolerance;

        on_x_face || on_y_face || on_z_face
    }

    fn intersects(&self, other: &Self) -> bool {
        let self_min = <Self as BBox3DProperties<T>>::min(self);
        let self_max = <Self as BBox3DProperties<T>>::max(self);
        let other_min = <Self as BBox3DProperties<T>>::min(other);
        let other_max = <Self as BBox3DProperties<T>>::max(other);

        self_min.0 <= other_max.0
            && self_max.0 >= other_min.0
            && self_min.1 <= other_max.1
            && self_max.1 >= other_min.1
            && self_min.2 <= other_max.2
            && self_max.2 >= other_min.2
    }

    fn contains_bbox(&self, other: &Self) -> bool {
        let self_min = <Self as BBox3DProperties<T>>::min(self);
        let self_max = <Self as BBox3DProperties<T>>::max(self);
        let other_min = <Self as BBox3DProperties<T>>::min(other);
        let other_max = <Self as BBox3DProperties<T>>::max(other);

        self_min.0 <= other_min.0
            && self_min.1 <= other_min.1
            && self_min.2 <= other_min.2
            && self_max.0 >= other_max.0
            && self_max.1 >= other_max.1
            && self_max.2 >= other_max.2
    }

    fn intersection(&self, other: &Self) -> Option<Self> {
        if !<Self as BBox3DMeasure<T>>::intersects(self, other) {
            return None;
        }

        let self_min = <Self as BBox3DProperties<T>>::min(self);
        let self_max = <Self as BBox3DProperties<T>>::max(self);
        let other_min = <Self as BBox3DProperties<T>>::min(other);
        let other_max = <Self as BBox3DProperties<T>>::max(other);

        let min_x = self_min.0.max(other_min.0);
        let min_y = self_min.1.max(other_min.1);
        let min_z = self_min.2.max(other_min.2);
        let max_x = self_max.0.min(other_max.0);
        let max_y = self_max.1.min(other_max.1);
        let max_z = self_max.2.min(other_max.2);

        Some(<Self as BBox3DConstructor<T>>::new(
            (min_x, min_y, min_z),
            (max_x, max_y, max_z),
        ))
    }

    fn union(&self, other: &Self) -> Self {
        let self_min = <Self as BBox3DProperties<T>>::min(self);
        let self_max = <Self as BBox3DProperties<T>>::max(self);
        let other_min = <Self as BBox3DProperties<T>>::min(other);
        let other_max = <Self as BBox3DProperties<T>>::max(other);

        let min_x = self_min.0.min(other_min.0);
        let min_y = self_min.1.min(other_min.1);
        let min_z = self_min.2.min(other_min.2);
        let max_x = self_max.0.max(other_max.0);
        let max_y = self_max.1.max(other_max.1);
        let max_z = self_max.2.max(other_max.2);

        <Self as BBox3DConstructor<T>>::new((min_x, min_y, min_z), (max_x, max_y, max_z))
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        // 内部にある場合は距離0
        if <Self as BBox3DMeasure<T>>::contains_point(self, point) {
            return T::ZERO;
        }

        let min = <Self as BBox3DProperties<T>>::min(self);
        let max = <Self as BBox3DProperties<T>>::max(self);

        // 外部の場合、最も近い面への距離
        let dx = if point.0 < min.0 {
            min.0 - point.0
        } else if point.0 > max.0 {
            point.0 - max.0
        } else {
            T::ZERO
        };

        let dy = if point.1 < min.1 {
            min.1 - point.1
        } else if point.1 > max.1 {
            point.1 - max.1
        } else {
            T::ZERO
        };

        let dz = if point.2 < min.2 {
            min.2 - point.2
        } else if point.2 > max.2 {
            point.2 - max.2
        } else {
            T::ZERO
        };

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T) {
        let min = <Self as BBox3DProperties<T>>::min(self);
        let max = <Self as BBox3DProperties<T>>::max(self);
        let clamped_x = point.0.max(min.0).min(max.0);
        let clamped_y = point.1.max(min.1).min(max.1);
        let clamped_z = point.2.max(min.2).min(max.2);
        (clamped_x, clamped_y, clamped_z)
    }

    fn distance_to_bbox(&self, other: &Self) -> T {
        if <Self as BBox3DMeasure<T>>::intersects(self, other) {
            return T::ZERO;
        }

        let self_center = <Self as BBox3DProperties<T>>::center(self);
        let other_center = <Self as BBox3DProperties<T>>::center(other);

        // 簡易実装: 中心間距離から各境界ボックスのサイズを引く
        let two = T::ONE + T::ONE;
        let dx = (self_center.0 - other_center.0).abs()
            - (<Self as BBox3DProperties<T>>::width(self)
                + <Self as BBox3DProperties<T>>::width(other))
                / two;
        let dy = (self_center.1 - other_center.1).abs()
            - (<Self as BBox3DProperties<T>>::height(self)
                + <Self as BBox3DProperties<T>>::height(other))
                / two;
        let dz = (self_center.2 - other_center.2).abs()
            - (<Self as BBox3DProperties<T>>::depth(self)
                + <Self as BBox3DProperties<T>>::depth(other))
                / two;

        let dx = dx.max(T::ZERO);
        let dy = dy.max(T::ZERO);
        let dz = dz.max(T::ZERO);

        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    fn expand(&self, margin: T) -> Self {
        let min = <Self as BBox3DProperties<T>>::min(self);
        let max = <Self as BBox3DProperties<T>>::max(self);
        <Self as BBox3DConstructor<T>>::new(
            (min.0 - margin, min.1 - margin, min.2 - margin),
            (max.0 + margin, max.1 + margin, max.2 + margin),
        )
    }

    fn shrink(&self, margin: T) -> Option<Self> {
        let min = <Self as BBox3DProperties<T>>::min(self);
        let max = <Self as BBox3DProperties<T>>::max(self);
        let new_min = (min.0 + margin, min.1 + margin, min.2 + margin);
        let new_max = (max.0 - margin, max.1 - margin, max.2 - margin);

        if new_min.0 <= new_max.0 && new_min.1 <= new_max.1 && new_min.2 <= new_max.2 {
            Some(<Self as BBox3DConstructor<T>>::new(new_min, new_max))
        } else {
            None
        }
    }

    fn extend_to_include_point(&self, point: (T, T, T)) -> Self {
        let min = <Self as BBox3DProperties<T>>::min(self);
        let max = <Self as BBox3DProperties<T>>::max(self);
        <Self as BBox3DConstructor<T>>::new(
            (min.0.min(point.0), min.1.min(point.1), min.2.min(point.2)),
            (max.0.max(point.0), max.1.max(point.1), max.2.max(point.2)),
        )
    }

    fn extend_to_include_bbox(&self, other: &Self) -> Self {
        <Self as BBox3DMeasure<T>>::union(self, other)
    }

    fn projection_interval(&self, axis: (T, T, T)) -> (T, T) {
        let vertices = <Self as BBox3DProperties<T>>::vertices(self);
        let mut min_proj = axis.0 * vertices[0].0 + axis.1 * vertices[0].1 + axis.2 * vertices[0].2;
        let mut max_proj = min_proj;

        for &vertex in &vertices[1..] {
            let proj = axis.0 * vertex.0 + axis.1 * vertex.1 + axis.2 * vertex.2;
            min_proj = min_proj.min(proj);
            max_proj = max_proj.max(proj);
        }

        (min_proj, max_proj)
    }
}

// ============================================================================
// Legacy Foundation Trait Implementation (互換性維持)
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for BBox3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::BBox
    }

    fn bounding_box(&self) -> Self::BBox {
        *self // 境界ボックス自身がその境界ボックス
    }

    fn measure(&self) -> Option<T> {
        Some(<Self as BBox3DMeasure<T>>::volume(self)) // Core Traitのvolume()を使用
    }
}

impl<T: Scalar> TolerantEq<T> for BBox3D<T> {
    fn tolerant_eq(&self, other: &Self, tolerance: T) -> bool {
        // 最小点と最大点の距離をチェック
        let min_distance = self.min().distance_to(&other.min());
        let max_distance = self.max().distance_to(&other.max());

        // 許容誤差として単一のスカラー値を使用
        min_distance <= tolerance && max_distance <= tolerance
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point3D;

    #[test]
    fn test_extension_foundation() {
        let bbox = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 5.0, 3.0));

        assert_eq!(bbox.primitive_kind(), PrimitiveKind::BBox);
        assert!(bbox.measure().is_some());
        assert_eq!(bbox.measure().unwrap(), bbox.volume());

        let self_bbox = bbox.bounding_box();
        assert_eq!(bbox, self_bbox);
    }

    #[test]
    fn test_tolerant_eq() {
        let bbox1 = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 5.0, 3.0));

        let bbox2 = BBox3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(10.0, 5.0, 3.0));

        let bbox3 = BBox3D::new(Point3D::new(1.0, 1.0, 1.0), Point3D::new(11.0, 6.0, 4.0));

        let tolerance = 0.01; // スカラー値の許容誤差

        assert!(bbox1.tolerant_eq(&bbox2, tolerance));
        assert!(!bbox1.tolerant_eq(&bbox3, tolerance));
    }
}
