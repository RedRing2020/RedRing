//! LineSegment Core Traits - LineSegment形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;

// ============================================================================
// Type Aliases for Complex Types
// ============================================================================

/// 2つの2D点を表す型エイリアス（LineSegment2D用）
pub type TwoPoints2D<T> = ((T, T), (T, T));

/// 2つの3D点を表す型エイリアス（LineSegment3D用）
pub type TwoPoints3D<T> = ((T, T, T), (T, T, T));

/// 2D方向ベクトルを表す型エイリアス
pub type Direction2D<T> = (T, T);

/// 3D方向ベクトルを表す型エイリアス
pub type Direction3D<T> = (T, T, T);

// ============================================================================
// 1. Constructor Traits - LineSegment生成機能
// ============================================================================

/// LineSegment2D生成のためのConstructorトレイト
pub trait LineSegment2DConstructor<T: Scalar> {
    /// 2つの点から線分を作成
    fn new(start: (T, T), end: (T, T)) -> Self
    where
        Self: Sized;

    /// 起点、方向、長さから線分を作成
    fn from_point_direction_length(start: (T, T), direction: Direction2D<T>, length: T) -> Self
    where
        Self: Sized;

    /// 中点と長さから水平線分を作成
    fn horizontal_from_center(center: (T, T), length: T) -> Self
    where
        Self: Sized;

    /// 中点と長さから垂直線分を作成
    fn vertical_from_center(center: (T, T), length: T) -> Self
    where
        Self: Sized;

    /// 原点中心の単位線分（X軸方向、長さ1）
    fn unit_x() -> Self
    where
        Self: Sized;

    /// 原点中心の単位線分（Y軸方向、長さ1）
    fn unit_y() -> Self
    where
        Self: Sized;

    /// 退化した線分（同一点、長さ0）
    fn degenerate(point: (T, T)) -> Self
    where
        Self: Sized;

    /// 2つの点を結ぶ線分（点の順序を考慮）
    fn from_ordered_points(points: TwoPoints2D<T>) -> Self
    where
        Self: Sized;
}

/// LineSegment3D生成のためのConstructorトレイト
pub trait LineSegment3DConstructor<T: Scalar> {
    /// 2つの3D点から線分を作成
    fn new(start: (T, T, T), end: (T, T, T)) -> Self
    where
        Self: Sized;

    /// 起点、方向、長さから3D線分を作成
    fn from_point_direction_length(start: (T, T, T), direction: Direction3D<T>, length: T) -> Self
    where
        Self: Sized;

    /// X軸方向の単位線分
    fn unit_x() -> Self
    where
        Self: Sized;

    /// Y軸方向の単位線分
    fn unit_y() -> Self
    where
        Self: Sized;

    /// Z軸方向の単位線分
    fn unit_z() -> Self
    where
        Self: Sized;

    /// 退化した3D線分（同一点、長さ0）
    fn degenerate(point: (T, T, T)) -> Self
    where
        Self: Sized;

    /// 2つの点を結ぶ3D線分（点の順序を考慮）
    fn from_ordered_points(points: TwoPoints3D<T>) -> Self
    where
        Self: Sized;

    /// XY平面上の2D線分から3D線分を作成（Z=0）
    fn from_2d_in_xy_plane(start_2d: (T, T), end_2d: (T, T)) -> Self
    where
        Self: Sized;

    /// XZ平面上の2D線分から3D線分を作成（Y=0）
    fn from_2d_in_xz_plane(start_2d: (T, T), end_2d: (T, T)) -> Self
    where
        Self: Sized;

    /// YZ平面上の2D線分から3D線分を作成（X=0）
    fn from_2d_in_yz_plane(start_2d: (T, T), end_2d: (T, T)) -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - LineSegment基本情報取得
// ============================================================================

/// LineSegment2D基本プロパティ取得トレイト
pub trait LineSegment2DProperties<T: Scalar> {
    /// 開始点を取得
    fn start(&self) -> (T, T);

    /// 終了点を取得
    fn end(&self) -> (T, T);

    /// 中点を取得
    fn midpoint(&self) -> (T, T);

