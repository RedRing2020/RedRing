/// 楕円体サーフェスプリミティブ

use crate::geometry3d::Point3D;
use geo_core::{Vector3D, Scalar};
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

/// 3D楕円体
#[derive(Debug, Clone)]
pub struct Ellipsoid {
    center: Point3D,    // 中心点
    semi_axes: [f64; 3], // 半軸長 [a, b, c]
}

impl Ellipsoid {
    /// 新しい楕円体を作成
    pub fn new(center: Point3D, semi_axis_a: f64, semi_axis_b: f64, semi_axis_c: f64) -> Option<Self> {
        if semi_axis_a <= 0.0 || semi_axis_b <= 0.0 || semi_axis_c <= 0.0 {
            None
        } else {
            Some(Self {
                center,
                semi_axes: [semi_axis_a, semi_axis_b, semi_axis_c],
            })
        }
    }

    /// 球を作成（半径が等しい楕円体）
    pub fn sphere(center: Point3D, radius: f64) -> Option<Self> {
        Self::new(center, radius, radius, radius)
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point3D {
        &self.center
    }

    /// X軸方向の半軸長（a）を取得
    pub fn semi_axis_a(&self) -> f64 {
        self.semi_axes[0]
    }

    /// Y軸方向の半軸長（b）を取得
    pub fn semi_axis_b(&self) -> f64 {
        self.semi_axes[1]
    }

    /// Z軸方向の半軸長（c）を取得
    pub fn semi_axis_c(&self) -> f64 {
        self.semi_axes[2]
    }

    /// 半軸長の配列を取得
    pub fn semi_axes(&self) -> [f64; 3] {
        self.semi_axes
    }

    /// パラメトリック評価 (u: [0, 2π], v: [-π/2, π/2])
    /// u: 経度角、v: 緯度角
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let x = self.semi_axes[0] * v.cos() * u.cos();
        let y = self.semi_axes[1] * v.cos() * u.sin();
        let z = self.semi_axes[2] * v.sin();

        Point3D::new(
            self.center.x() + x,
            self.center.y() + y,
            self.center.z() + z,
        )
    }

    /// 指定パラメータでの法線ベクトル
    pub fn normal(&self, u: f64, v: f64) -> Vector3D {
        // 楕円体の法線は勾配ベクトルに比例
        let x = v.cos() * u.cos() / self.semi_axes[0];
        let y = v.cos() * u.sin() / self.semi_axes[1];
        let z = v.sin() / self.semi_axes[2];

        // 正規化
        let length = (x * x + y * y + z * z).sqrt();
        if length > 1e-12 {
            Vector3D::new(
                Scalar::new(x / length),
                Scalar::new(y / length),
                Scalar::new(z / length),
            )
        } else {
            Vector3D::new(Scalar::new(0.0), Scalar::new(0.0), Scalar::new(1.0))
        }
    }

    /// 楕円体の表面積（近似）
    pub fn surface_area(&self) -> f64 {
        let a = self.semi_axes[0];
        let b = self.semi_axes[1];
        let c = self.semi_axes[2];

        // 楕円体の表面積は厳密には楕円積分で表現される
        // ここではNomoto近似式を使用
        let p = 1.6075;
        let ap = a.powf(p);
        let bp = b.powf(p);
        let cp = c.powf(p);

        4.0 * std::f64::consts::PI * ((ap * bp + ap * cp + bp * cp) / 3.0).powf(1.0 / p)
    }

    /// 楕円体の体積
    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.semi_axes[0] * self.semi_axes[1] * self.semi_axes[2]
    }

    /// 点が楕円体表面上にあるかを判定
    pub fn contains_point(&self, point: &Point3D, tolerance: f64) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let dz = point.z() - self.center.z();

        let value = (dx / self.semi_axes[0]).powi(2)
                  + (dy / self.semi_axes[1]).powi(2)
                  + (dz / self.semi_axes[2]).powi(2);

        (value - 1.0).abs() < tolerance
    }

    /// 点が楕円体内部にあるかを判定
    pub fn contains_point_inside(&self, point: &Point3D) -> bool {
        let dx = point.x() - self.center.x();
        let dy = point.y() - self.center.y();
        let dz = point.z() - self.center.z();

        let value = (dx / self.semi_axes[0]).powi(2)
                  + (dy / self.semi_axes[1]).powi(2)
                  + (dz / self.semi_axes[2]).powi(2);

        value < 1.0
    }

    /// 楕円体が球かどうかを判定
    pub fn is_sphere(&self) -> bool {
        let tolerance = 1e-10;
        (self.semi_axes[0] - self.semi_axes[1]).abs() < tolerance
            && (self.semi_axes[1] - self.semi_axes[2]).abs() < tolerance
    }

    /// 偏心率（離心率）を計算（最大・最小軸のみ考慮）
    pub fn eccentricity(&self) -> f64 {
        let mut axes = self.semi_axes;
        axes.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let a = axes[0]; // 最大軸
        let c = axes[2]; // 最小軸

        if a <= c {
            0.0 // 球の場合
        } else {
            (1.0 - (c / a).powi(2)).sqrt()
        }
    }

    /// 移動した新しい楕円体を作成
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Ellipsoid {
        Self {
            center: self.center.translate(dx, dy, dz),
            semi_axes: self.semi_axes,
        }
    }

    /// 指定点を中心にスケール
    pub fn scale(&self, factor: f64, center: &Point3D) -> Option<Ellipsoid> {
        if factor <= 0.0 {
            return None;
        }

        let new_center = self.center.scale(factor, center);
        let new_semi_axes = [
            self.semi_axes[0] * factor,
            self.semi_axes[1] * factor,
            self.semi_axes[2] * factor,
        ];

        Some(Self {
            center: new_center,
            semi_axes: new_semi_axes,
        })
    }

    /// 軸方向に異なるスケールを適用
    pub fn scale_axes(&self, scale_x: f64, scale_y: f64, scale_z: f64) -> Option<Ellipsoid> {
        if scale_x <= 0.0 || scale_y <= 0.0 || scale_z <= 0.0 {
            return None;
        }

        let new_semi_axes = [
            self.semi_axes[0] * scale_x,
            self.semi_axes[1] * scale_y,
            self.semi_axes[2] * scale_z,
        ];

        Some(Self {
            center: self.center.clone(),
            semi_axes: new_semi_axes,
        })
    }
}

