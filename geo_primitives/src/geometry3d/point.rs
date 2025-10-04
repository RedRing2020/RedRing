/// 3D点プリミティブ
///
/// geo_core::Point3Dをベースとした3D点の拡張実装

use geo_core::{Point3D as CorePoint3D, Vector3D as CoreVector3D, Scalar};

/// 3D点（geo_core統合版）
#[derive(Debug, Clone)]
pub struct Point3D {
    inner: CorePoint3D,
}

impl Point3D {
    /// 新しい3D点を作成
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self {
            inner: CorePoint3D::from_f64(x, y, z),
        }
    }

    /// geo_core::Point3Dから作成
    pub fn from_geo_core(point: CorePoint3D) -> Self {
        Self { inner: point }
    }

    /// geo_core::Point3Dに変換
    pub fn to_geo_core(&self) -> CorePoint3D {
        self.inner.clone()
    }

    /// x座標を取得
    pub fn x(&self) -> f64 {
        self.inner.x().value()
    }

    /// y座標を取得
    pub fn y(&self) -> f64 {
        self.inner.y().value()
    }

    /// z座標を取得
    pub fn z(&self) -> f64 {
        self.inner.z().value()
    }

    /// 原点
    pub fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Point3D) -> f64 {
        let dx = self.x() - other.x();
        let dy = self.y() - other.y();
        let dz = self.z() - other.z();
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2つの点の中点を計算
    pub fn midpoint(&self, other: &Point3D) -> Point3D {
        Self::new(
            (self.x() + other.x()) / 2.0,
            (self.y() + other.y()) / 2.0,
            (self.z() + other.z()) / 2.0,
        )
    }

    /// 配列形式で取得
    pub fn to_array(&self) -> [f64; 3] {
        [self.x(), self.y(), self.z()]
    }

    /// 配列から生成
    pub fn from_array(arr: [f64; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }

    /// 点を3Dベクトルに変換（原点からのベクトル）
    pub fn to_vector(&self) -> CoreVector3D {
        CoreVector3D::new(Scalar::new(self.x()), Scalar::new(self.y()), Scalar::new(self.z()))
    }

    /// 他の点に向かうベクトルを計算
    pub fn vector_to(&self, other: &Point3D) -> CoreVector3D {
        CoreVector3D::new(
            Scalar::new(other.x() - self.x()),
            Scalar::new(other.y() - self.y()),
            Scalar::new(other.z() - self.z()),
        )
    }

    /// 移動
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Point3D {
        Self::new(self.x() + dx, self.y() + dy, self.z() + dz)
    }

    /// X軸周りの回転
    pub fn rotate_x(&self, angle_rad: f64) -> Point3D {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self::new(
            self.x(),
            self.y() * cos_a - self.z() * sin_a,
            self.y() * sin_a + self.z() * cos_a,
        )
    }

    /// Y軸周りの回転
    pub fn rotate_y(&self, angle_rad: f64) -> Point3D {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self::new(
            self.x() * cos_a + self.z() * sin_a,
            self.y(),
            -self.x() * sin_a + self.z() * cos_a,
        )
    }

    /// Z軸周りの回転
    pub fn rotate_z(&self, angle_rad: f64) -> Point3D {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self::new(
            self.x() * cos_a - self.y() * sin_a,
            self.x() * sin_a + self.y() * cos_a,
            self.z(),
        )
    }

    /// 指定点中心でのスケール
    pub fn scale(&self, factor: f64, center: &Point3D) -> Point3D {
        Self::new(
            center.x() + factor * (self.x() - center.x()),
            center.y() + factor * (self.y() - center.y()),
            center.z() + factor * (self.z() - center.z()),
        )
    }

    /// XY平面への投影
    pub fn project_xy(&self) -> crate::geometry2d::Point2D {
        crate::geometry2d::Point2D::new(self.x(), self.y())
    }

    /// XZ平面への投影
    pub fn project_xz(&self) -> crate::geometry2d::Point2D {
        crate::geometry2d::Point2D::new(self.x(), self.z())
    }

    /// YZ平面への投影
    pub fn project_yz(&self) -> crate::geometry2d::Point2D {
        crate::geometry2d::Point2D::new(self.y(), self.z())
    }
}

impl PartialEq for Point3D {
    fn eq(&self, other: &Self) -> bool {
        (self.x() - other.x()).abs() < 1e-10
            && (self.y() - other.y()).abs() < 1e-10
            && (self.z() - other.z()).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point3d_creation() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        assert_eq!(p.x(), 1.0);
        assert_eq!(p.y(), 2.0);
        assert_eq!(p.z(), 3.0);
    }

    #[test]
    fn test_point3d_distance() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(3.0, 4.0, 0.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point3d_midpoint() {
        let p1 = Point3D::new(0.0, 0.0, 0.0);
        let p2 = Point3D::new(6.0, 8.0, 10.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x(), 3.0);
        assert_eq!(mid.y(), 4.0);
        assert_eq!(mid.z(), 5.0);
    }

    #[test]
    fn test_point3d_rotate_z() {
        let p = Point3D::new(1.0, 0.0, 0.0);
        let rotated = p.rotate_z(std::f64::consts::PI / 2.0); // 90度回転
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
        assert_eq!(rotated.z(), 0.0);
    }

    #[test]
    fn test_point3d_translate() {
        let p = Point3D::new(1.0, 2.0, 3.0);
        let translated = p.translate(4.0, 5.0, 6.0);
        assert_eq!(translated.x(), 5.0);
        assert_eq!(translated.y(), 7.0);
        assert_eq!(translated.z(), 9.0);
    }

    #[test]
    fn test_point3d_projection() {
        let p = Point3D::new(1.0, 2.0, 3.0);

        let xy = p.project_xy();
        assert_eq!(xy.x(), 1.0);
        assert_eq!(xy.y(), 2.0);

        let xz = p.project_xz();
        assert_eq!(xz.x(), 1.0);
        assert_eq!(xz.y(), 3.0);

        let yz = p.project_yz();
        assert_eq!(yz.x(), 2.0);
        assert_eq!(yz.y(), 3.0);
    }

    #[test]
    fn test_point3d_scale() {
        let p = Point3D::new(2.0, 4.0, 6.0);
        let center = Point3D::new(1.0, 1.0, 1.0);
        let scaled = p.scale(2.0, &center);
        assert_eq!(scaled.x(), 3.0); // 1 + 2*(2-1) = 3
        assert_eq!(scaled.y(), 7.0); // 1 + 2*(4-1) = 7
        assert_eq!(scaled.z(), 11.0); // 1 + 2*(6-1) = 11
    }
}
