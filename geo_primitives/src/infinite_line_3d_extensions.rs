//! InfiniteLine3D Extension 機能
//!
//! Extension Foundation パターンに基づく InfiniteLine3D の拡張実装

use crate::{Direction3D, InfiniteLine3D, Point2D, Point3D, Vector2D, Vector3D};
use geo_foundation::Scalar;

// ============================================================================
// Extension Methods Implementation
// ============================================================================

impl<T: Scalar> InfiniteLine3D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// X軸に平行な直線を作成
    pub fn x_axis(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_x()).unwrap()
    }

    /// Y軸に平行な直線を作成
    pub fn y_axis(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_y()).unwrap()
    }

    /// Z軸に平行な直線を作成
    pub fn z_axis(point: Point3D<T>) -> Self {
        Self::new(point, Vector3D::unit_z()).unwrap()
    }

    /// 原点を通るX軸
    pub fn origin_x_axis() -> Self {
        Self::x_axis(Point3D::origin())
    }

    /// 原点を通るY軸
    pub fn origin_y_axis() -> Self {
        Self::y_axis(Point3D::origin())
    }

    /// 原点を通るZ軸
    pub fn origin_z_axis() -> Self {
        Self::z_axis(Point3D::origin())
    }

    // ========================================================================
    // Advanced Geometric Analysis (Extension)
    // ========================================================================

    /// 軸に平行かどうかを判定
    pub fn is_parallel_to_axis(&self, axis: Vector3D<T>, tolerance: T) -> bool {
        let normalized_axis =
            Direction3D::from_vector(axis.normalize()).unwrap_or(Direction3D::positive_x());
        let dot_product = self.direction().dot(&normalized_axis).abs();
        (dot_product - T::ONE).abs() <= tolerance
    }

    /// X軸に平行かどうかを判定
    pub fn is_parallel_to_x_axis(&self, tolerance: T) -> bool {
        self.is_parallel_to_axis(Vector3D::unit_x(), tolerance)
    }

    /// Y軸に平行かどうかを判定
    pub fn is_parallel_to_y_axis(&self, tolerance: T) -> bool {
        self.is_parallel_to_axis(Vector3D::unit_y(), tolerance)
    }

    /// Z軸に平行かどうかを判定
    pub fn is_parallel_to_z_axis(&self, tolerance: T) -> bool {
        self.is_parallel_to_axis(Vector3D::unit_z(), tolerance)
    }

    // ========================================================================
    // Advanced Relationship Analysis (Extension)
    // ========================================================================

    /// 他の直線との関係を判定
    pub fn relationship_with(&self, other: &Self, tolerance: T) -> LineRelationship {
        // 方向ベクトルの平行性チェック
        let cross_product = self.direction().cross(&other.direction());
        let is_parallel = cross_product.length() <= tolerance;

        if is_parallel {
            // 平行な場合、同一直線かチェック
            if self.contains_point(&other.point(), tolerance) {
                LineRelationship::Coincident
            } else {
                LineRelationship::Parallel
            }
        } else {
            // 平行でない場合、交差か非交差かチェック
            let to_other_point = Vector3D::new(
                other.point().x() - self.point().x(),
                other.point().y() - self.point().y(),
                other.point().z() - self.point().z(),
            );

            // スカラ三重積でねじれ位置判定
            let scalar_triple_product = to_other_point.dot(&cross_product);

            if scalar_triple_product.abs() <= tolerance {
                LineRelationship::Intersecting
            } else {
                LineRelationship::Skew
            }
        }
    }

    /// 直線が平行かを判定
    pub fn is_parallel(&self, other: &Self, tolerance: T) -> bool {
        matches!(
            self.relationship_with(other, tolerance),
            LineRelationship::Parallel | LineRelationship::Coincident
        )
    }

    /// 直線が同一かを判定
    pub fn is_coincident(&self, other: &Self, tolerance: T) -> bool {
        matches!(
            self.relationship_with(other, tolerance),
            LineRelationship::Coincident
        )
    }

    /// 直線が交差するかを判定
    pub fn is_intersecting(&self, other: &Self, tolerance: T) -> bool {
        matches!(
            self.relationship_with(other, tolerance),
            LineRelationship::Intersecting
        )
    }

    /// 直線がねじれ位置にあるかを判定
    pub fn is_skew(&self, other: &Self, tolerance: T) -> bool {
        matches!(
            self.relationship_with(other, tolerance),
            LineRelationship::Skew
        )
    }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================

    /// 直線を平行移動
    pub fn translate(&self, offset: Vector3D<T>) -> Self {
        Self::new(
            Point3D::new(
                self.point().x() + offset.x(),
                self.point().y() + offset.y(),
                self.point().z() + offset.z(),
            ),
            self.direction().as_vector(),
        )
        .unwrap()
    }

    /// 方向を反転
    pub fn reverse(&self) -> Self {
        Self::new(self.point(), (-self.direction()).as_vector()).unwrap()
    }

    /// 指定点周りの回転（軸と角度指定）
    pub fn rotate_around_point(
        &self,
        center: Point3D<T>,
        axis: Vector3D<T>,
        angle: T,
    ) -> Option<Self> {
        let normalized_axis = axis.normalize();
        if normalized_axis.length() <= T::EPSILON {
            return None;
        }

        // Rodriguesの回転公式を使用
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 点の回転
        let to_point = Vector3D::new(
            self.point().x() - center.x(),
            self.point().y() - center.y(),
            self.point().z() - center.z(),
        );

        let rotated_to_point = to_point * cos_angle
            + normalized_axis.cross(&to_point) * sin_angle
            + normalized_axis * (normalized_axis.dot(&to_point) * (T::ONE - cos_angle));

        let new_point = Point3D::new(
            center.x() + rotated_to_point.x(),
            center.y() + rotated_to_point.y(),
            center.z() + rotated_to_point.z(),
        );

        // 方向ベクトルの回転
        let rotated_direction = self.direction() * cos_angle
            + normalized_axis.cross(&self.direction()) * sin_angle
            + normalized_axis * (normalized_axis.dot(&self.direction()) * (T::ONE - cos_angle));

        Some(Self::new(new_point, rotated_direction).unwrap())
    }

    /// 原点周りの回転
    pub fn rotate_around_origin(&self, axis: Vector3D<T>, angle: T) -> Option<Self> {
        self.rotate_around_point(Point3D::origin(), axis, angle)
    }

    // ========================================================================
    // Conversion Methods (Extension)
    // ========================================================================

    /// 2次元投影（Z成分を無視）
    pub fn to_2d(&self) -> crate::InfiniteLine2D<T> {
        let point_2d = Point2D::new(self.point().x(), self.point().y());
        let direction_2d = Vector2D::new(self.direction().x(), self.direction().y());
        crate::InfiniteLine2D::new(point_2d, direction_2d).unwrap()
    }

    /// XY平面への投影
    pub fn project_to_xy_plane(&self) -> crate::InfiniteLine2D<T> {
        self.to_2d()
    }

    /// XZ平面への投影
    pub fn project_to_xz_plane(&self) -> crate::InfiniteLine2D<T> {
        let projected_point = Point2D::new(self.point().x(), self.point().z());
        let projected_direction = Vector2D::new(self.direction().x(), self.direction().z());
        crate::InfiniteLine2D::new(projected_point, projected_direction).unwrap()
    }

    /// YZ平面への投影
    pub fn project_to_yz_plane(&self) -> crate::InfiniteLine2D<T> {
        let projected_point = Point2D::new(self.point().y(), self.point().z());
        let projected_direction = Vector2D::new(self.direction().y(), self.direction().z());
        crate::InfiniteLine2D::new(projected_point, projected_direction).unwrap()
    }

    // ========================================================================
    // Advanced Helper Methods (Extension)
    // ========================================================================

    /// 直線上の最も近い点を取得（project_pointのエイリアス）
    pub fn closest_point(&self, point: &Point3D<T>) -> Point3D<T> {
        self.project_point(point)
    }

    /// 方向を反転（reverse_direction エイリアス）
    pub fn reverse_direction(&self) -> Self {
        self.reverse()
    }

    /// 直線間の最短距離を計算
    pub fn distance_between_lines(&self, other: &Self) -> T {
        let direction_cross = self.direction().cross(&other.direction());
        let cross_length = direction_cross.length();

        if cross_length <= T::EPSILON {
            // 平行線の場合
            return self.distance_to_point(&other.point());
        }

        // ねじれ位置の場合
        let to_other_point = Vector3D::new(
            other.point().x() - self.point().x(),
            other.point().y() - self.point().y(),
            other.point().z() - self.point().z(),
        );

        to_other_point.dot(&direction_cross).abs() / cross_length
    }
}

/// 3次元直線間の関係を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineRelationship {
    /// 同一直線
    Coincident,
    /// 平行
    Parallel,
    /// 交差
    Intersecting,
    /// ねじれ位置
    Skew,
}
