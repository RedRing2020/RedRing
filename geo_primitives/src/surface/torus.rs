/// トーラスサーフェスプリミティブ

use crate::geometry3d::Point3D;
use geo_core::{Vector3D, Scalar};
use geo_core::vector::Vector;
use geo_core::tolerance::ToleranceContext;
use crate::{GeometricPrimitive, PrimitiveKind, BoundingBox};

/// 3Dトーラス（ドーナツ形状）
#[derive(Debug, Clone)]
pub struct Torus {
    center: Point3D,        // 中心点
    major_radius: f64,      // 主半径（トーラスの中心軸からチューブ中心までの距離）
    minor_radius: f64,      // 副半径（チューブの半径）
    axis: Vector3D,         // 軸方向（正規化済み）
}

impl Torus {
    /// 新しいトーラスを作成
    pub fn new(center: Point3D, major_radius: f64, minor_radius: f64, axis: Vector3D) -> Option<Self> {
        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return None;
        }

        if major_radius <= minor_radius {
            return None; // 主半径が副半径より大きくなければならない
        }

        let axis_length = (axis.x().value().powi(2) + axis.y().value().powi(2) + axis.z().value().powi(2)).sqrt();
        let normalized_axis = if axis_length > 1e-12 {
            Vector3D::new(
                Scalar::new(axis.x().value() / axis_length),
                Scalar::new(axis.y().value() / axis_length),
                Scalar::new(axis.z().value() / axis_length),
            )
        } else {
            Vector3D::z_axis() // unit_z
        };

