/// Direction trait - STEP形式に倣った方向ベクトルの抽象化
///
/// STEPにおけるDIRECTIONエンティティの抽象化。
/// 正規化されたベクトルとして表現され、CAD形状処理に適した操作を提供する。

// use crate::traits::Normalizable;
use std::fmt::Debug;

/// 方向ベクトルの抽象化トレイト
///
/// STEPのDIRECTIONエンティティに対応し、常に正規化された（長さ=1）ベクトルを表現する。
/// CAD操作における方向性を持つ要素（軸、法線、切線など）に使用される。
pub trait Direction: Debug + Clone + PartialEq {
    /// 関連するベクトル型
    type Vector: Clone + Debug;

    /// 関連するスカラー型（通常はf64）
    type Scalar: Clone + Debug + PartialEq;

    /// ベクトルから方向を作成
    ///
    /// # Arguments
    /// * `vector` - 元となるベクトル（ゼロベクトルの場合はNoneを返す）
    ///
    /// # Returns
    /// 正規化された方向ベクトル、またはゼロベクトルの場合はNone
    fn from_vector(vector: Self::Vector) -> Option<Self>;

    /// 成分から方向を作成（2D用）
    fn from_components_2d(x: Self::Scalar, y: Self::Scalar) -> Option<Self>
    where
        Self: Sized;

    /// 成分から方向を作成（3D用）
    fn from_components_3d(x: Self::Scalar, y: Self::Scalar, z: Self::Scalar) -> Option<Self>
    where
        Self: Sized;

    /// 方向ベクトルを元のベクトル型として取得
    fn to_vector(&self) -> Self::Vector;

    /// 内積計算
    fn dot(&self, other: &Self) -> Self::Scalar;

    /// 方向の反転
    fn reverse(&self) -> Self;

    /// 方向が平行かどうかを判定（許容誤差考慮）
    fn is_parallel(&self, other: &Self, tolerance: Self::Scalar) -> bool;

    /// 方向が垂直かどうかを判定（許容誤差考慮）
    fn is_perpendicular(&self, other: &Self, tolerance: Self::Scalar) -> bool;

    /// 方向が同じかどうかを判定（許容誤差考慮）
    fn is_same_direction(&self, other: &Self, tolerance: Self::Scalar) -> bool;

    /// 方向が反対かどうかを判定（許容誤差考慮）
    fn is_opposite_direction(&self, other: &Self, tolerance: Self::Scalar) -> bool;
}

/// 2D方向ベクトルの追加機能
pub trait Direction2D: Direction {
    /// 90度回転した方向を取得
    fn perpendicular(&self) -> Self;

    /// 角度（ラジアン）から方向を作成
    fn from_angle(angle: Self::Scalar) -> Self;

    /// 方向の角度（ラジアン）を取得
    fn to_angle(&self) -> Self::Scalar;

    /// X軸の正方向
    fn x_axis() -> Self;

    /// Y軸の正方向
    fn y_axis() -> Self;
}

/// 3D方向ベクトルの追加機能
pub trait Direction3D: Direction {
    /// 外積計算
    fn cross(&self, other: &Self) -> Self::Vector;

    /// 指定した軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: Self::Scalar) -> Self;

    /// 任意の軸に対する直交ベクトルを生成
    fn any_perpendicular(&self) -> Self;

    /// 正規直交基底を構築（この方向をZ軸とする）
    fn build_orthonormal_basis(&self) -> (Self, Self, Self);

    /// X軸の正方向
    fn x_axis() -> Self;

    /// Y軸の正方向
    fn y_axis() -> Self;

    /// Z軸の正方向
    fn z_axis() -> Self;
}

/// STEP互換性のためのマーカートレイト
///
/// STEPファイルとの相互運用性を示すマーカー。
/// 将来的なSTEPエクスポート/インポート機能で使用予定。
pub trait StepCompatible {
    /// STEP表現の文字列を生成
    fn to_step_string(&self) -> String;

    /// STEP表現から解析（将来実装予定）
    fn from_step_string(step_str: &str) -> Result<Self, String>
    where
        Self: Sized;
}