    /// 線分の方向ベクトルを取得
    fn direction_vector(&self) -> Direction2D<T>;

    /// 正規化された方向ベクトルを取得
    fn unit_direction(&self) -> Option<Direction2D<T>>;

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 線分の長さの二乗を取得（計算効率向上）
    fn length_squared(&self) -> T;

    /// 線分が退化しているか判定（長さが0またはほぼ0）
    fn is_degenerate(&self) -> bool;

    /// 線分が有効かどうか判定
    fn is_valid(&self) -> bool;

    /// 線分が水平かどうか判定
    fn is_horizontal(&self) -> bool;

    /// 線分が垂直かどうか判定
    fn is_vertical(&self) -> bool;

    /// 線分の傾きを取得（垂直線の場合はNone）
    fn slope(&self) -> Option<T>;

    /// 線分の境界ボックスを取得
    fn bounding_box(&self) -> TwoPoints2D<T>;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

/// LineSegment3D基本プロパティ取得トレイト
pub trait LineSegment3DProperties<T: Scalar> {
    /// 開始点を取得
    fn start(&self) -> (T, T, T);

    /// 終了点を取得
    fn end(&self) -> (T, T, T);

    /// 中点を取得
    fn midpoint(&self) -> (T, T, T);

    /// 線分の方向ベクトルを取得
    fn direction_vector(&self) -> Direction3D<T>;

    /// 正規化された方向ベクトルを取得
    fn unit_direction(&self) -> Option<Direction3D<T>>;

    /// 線分の長さを取得
    fn length(&self) -> T;

    /// 線分の長さの二乗を取得（計算効率向上）
    fn length_squared(&self) -> T;

    /// 線分が退化しているか判定（長さが0またはほぼ0）
    fn is_degenerate(&self) -> bool;

    /// 線分が有効かどうか判定
    fn is_valid(&self) -> bool;

    /// X軸に平行かどうか判定
    fn is_parallel_to_x_axis(&self) -> bool;

    /// Y軸に平行かどうか判定
    fn is_parallel_to_y_axis(&self) -> bool;

    /// Z軸に平行かどうか判定
    fn is_parallel_to_z_axis(&self) -> bool;

    /// XY平面に平行かどうか判定
    fn is_parallel_to_xy_plane(&self) -> bool;

    /// XZ平面に平行かどうか判定
    fn is_parallel_to_xz_plane(&self) -> bool;

    /// YZ平面に平行かどうか判定
    fn is_parallel_to_yz_plane(&self) -> bool;

    /// XY平面への射影を取得
    fn xy_projection(&self) -> TwoPoints2D<T>;

    /// XZ平面への射影を取得
    fn xz_projection(&self) -> TwoPoints2D<T>;

    /// YZ平面への射影を取得
    fn yz_projection(&self) -> TwoPoints2D<T>;

    /// 線分の境界ボックスを取得
    fn bounding_box(&self) -> TwoPoints3D<T>;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}

// ============================================================================
// 3. Measure Traits - LineSegment計量・関係演算機能
// ============================================================================

/// LineSegment2D計量・関係演算機能トレイト
pub trait LineSegment2DMeasure<T: Scalar> {
    /// 線分の長さ（測度）
    fn measure(&self) -> T;

    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 点に最も近い線分上の点を取得
    fn closest_point_to(&self, point: (T, T)) -> (T, T);

    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 点に対応するパラメータを取得（線分上にない場合はNone）
    fn parameter_at_point(&self, point: (T, T)) -> Option<T>;

    /// 他の線分と交差するかを判定
    fn intersects_segment(&self, other: &Self) -> bool;

    /// 他の線分との交点を計算
    fn intersection_with_segment(&self, other: &Self) -> Option<(T, T)>;

    /// 他の線分と平行かを判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の線分と垂直かを判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の線分との角度を計算（ラジアン）
    fn angle_with_segment(&self, other: &Self) -> T;

    /// 他の線分との最短距離を計算
    fn distance_to_segment(&self, other: &Self) -> T;

    /// 線分を反転（開始点と終了点を交換）
    fn reverse(&self) -> Self
    where
        Self: Sized;

