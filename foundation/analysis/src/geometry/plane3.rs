//! 3次元平面（数値計算用）
//!
//! 純粋な数値計算用の平面定義
//! 参照点と法線ベクトルで構成される無限平面

use crate::{
    linalg::{point3::Point3, vector::Vector3},
    Scalar,
};

/// 3次元平面（無限平面）
///
/// ax + by + cz + d = 0 の形式で表現される平面
/// または参照点と法線ベクトルで定義される平面
/// CADの座標系を持つ Plane3D とは異なり、純粋な数値計算用
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane3<T: Scalar> {
    /// 平面上の参照点
    reference_point: Point3<T>,
    /// 平面の法線ベクトル（正規化されている）
    normal: Vector3<T>,
}

impl<T: Scalar> Plane3<T> {
    /// 点と法線ベクトルから平面を作成
    pub fn from_point_and_normal(point: Point3<T>, normal: Vector3<T>) -> Option<Self> {
        let length = normal.norm();
        if length == T::ZERO {
            return None;
        }

        let normalized_normal = normal / length;
        Some(Self {
            reference_point: point,
            normal: normalized_normal,
        })
    }

    /// 平面方程式の係数から作成
    /// ax + by + cz + d = 0
    pub fn from_coefficients(a: T, b: T, c: T, d: T) -> Option<Self> {
        let normal = Vector3::new(a, b, c);
        let normal_length = normal.norm();

        if normal_length == T::ZERO {
            return None;
        }

        let normalized_normal = normal / normal_length;

        // 平面上の点を見つける（最も座標値の大きい軸を使用）
        let abs_a = a.abs();
        let abs_b = b.abs();
        let abs_c = c.abs();

        let point = if abs_a >= abs_b && abs_a >= abs_c {
            Point3::new(-d / a, T::ZERO, T::ZERO)
        } else if abs_b >= abs_c {
            Point3::new(T::ZERO, -d / b, T::ZERO)
        } else {
            Point3::new(T::ZERO, T::ZERO, -d / c)
        };

        Some(Self {
            reference_point: point,
            normal: normalized_normal,
        })
    }

    /// 参照点を取得
    pub fn reference_point(&self) -> Point3<T> {
        self.reference_point
    }

    /// 法線ベクトルを取得
    pub fn normal(&self) -> Vector3<T> {
        self.normal
    }

    /// 点から平面までの符号付き距離
    pub fn distance_to_point(&self, point: Point3<T>) -> T {
        let to_point = point - self.reference_point; // Point3 - Point3 = Vector3
        to_point.dot(&self.normal)
    }

    /// 点が平面上にあるかチェック
    pub fn contains_point(&self, point: Point3<T>, tolerance: T) -> bool {
        self.distance_to_point(point).abs() <= tolerance
    }

    /// 点を平面に投影
    pub fn project_point(&self, point: Point3<T>) -> Point3<T> {
        let distance = self.distance_to_point(point);
        point - self.normal * distance // Point3 - Vector3 = Point3
    }

    /// 平面方程式の係数を取得
    /// Returns (a, b, c, d) where ax + by + cz + d = 0
    pub fn equation_coefficients(&self) -> (T, T, T, T) {
        let a = self.normal.x();
        let b = self.normal.y();
        let c = self.normal.z();
        let d = -(a * self.reference_point.x()
            + b * self.reference_point.y()
            + c * self.reference_point.z());
        (a, b, c, d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infinite_plane_creation() {
        let point = Point3::new(1.0, 2.0, 3.0);
        let normal = Vector3::new(0.0, 0.0, 1.0);

        let plane = Plane3::from_point_and_normal(point, normal).unwrap();

        assert_eq!(plane.reference_point(), point);
        assert_eq!(plane.normal(), Vector3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_distance_calculation() {
        let plane =
            Plane3::from_point_and_normal(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0))
                .unwrap();

        let test_point = Point3::new(1.0, 2.0, 5.0);
        assert_eq!(plane.distance_to_point(test_point), 5.0);
    }

    #[test]
    fn test_projection() {
        let plane =
            Plane3::from_point_and_normal(Point3::new(0.0, 0.0, 0.0), Vector3::new(0.0, 0.0, 1.0))
                .unwrap();

        let test_point = Point3::new(1.0, 2.0, 5.0);
        let projected = plane.project_point(test_point);

        assert_eq!(projected, Point3::new(1.0, 2.0, 0.0));
    }
}
