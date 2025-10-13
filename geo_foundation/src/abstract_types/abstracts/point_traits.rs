//! Point2D 抽象トレイト定義
//!
//! Foundation統一システムに基づく最小責務のPoint2D抽象化

use crate::Scalar;

/// Point2D の最小責務抽象トレイト
/// 基本属性のみを提供し、Foundation拡張は別トレイトで実装
pub trait Point2D<T: Scalar> {
    /// X座標を取得
    fn x(&self) -> T;

    /// Y座標を取得  
    fn y(&self) -> T;
}
