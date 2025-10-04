/// 点プリミティブ
///
/// geo_core::Point2Dをベースとした2D点の拡張実装

use geo_core::{Point2D as CorePoint2D, Vector2D};

/// 2D点（geo_core統合版）
#[derive(Debug, Clone)]
pub struct Point2D {
    inner: CorePoint2D,
}

impl Point2D {
    /// 新しい2D点を作成
    pub fn new(x: f64, y: f64) -> Self {
        Self { inner: CorePoint2D::new(x, y) }
    }

    /// geo_core::Point2Dから作成
    pub fn from_geo_core(point: CorePoint2D) -> Self {
        Self { inner: point }
    }

    /// geo_core::Point2Dに変換
    pub fn to_geo_core(&self) -> CorePoint2D {
        self.inner.clone()
    }

    /// x座標を取得
    pub fn x(&self) -> f64 { self.inner.x() }

    /// y座標を取得
    pub fn y(&self) -> f64 { self.inner.y() }

    /// 原点
    pub fn origin() -> Self {
        Self::new(0.0, 0.0)
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Point2D) -> f64 {
        let dx = self.x() - other.x();
        let dy = self.y() - other.y();
        (dx * dx + dy * dy).sqrt()
    }

    /// 2つの点の中点を計算
    pub fn midpoint(&self, other: &Point2D) -> Point2D {
        Self::new(
            (self.x() + other.x()) / 2.0,
            (self.y() + other.y()) / 2.0,
        )
    }

    /// 配列形式で取得
    pub fn to_array(&self) -> [f64; 2] {
        [self.x(), self.y()]
    }

    /// 配列から生成
    pub fn from_array(arr: [f64; 2]) -> Self {
        Self::new(arr[0], arr[1])
    }

    /// 点を2Dベクトルに変換（原点からのベクトル）
    pub fn to_vector(&self) -> geo_core::Vector2D { Vector2D::new(self.x(), self.y()) }

    /// 他の点に向かうベクトルを計算
    pub fn vector_to(&self, other: &Point2D) -> geo_core::Vector2D { Vector2D::new(other.x() - self.x(), other.y() - self.y()) }

    /// スカラー値で移動
    pub fn translate(&self, dx: f64, dy: f64) -> Point2D {
        Self::new(self.x() + dx, self.y() + dy)
    }

    /// 原点中心での回転
    pub fn rotate(&self, angle_rad: f64) -> Point2D {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self::new(
            self.x() * cos_a - self.y() * sin_a,
            self.x() * sin_a + self.y() * cos_a,
        )
    }

    /// 指定点中心での回転
    pub fn rotate_around(&self, center: &Point2D, angle_rad: f64) -> Point2D {
        let translated = Self::new(self.x() - center.x(), self.y() - center.y());
        let rotated = translated.rotate(angle_rad);
        Self::new(rotated.x() + center.x(), rotated.y() + center.y())
    }
}

impl PartialEq for Point2D {
    fn eq(&self, other: &Self) -> bool {
        (self.x() - other.x()).abs() < 1e-10 && (self.y() - other.y()).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point2d_creation() {
        let p = Point2D::new(3.0, 4.0);
        assert_eq!(p.x(), 3.0);
        assert_eq!(p.y(), 4.0);
    }

    #[test]
    fn test_point2d_distance() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(3.0, 4.0);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_point2d_midpoint() {
        let p1 = Point2D::new(0.0, 0.0);
        let p2 = Point2D::new(4.0, 6.0);
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x(), 2.0);
        assert_eq!(mid.y(), 3.0);
    }

    #[test]
    fn test_point2d_rotate() {
        let p = Point2D::new(1.0, 0.0);
        let rotated = p.rotate(std::f64::consts::PI / 2.0); // 90度回転
        assert!((rotated.x() - 0.0).abs() < 1e-10);
        assert!((rotated.y() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_point2d_translate() {
        let p = Point2D::new(1.0, 2.0);
        let translated = p.translate(3.0, 4.0);
        assert_eq!(translated.x(), 4.0);
        assert_eq!(translated.y(), 6.0);
    }
}
