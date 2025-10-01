use crate::geometry::geometry2d::{ line::Line, vector::Vector };
use crate::geometry::geometry2d;
use crate::geometry_trait::point_ops::PointOps;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// 新しい点を生成
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// x座標を返す
    pub fn x(&self) -> f64 { self.x }

    /// y座標を返す
    pub fn y(&self) -> f64 { self.y }

    /// 原点
    pub const ORIGIN: Point = Point { x: 0.0, y: 0.0 };

    /// 点をベクトルに変換
    pub fn to_vector(&self) -> geometry2d::vector::Vector {
        Vector::new(self.x, self.y)
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// 点を配列形式で取得（互換性用）
    pub fn to_array(&self) -> [f64; 2] {
        [self.x, self.y]
    }

    /// 配列から点を生成
    pub fn from_array(arr: [f64; 2]) -> Self {
        Self { x: arr[0], y: arr[1] }
    }

    /// 線分上の点までの距離を計算
    pub fn distance_to_point_on_line(&self, line: &Line) -> f64 {
        line.distance_to_point(self)
    }
}

impl PointOps for geometry2d::point::Point {
    /// 原点を返す
    fn origin() -> Self {
        Point::ORIGIN
    }

    /// dx, dy を加算した新しい点を返す
    fn add(&self, other: &Self) -> Self {
        Point::new(self.x + other.x, self.y + other.y)
    }

    /// 他の点との差分ベクトルを返す
    fn sub(&self, other: &Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y)
    }

    /// スカラー値で乗算した新しい点を返す
    fn mul(&self, scalar: f64) -> Self {
        Self::new(self.x * scalar, self.y * scalar)
    }

    /// スカラー値で除算した新しい点を返す
    fn div(&self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar)
    }

    /// 他の点を scale 倍して加算した新しい点を返す
    fn add_scaled(&self, other: &Self, scale: f64) -> Self {
        Self::new(self.x + other.x * scale, self.y + other.y * scale)
    }
}
