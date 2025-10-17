//! Point3D拡張機能
//!
//! Extension Foundation パターンに基づく Point3D の拡張実装
//! 高度な構築・変換・空間関係メソッドを提供

use crate::{Point3D, Vector3D};
use geo_foundation::{Angle, Scalar};

// ============================================================================
// Extension Implementation (高度な機能)
// ============================================================================

impl<T: Scalar> Point3D<T> {
    // ========================================================================
    // Advanced Construction Methods (Extension)
    // ========================================================================

    /// 球面座標から点を作成（r, θ, φ）
    pub fn from_spherical(radius: T, theta: Angle<T>, phi: Angle<T>) -> Self {
        let sin_phi = phi.to_radians().sin();
        let cos_phi = phi.to_radians().cos();
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();

        Self::new(
            radius * sin_phi * cos_theta,
            radius * sin_phi * sin_theta,
            radius * cos_phi,
        )
    }

    /// 円筒座標から点を作成（r, θ, z）
    pub fn from_cylindrical(radius: T, theta: Angle<T>, z: T) -> Self {
        let sin_theta = theta.to_radians().sin();
        let cos_theta = theta.to_radians().cos();

        Self::new(radius * cos_theta, radius * sin_theta, z)
    }

    // ========================================================================
    // Interpolation Methods (Extension)
    // ========================================================================

    /// 線形補間
    pub fn lerp(&self, other: &Self, t: T) -> Self {
        Point3D::new(
            self.x() + t * (other.x() - self.x()),
            self.y() + t * (other.y() - self.y()),
            self.z() + t * (other.z() - self.z()),
        )
    }

    /// 中点計算
    pub fn midpoint(&self, other: &Self) -> Self {
        let half = T::ONE / (T::ONE + T::ONE);
        self.lerp(other, half)
    }

    /// 重心計算（3点）
    pub fn centroid_3(p1: &Self, p2: &Self, p3: &Self) -> Self {
        let one_third = T::ONE / (T::ONE + T::ONE + T::ONE);
        Point3D::new(
            (p1.x() + p2.x() + p3.x()) * one_third,
            (p1.y() + p2.y() + p3.y()) * one_third,
            (p1.z() + p2.z() + p3.z()) * one_third,
        )
    }

    /// 重心計算（複数点）
    pub fn centroid(points: &[Self]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let count = T::from_usize(points.len());
        let sum = points.iter().fold(Self::origin(), |acc, p| {
            Self::new(acc.x() + p.x(), acc.y() + p.y(), acc.z() + p.z())
        });

        Some(Self::new(sum.x() / count, sum.y() / count, sum.z() / count))
    }

    // ========================================================================
    // Transformation Methods (Extension)
    // ========================================================================

    /// X軸反射
    pub fn reflect_x(&self) -> Self {
        Self::new(-self.x(), self.y(), self.z())
    }

    /// Y軸反射
    pub fn reflect_y(&self) -> Self {
        Self::new(self.x(), -self.y(), self.z())
    }

    /// Z軸反射
    pub fn reflect_z(&self) -> Self {
        Self::new(self.x(), self.y(), -self.z())
    }

    /// XY平面反射
    pub fn reflect_xy(&self) -> Self {
        Self::new(self.x(), self.y(), -self.z())
    }

    /// XZ平面反射
    pub fn reflect_xz(&self) -> Self {
        Self::new(self.x(), -self.y(), self.z())
    }

    /// YZ平面反射
    pub fn reflect_yz(&self) -> Self {
        Self::new(-self.x(), self.y(), self.z())
    }

    /// 原点反射
    pub fn reflect_origin(&self) -> Self {
        Self::new(-self.x(), -self.y(), -self.z())
    }

    /// 指定点を中心とした回転（X軸周り）
    pub fn rotate_x(&self, center: &Self, angle: Angle<T>) -> Self {
        let translated = *self - Vector3D::new(center.x(), center.y(), center.z());
        let cos_a = angle.to_radians().cos();
        let sin_a = angle.to_radians().sin();

        let rotated = Self::new(
            translated.x(),
            translated.y() * cos_a - translated.z() * sin_a,
            translated.y() * sin_a + translated.z() * cos_a,
        );

        rotated + Vector3D::new(center.x(), center.y(), center.z())
    }

    /// 指定点を中心とした回転（Y軸周り）
    pub fn rotate_y(&self, center: &Self, angle: Angle<T>) -> Self {
        let translated = *self - Vector3D::new(center.x(), center.y(), center.z());
        let cos_a = angle.to_radians().cos();
        let sin_a = angle.to_radians().sin();

        let rotated = Self::new(
            translated.x() * cos_a + translated.z() * sin_a,
            translated.y(),
            -translated.x() * sin_a + translated.z() * cos_a,
        );

        rotated + Vector3D::new(center.x(), center.y(), center.z())
    }

