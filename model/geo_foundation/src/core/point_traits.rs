//! Point 抽象トレイト定義
//!
//! Foundation統一システムに基づく最小責務のPoint抽象化

use crate::Scalar;

/// Point2D の最小責務抽象トレイト
/// 基本属性のみを提供し、Foundation拡張は別トレイトで実装
pub trait Point2D<T: Scalar> {
    /// X座標を取得
    fn x(&self) -> T;

    /// Y座標を取得
    fn y(&self) -> T;
}

/// Point3D の最小責務抽象トレイト
/// Point2Dを継承し、Z座標を追加
pub trait Point3D<T: Scalar>: Point2D<T> {
    /// Z座標を取得
    fn z(&self) -> T;
}

/// Point2D作成のための抽象トレイト
/// NURBSなどの高レベル実装がコンストラクタを呼び出すためのインターフェース
pub trait Point2DConstructor<T: Scalar> {
    /// 新しいPoint2Dインスタンスを作成
    fn new(x: T, y: T) -> Self;
}

/// Point3D作成のための抽象トレイト
/// NURBSなどの高レベル実装がコンストラクタを呼び出すためのインターフェース
pub trait Point3DConstructor<T: Scalar> {
    /// 新しいPoint3Dインスタンスを作成
    fn new(x: T, y: T, z: T) -> Self;
}