impl GeometricPrimitive for Ellipsoid {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Ellipsoid
    }

    fn bounding_box(&self) -> BoundingBox {
        let min_x = self.center.x() - self.semi_axes[0];
        let max_x = self.center.x() + self.semi_axes[0];
        let min_y = self.center.y() - self.semi_axes[1];
        let max_y = self.center.y() + self.semi_axes[1];
        let min_z = self.center.z() - self.semi_axes[2];
        let max_z = self.center.z() + self.semi_axes[2];

        BoundingBox::new(
            Point3D::new(min_x, min_y, min_z).to_geo_core(),
            Point3D::new(max_x, max_y, max_z).to_geo_core(),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

impl PartialEq for Ellipsoid {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && (self.semi_axes[0] - other.semi_axes[0]).abs() < 1e-10
            && (self.semi_axes[1] - other.semi_axes[1]).abs() < 1e-10
            && (self.semi_axes[2] - other.semi_axes[2]).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsoid_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipsoid = Ellipsoid::new(center, 2.0, 3.0, 4.0).unwrap();
        assert_eq!(ellipsoid.semi_axis_a(), 2.0);
        assert_eq!(ellipsoid.semi_axis_b(), 3.0);
        assert_eq!(ellipsoid.semi_axis_c(), 4.0);
    }

    #[test]
    fn test_ellipsoid_invalid_axes() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        assert!(Ellipsoid::new(center.clone(), -1.0, 3.0, 4.0).is_none());
        assert!(Ellipsoid::new(center.clone(), 2.0, 0.0, 4.0).is_none());
        assert!(Ellipsoid::new(center, 2.0, 3.0, -1.0).is_none());
    }

    #[test]
    fn test_ellipsoid_sphere_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Ellipsoid::sphere(center, 5.0).unwrap();
        assert!(sphere.is_sphere());
        assert_eq!(sphere.semi_axis_a(), 5.0);
        assert_eq!(sphere.semi_axis_b(), 5.0);
        assert_eq!(sphere.semi_axis_c(), 5.0);
    }

    #[test]
    fn test_ellipsoid_volume() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipsoid = Ellipsoid::new(center, 1.0, 2.0, 3.0).unwrap();
        let expected_volume = (4.0 / 3.0) * std::f64::consts::PI * 1.0 * 2.0 * 3.0;
        assert!((ellipsoid.volume() - expected_volume).abs() < 1e-10);
    }

    #[test]
    fn test_ellipsoid_contains_point() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipsoid = Ellipsoid::new(center, 2.0, 3.0, 4.0).unwrap();

        let inside = Point3D::new(1.0, 1.0, 1.0);
        let outside = Point3D::new(3.0, 3.0, 3.0);

        assert!(ellipsoid.contains_point_inside(&inside));
        assert!(!ellipsoid.contains_point_inside(&outside));
    }

    #[test]
    fn test_ellipsoid_is_sphere() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Ellipsoid::new(center.clone(), 5.0, 5.0, 5.0).unwrap();
        let ellipsoid = Ellipsoid::new(center, 2.0, 3.0, 4.0).unwrap();

        assert!(sphere.is_sphere());
        assert!(!ellipsoid.is_sphere());
    }

    #[test]
    fn test_ellipsoid_eccentricity() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let sphere = Ellipsoid::new(center.clone(), 5.0, 5.0, 5.0).unwrap();
        let ellipsoid = Ellipsoid::new(center, 4.0, 3.0, 2.0).unwrap(); // a=4, c=2

        assert!((sphere.eccentricity() - 0.0).abs() < 1e-10);

        let expected_ecc = (1.0f64 - (2.0f64 / 4.0f64).powi(2)).sqrt(); // sqrt(1 - (c/a)²)
        assert!((ellipsoid.eccentricity() - expected_ecc).abs() < 1e-10);
    }

    #[test]
    fn test_ellipsoid_translate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipsoid = Ellipsoid::new(center, 2.0, 3.0, 4.0).unwrap();
        let translated = ellipsoid.translate(1.0, 2.0, 3.0);

        assert_eq!(translated.center().x(), 1.0);
        assert_eq!(translated.center().y(), 2.0);
        assert_eq!(translated.center().z(), 3.0);
        assert_eq!(translated.semi_axes(), [2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_ellipsoid_scale() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipsoid = Ellipsoid::new(center.clone(), 2.0, 3.0, 4.0).unwrap();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scaled = ellipsoid.scale(2.0, &scale_center).unwrap();

        assert_eq!(scaled.center(), &center);
        assert_eq!(scaled.semi_axes(), [4.0, 6.0, 8.0]);
    }

    #[test]
    fn test_ellipsoid_scale_axes() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ellipsoid = Ellipsoid::new(center.clone(), 2.0, 3.0, 4.0).unwrap();
        let scaled = ellipsoid.scale_axes(2.0, 1.5, 0.5).unwrap();

        assert_eq!(scaled.center(), &center);
        assert_eq!(scaled.semi_axes(), [4.0, 4.5, 2.0]);
    }
}
