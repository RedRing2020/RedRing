//! Ray3D拡張機能の実装
//!
//! Core Foundation パターンに基づく Ray3D の拡張機能
//! 変換機能、高度な幾何操作を提供

use crate::{BBox3D, Point3D, Ray3D, Vector3D};
use geo_foundation::Scalar;

// ============================================================================
// Ray3D Transform Extensions
// ============================================================================

impl<T: Scalar> Ray3D<T> {
    // ========================================================================
    // Transform Methods
    // ========================================================================

    /// 平行移動
    pub fn translate(&self, offset: &Vector3D<T>) -> Self {
        let new_origin = self.origin() + *offset;
        Self::new(new_origin, self.direction()).unwrap()
    }

    /// 均一スケール
    pub fn scale_uniform(&self, center: &Point3D<T>, factor: T) -> Self {
        let relative_origin = Vector3D::from_points(center, &self.origin());
        let scaled_origin = relative_origin * factor;
        let new_origin = *center + scaled_origin;

        // 方向ベクトルはスケールされない（正規化済み）
        Self::new(new_origin, self.direction()).unwrap()
    }

    /// Ray の方向を新しい方向に設定
    pub fn with_direction(&self, new_direction: Vector3D<T>) -> Option<Self> {
        Self::new(self.origin(), new_direction)
    }

    /// Ray の起点を新しい点に設定
    pub fn with_origin(&self, new_origin: Point3D<T>) -> Self {
        Self::new(new_origin, self.direction()).unwrap()
    }

    /// 指定した長さで切断してLineSegment3Dに変換
    pub fn to_line_segment(&self, length: T) -> crate::LineSegment3D<T> {
        let end_point = self.point_at_parameter(length);
        crate::LineSegment3D::new(self.origin(), end_point).unwrap()
    }

    /// 指定範囲での境界ボックスを計算
    ///
    /// # 引数
    /// * `max_parameter` - 最大パラメータ値
    ///
    /// # 戻り値
    /// [0, max_parameter] 範囲での境界ボックス
    pub fn bounding_box_for_range(&self, max_parameter: T) -> BBox3D<T> {
        let start_point = self.origin();
        let end_point = self.point_at_parameter(max_parameter);

        BBox3D::from_point_collection(&[start_point, end_point]).unwrap()
    }

    // ========================================================================
    // Advanced Geometric Operations
    // ========================================================================

    /// 指定した平面との交点を計算
    ///
    /// # 引数
    /// * `plane_point` - 平面上の点
    /// * `plane_normal` - 平面の法線ベクトル（正規化済み）
    ///
    /// # 戻り値
    /// 交点が存在し、Ray の正の方向にある場合は Some(point)、そうでなければ None
    pub fn intersect_plane(
        &self,
        plane_point: &Point3D<T>,
        plane_normal: &Vector3D<T>,
    ) -> Option<Point3D<T>> {
        let denom = self.direction().dot(plane_normal);

        // Ray が平面に平行な場合
        if denom.abs() < T::EPSILON {
            return None;
        }

        let to_plane = Vector3D::from_points(&self.origin(), plane_point);
        let t = to_plane.dot(plane_normal) / denom;

        // 交点が Ray の正の方向にある場合のみ
        if t >= T::ZERO {
            Some(self.point_at_parameter(t))
        } else {
            None
        }
    }

    /// 球との交点を計算
    ///
    /// # 引数
    /// * `sphere_center` - 球の中心
    /// * `sphere_radius` - 球の半径
    ///
    /// # 戻り値
    /// 交点のリスト（0個、1個、または2個）
    pub fn intersect_sphere(
        &self,
        sphere_center: &Point3D<T>,
        sphere_radius: T,
    ) -> Vec<Point3D<T>> {
        let to_center = Vector3D::from_points(&self.origin(), sphere_center);
        let direction = self.direction();
        let a = direction.dot(&direction); // 常に1（正規化済み）
        let b = -to_center.dot(&direction) * (T::ONE + T::ONE);
        let c = to_center.dot(&to_center) - sphere_radius * sphere_radius;

        let discriminant = b * b - (T::ONE + T::ONE + T::ONE + T::ONE) * a * c;

        if discriminant < T::ZERO {
            // 交点なし
            return Vec::new();
        }

        let sqrt_discriminant = discriminant.sqrt();
        let two_a = (T::ONE + T::ONE) * a;

        let t1 = (-b - sqrt_discriminant) / two_a;
        let t2 = (-b + sqrt_discriminant) / two_a;

        let mut intersections = Vec::new();

        // t >= 0 の交点のみ追加
        if t1 >= T::ZERO {
            intersections.push(self.point_at_parameter(t1));
        }
        if t2 >= T::ZERO && (discriminant > T::EPSILON) {
            intersections.push(self.point_at_parameter(t2));
        }

        intersections
    }
}
