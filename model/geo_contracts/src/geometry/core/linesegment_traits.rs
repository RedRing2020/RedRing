//! LineSegment Core Traits - LineSegment形状のCore機能統合
//!
//! Foundation Pattern Phase 1 + Phase 2 実装
//! Transform機能は共通のAnalysisTransformトレイトを使用

use analysis::abstract_types::Scalar;

/// LineSegment2D生成のためのConstructorトレイト
pub trait LineSegment2DConstructor<T: Scalar> {
    /// 2つの点から線分を作成
    fn new(start: (T, T), end: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 起点、方向、長さから線分を作成
    fn from_point_direction_length(start: (T, T), direction: (T, T), length: T) -> Option<Self>
    where
        Self: Sized;

    /// X軸方向の単位線分（原点から(1,0)まで）
    fn unit_x() -> Self
    where
        Self: Sized;

    /// 中点と長さから水平線分を作成
    fn from_midpoint_length_horizontal(midpoint: (T, T), length: T) -> Option<Self>
    where
        Self: Sized;

    /// 中点と長さから垂直線分を作成
    fn from_midpoint_length_vertical(midpoint: (T, T), length: T) -> Option<Self>
    where
        Self: Sized;

    /// Y軸方向の単位線分（原点から(0,1)まで）
    fn unit_y() -> Self
    where
        Self: Sized;
}

/// LineSegment3D生成のためのConstructorトレイト
pub trait LineSegment3DConstructor<T: Scalar> {
    /// 2つの3D点から線分を作成
    fn new(start: (T, T, T), end: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 起点、方向、長さから3D線分を作成
    fn from_point_direction_length(
        start: (T, T, T),
        direction: (T, T, T),
        length: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// X軸方向の単位線分（原点から(1,0,0)まで）
    fn unit_x() -> Self
    where
        Self: Sized;

    /// Y軸方向の単位線分（原点から(0,1,0)まで）
    fn unit_y() -> Self
    where
        Self: Sized;

    /// Z軸方向の単位線分（原点から(0,0,1)まで）
    fn unit_z() -> Self
    where
        Self: Sized;

    /// XY平面上の水平線分（Z=0）
    fn horizontal_xy(midpoint: (T, T, T), length: T) -> Option<Self>
    where
        Self: Sized;
}

/// LineSegment2D基本プロパティ取得トレイト
pub trait LineSegment2DProperties<T: Scalar> {
    /// 拘束点としての開始点を取得
    fn start(&self) -> (T, T);

    /// 拘束点としての終了点を取得
    fn end(&self) -> (T, T);

    /// 中点を取得
    fn midpoint(&self) -> (T, T);

    /// 拘束点間距離としての長さを取得
    fn length(&self) -> T;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;

    /// 単位長さ（長さ1）の線分かどうか
    fn is_unit_length(&self) -> bool;

    /// 水平線分（Y座標が一定）かどうか
    fn is_horizontal(&self) -> bool;

    /// 垂直線分（X座標が一定）かどうか
    fn is_vertical(&self) -> bool;
}

/// LineSegment3D基本プロパティ取得トレイト
pub trait LineSegment3DProperties<T: Scalar> {
    /// 拘束点としての開始点を取得
    fn start(&self) -> (T, T, T);

    /// 拘束点としての終了点を取得
    fn end(&self) -> (T, T, T);

    /// 中点を取得
    fn midpoint(&self) -> (T, T, T);

    /// 拘束点間距離としての長さを取得
    fn length(&self) -> T;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;

    /// 単位長さ（長さ1）の線分かどうか
    fn is_unit_length(&self) -> bool;

    /// XY平面上（Z座標が一定）の線分かどうか
    fn is_on_xy_plane(&self) -> bool;

    /// YZ平面上（X座標が一定）の線分かどうか
    fn is_on_yz_plane(&self) -> bool;
}

pub trait LineSegment2DDerived<T: Scalar> {
    /// 互換目的の測度。意味は拘束点間距離とする
    fn measure(&self) -> T;

    /// 方向ベクトルを取得
    fn direction_vector(&self) -> (T, T);

    /// 線分のベクトル表現（始点から終点）
    fn as_vector(&self) -> (T, T);
}

pub trait LineSegment2DDistance<T: Scalar> {
    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait LineSegment2DContainment<T: Scalar> {
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T)) -> bool;
}

pub trait LineSegment2DEvaluation<T: Scalar> {
    /// 正規化パラメータt（0<=t<=1）で support line 上の評価点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);
}

pub trait LineSegment2DProjection<T: Scalar> {
    /// 点から線分への最近点を計算
    fn closest_point_to(&self, point: (T, T)) -> (T, T);
}

pub trait LineSegment3DDerived<T: Scalar> {
    /// 互換目的の測度。意味は拘束点間距離とする
    fn measure(&self) -> T;

    /// 方向ベクトルを取得
    fn direction_vector(&self) -> (T, T, T);

    /// 線分のベクトル表現（始点から終点）
    fn as_vector(&self) -> (T, T, T);
}

pub trait LineSegment3DDistance<T: Scalar> {
    /// 点から線分への最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;
}

pub trait LineSegment3DContainment<T: Scalar> {
    /// 点が線分上にあるかを判定
    fn contains_point(&self, point: (T, T, T)) -> bool;
}

pub trait LineSegment3DEvaluation<T: Scalar> {
    /// 正規化パラメータt（0<=t<=1）で support line 上の評価点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}

pub trait LineSegment3DProjection<T: Scalar> {
    /// 点から線分への最近点を計算
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);
}

/// LineSegment2Dの3つのCore機能統合トレイト
pub trait LineSegment2DCore<T: Scalar>:
    LineSegment2DConstructor<T> + LineSegment2DProperties<T>
{
}

/// LineSegment3Dの3つのCore機能統合トレイト
pub trait LineSegment3DCore<T: Scalar>:
    LineSegment3DConstructor<T> + LineSegment3DProperties<T>
{
}

impl<T: Scalar, L> LineSegment2DCore<T> for L where
    L: LineSegment2DConstructor<T> + LineSegment2DProperties<T>
{
}

impl<T: Scalar, L> LineSegment3DCore<T> for L where
    L: LineSegment3DConstructor<T> + LineSegment3DProperties<T>
{
}
