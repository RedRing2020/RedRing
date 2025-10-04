/// 3D方向ベクトル
/// 正規化された3次元方向ベクトルを表現

use geo_core::{Vector3D, Scalar};
use crate::util::axis_angle::rodrigues_f64;
use geo_core::{ToleranceContext, TolerantEq};

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
    pub fn dot(&self, other: &Self) -> f64 { self.x * other.x + self.y * other.y + self.z * other.z }

    /// 外積
    pub fn cross(&self, other: &Self) -> Vector3D {
        Vector3D::new(
            Scalar::new(self.y * other.z - self.z * other.y),
            Scalar::new(self.z * other.x - self.x * other.z),
            Scalar::new(self.x * other.y - self.y * other.x),
        )
    }

    /// (x,y,z) タプルを取得（回転ユーティリティとの橋渡し用）
    pub fn as_tuple(&self) -> (f64,f64,f64) { (self.x, self.y, self.z) }

    /// 任意軸 (axis) 回りに angle (rad) だけ回転した新しい方向を返す。
    /// axis は非ゼロである必要がある（内部で正規化）。
    pub fn rotate_around_axis(&self, axis: &Direction3D, angle: f64) -> Self {
        // Rodrigues を f64 ドメインで実行
        let (rx, ry, rz) = rodrigues_f64(axis.as_tuple(), self.as_tuple(), angle);
        // 正規化（数値誤差吸収）
        if let Some(dir) = Direction3D::from_f64(rx, ry, rz) {
            dir
        } else {
            // 角度が 0 や 2π の数値丸めでゼロに近づいた等の場合は self を返す
            self.clone()
        }
    }
}

impl TolerantEq for Direction3D {
    fn tolerant_eq(&self, other: &Self, context: &ToleranceContext) -> bool {
        (self.x - other.x).abs() < context.linear &&
        (self.y - other.y).abs() < context.linear &&
        (self.z - other.z).abs() < context.linear
    }
}
