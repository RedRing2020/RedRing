//! Plane変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 全幾何プリミティブで共通利用可能な統一インターフェース

use crate::{Plane3D, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Plane3D<T> {
    type Transformed = Plane3D<T>;
    type Vector2D = Vector3D<T>; // 3D なので Vector3D を使用
    type Point2D = Point3D<T>; // 3D なので Point3D を使用
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        // 平面の参照点を平行移動
        let new_point = Point3D::new(
            self.point().x() + translation.x(),
            self.point().y() + translation.y(),
            self.point().z() + translation.z(),
        );

        // 法線ベクトルは変わらない
        Plane3D::from_point_and_normal(new_point, self.normal()).unwrap()
    }

    /// 回転（Z軸周りの回転として実装）
    fn rotate(&self, center: Self::Point2D, angle: Self::Angle) -> Self::Transformed {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let translated_point = Vector3D::new(
            self.point().x() - center.x(),
            self.point().y() - center.y(),
            self.point().z() - center.z(),
        );

        let rotated_point = Vector3D::new(
            translated_point.x() * cos_a - translated_point.y() * sin_a,
            translated_point.x() * sin_a + translated_point.y() * cos_a,
            translated_point.z(),
        );

        let new_point = Point3D::new(
            rotated_point.x() + center.x(),
            rotated_point.y() + center.y(),
            rotated_point.z() + center.z(),
        );

        let rotated_normal = Vector3D::new(
            self.normal().x() * cos_a - self.normal().y() * sin_a,
            self.normal().x() * sin_a + self.normal().y() * cos_a,
            self.normal().z(),
        );

        Plane3D::from_point_and_normal(new_point, rotated_normal).unwrap()
    }

    /// 指定中心でのスケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        // 点をスケール
        let translated_point = Vector3D::new(
            self.point().x() - center.x(),
            self.point().y() - center.y(),
            self.point().z() - center.z(),
        );

        let scaled_point = Vector3D::new(
            translated_point.x() * factor,
            translated_point.y() * factor,
            translated_point.z() * factor,
        );

        let new_point = Point3D::new(
            scaled_point.x() + center.x(),
            scaled_point.y() + center.y(),
            scaled_point.z() + center.z(),
        );

        // 法線ベクトルはスケールの逆数でスケール（正規化を保つため）
        let inv_factor = T::ONE / factor;
        let scaled_normal = Vector3D::new(
            self.normal().x() * inv_factor,
            self.normal().y() * inv_factor,
            self.normal().z() * inv_factor,
        )
        .normalize();

        Plane3D::from_point_and_normal(new_point, scaled_normal).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_foundation::extensions::BasicTransform;

    #[test]
    fn test_translate() {
        let point = Point3D::new(0.0, 0.0, 0.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        let translation = Vector3D::new(1.0, 2.0, 3.0);
        let translated = plane.translate(translation);

        let expected_point = Point3D::new(1.0, 2.0, 3.0);
        assert!((translated.point().x() - expected_point.x()).abs() < 1e-10);
        assert!((translated.point().y() - expected_point.y()).abs() < 1e-10);
        assert!((translated.point().z() - expected_point.z()).abs() < 1e-10);
        assert!((translated.normal().x() - normal.x()).abs() < 1e-10);
        assert!((translated.normal().y() - normal.y()).abs() < 1e-10);
        assert!((translated.normal().z() - normal.z()).abs() < 1e-10);
    }

    #[test]
    fn test_rotate() {
        let point = Point3D::new(1.0, 0.0, 0.0);
        let normal = Vector3D::new(1.0, 0.0, 0.0);
        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        let center = Point3D::new(0.0, 0.0, 0.0);
        let angle = Angle::from_degrees(90.0);
        let rotated = plane.rotate(center, angle);

        // 90度回転後、点は(0, 1, 0)、法線は(0, 1, 0)になるはず
        let expected_point = Point3D::new(0.0, 1.0, 0.0);
        let expected_normal = Vector3D::new(0.0, 1.0, 0.0);

        assert!((rotated.point().x() - expected_point.x()).abs() < 1e-10);
        assert!((rotated.point().y() - expected_point.y()).abs() < 1e-10);
        assert!((rotated.point().z() - expected_point.z()).abs() < 1e-10);
        assert!((rotated.normal().x() - expected_normal.x()).abs() < 1e-10);
        assert!((rotated.normal().y() - expected_normal.y()).abs() < 1e-10);
        assert!((rotated.normal().z() - expected_normal.z()).abs() < 1e-10);
    }

    #[test]
    fn test_scale() {
        let point = Point3D::new(1.0, 1.0, 1.0);
        let normal = Vector3D::new(0.0, 0.0, 1.0);
        let plane = Plane3D::from_point_and_normal(point, normal).unwrap();

        let center = Point3D::new(0.0, 0.0, 0.0);
        let factor = 2.0;
        let scaled = plane.scale(center, factor);

        // 点は2倍にスケール、法線は正規化を維持
        let expected_point = Point3D::new(2.0, 2.0, 2.0);
        assert!((scaled.point().x() - expected_point.x()).abs() < 1e-10);
        assert!((scaled.point().y() - expected_point.y()).abs() < 1e-10);
        assert!((scaled.point().z() - expected_point.z()).abs() < 1e-10);

        // 法線の長さは1を維持
        let normal_magnitude = (scaled.normal().x() * scaled.normal().x()
            + scaled.normal().y() * scaled.normal().y()
            + scaled.normal().z() * scaled.normal().z())
        .sqrt();
        assert!((normal_magnitude - 1.0).abs() < 1e-10);
    }
}