    /// 指定点を中心とした回転（Z軸周り）
    pub fn rotate_z(&self, center: &Self, angle: Angle<T>) -> Self {
        let translated = *self - Vector3D::new(center.x(), center.y(), center.z());
        let cos_a = angle.to_radians().cos();
        let sin_a = angle.to_radians().sin();

        let rotated = Self::new(
            translated.x() * cos_a - translated.y() * sin_a,
            translated.x() * sin_a + translated.y() * cos_a,
            translated.z(),
        );

        rotated + Vector3D::new(center.x(), center.y(), center.z())
    }

    /// スケール変換（中心点指定）
    pub fn scale(&self, center: &Self, factor: T) -> Self {
        let offset = *self - Vector3D::new(center.x(), center.y(), center.z());
        let scaled_offset = Vector3D::new(
            offset.x() * factor,
            offset.y() * factor,
            offset.z() * factor,
        );
        *center + scaled_offset
    }

    // ========================================================================
    // Conversion Methods (Extension)
    // ========================================================================

    /// ベクトルとして取得
    pub fn to_vector(&self) -> Vector3D<T> {
        Vector3D::new(self.x(), self.y(), self.z())
    }

    /// 他の点への方向ベクトルを計算
    pub fn direction_to(&self, other: &Self) -> Vector3D<T> {
        Vector3D::new(
            other.x() - self.x(),
            other.y() - self.y(),
            other.z() - self.z(),
        )
    }

    /// ベクトルから点を作成
    pub fn from_vector(vector: Vector3D<T>) -> Self {
        Self::new(vector.x(), vector.y(), vector.z())
    }

    /// 球面座標に変換（r, θ, φ）
    pub fn to_spherical(&self) -> (T, Angle<T>, Angle<T>) {
        let r = self.norm();
        if r <= T::EPSILON {
            return (T::ZERO, Angle::zero(), Angle::zero());
        }

        let theta = Angle::from_radians(self.y().atan2(self.x()));
        let phi = Angle::from_radians((self.z() / r).acos());

        (r, theta, phi)
    }

    /// 円筒座標に変換（r, θ, z）
    pub fn to_cylindrical(&self) -> (T, Angle<T>, T) {
        let r = (self.x() * self.x() + self.y() * self.y()).sqrt();
        let theta = Angle::from_radians(self.y().atan2(self.x()));

        (r, theta, self.z())
    }

    // ========================================================================
    // Predicate Methods (Extension)
    // ========================================================================

    /// 原点判定
    pub fn is_origin(&self) -> bool {
        self.norm() <= T::EPSILON
    }

    /// 近似等価判定
    pub fn is_approximately_equal(&self, other: &Self, tolerance: T) -> bool {
        self.distance_to(other) <= tolerance
    }

    /// 点が指定した平面上にあるかを判定（Z=定数）
    pub fn on_z_plane(&self, z: T, tolerance: T) -> bool {
        (self.z() - z).abs() <= tolerance
    }

    /// 点が指定した球内にあるかを判定
    pub fn inside_sphere(&self, center: &Self, radius: T) -> bool {
        self.distance_to(center) <= radius
    }

    /// 点が指定した立方体内にあるかを判定
    pub fn inside_box(&self, min: &Self, max: &Self) -> bool {
        self.x() >= min.x()
            && self.x() <= max.x()
            && self.y() >= min.y()
            && self.y() <= max.y()
            && self.z() >= min.z()
            && self.z() <= max.z()
    }

    // ========================================================================
    // Advanced Calculation Methods (Extension)
    // ========================================================================

    /// 3つの点で形成される三角形の面積
    pub fn triangle_area(p1: &Self, p2: &Self, p3: &Self) -> T {
        let v1 = Vector3D::from_points(p1, p2);
        let v2 = Vector3D::from_points(p1, p3);
        let cross = v1.cross(&v2);
        cross.length() / (T::ONE + T::ONE)
    }

    /// 4つの点で形成される四面体の体積
    pub fn tetrahedron_volume(p1: &Self, p2: &Self, p3: &Self, p4: &Self) -> T {
        let v1 = Vector3D::from_points(p1, p2);
        let v2 = Vector3D::from_points(p1, p3);
        let v3 = Vector3D::from_points(p1, p4);

        let scalar_triple_product = v1.dot(&v2.cross(&v3));
        scalar_triple_product.abs() / (T::ONE + T::ONE + T::ONE + T::ONE + T::ONE + T::ONE)
        // /6
    }
}

// ============================================================================
// Extension Foundation trait implementations
// ============================================================================

// TODO: Extension Foundation traits need migration to new traits system
/*
impl<T: Scalar> ExtensionFoundation<T> for Point3D<T> {
    // 高度な変形操作
    // 空間関係操作
    // 次元変換操作
}
*/
