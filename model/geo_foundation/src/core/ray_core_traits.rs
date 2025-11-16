//! Ray Core Traits - Ray形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;
use analysis::linalg::{
    point2::Point2,
    point3::Point3,
    vector::{Vector2, Vector3},
};

// ============================================================================
// 1. Constructor Traits - Ray生成機能
// ============================================================================

/// Ray2D生成のためのConstructorトレイト
pub trait Ray2DConstructor<T: Scalar> {
    /// 起点と方向ベクトルからRayを作成
    /// 方向ベクトルが自動的に正規化される
    fn new(origin: Point2<T>, direction: Vector2<T>) -> Option<Self>
    where
        Self: Sized;

    /// 2点を通るRayを作成
    /// 第1点が起点、第2点が通過点となる
    fn from_points(start: Point2<T>, through: Point2<T>) -> Option<Self>
    where
        Self: Sized;

    /// X軸正方向のRayを作成
    fn along_positive_x(origin: Point2<T>) -> Self
    where
        Self: Sized;

    /// Y軸正方向のRayを作成
    fn along_positive_y(origin: Point2<T>) -> Self
    where
        Self: Sized;

    /// X軸負方向のRayを作成
    fn along_negative_x(origin: Point2<T>) -> Self
    where
        Self: Sized;

    /// Y軸負方向のRayを作成
    fn along_negative_y(origin: Point2<T>) -> Self
    where
        Self: Sized;

    /// 原点からX軸正方向のRayを作成
    fn x_axis() -> Self
    where
        Self: Sized;

    /// 原点からY軸正方向のRayを作成
    fn y_axis() -> Self
    where
        Self: Sized;
}

/// Ray3D生成のためのConstructorトレイト
pub trait Ray3DConstructor<T: Scalar> {
    /// 起点と方向ベクトルからRayを作成
    /// 方向ベクトルが自動的に正規化される
    fn new(origin: Point3<T>, direction: Vector3<T>) -> Option<Self>
    where
        Self: Sized;

    /// 2点を通るRayを作成
    /// 第1点が起点、第2点が通過点となる
    fn from_points(start: Point3<T>, through: Point3<T>) -> Option<Self>
    where
        Self: Sized;

    /// X軸正方向のRayを作成
    fn along_positive_x(origin: Point3<T>) -> Self
    where
        Self: Sized;

    /// Y軸正方向のRayを作成
    fn along_positive_y(origin: Point3<T>) -> Self
    where
        Self: Sized;

    /// Z軸正方向のRayを作成
    fn along_positive_z(origin: Point3<T>) -> Self
    where
        Self: Sized;

    /// X軸負方向のRayを作成
    fn along_negative_x(origin: Point3<T>) -> Self
    where
        Self: Sized;

    /// Y軸負方向のRayを作成
    fn along_negative_y(origin: Point3<T>) -> Self
    where
        Self: Sized;

    /// Z軸負方向のRayを作成
    fn along_negative_z(origin: Point3<T>) -> Self
    where
        Self: Sized;

    /// 原点からX軸正方向のRayを作成
    fn x_axis() -> Self
    where
        Self: Sized;

    /// 原点からY軸正方向のRayを作成
    fn y_axis() -> Self
    where
        Self: Sized;

    /// 原点からZ軸正方向のRayを作成
    fn z_axis() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - Ray属性アクセス機能
// ============================================================================

/// Ray2D属性アクセスのためのPropertiesトレイト
pub trait Ray2DProperties<T: Scalar> {
    /// 起点を取得
    fn origin(&self) -> Point2<T>;

    /// 方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Vector2<T>;

    /// 起点のX座標
    fn origin_x(&self) -> T;

    /// 起点のY座標
    fn origin_y(&self) -> T;

    /// 方向ベクトルのX成分
    fn direction_x(&self) -> T;

    /// 方向ベクトルのY成分
    fn direction_y(&self) -> T;

    /// Rayが有効かどうか（方向ベクトルが非ゼロ）
    fn is_valid(&self) -> bool;

    /// 次元数を取得（常に2）
    fn dimension(&self) -> usize {
        2
    }
}

/// Ray3D属性アクセスのためのPropertiesトレイト
pub trait Ray3DProperties<T: Scalar> {
    /// 起点を取得
    fn origin(&self) -> Point3<T>;

    /// 方向ベクトルを取得（正規化済み）
    fn direction(&self) -> Vector3<T>;

