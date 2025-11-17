//! Ellipse Core Traits - Ellipse形状の3つのCore機能統合
//!
//! Foundation パターンに基づく楕円のCore機能統合
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

// ============================================================================
// 1. Constructor Traits - Ellipse生成機能
// ============================================================================

/// Ellipse2D生成のためのConstructorトレイト
pub trait Ellipse2DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、長軸半径、短軸半径、回転角）
    fn new(center: (T, T), semi_major_axis: T, semi_minor_axis: T, rotation: T) -> Option<Self>
    where
        Self: Sized;

    /// 中心点と軸端点から作成
    fn from_center_and_axes(
        center: (T, T),
        major_axis_endpoint: (T, T),
        minor_axis_endpoint: (T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    /// 5点から楕円を構築（一般的な楕円フィット）
    fn from_five_points(p1: (T, T), p2: (T, T), p3: (T, T), p4: (T, T), p5: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 円から楕円を作成（スケール変換）
    fn from_circle(center: (T, T), radius: T, x_scale: T, y_scale: T, rotation: T) -> Option<Self>
    where
        Self: Sized;

    /// 単位楕円作成（原点中心、a=1, b=1, 回転なし）
    fn unit_ellipse() -> Self
    where
        Self: Sized;
}

/// Ellipse3D生成のためのConstructorトレイト
pub trait Ellipse3DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、平面法線、長軸半径、短軸半径、長軸方向）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        major_axis_direction: (T, T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    /// 完全な座標系で作成
    fn new_with_coordinate_system(
        center: (T, T, T),
        normal: (T, T, T),
        major_axis_direction: (T, T, T),
        minor_axis_direction: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の楕円作成
    fn new_xy_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XZ平面上の楕円作成
    fn new_xz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// YZ平面上の楕円作成
    fn new_yz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面単位楕円
    fn unit_ellipse_xy() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - Ellipse基本情報アクセス
// ============================================================================

/// Ellipse2D基本プロパティアクセス
pub trait Ellipse2DProperties<T: Scalar> {
    /// 楕円の中心座標を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;

    /// 楕円の焦点間距離を取得
    fn focal_distance(&self) -> T;

    /// 第1焦点の座標を取得
    fn focus1(&self) -> (T, T);

    /// 第2焦点の座標を取得
    fn focus2(&self) -> (T, T);

    /// Analysis層互換の座標変換
    fn to_analysis_vector(&self) -> Vector2<T>;

    /// 中心点をタプルとして取得
    fn center_tuple(&self) -> (T, T);
}

/// Ellipse3D基本プロパティアクセス
pub trait Ellipse3DProperties<T: Scalar>: Ellipse2DProperties<T> {
    /// 楕円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> (T, T, T);

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> (T, T, T);

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> (T, T, T);

    /// 3D中心座標を取得
    fn center_3d(&self) -> (T, T, T);

    /// Analysis層互換の3D座標変換
    fn to_analysis_vector_3d(&self) -> Vector3<T>;

    /// 3D中心点をタプルとして取得
    fn center_3d_tuple(&self) -> (T, T, T);
}

// ============================================================================
// 3. Measure Traits - Ellipse計量・関係演算
// ============================================================================

/// Ellipse2D計量機能
pub trait Ellipse2DMeasure<T: Scalar> {
    /// 楕円の面積を計算
    fn area(&self) -> T;

    /// 楕円の周長を計算（近似）
    fn perimeter(&self) -> T;

    /// 楕円の正確な周長を計算（数値積分）
    fn perimeter_exact(&self, tolerance: T) -> T;

    /// 楕円の離心率を計算
    fn eccentricity(&self) -> T;

    /// 点が楕円内部にあるかを判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 点が楕円境界上にあるかを判定（許容誤差考慮）
    fn on_ellipse(&self, point: (T, T), tolerance: T) -> bool;

    /// 点から楕円への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 楕円が円に近いかを判定
    fn is_nearly_circular(&self, tolerance: T) -> bool;

    /// 楕円が完全な円かを判定
    fn is_circle(&self) -> bool;

    /// 他の楕円との交点を計算
    fn intersection_with_ellipse(&self, other: &Self) -> Vec<(T, T)>;

    /// 直線との交点を計算
    fn intersection_with_line(&self, line_point: (T, T), line_direction: (T, T)) -> Vec<(T, T)>;
}

/// Ellipse3D計量機能
pub trait Ellipse3DMeasure<T: Scalar>: Ellipse2DMeasure<T> {
    /// 3D空間での点が楕円内部にあるかを判定
    fn contains_point_3d(&self, point: (T, T, T)) -> bool;

    /// 3D空間での点から楕円への最短距離を計算
    fn distance_to_point_3d(&self, point: (T, T, T)) -> T;

    /// 3D空間での直線との交点を計算
    fn intersection_with_line_3d(
        &self,
        line_point: (T, T, T),
        line_direction: (T, T, T),
    ) -> Vec<(T, T, T)>;

    /// 平面との交点を計算
    fn intersection_with_plane(
        &self,
        plane_point: (T, T, T),
        plane_normal: (T, T, T),
    ) -> Vec<(T, T, T)>;
}
