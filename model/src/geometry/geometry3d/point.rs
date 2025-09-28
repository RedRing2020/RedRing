use super::vector::Vector;
use crate::geometry::geometry3d;
use crate::geometry_trait::point_ops::PointOps;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
}

impl Point {
    /// コンストラクタ：明示的に座標を指定
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// ゲッター：各成分を取得
    pub fn x(&self) -> f64 {
        self.x
    }

    pub fn y(&self) -> f64 {
        self.y
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    /// 他の点との距離を計算
    pub fn distance_to(&self, other: &Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 自身をベクトルとして取得（原点からの位置ベクトル）
    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x, self.y, self.z)
    }

    /// ベクトルで平行移動
    pub fn translate(&self, v: &Vector) -> Self {
        Self {
            x: self.x + v.x(),
            y: self.y + v.y(),
            z: self.z + v.z(),
        }
    }
}

impl PointOps for geometry3d::point::Point {
    fn origin() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    fn add_scaled(&self, other: &Self, scale: f64) -> Self {
        Self::new(self.x + other.x * scale, self.y + other.y * scale, self.z + other.z * scale)
    }

    fn div(&self, scalar: f64) -> Self {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}
