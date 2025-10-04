/// 3D方向ベクトル
/// 正規化された3次元方向ベクトルを表現

use geo_core::{Vector3D, Scalar};

#[derive(Debug, Clone)]
pub struct Direction3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Direction3D {
    /// f64成分から方向ベクトルを作成
    pub fn from_f64(x: f64, y: f64, z: f64) -> Option<Self> {
        let magnitude = (x * x + y * y + z * z).sqrt();
        if magnitude > 1e-12 {
            Some(Self {
                x: x / magnitude,
                y: y / magnitude,
                z: z / magnitude,
            })
        } else {
            None
        }
    }

    /// Vector3Dから方向ベクトルを作成
    pub fn from_vector(v: &Vector3D) -> Option<Self> {
        Self::from_f64(v.x().value(), v.y().value(), v.z().value())
    }

    /// X軸単位ベクトル
    pub fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0, z: 0.0 }
    }

    /// Y軸単位ベクトル
    pub fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0, z: 0.0 }
    }

    /// Z軸単位ベクトル
    pub fn unit_z() -> Self {
        Self { x: 0.0, y: 0.0, z: 1.0 }
    }

    /// X成分を取得
    pub fn x(&self) -> f64 { self.x }

    /// Y成分を取得
    pub fn y(&self) -> f64 { self.y }

    /// Z成分を取得
    pub fn z(&self) -> f64 { self.z }

    /// Vector3Dに変換
    pub fn to_vector(&self) -> Vector3D {
        Vector3D::new(Scalar::new(self.x), Scalar::new(self.y), Scalar::new(self.z))
    }

    /// 反転
    pub fn negate(&self) -> Self {
        Self { x: -self.x, y: -self.y, z: -self.z }
    }

    /// 内積
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// 外積
    pub fn cross(&self, other: &Self) -> Vector3D {
        Vector3D::new(
            Scalar::new(self.y * other.z - self.z * other.y),
            Scalar::new(self.z * other.x - self.x * other.z),
            Scalar::new(self.x * other.y - self.y * other.x),
        )
    }
}
