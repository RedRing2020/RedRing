//! Ellipse Core Traits - Ellipse形状のCore機能統合
//!
//! Foundation Pattern Phase 1 + Phase 2 実装
//! 3-5-4 パターン: Constructor(3) + Properties(5) + Measure(4)

use analysis::abstract_types::Scalar;

// ============================================================================
// Ellipse2D Core Traits
// ============================================================================

/// Ellipse2D Constructor トレイト（3+3メソッド）
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

/// Ellipse2D Properties トレイト（5+3メソッド）
pub trait Ellipse2DProperties<T: Scalar> {
    /// 中心座標を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;

    /// 離心率を取得
    fn eccentricity(&self) -> T;

    /// 焦点間距離を取得
    fn focal_distance(&self) -> T;

    /// 第1焦点を取得
    fn focus1(&self) -> (T, T);

    /// 第2焦点を取得
    fn focus2(&self) -> (T, T);
}

/// Ellipse2D Measure トレイト（4+4メソッド）
pub trait Ellipse2DMeasure<T: Scalar> {
    /// 楕円の面積（測度）を計算
    fn measure(&self) -> T;

    /// 楕円の周長を計算（近似）
    fn perimeter(&self) -> T;

    /// 点が楕円内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 楕円が円かどうか判定
    fn is_circle(&self) -> bool;

    /// パラメータ t における楕円上の点を取得（0 <= t < 2π）
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 点から楕円周への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 楕円周上の点かどうか判定
    fn point_on_boundary(&self, point: (T, T)) -> bool;

    /// 楕円の線形離心率を取得
    fn linear_eccentricity(&self) -> T;
}

/// Ellipse2D Core トレイト（統合インターフェース）
pub trait Ellipse2DCore<T: Scalar>:
    Ellipse2DConstructor<T> + Ellipse2DProperties<T> + Ellipse2DMeasure<T>
{
}

// ============================================================================
// Ellipse3D Core Traits
// ============================================================================

/// Ellipse3D Constructor トレイト（3+3メソッド）
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

/// Ellipse3D Properties トレイト（5+3メソッド）
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

    /// 離心率を取得
    fn eccentricity(&self) -> T;
}

/// Ellipse3D Measure トレイト（4+4メソッド）
pub trait Ellipse3DMeasure<T: Scalar> {
    /// 3D空間での点が楕円内部にあるか判定
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

    /// 楕円の面積を計算
    fn measure(&self) -> T;

    /// 楕円の周長を計算（近似）
    fn perimeter(&self) -> T;

    /// パラメータ t における楕円上の点を取得（0 <= t < 2π）
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    /// 楕円が円かどうか判定
    fn is_circle(&self) -> bool;
}

/// Ellipse3D Core トレイト（統合インターフェース）
pub trait Ellipse3DCore<T: Scalar>:
    Ellipse3DConstructor<T> + Ellipse3DProperties<T> + Ellipse3DMeasure<T>
{
}
