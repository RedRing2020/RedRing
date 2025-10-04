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

    /// 指定軸周りで回転
    pub fn rotate_around_axis(&self, axis: &Self, angle: f64) -> Self {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        let one_minus_cos = 1.0 - cos_angle;

        let k_dot_v = axis.x * self.x + axis.y * self.y + axis.z * self.z;

        let rotated_x = self.x * cos_angle + 
                       (axis.y * self.z - axis.z * self.y) * sin_angle + 
                       axis.x * k_dot_v * one_minus_cos;
        let rotated_y = self.y * cos_angle + 
                       (axis.z * self.x - axis.x * self.z) * sin_angle + 
                       axis.y * k_dot_v * one_minus_cos;
        let rotated_z = self.z * cos_angle + 
                       (axis.x * self.y - axis.y * self.x) * sin_angle + 
                       axis.z * k_dot_v * one_minus_cos;

        Self::from_f64(rotated_x, rotated_y, rotated_z).expect("Rotation should preserve unit length")
    }

    /// 許容誤差を考慮した等価比較
    pub fn tolerant_eq(&self, other: &Self, tolerance: &geo_core::ToleranceContext) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        let dz = (self.z - other.z).abs();
        dx < tolerance.linear && dy < tolerance.linear && dz < tolerance.linear
    }
}