        Some(Self {
            center,
            major_radius,
            minor_radius,
            axis: normalized_axis,
        })
    }

    /// Z軸を軸とするトーラスを作成
    pub fn new_z_axis(center: Point3D, major_radius: f64, minor_radius: f64) -> Option<Self> {
                Self::new(center, major_radius, minor_radius, Vector3D::z_axis())
    }

    /// 中心点を取得
    pub fn center(&self) -> &Point3D {
        &self.center
    }

    /// 主半径を取得
    pub fn major_radius(&self) -> f64 {
        self.major_radius
    }

    /// 副半径を取得
    pub fn minor_radius(&self) -> f64 {
        self.minor_radius
    }

    /// 軸方向ベクトルを取得
    pub fn axis(&self) -> &Vector3D {
        &self.axis
    }

    /// パラメトリック評価 (u: [0, 2π], v: [0, 2π])
    /// u: 主円周角、v: 副円周角
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let tolerance = ToleranceContext::default();
        // 軸に垂直な2つの単位ベクトルを計算
        let temp = if self.axis.x().value().abs() < 0.9 {
            Vector3D::x_axis()
        } else {
            Vector3D::y_axis()
        };

        let u_vec = self.axis.cross(&temp).normalize(&tolerance).unwrap();
        let v_vec = self.axis.cross(&u_vec).normalize(&tolerance).unwrap();

        // 主円上の点（チューブの中心線）
        let major_circle_point = u_vec.clone() * Scalar::from_f64(self.major_radius * u.cos()) + v_vec.clone() * Scalar::from_f64(self.major_radius * u.sin());

        // チューブの半径方向
        let tube_radial = u_vec * Scalar::from_f64(u.cos() * v.cos()) + v_vec * Scalar::from_f64(u.sin() * v.cos()) + self.axis.clone() * Scalar::from_f64(v.sin());

        let final_point = major_circle_point + tube_radial * Scalar::from_f64(self.minor_radius);

        Point3D::new(
            self.center.x() + final_point.x().value(),
            self.center.y() + final_point.y().value(),
            self.center.z() + final_point.z().value(),
        )
    }

    /// 指定パラメータでの法線ベクトル
    pub fn normal(&self, u: f64, v: f64) -> Vector3D {
        let tolerance = ToleranceContext::default();
        // 軸に垂直な2つの単位ベクトルを計算
        let temp = if self.axis.x().value().abs() < 0.9 {
            Vector3D::x_axis()
        } else {
            Vector3D::y_axis()
        };

        let u_vec = self.axis.cross(&temp).normalize(&tolerance).unwrap();
        let v_vec = self.axis.cross(&u_vec).normalize(&tolerance).unwrap();

        // チューブの半径方向が法線方向
        let normal = u_vec * Scalar::from_f64(u.cos() * v.cos()) + v_vec * Scalar::from_f64(u.sin() * v.cos()) + self.axis.clone() * Scalar::from_f64(v.sin());

        normal.normalize(&tolerance).unwrap_or(Vector3D::z_axis())
    }

    /// トーラスの表面積
    pub fn surface_area(&self) -> f64 {
        4.0 * std::f64::consts::PI * std::f64::consts::PI * self.major_radius * self.minor_radius
    }

    /// トーラスの体積
    pub fn volume(&self) -> f64 {
        2.0 * std::f64::consts::PI * std::f64::consts::PI * self.major_radius * self.minor_radius * self.minor_radius
    }

    /// 点がトーラス表面上にあるかを判定
    pub fn contains_point(&self, point: &Point3D, tolerance: f64) -> bool {
        let to_point = self.center.vector_to(point);

        // 軸への投影
        let axis_projection = to_point.dot(&self.axis).value();

        // 軸に垂直な成分
        let perpendicular = to_point - self.axis.clone() * Scalar::from_f64(axis_projection);
        let distance_from_axis = perpendicular.norm().value();

        // 主円からの距離
        let distance_from_major_circle = (distance_from_axis - self.major_radius).abs();

        // 軸方向の距離と主円からの距離でトーラス表面までの距離を計算
        let distance_to_surface = (axis_projection * axis_projection + distance_from_major_circle * distance_from_major_circle).sqrt();

        (distance_to_surface - self.minor_radius).abs() < tolerance
    }

    /// 点がトーラス内部にあるかを判定
    pub fn contains_point_inside(&self, point: &Point3D) -> bool {
        let to_point = self.center.vector_to(point);

        // 軸への投影
        let axis_projection = to_point.dot(&self.axis).value();

        // 軸に垂直な成分
        let perpendicular = to_point - self.axis.clone() * Scalar::from_f64(axis_projection);
        let distance_from_axis = perpendicular.norm().value();

        // 主円からの距離が主半径より小さい場合は中央のホール部分
        if distance_from_axis <= self.major_radius - self.minor_radius {
            return false;
        }

        // 主円からの距離が主半径+副半径より大きい場合は外部
        if distance_from_axis >= self.major_radius + self.minor_radius {
            return false;
        }

        // 主円上の最近点までの距離を計算
        let distance_from_major_circle = (distance_from_axis - self.major_radius).abs();

        // 軸方向の距離と主円からの距離でトーラス表面までの距離を計算
        let distance_to_surface = (axis_projection * axis_projection + distance_from_major_circle * distance_from_major_circle).sqrt();

        // 表面上の点は内部ではない（厳密に小さい場合のみ内部）
        distance_to_surface < self.minor_radius && distance_to_surface > 1e-10
    }

    /// トーラスの種類を判定
    pub fn torus_type(&self) -> TorusType {
        if self.major_radius > self.minor_radius {
            TorusType::Ring // 通常のドーナツ型
        } else if self.major_radius == self.minor_radius {
            TorusType::Horn // ホーン型（特異点あり）
        } else {
            TorusType::Spindle // スピンドル型（自己交差）
        }
    }

    /// 移動した新しいトーラスを作成
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Torus {
        Self {
            center: self.center.translate(dx, dy, dz),
            major_radius: self.major_radius,
            minor_radius: self.minor_radius,
            axis: self.axis.clone(),
        }
    }

    /// 指定点を中心にスケール
    pub fn scale(&self, factor: f64, center: &Point3D) -> Option<Torus> {
        if factor <= 0.0 {
            return None;
        }

        let new_center = self.center.scale(factor, center);
        let new_major_radius = self.major_radius * factor;
        let new_minor_radius = self.minor_radius * factor;

        Self::new(new_center, new_major_radius, new_minor_radius, self.axis.clone())
    }

    /// 軸周りに回転
    pub fn rotate_around_axis(&self, _angle: f64) -> Torus {
        // トーラスは軸周りの回転に対して不変
        self.clone()
    }
}

/// トーラスの種類を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TorusType {
    Ring,    // R > r（通常のドーナツ型）
    Horn,    // R = r（ホーン型、特異点あり）
    Spindle, // R < r（スピンドル型、自己交差）
}

impl GeometricPrimitive for Torus {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Torus
    }

    fn bounding_box(&self) -> BoundingBox {
        // 簡易実装：軸が任意方向の場合の正確な計算は複雑
        let outer_radius = self.major_radius + self.minor_radius;

        let min_x = self.center.x() - outer_radius;
        let max_x = self.center.x() + outer_radius;
        let min_y = self.center.y() - outer_radius;
        let max_y = self.center.y() + outer_radius;
        let min_z = self.center.z() - outer_radius;
        let max_z = self.center.z() + outer_radius;

        BoundingBox::new(
            Point3D::new(min_x, min_y, min_z).to_geo_core(),
            Point3D::new(max_x, max_y, max_z).to_geo_core(),
        )
    }

    fn measure(&self) -> Option<f64> {
        Some(self.surface_area())
    }
}

