//! InfiniteLine Core Traits - InfiniteLine形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;

// ============================================================================
// Type Aliases for Complex Types
// ============================================================================

/// 2つの3D点を表す型エイリアス（複雑な型の簡素化）
pub type TwoPoints3D<T> = ((T, T, T), (T, T, T));

/// 2つの2D点を表す型エイリアス（複雑な型の簡素化）
pub type TwoPoints2D<T> = ((T, T), (T, T));

// ============================================================================
// 1. Constructor Traits - InfiniteLine生成機能
// ============================================================================

/// InfiniteLine2D生成のためのConstructorトレイト
pub trait InfiniteLine2DConstructor<T: Scalar> {
    /// 基本コンストラクタ（点と方向ベクトル）
    fn new(point: (T, T), direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 2点から無限直線を作成
    fn from_two_points(p1: (T, T), p2: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// X軸に平行な直線（Y座標指定）
    fn horizontal(y: T) -> Self
    where
        Self: Sized;

    /// Y軸に平行な直線（X座標指定）
    fn vertical(x: T) -> Self
    where
        Self: Sized;

    /// X軸（y=0の水平線）
    fn x_axis() -> Self
    where
        Self: Sized;

    /// Y軸（x=0の垂直線）
    fn y_axis() -> Self
    where
        Self: Sized;

    /// 原点を通る指定方向の直線
    fn through_origin(direction: (T, T)) -> Option<Self>
    where
        Self: Sized;

    // ========== Phase 2: 追加メソッド ==========

    /// 指定角度（ラジアン）の直線を原点を通るように作成
    fn from_angle(angle: T) -> Self
    where
        Self: Sized;

    /// 指定点を通る指定角度の直線を作成
    fn from_point_and_angle(point: (T, T), angle: T) -> Self
    where
        Self: Sized;

    /// 指定点を通り、指定直線に垂直な直線を作成
    fn perpendicular_through(point: (T, T), other: &Self) -> Self
    where
        Self: Sized;
}

/// InfiniteLine3D生成のためのConstructorトレイト
pub trait InfiniteLine3DConstructor<T: Scalar> {
    /// 基本コンストラクタ（点と方向ベクトル）
    fn new(point: (T, T, T), direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 2点から無限直線を作成
    fn from_two_points(p1: (T, T, T), p2: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// X軸に平行な直線
    fn x_parallel(point: (T, T, T)) -> Self
    where
        Self: Sized;

    /// Y軸に平行な直線
    fn y_parallel(point: (T, T, T)) -> Self
    where
        Self: Sized;

    /// Z軸に平行な直線
    fn z_parallel(point: (T, T, T)) -> Self
    where
        Self: Sized;

    /// X軸（原点を通るX方向）
    fn x_axis() -> Self
    where
        Self: Sized;

    /// Y軸（原点を通るY方向）
    fn y_axis() -> Self
    where
        Self: Sized;

    /// Z軸（原点を通るZ方向）
    fn z_axis() -> Self
    where
        Self: Sized;

    /// 原点を通る指定方向の直線
    fn through_origin(direction: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    // ========== Phase 2: 追加メソッド ==========

    /// XY平面上で指定角度の直線を原点を通るように作成
    fn from_xy_angle(angle: T) -> Self
    where
        Self: Sized;

    /// 指定点を通りXY平面上で指定角度の直線を作成
    fn from_point_and_xy_angle(point: (T, T, T), angle: T) -> Self
    where
        Self: Sized;

    /// 指定点を通り、指定直線に垂直かつ指定平面内の直線を作成
    fn perpendicular_in_plane(
        point: (T, T, T),
        other: &Self,
        plane_normal: (T, T, T),
    ) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - InfiniteLine基本情報取得
// ============================================================================

/// InfiniteLine2D基本プロパティ取得トレイト
pub trait InfiniteLine2DProperties<T: Scalar> {
    /// 直線上の点を取得
    fn point(&self) -> (T, T);

    /// 方向ベクトルを取得（正規化済み）
    fn direction(&self) -> (T, T);

    /// 法線ベクトルを取得
    fn normal(&self) -> (T, T);

    /// 傾きを取得（垂直線の場合はNone）
    fn slope(&self) -> Option<T>;

    /// Y切片を取得（垂直線の場合はNone）
    fn y_intercept(&self) -> Option<T>;

    /// X切片を取得（水平線の場合はNone）
    fn x_intercept(&self) -> Option<T>;

    /// 水平線かどうか判定
    fn is_horizontal(&self) -> bool;

    /// 垂直線かどうか判定
    fn is_vertical(&self) -> bool;

    /// 原点を通るかどうか判定
    fn passes_through_origin(&self) -> bool;

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;

    // ========== Phase 2: 追加メソッド ==========

    /// 直線の方向角度（ラジアン）を取得
    fn angle(&self) -> T;

    /// 指定点が直線の上側にあるか判定
    fn is_above(&self, point: (T, T)) -> bool;

    /// 指定点が直線の下側にあるか判定
    fn is_below(&self, point: (T, T)) -> bool;
}

/// InfiniteLine3D基本プロパティ取得トレイト
pub trait InfiniteLine3DProperties<T: Scalar> {
    /// 直線上の点を取得
    fn point(&self) -> (T, T, T);

    /// 方向ベクトルを取得（正規化済み）
    fn direction(&self) -> (T, T, T);

    /// X軸に平行かどうか判定
    fn is_x_parallel(&self) -> bool;

    /// Y軸に平行かどうか判定
    fn is_y_parallel(&self) -> bool;

    /// Z軸に平行かどうか判定
    fn is_z_parallel(&self) -> bool;

    /// XY平面に平行かどうか判定
    fn is_xy_parallel(&self) -> bool;

    /// XZ平面に平行かどうか判定
    fn is_xz_parallel(&self) -> bool;

    /// YZ平面に平行かどうか判定
    fn is_yz_parallel(&self) -> bool;

    /// 原点を通るかどうか判定
    fn passes_through_origin(&self) -> bool;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;

    // ========== Phase 2: 追加メソッド ==========

    /// XY平面上での方向角度（azimuth）を取得
    fn xy_angle(&self) -> T;

    /// 指定平面上にあるか判定
    fn is_on_plane(&self, plane_normal: (T, T, T), plane_point: (T, T, T)) -> bool;

    /// 座標軸に平行か判定（いずれかの軸）
    fn is_axis_aligned(&self) -> bool;
}

// ============================================================================
// 3. Measure Traits - InfiniteLine計量・関係演算機能
// ============================================================================

/// InfiniteLine2D計量・関係演算機能トレイト
pub trait InfiniteLine2DMeasure<T: Scalar> {
    /// パラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 点から直線への距離
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 点が直線上にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 点を直線上に投影
    fn project_point(&self, point: (T, T)) -> (T, T);

    /// 点の直線に対するパラメータを取得
    fn parameter_for_point(&self, point: (T, T)) -> T;

    /// 他の直線との交点を計算
    fn intersection(&self, other: &Self) -> Option<(T, T)>;

    /// 他の直線と平行かどうか判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の直線と垂直かどうか判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の直線と同一かどうか判定
    fn is_same_line(&self, other: &Self) -> bool;

    /// 他の直線との角度を計算
    fn angle_to(&self, other: &Self) -> T;

    /// 直線を反転（方向を逆にする）
    fn reverse(&self) -> Self
    where
        Self: Sized;

    // ========== Phase 2: 追加メソッド ==========

    /// 点を直線に対して鏡面反射
    fn mirror_point(&self, point: (T, T)) -> (T, T);

    /// 指定距離だけ離れた平行な直線を作成
    fn offset(&self, distance: T) -> Self
    where
        Self: Sized;

    /// 原点周りに指定角度だけ回転
    fn rotate_around_origin(&self, angle: T) -> Self
    where
        Self: Sized;

    /// 指定点周りに指定角度だけ回転
    fn rotate_around_point(&self, center: (T, T), angle: T) -> Self
    where
        Self: Sized;
}

/// InfiniteLine3D計量・関係演算機能トレイト
pub trait InfiniteLine3DMeasure<T: Scalar> {
    /// パラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 点から直線への距離
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// 点が直線上にあるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点を直線上に投影
    fn project_point(&self, point: (T, T, T)) -> (T, T, T);

    /// 点の直線に対するパラメータを取得
    fn parameter_for_point(&self, point: (T, T, T)) -> T;

    /// 他の直線との最短距離
    fn distance_to_line(&self, other: &Self) -> T;

    /// 他の直線との最接近点を計算
    fn closest_points(&self, other: &Self) -> Option<TwoPoints3D<T>>;

    /// 他の直線と平行かどうか判定
    fn is_parallel_to(&self, other: &Self) -> bool;

    /// 他の直線と垂直かどうか判定
    fn is_perpendicular_to(&self, other: &Self) -> bool;

    /// 他の直線と同一かどうか判定
    fn is_same_line(&self, other: &Self) -> bool;

    /// 他の直線と交差するかどうか判定
    fn intersects(&self, other: &Self) -> bool;

    /// 他の直線とねじれの位置にあるかどうか判定
    fn is_skew_to(&self, other: &Self) -> bool;

    /// 他の直線との角度を計算
    fn angle_to(&self, other: &Self) -> T;

    /// 直線を反転（方向を逆にする）
    fn reverse(&self) -> Self
    where
        Self: Sized;

    // ========== Phase 2: 追加メソッド ==========

    /// 点を直線に対して鏡面反射
    fn mirror_point(&self, point: (T, T, T)) -> (T, T, T);

    /// 指定軸周りに指定角度だけ回転
    fn rotate_around_axis(&self, axis_point: (T, T, T), axis_direction: (T, T, T), angle: T) -> Option<Self>
    where
        Self: Sized;

    /// 平面との交点を計算
    fn intersection_with_plane(
        &self,
        plane_point: (T, T, T),
        plane_normal: (T, T, T),
    ) -> Option<(T, T, T)>;

    /// 他の直線との交点を計算（交差する場合）
    fn intersection_with_line(&self, other: &Self) -> Option<(T, T, T)>;
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// InfiniteLine2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait InfiniteLine2DCore<T: Scalar>:
    InfiniteLine2DConstructor<T> + InfiniteLine2DProperties<T> + InfiniteLine2DMeasure<T>
{
}

/// InfiniteLine3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait InfiniteLine3DCore<T: Scalar>:
    InfiniteLine3DConstructor<T> + InfiniteLine3DProperties<T> + InfiniteLine3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, L> InfiniteLine2DCore<T> for L where
    L: InfiniteLine2DConstructor<T> + InfiniteLine2DProperties<T> + InfiniteLine2DMeasure<T>
{
}

impl<T: Scalar, L> InfiniteLine3DCore<T> for L where
    L: InfiniteLine3DConstructor<T> + InfiniteLine3DProperties<T> + InfiniteLine3DMeasure<T>
{
}