    /// 線分を指定倍率で延長
    fn extend(&self, factor: T) -> Self
    where
        Self: Sized;

    /// 線分を両端から指定長さ分延長
    fn extend_by_length(&self, start_extension: T, end_extension: T) -> Self
    where
        Self: Sized;

    /// 線分を指定倍率で縮小
    fn shrink(&self, factor: T) -> Option<Self>
    where
        Self: Sized;

    /// 線分を中心から指定長さに調整
    fn resize_from_center(&self, new_length: T) -> Option<Self>
    where
        Self: Sized;

    /// 線分を分割（指定パラメータ位置で）
    fn split_at_parameter(&self, t: T) -> Option<(Self, Self)>
    where
        Self: Sized;

    /// 線分を等分割
    fn subdivide(&self, segments: u32) -> Vec<Self>
    where
        Self: Sized;
}

/// LineSegment3D計量・関係演算機能トレイト
pub trait LineSegment3DMeasure<T: Scalar> {
    /// 線分の長さ（測度）
    fn measure(&self) -> T;

    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// 点に最も近い線分上の点を取得
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);

    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 点に対応するパラメータを取得（線分上にない場合はNone）
    fn parameter_at_point(&self, point: (T, T, T)) -> Option<T>;

    /// 他の線分と交差するかを判定（スキュー線も考慮）
    fn intersects_segment(&self, other: &Self) -> bool;

    /// 他の線分との交点を計算（同一平面上の場合）
    fn intersection_with_segment(&self, other: &Self) -> Option<(T, T, T)>;

    /// 他の線分と平行かを判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の線分と垂直かを判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の線分と同一平面上にあるかを判定
    fn is_coplanar_with(&self, other: &Self) -> bool;

    /// 他の線分とスキュー（ねじれ）関係にあるかを判定
    fn is_skew_to(&self, other: &Self) -> bool;

    /// 他の線分との角度を計算（ラジアン）
    fn angle_with_segment(&self, other: &Self) -> T;

    /// 他の線分との最短距離を計算
    fn distance_to_segment(&self, other: &Self) -> T;

    /// 平面との交点を計算
    fn intersection_with_plane(
        &self,
        plane_point: (T, T, T),
        plane_normal: Direction3D<T>,
    ) -> Option<(T, T, T)>;

    /// 指定軸への射影の長さを計算
    fn projection_length_on_axis(&self, axis: Direction3D<T>) -> T;

    /// 線分を反転（開始点と終了点を交換）
    fn reverse(&self) -> Self
    where
        Self: Sized;

    /// 線分を指定倍率で延長
    fn extend(&self, factor: T) -> Self
    where
        Self: Sized;

    /// 線分を両端から指定長さ分延長
    fn extend_by_length(&self, start_extension: T, end_extension: T) -> Self
    where
        Self: Sized;

    /// 線分を指定倍率で縮小
    fn shrink(&self, factor: T) -> Option<Self>
    where
        Self: Sized;

    /// 線分を中心から指定長さに調整
    fn resize_from_center(&self, new_length: T) -> Option<Self>
    where
        Self: Sized;

    /// 線分を分割（指定パラメータ位置で）
    fn split_at_parameter(&self, t: T) -> Option<(Self, Self)>
    where
        Self: Sized;

    /// 線分を等分割
    fn subdivide(&self, segments: u32) -> Vec<Self>
    where
        Self: Sized;
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// LineSegment2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait LineSegment2DCore<T: Scalar>:
    LineSegment2DConstructor<T> + LineSegment2DProperties<T> + LineSegment2DMeasure<T>
{
}

/// LineSegment3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait LineSegment3DCore<T: Scalar>:
    LineSegment3DConstructor<T> + LineSegment3DProperties<T> + LineSegment3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, L> LineSegment2DCore<T> for L where
    L: LineSegment2DConstructor<T> + LineSegment2DProperties<T> + LineSegment2DMeasure<T>
{
}

impl<T: Scalar, L> LineSegment3DCore<T> for L where
    L: LineSegment3DConstructor<T> + LineSegment3DProperties<T> + LineSegment3DMeasure<T>
{
}
