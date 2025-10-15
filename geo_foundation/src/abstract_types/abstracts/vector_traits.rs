//! Vector Traits - ベクトルの最小責務抽象化
//!
//! abstracts 層での Vector 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 2Dベクトルの最小責務抽象化
///
/// 実装に依存しない純粋なインターフェース定義
pub trait Vector2D<T: Scalar> {
    /// X成分を取得
    fn x(&self) -> T;

    /// Y成分を取得
    fn y(&self) -> T;
}

/// 3Dベクトルの最小責務抽象化
pub trait Vector3D<T: Scalar>: Vector2D<T> {
    /// Z成分を取得
    fn z(&self) -> T;
}

/// ベクトルの計量最小責務
pub trait VectorMetrics<T: Scalar>: Vector2D<T> {
    /// ベクトルの長さ（ノルム）を計算
    fn length(&self) -> T;

    /// ベクトルの長さの2乗を計算
    fn length_squared(&self) -> T;

    /// 正規化されたベクトルを取得
    fn normalize(&self) -> Self;
}

/// ベクトルの演算最小責務
pub trait VectorOps<T: Scalar>: Vector2D<T> {
    /// ベクトルの加算
    fn add(&self, other: &Self) -> Self;

    /// ベクトルの減算
    fn subtract(&self, other: &Self) -> Self;

    /// スカラー倍
    fn scale(&self, scalar: T) -> Self;

    /// 内積
    fn dot(&self, other: &Self) -> T;
}

/// 2Dベクトルの2D特有演算
pub trait Vector2DOps<T: Scalar>: Vector2D<T> {
    /// 2D外積（Z成分のスカラー値）
    fn cross(&self, other: &Self) -> T;

    /// 90度回転（反時計回り）
    fn rotate_90(&self) -> Self;

    /// 指定角度回転
    fn rotate(&self, angle: T) -> Self;
}

/// 3Dベクトルの3D特有演算
pub trait Vector3DOps<T: Scalar>: Vector3D<T> {
    /// 3D外積
    fn cross(&self, other: &Self) -> Self;
}