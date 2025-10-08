//! Arc - 円弧の抽象化トレイト
//!
//! CAD/CAM システムで使用される円弧の抽象化インターフェース

use crate::abstract_types::{Angle, Scalar};
use std::fmt::Debug;

/// 2D円弧の抽象化トレイト
///
/// 2次元平面上の円弧を表現する共通インターフェース
pub trait Arc2D: Debug + Clone {
    /// 点の型（通常はPoint2D<T>）
    type Point;
    /// スカラー型（f32またはf64）
    type Scalar: Scalar;
    /// 円の型
    type Circle;
    /// ベクトル型
    type Vector;

    /// 円弧の基底円を取得
    fn circle(&self) -> &Self::Circle;

    /// 開始角度を取得
    fn start_angle(&self) -> Angle<Self::Scalar>;

    /// 終了角度を取得
    fn end_angle(&self) -> Angle<Self::Scalar>;

    /// 指定角度での点を取得
    fn point_at_angle(&self, angle: Self::Scalar) -> Self::Point;

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    fn angle_contains(&self, angle: Angle<Self::Scalar>) -> bool;

    /// 円弧の角度範囲を取得
    fn angle_span(&self) -> Angle<Self::Scalar>;

    /// 円弧の弧長を計算
    fn arc_length(&self) -> Self::Scalar;

    /// 円弧の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 円弧の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 円弧の中点を取得
    fn midpoint(&self) -> Self::Point;

    /// 円弧の中心を取得
    fn center(&self) -> Self::Point;

    /// 円弧の半径を取得
    fn radius(&self) -> Self::Scalar;

    /// 点が円弧上にあるかチェック
    fn contains_point(&self, point: Self::Point) -> bool;

    /// 円弧を反転（開始と終了を入れ替え）
    fn reverse(&self) -> Self;

    /// 円弧をスケール
    fn scale(&self, factor: Self::Scalar) -> Self;

    /// 円弧を平行移動
    fn translate(&self, dx: Self::Scalar, dy: Self::Scalar) -> Self;

    /// 円弧を平行移動（Vector版）
    fn translate_by_vector(&self, vector: &Self::Vector) -> Self;

    /// 円弧を回転
    fn rotate(&self, angle: Angle<Self::Scalar>) -> Self;
}

/// 3D円弧の抽象化トレイト
///
/// 3次元空間上の円弧を表現する共通インターフェース
pub trait Arc3D: Debug + Clone {
    /// 点の型（通常はPoint3D<T>）
    type Point;
    /// スカラー型（f32またはf64）
    type Scalar: Scalar;
    /// 円の型
    type Circle;
    /// ベクトル型
    type Vector;

    /// 円弧の基底円を取得
    fn circle(&self) -> &Self::Circle;

    /// 開始角度を取得
    fn start_angle(&self) -> Angle<Self::Scalar>;

    /// 終了角度を取得
    fn end_angle(&self) -> Angle<Self::Scalar>;

    /// 指定角度での点を取得
    fn point_at_angle(&self, angle: Self::Scalar) -> Self::Point;

    /// 指定された角度が円弧の角度範囲内にあるかを判定
    fn angle_contains(&self, angle: Angle<Self::Scalar>) -> bool;

    /// 円弧の角度範囲を取得
    fn angle_span(&self) -> Angle<Self::Scalar>;

    /// 円弧の弧長を計算
    fn arc_length(&self) -> Self::Scalar;

    /// 円弧の開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 円弧の終了点を取得
    fn end_point(&self) -> Self::Point;

    /// 円弧の中点を取得
    fn midpoint(&self) -> Self::Point;

    /// 円弧の中心を取得
    fn center(&self) -> Self::Point;

    /// 円弧の半径を取得
    fn radius(&self) -> Self::Scalar;

    /// 点が円弧上にあるかチェック
    fn contains_point(&self, point: Self::Point) -> bool;

    /// 円弧を反転（開始と終了を入れ替え）
    fn reverse(&self) -> Self;

    /// 円弧をスケール
    fn scale(&self, factor: Self::Scalar) -> Self;

    /// 円弧を平行移動
    fn translate(&self, dx: Self::Scalar, dy: Self::Scalar, dz: Self::Scalar) -> Self;

    /// 円弧を平行移動（Vector版）
    fn translate_by_vector(&self, vector: &Self::Vector) -> Self;

    /// 円弧を回転
    fn rotate(&self, angle: Angle<Self::Scalar>) -> Self;
}
