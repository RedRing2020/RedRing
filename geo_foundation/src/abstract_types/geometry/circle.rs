//! Circle - 円の最小責務抽象化
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本Circleトレイト = 円の基本属性のみ
//! ```text
//! Circle Trait = 基本属性のみ
//! ├── 中心座標 (center)
//! ├── 半径 (radius)
//! └── 基本生成 (new)
//!
//! 除外される責務:
//! ├── 幾何計算 (area, circumference) → CircleMetrics
//! ├── 判定操作 (contains_point, intersects) → CircleContainment
//! ├── 変換操作 (translate, scale) → CircleTransform
//! └── 高度な生成 (from_three_points) → CircleBuilder
//! ```
//!
//! ## 拡張トレイト群による機能分離
//! ```text
//! Circle2D/Circle3D: 次元特化
//! CircleMetrics: 面積・周長計算
//! CircleContainment: 包含・交差判定
//! CircleTransform: 変換操作
//! ```

use crate::Scalar;

/// 2D円の最小責務トレイト
///
/// 円の基本属性（中心・半径）のアクセスのみを提供。
/// 計算や変換などの機能は拡張トレイトで分離。
pub trait Circle2D<T: Scalar> {
    /// 点の型
    type Point;

    /// 円の中心座標を取得
    fn center(&self) -> Self::Point;

    /// 円の半径を取得
    fn radius(&self) -> T;
}

/// 3D円の最小責務トレイト
///
/// 2D円に法線ベクトルを追加。3D空間での円の基本属性のみ。
pub trait Circle3D<T: Scalar>: Circle2D<T> {
    /// ベクトル型の定義
    type Vector;

    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;
}

/// 円の計量拡張トレイト
///
/// 面積・周長などの計算機能を提供。基本トレイトとは分離。
pub trait CircleMetrics<T: Scalar>: Circle2D<T> {
    /// 円の面積を計算
    fn area(&self) -> T;

    /// 円の周長を計算
    fn circumference(&self) -> T;

    /// 円の直径を計算
    fn diameter(&self) -> T {
        self.radius() + self.radius() // 2 * radius
    }
}

/// 円の包含・交差判定拡張
///
/// 点や他の図形との位置関係判定を提供。
pub trait CircleContainment<T: Scalar>: Circle2D<T> {
    /// 点が円内部にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が円周上にあるかを判定（許容誤差考慮）
    fn on_circle(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 他の円と交差するかを判定
    fn intersects_circle(&self, other: &Self) -> bool;
}

/// 円の変換操作拡張
///
/// 移動・拡大縮小などの変換操作を提供。
pub trait CircleTransform<T: Scalar>: Circle2D<T> {
    /// 円を移動
    fn translate(&self, offset: &Self::Point) -> Self;

    /// 円を拡大縮小
    fn scale(&self, factor: T) -> Self;
}

// Arc トレイトは arc.rs で定義されています
// 重複を避けるため、ここからは削除しました
