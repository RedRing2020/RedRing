//! ベクトル拡張トレイト
//!
//! 基本Vectorトレイトを拡張した高度なベクトル操作

use super::vector::{Vector, Vector2D as Vector2DTrait, Vector3D as Vector3DTrait};

/// 2Dベクトル専用の拡張トレイト
pub trait Vector2DExt: Vector2DTrait {
    /// 角度（ラジアン）から2Dベクトルを作成
    fn from_angle(angle: Self::Scalar) -> Self
    where
        Self: Sized;

    /// ベクトルの角度（ラジアン）を取得
    fn angle(&self) -> Self::Scalar;

    /// 他のベクトルとの角度差を取得
    fn angle_to(&self, other: &Self) -> Self::Scalar;
}

/// 3Dベクトル専用の拡張トレイト
pub trait Vector3DExt: Vector3DTrait {
    /// 指定した軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: Self::Scalar) -> Self;

    /// 任意の軸に対する直交ベクトルを生成
    fn any_perpendicular(&self) -> Self;

    /// 正規直交基底を構築（このベクトルをZ軸とする）
    fn build_orthonormal_basis(&self) -> (Self, Self, Self);
}

/// デフォルトの許容誤差
pub const DEFAULT_VECTOR_TOLERANCE: f64 = 1e-10;
