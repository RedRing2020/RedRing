//! Arc - 円弧の最小責務抽象化
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本Arcトレイト = 円弧の基本属性のみ
//! ```text
//! Arc Trait = 基本属性のみ
//! ├── 基底円 (circle)
//! ├── 角度範囲 (start_angle, end_angle)
//! ├── 基本生成 (new)
//! └── 基本形状判定 (is_full_circle)
//!
//! 除外される責務:
//! ├── 計量演算 (arc_length, area) → ArcMetrics
//! ├── 点判定 (contains_point, on_arc) → ArcContainment
//! ├── 変換操作 (translate, rotate) → ArcTransform
//! └── 高度な生成 (from_three_points) → ArcBuilder
//! ```

use crate::{Angle, Scalar};
use std::fmt::Debug;

/// 2D円弧の最小責務トレイト
///
/// 円弧の基本属性（基底円・角度範囲）のみを提供。
/// 計算や変換などの機能は拡張トレイトで分離。
pub trait Arc2D<T: Scalar>: Debug + Clone {
    /// 円の型
    type Circle;

    /// 点の型
    type Point;

    /// 角度の型
    type Angle;

    /// 円弧の基底円を取得
    fn circle(&self) -> &Self::Circle;

    /// 開始角度を取得
    fn start_angle(&self) -> Self::Angle;

    /// 終了角度を取得
    fn end_angle(&self) -> Self::Angle;

    /// 完全な円（360度）かどうかを判定
    fn is_full_circle(&self) -> bool;

    /// 開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 終了点を取得
    fn end_point(&self) -> Self::Point;
}

/// 円弧の計量演算拡張
///
/// 弧長・面積・中心角などの計算機能を提供。
pub trait ArcMetrics<T: Scalar>: Arc2D<T> {
    /// 弧長を計算
    fn arc_length(&self) -> T;

    /// 扇形の面積を計算
    fn sector_area(&self) -> T;

    /// 中心角を計算（角度型で返す）
    fn central_angle(&self) -> Self::Angle;
}

/// 円弧の包含・角度判定拡張
///
/// 点の包含判定や角度範囲チェックを提供。
pub trait ArcContainment<T: Scalar>: Arc2D<T> {
    /// 点が円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 角度が円弧の角度範囲内にあるかを判定
    fn contains_angle(&self, angle: Self::Angle) -> bool;

    /// 指定角度での円弧上の点を取得
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;
}

/// 円弧の変換操作拡張
///
/// 移動・回転・拡大縮小などの変換操作を提供。
pub trait ArcTransform<T: Scalar>: Arc2D<T> {
    /// 2D固有の平行移動（x, y座標指定）
    fn translate_xy(&self, dx: T, dy: T) -> Self;

    /// 2D固有の回転（原点基準）
    fn rotate_2d(&self, angle: Angle<T>) -> Self;

    /// 拡大縮小
    fn scale(&self, factor: T) -> Self;
}

/// 円弧の点列生成拡張
///
/// 円弧の分割や点列生成機能を提供。
pub trait ArcSampling<T: Scalar>: Arc2D<T> {
    /// 円弧を指定数に分割した点列を生成
    fn sample_points(&self, num_points: usize) -> Vec<Self::Point>;

    /// 指定された弧長間隔で点列を生成
    fn sample_by_arc_length(&self, arc_length_step: T) -> Vec<Self::Point>;
}

/// 3D円弧の最小責務トレイト
///
/// 2D円弧に法線ベクトルを追加した3D空間での円弧。
pub trait Arc3D<T: Scalar>: Arc2D<T> {
    /// 方向型
    type Direction;

    /// 円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Direction;
}