    /// 起点のX座標
    fn origin_x(&self) -> T;

    /// 起点のY座標
    fn origin_y(&self) -> T;

    /// 起点のZ座標
    fn origin_z(&self) -> T;

    /// 方向ベクトルのX成分
    fn direction_x(&self) -> T;

    /// 方向ベクトルのY成分
    fn direction_y(&self) -> T;

    /// 方向ベクトルのZ成分
    fn direction_z(&self) -> T;

    /// Rayが有効かどうか（方向ベクトルが非ゼロ）
    fn is_valid(&self) -> bool;

    /// 次元数を取得（常に3）
    fn dimension(&self) -> usize {
        3
    }
}

// ============================================================================
// 3. Measure Traits - Ray測定・計算機能
// ============================================================================

/// Ray2D測定のためのMeasureトレイト
pub trait Ray2DMeasure<T: Scalar> {
    /// パラメータtでの点を計算
    /// point = origin + t * direction (t >= 0)
    fn point_at_parameter(&self, t: T) -> Point2<T>;

    /// 指定点に最も近いRay上の点を取得
    fn closest_point(&self, point: &Point2<T>) -> Point2<T>;

    /// 指定点からRayまでの最短距離
    fn distance_to_point(&self, point: &Point2<T>) -> T;

    /// 指定点がRay上にあるかどうか
    fn contains_point(&self, point: &Point2<T>) -> bool;

    /// 指定点に対応するパラメータtを計算
    /// Ray上にない場合は最も近い点のパラメータを返す
    fn parameter_for_point(&self, point: &Point2<T>) -> T;

    /// 指定方向を向いているかどうか
    fn points_towards(&self, direction: &Vector2<T>) -> bool;

    /// 他のRayと平行かどうか
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他のRayと同じ方向かどうか
    fn is_same_direction(&self, other: &Self) -> bool;

    /// 他のRayと逆方向かどうか
    fn is_opposite_direction(&self, other: &Self) -> bool;

    /// Rayの逆方向を作成
    fn reverse(&self) -> Self
    where
        Self: Sized;

    /// 指定した分だけ起点を移動
    fn translate(&self, offset: Vector2<T>) -> Self
    where
        Self: Sized;
}

/// Ray3D測定のためのMeasureトレイト
pub trait Ray3DMeasure<T: Scalar> {
    /// パラメータtでの点を計算
    /// point = origin + t * direction (t >= 0)
    fn point_at_parameter(&self, t: T) -> Point3<T>;

    /// 指定点に最も近いRay上の点を取得
    fn closest_point(&self, point: &Point3<T>) -> Point3<T>;

    /// 指定点からRayまでの最短距離
    fn distance_to_point(&self, point: &Point3<T>) -> T;

    /// 指定点がRay上にあるかどうか
    fn contains_point(&self, point: &Point3<T>) -> bool;

    /// 指定点に対応するパラメータtを計算
    /// Ray上にない場合は最も近い点のパラメータを返す
    fn parameter_for_point(&self, point: &Point3<T>) -> T;

    /// 指定方向を向いているかどうか
    fn points_towards(&self, direction: &Vector3<T>) -> bool;

    /// 他のRayと平行かどうか
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他のRayと同じ方向かどうか
    fn is_same_direction(&self, other: &Self) -> bool;

    /// 他のRayと逆方向かどうか
    fn is_opposite_direction(&self, other: &Self) -> bool;

    /// Rayの逆方向を作成
    fn reverse(&self) -> Self
    where
        Self: Sized;

    /// 指定した分だけ起点を移動
    fn translate(&self, offset: Vector3<T>) -> Self
    where
        Self: Sized;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// Ray2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait Ray2DCore<T: Scalar>: Ray2DConstructor<T> + Ray2DProperties<T> + Ray2DMeasure<T> {}

/// Ray3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait Ray3DCore<T: Scalar>: Ray3DConstructor<T> + Ray3DProperties<T> + Ray3DMeasure<T> {}

// ============================================================================
// 自動実装 - 3つのトレイトを実装している型は自動的にCoreトレイトも実装
// ============================================================================

impl<T: Scalar, R> Ray2DCore<T> for R where
    R: Ray2DConstructor<T> + Ray2DProperties<T> + Ray2DMeasure<T>
{
}

impl<T: Scalar, R> Ray3DCore<T> for R where
    R: Ray3DConstructor<T> + Ray3DProperties<T> + Ray3DMeasure<T>
{
}