impl PartialEq for Torus {
    fn eq(&self, other: &Self) -> bool {
        self.center == other.center
            && (self.major_radius - other.major_radius).abs() < 1e-10
            && (self.minor_radius - other.minor_radius).abs() < 1e-10
            && self.axis == other.axis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_torus_creation() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let torus = Torus::new_z_axis(center, 3.0, 1.0).unwrap();
        assert_eq!(torus.major_radius(), 3.0);
        assert_eq!(torus.minor_radius(), 1.0);
        assert_eq!(torus.axis(), &Vector3D::z_axis());
    }

    #[test]
    fn test_torus_invalid_radii() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        // 負の半径
        assert!(Torus::new_z_axis(center.clone(), -1.0, 1.0).is_none());
        assert!(Torus::new_z_axis(center.clone(), 3.0, -1.0).is_none());
        // ゼロ半径
        assert!(Torus::new_z_axis(center.clone(), 0.0, 1.0).is_none());
        assert!(Torus::new_z_axis(center.clone(), 3.0, 0.0).is_none());
        // 主半径が副半径より小さい（スピンドル型は作成可能だが、通常は避ける）
        assert!(Torus::new_z_axis(center, 1.0, 3.0).is_none());
    }

    #[test]
    fn test_torus_volume_and_surface_area() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let torus = Torus::new_z_axis(center, 3.0, 1.0).unwrap();

        let expected_volume = 2.0 * std::f64::consts::PI * std::f64::consts::PI * 3.0 * 1.0 * 1.0;
        let expected_surface_area = 4.0 * std::f64::consts::PI * std::f64::consts::PI * 3.0 * 1.0;

        assert!((torus.volume() - expected_volume).abs() < 1e-10);
        assert!((torus.surface_area() - expected_surface_area).abs() < 1e-10);
    }

    #[test]
    fn test_torus_type() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let ring_torus = Torus::new_z_axis(center, 3.0, 1.0).unwrap();

        assert_eq!(ring_torus.torus_type(), TorusType::Ring);
    }

    #[test]
    fn test_torus_contains_point_inside() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let torus = Torus::new_z_axis(center, 3.0, 1.0).unwrap();

        // 中心は内部にない（ホールがあるため）
        let center_point = Point3D::new(0.0, 0.0, 0.0);
        assert!(!torus.contains_point_inside(&center_point));

        // 主円上の点（表面上なので内部ではない）
        let on_major_circle = Point3D::new(3.0, 0.0, 0.0);
        assert!(!torus.contains_point_inside(&on_major_circle));

        // チューブ内部の点
        let inside_tube = Point3D::new(2.5, 0.0, 0.0);
        assert!(torus.contains_point_inside(&inside_tube));

        // 明確に外部の点
        let outside = Point3D::new(5.0, 0.0, 0.0);
        assert!(!torus.contains_point_inside(&outside));
    }

    #[test]
    fn test_torus_translate() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let torus = Torus::new_z_axis(center, 3.0, 1.0).unwrap();
        let translated = torus.translate(1.0, 2.0, 3.0);

        assert_eq!(translated.center().x(), 1.0);
        assert_eq!(translated.center().y(), 2.0);
        assert_eq!(translated.center().z(), 3.0);
        assert_eq!(translated.major_radius(), 3.0);
        assert_eq!(translated.minor_radius(), 1.0);
    }

    #[test]
    fn test_torus_scale() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let torus = Torus::new_z_axis(center.clone(), 3.0, 1.0).unwrap();
        let scale_center = Point3D::new(0.0, 0.0, 0.0);
        let scaled = torus.scale(2.0, &scale_center).unwrap();

        assert_eq!(scaled.center(), &center);
        assert_eq!(scaled.major_radius(), 6.0);
        assert_eq!(scaled.minor_radius(), 2.0);
    }

    #[test]
    fn test_torus_custom_axis() {
        let center = Point3D::new(0.0, 0.0, 0.0);
        let axis = Vector3D::x_axis();
        let torus = Torus::new(center, 3.0, 1.0, axis).unwrap();

        assert_eq!(torus.axis(), &Vector3D::x_axis());
        assert_eq!(torus.major_radius(), 3.0);
        assert_eq!(torus.minor_radius(), 1.0);
    }
}
