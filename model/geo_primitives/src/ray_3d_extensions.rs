//! Ray3D Extensions 実装
//!
//! Foundation統一システムに基づくRay3Dの拡張機能
//! Core機能は ray_3d.rs を参照

use crate::{BBox3D, InfiniteLine3D, Point3D, Ray3D, Vector3D};
use geo_foundation::Scalar;

// ============================================================================
// Core trait implementations
// ============================================================================

impl<T: Scalar> Clone for Ray3D<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Scalar> Copy for Ray3D<T> {}

impl<T: Scalar> PartialEq for Ray3D<T> {
    fn eq(&self, other: &Self) -> bool {
        self.origin() == other.origin() && self.direction_vector() == other.direction_vector()
    }
}

impl<T: Scalar> std::fmt::Debug for Ray3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ray3D")
            .field("origin", &self.origin())
            .field("direction", &self.direction_vector())
            .finish()
    }
}

impl<T: Scalar> std::fmt::Display for Ray3D<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ray3D(origin: {:?}, direction: {:?})",
            self.origin(),
            self.direction_vector()
        )
    }
}

// ============================================================================
// Extended geometric operations (moved from core)
// ============================================================================

impl<T: Scalar> Ray3D<T> {
    /// InfiniteLine3D への変換
    ///
    /// # 戻り値
    /// 同じ起点と方向を持つ無限直線
    pub fn to_infinite_line(&self) -> InfiniteLine3D<T> {
        InfiniteLine3D::new(self.origin(), self.direction_vector())
            .expect("Ray direction should always create valid InfiniteLine3D")
    }

    /// 境界ボックスを取得（無限のため最大値を使用）
    pub fn bounding_box(&self) -> BBox3D<T> {
        // Ray は無限に延びるため、方向に基づいて最大値を設定
        BBox3D::from_points(&[self.origin(), Point3D::new(T::MAX, T::MAX, T::MAX)])
            .unwrap_or_else(|| BBox3D::from_point(self.origin()))
    }

    /// パラメータの範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::INFINITY)
    }

    /// パラメータ t での接線ベクトルを取得
    pub fn tangent_at_parameter(&self, _t: T) -> Vector3D<T> {
        self.direction_vector()
    }

    /// 点が境界上にあるかを判定（Ray の場合は起点のみ）
    pub fn on_boundary(&self, point: &Point3D<T>, tolerance: T) -> bool {
        self.origin().distance_to(point) <= tolerance
    }

    /// 点までの距離を計算
    pub fn distance_to_point(&self, point: &Point3D<T>) -> T {
        let to_point = *point - self.origin();
        let projection_length = self.direction_vector().dot(&to_point);

        if projection_length <= T::ZERO {
            // 点が Ray の起点より後ろにある場合
            self.origin().distance_to(point)
        } else {
            // 点を Ray に垂直投影した点までの距離
            let direction_offset = self.direction_vector() * projection_length;
            let projection = Point3D::new(
                self.origin().x() + direction_offset.x(),
                self.origin().y() + direction_offset.y(),
                self.origin().z() + direction_offset.z(),
            );
            point.distance_to(&projection)
        }
    }

    /// Ray が指定した点の方向を向いているかを判定
    pub fn points_towards(&self, target: &Point3D<T>) -> bool {
        let to_target = *target - self.origin();
        self.direction_vector().dot(&to_target) > T::ZERO
    }

    /// 点が Ray の前方にあるかを判定
    pub fn is_point_ahead(&self, point: &Point3D<T>) -> bool {
        let to_point = *point - self.origin();
        self.direction_vector().dot(&to_point) > T::ZERO
    }

    /// 点が Ray の後方にあるかを判定
    pub fn is_point_behind(&self, point: &Point3D<T>) -> bool {
        !self.is_point_ahead(point)
    }

    /// Ray 上で指定した点に最も近い点を取得
    pub fn closest_point_on_ray(&self, point: &Point3D<T>) -> Point3D<T> {
        let to_point = *point - self.origin();
        let projection_length = self.direction_vector().dot(&to_point);

        if projection_length <= T::ZERO {
            // 投影が起点より後ろの場合は起点を返す
            self.origin()
        } else {
            // 投影した点を返す
            {
                let direction_offset = self.direction_vector() * projection_length;
                Point3D::new(
                    self.origin().x() + direction_offset.x(),
                    self.origin().y() + direction_offset.y(),
                    self.origin().z() + direction_offset.z(),
                )
            }
        }
    }

    /// 指定した点に最も近い Ray 上の点のパラメータを取得
    pub fn closest_parameter(&self, point: &Point3D<T>) -> T {
        let t = self.parameter_for_point(point);
        t.max(T::ZERO) // t < 0 の場合は 0 (起点) を返す
    }

    // ========================================================================
    // Transform Methods
    // ========================================================================

    /// 平行移動
    pub fn translate(&self, offset: &Vector3D<T>) -> Self {
        let new_origin = Point3D::new(
            self.origin().x() + offset.x(),
            self.origin().y() + offset.y(),
            self.origin().z() + offset.z(),
        );
        Self::new(new_origin, self.direction().as_vector()).unwrap()
    }

    /// 均一スケール
    pub fn scale_uniform(&self, center: &Point3D<T>, factor: T) -> Self {
        let relative_origin = Vector3D::from_points(center, &self.origin());
        let scaled_origin = relative_origin * factor;
        let new_origin = Point3D::new(
            center.x() + scaled_origin.x(),
            center.y() + scaled_origin.y(),
            center.z() + scaled_origin.z(),
        );

        // 方向ベクトルはスケールされない（正規化済み）
        Self::new(new_origin, self.direction().as_vector()).unwrap()
    }

    /// Ray の方向を新しい方向に設定
    pub fn with_direction(&self, new_direction: Vector3D<T>) -> Option<Self> {
        Self::new(self.origin(), new_direction)
    }

    /// Ray の起点を新しい点に設定
    pub fn with_origin(&self, new_origin: Point3D<T>) -> Self {
        Self::new(new_origin, self.direction().as_vector()).unwrap()
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
        let denom = self.direction().as_vector().dot(plane_normal);

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
