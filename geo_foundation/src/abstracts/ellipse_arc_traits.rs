//! EllipseArc Traits - 楕円弧の最小責務抽象化
//!
//! abstracts 層での EllipseArc 最小責務トレイト定義
//! 実装詳細を含まない純粋なインターフェース定義
//!
//! # 設計方針: 最小責務原則
//!
//! ## 基本EllipseArcトレイト = 楕円弧の基本属性のみ
//! ```text
//! EllipseArc Trait = 基本属性のみ
//! ├── 基底楕円 (ellipse)
//! ├── 角度範囲 (start_angle, end_angle)
//! ├── 基本生成 (new)
//! └── 基本形状判定 (is_full_ellipse)
//!
//! 除外される責務:
//! ├── 計量演算 (arc_length, area) → EllipseArcMetrics
//! ├── 点判定 (contains_point, on_arc) → EllipseArcContainment
//! ├── 変換操作 (translate, rotate) → EllipseArcTransform
//! └── 高度な生成 (from_points) → EllipseArcBuilder
//! ```

use crate::Scalar;
use std::fmt::Debug;

/// 2D楕円弧の最小責務トレイト
///
/// 楕円弧の基本属性（基底楕円・角度範囲）のみを提供。
/// 計算や変換などの機能は拡張トレイトで分離。
pub trait EllipseArc2D<T: Scalar>: Debug + Clone {
    /// 楕円の型
    type Ellipse;

    /// 点の型
    type Point;

    /// 角度の型
    type Angle;

    /// 楕円弧の基底楕円を取得
    fn ellipse(&self) -> &Self::Ellipse;

    /// 開始角度を取得
    fn start_angle(&self) -> Self::Angle;

    /// 終了角度を取得
    fn end_angle(&self) -> Self::Angle;

    /// 完全な楕円（360度）かどうかを判定
    fn is_full_ellipse(&self) -> bool;

    /// 開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 終了点を取得
    fn end_point(&self) -> Self::Point;
}

/// 楕円弧の計量演算拡張
///
/// 弧長・面積・中心角などの計算機能を提供。
pub trait EllipseArcMetrics<T: Scalar>: EllipseArc2D<T> {
    /// 楕円弧長を計算
    fn arc_length(&self) -> T;

    /// 楕円扇形の面積を計算
    fn sector_area(&self) -> T;

    /// 中心角を計算
    fn central_angle(&self) -> Self::Angle;

    /// 楕円弧の中点を取得
    fn midpoint(&self) -> Self::Point;
}

/// 楕円弧の包含・角度判定拡張
///
/// 点の包含判定や角度範囲チェックを提供。
pub trait EllipseArcContainment<T: Scalar>: EllipseArc2D<T> {
    /// 点が楕円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 角度が楕円弧の角度範囲内にあるかを判定
    fn contains_angle(&self, angle: Self::Angle) -> bool;

    /// 指定角度での楕円弧上の点を取得
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;

    /// パラメータ t での楕円弧上の点を取得（0.0 = 開始点、1.0 = 終了点）
    fn point_at_parameter(&self, t: T) -> Self::Point;
}

/// 楕円弧の変換操作拡張
///
/// 移動・回転・拡大縮小などの変換操作を提供。
pub trait EllipseArcTransform<T: Scalar>: EllipseArc2D<T> {
    /// 2D固有の平行移動（x, y座標指定）
    fn translate_xy(&self, dx: T, dy: T) -> Self;

    /// 2D固有の回転（原点基準）
    fn rotate_2d(&self, angle: Self::Angle) -> Self;

    /// 拡大縮小（均等）
    fn scale(&self, factor: T) -> Self;

    /// 非均等拡大縮小（x軸・y軸別々）
    fn scale_xy(&self, scale_x: T, scale_y: T) -> Self;
}

/// 楕円弧の点列生成拡張
///
/// 楕円弧の分割や点列生成機能を提供。
pub trait EllipseArcSampling<T: Scalar>: EllipseArc2D<T> {
    /// 楕円弧を指定数に分割した点列を生成
    fn sample_points(&self, num_points: usize) -> Vec<Self::Point>;

    /// 指定された弧長間隔で点列を生成
    fn sample_by_arc_length(&self, arc_length_step: T) -> Vec<Self::Point>;

    /// 指定された角度間隔で点列を生成
    fn sample_by_angle(&self, angle_step: Self::Angle) -> Vec<Self::Point>;
}

/// 3D楕円弧の最小責務トレイト
///
/// 2D楕円弧に法線ベクトルを追加した3D空間での楕円弧。
pub trait EllipseArc3D<T: Scalar>: EllipseArc2D<T> {
    /// ベクトル型
    type Vector;

    /// 楕円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> Self::Vector;

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> Self::Vector;
}
