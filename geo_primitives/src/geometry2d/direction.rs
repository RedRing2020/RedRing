/// 2D方向ベクトル
/// 正規化された2次元方向ベクトルを表現

use geo_core::{Vector2D, Scalar, ToleranceContext, TolerantEq};

#[derive(Debug, Clone)]
pub struct Direction2D {
    x: f64,
    y: f64,
}

impl Direction2D {
    /// f64成分から方向ベクトルを作成
    pub fn from_f64(x: f64, y: f64) -> Option<Self> {
        let magnitude = (x * x + y * y).sqrt();
        if magnitude > 1e-12 {
            Some(Self { x: x / magnitude, y: y / magnitude })
        } else {
            None
        }
    }

    /// geo_core::Vector2Dから方向ベクトルを作成
    pub fn from_vector(v: &Vector2D) -> Option<Self> {
        Self::from_f64(v.x().value(), v.y().value())
    }

    /// x方向の単位ベクトル
    pub fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    /// y方向の単位ベクトル
    pub fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    /// x成分を取得
    pub fn x(&self) -> f64 { self.x }

    /// y成分を取得
    pub fn y(&self) -> f64 { self.y }

    /// Vector2Dに変換
    pub fn to_vector(&self) -> Vector2D {
        Vector2D::from_f64(self.x, self.y)
    }

    /// 方向ベクトルを反転
    pub fn negate(&self) -> Self {
        Self { x: -self.x, y: -self.y }
    }

    /// 回転
    pub fn rotate(&self, angle_rad: f64) -> Self {
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
        }
    }
}

impl TolerantEq for Direction2D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        (self.x - other.x).abs() < context.linear &&
        (self.y - other.y).abs() < context.linear
    }
}
