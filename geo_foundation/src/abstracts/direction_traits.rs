//! Direction Traits - 方向ベクトルの最小責務抽象化
//!
//! abstracts 層での Direction 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義

use crate::Scalar;

/// 2D方向ベクトルの最小責務抽象化
///
/// 正規化されたベクトル（単位ベクトル）を表現する純粋なインターフェース
/// Vector2Dを継承し、長さが1であることを保証
pub trait Direction2D<T: Scalar> {
    /// 関連するベクトル型
    type Vector;

    /// X成分を取得（-1.0 ≤ x ≤ 1.0）
    fn x(&self) -> T;

    /// Y成分を取得（-1.0 ≤ y ≤ 1.0）
    fn y(&self) -> T;

    /// 内部のベクトルを取得
    fn as_vector(&self) -> Self::Vector;
}

/// 3D方向ベクトルの最小責務抽象化
///
/// Direction2Dを継承し、Z成分を追加
pub trait Direction3D<T: Scalar>: Direction2D<T> {
    /// Z成分を取得（-1.0 ≤ z ≤ 1.0）
    fn z(&self) -> T;
}

/// 方向ベクトルの角度計算最小責務
pub trait DirectionAngular<T: Scalar>: Direction2D<T> {
    /// 関連する角度型
    type Angle;

    /// X軸からの角度を取得
    fn angle(&self) -> Self::Angle;

    /// 他の方向との角度差を計算
    fn angle_to(&self, other: &Self) -> Self::Angle;
}

/// 方向ベクトルの変換最小責務
pub trait DirectionTransform<T: Scalar>: Direction2D<T> {
    /// 関連する角度型
    type Angle;

    /// 指定角度だけ回転した新しい方向を取得
    fn rotate(&self, angle: Self::Angle) -> Self;

    /// 90度回転（反時計回り）
    fn rotate_90(&self) -> Self;

    /// 180度回転（反転）
    fn reverse(&self) -> Self;
}

/// 方向ベクトルの関係計算最小責務
pub trait DirectionRelations<T: Scalar>: Direction2D<T> {
    /// 他の方向と平行かどうかを判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の方向と垂直かどうかを判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の方向との内積を計算
    fn dot(&self, other: &Self) -> T;
}

/// 3D方向ベクトルの3D特有変換
pub trait Direction3DTransform<T: Scalar>: Direction3D<T> {
    /// 関連する角度型
    type Angle;

    /// 指定軸周りの回転
    fn rotate_around_axis(&self, axis: &Self, angle: Self::Angle) -> Self;

    /// 他の方向との外積（結果も正規化される）
    fn cross(&self, other: &Self) -> Self;
}
