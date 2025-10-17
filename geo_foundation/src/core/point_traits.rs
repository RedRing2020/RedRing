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
