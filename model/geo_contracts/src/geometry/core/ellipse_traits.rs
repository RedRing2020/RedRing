//! Ellipse Core Traits - Ellipse形状のCore機能統合
//!
//! Ellipse の trait定義を capability taxonomy に沿って分離する。

use analysis::abstract_types::Scalar;

/// Ellipse2D の生成 trait
pub trait Ellipse2DConstructor<T: Scalar>: Sized {
    /// 中心、長軸、短軸、回転角から楕円を作成
    fn new(center: (T, T), semi_major: T, semi_minor: T, rotation: T) -> Option<Self>;

    /// 単位楕円を作成（原点中心、a=1, b=1、回転なし）
    fn unit_ellipse() -> Self;

    /// 軸に平行な楕円を作成（回転なし）
    fn axis_aligned(center: (T, T), semi_major: T, semi_minor: T) -> Option<Self>;

    /// 円から楕円を作成
    fn from_circle(center: (T, T), radius: T) -> Self;

    /// 焦点と長半軸から楕円を作成
    fn from_foci_and_semi_major(focus1: (T, T), focus2: (T, T), semi_major: T) -> Option<Self>;

    /// 原点中心の楕円を作成
    fn centered_at_origin(semi_major: T, semi_minor: T, rotation: T) -> Option<Self>;
}

/// Ellipse2D の定義パラメータ
pub trait Ellipse2DProperties<T: Scalar> {
    /// 中心座標を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;
}

/// Ellipse2D の派生量
pub trait Ellipse2DDerived<T: Scalar> {
    /// 面積を返す
    fn area(&self) -> T;

    /// 閉曲線の主語彙としての周回長を返す
    fn circumference(&self) -> T;

    /// 離心率を返す
    fn eccentricity(&self) -> T;

    /// 焦点間距離を返す
    fn focal_distance(&self) -> T;

    /// 第1焦点を返す
    fn focus1(&self) -> (T, T);

    /// 第2焦点を返す
    fn focus2(&self) -> (T, T);

    /// 線形離心率を返す
    fn linear_eccentricity(&self) -> T;

    /// 円の特殊ケースかどうかを返す
    fn is_circle(&self) -> bool;
}

/// Ellipse2D の評価
pub trait Ellipse2DEvaluation<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T);
}

/// Ellipse2D の包含判定
pub trait Ellipse2DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T)) -> bool;
    fn point_on_boundary(&self, point: (T, T)) -> bool;
}

/// Ellipse2D の距離計算
pub trait Ellipse2DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T)) -> T;
}

pub trait Ellipse2DProjection<T: Scalar> {
    fn closest_point_to(&self, point: (T, T)) -> (T, T);
}

/// Ellipse2D の互換 Core trait
pub trait Ellipse2DCore<T: Scalar>: Ellipse2DConstructor<T> + Ellipse2DProperties<T> {}

/// Ellipse3D の生成 trait
pub trait Ellipse3DConstructor<T: Scalar>: Sized {
    /// 中心、法線、長軸、短軸、長軸方向から楕円を作成
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        major_axis_direction: (T, T, T),
    ) -> Option<Self>;

    /// 完全な座標系で楕円を作成
    fn new_with_coordinate_system(
        center: (T, T, T),
        normal: (T, T, T),
        major_axis_direction: (T, T, T),
        minor_axis_direction: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
    ) -> Option<Self>;

    /// XY平面上の楕円を作成
    fn new_xy_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>;

    /// XZ平面上の楕円を作成
    fn new_xz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>;

    /// YZ平面上の楕円を作成
    fn new_yz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
    ) -> Option<Self>;

    /// XY平面の単位楕円を作成
    fn unit_ellipse_xy() -> Self;
}

/// Ellipse3D の定義パラメータ
pub trait Ellipse3DProperties<T: Scalar> {
    /// 楕円平面の法線ベクトルを取得
    fn normal(&self) -> (T, T, T);

    /// 長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> (T, T, T);

    /// 短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> (T, T, T);

    /// 中心座標を取得
    fn center_3d(&self) -> (T, T, T);

    /// 中心座標をタプルとして取得
    fn center_3d_tuple(&self) -> (T, T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;
}

/// Ellipse3D の派生量
pub trait Ellipse3DDerived<T: Scalar> {
    /// 面積を返す
    fn area(&self) -> T;

    /// 閉曲線の主語彙としての周回長を返す
    fn circumference(&self) -> T;

    /// 離心率を返す
    fn eccentricity(&self) -> T;

    /// 焦点間距離を返す
    fn focal_distance(&self) -> T;

    /// 円の特殊ケースかどうかを返す
    fn is_circle(&self) -> bool;
}

/// Ellipse3D の評価
pub trait Ellipse3DEvaluation<T: Scalar> {
    fn point_at_parameter(&self, t: T) -> (T, T, T);
}

/// Ellipse3D の包含判定
pub trait Ellipse3DContainment<T: Scalar> {
    fn contains_point(&self, point: (T, T, T)) -> bool;

    fn contains_point_3d(&self, point: (T, T, T)) -> bool {
        self.contains_point(point)
    }
}

/// Ellipse3D の距離計算
pub trait Ellipse3DDistance<T: Scalar> {
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    fn distance_to_point_3d(&self, point: (T, T, T)) -> T {
        self.distance_to_point(point)
    }
}

/// Ellipse3D の射影
pub trait Ellipse3DProjection<T: Scalar> {
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);
}

/// Ellipse3D の互換 Core trait
pub trait Ellipse3DCore<T: Scalar>: Ellipse3DConstructor<T> + Ellipse3DProperties<T> {}

impl<T: Scalar, Ellipse> Ellipse2DCore<T> for Ellipse where
    Ellipse: Ellipse2DConstructor<T> + Ellipse2DProperties<T>
{
}

impl<T: Scalar, Ellipse> Ellipse3DCore<T> for Ellipse where
    Ellipse: Ellipse3DConstructor<T> + Ellipse3DProperties<T>
{
}
