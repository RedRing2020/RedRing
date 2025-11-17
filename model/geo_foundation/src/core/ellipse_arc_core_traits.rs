//! EllipseArc Core Traits - 楕円弧形状の3つのCore機能統合
//!
//! Foundation パターンに基づく楕円弧のCore機能統合
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;
use analysis::linalg::vector::{Vector2, Vector3};

// ============================================================================
// 1. Constructor Traits - EllipseArc生成機能
// ============================================================================

/// EllipseArc2D生成のためのConstructorトレイト
pub trait EllipseArc2DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、長軸半径、短軸半径、回転角、開始角、終了角）
    fn new(
        center: (T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 楕円と角度範囲から作成
    fn from_ellipse_and_angles(
        center: (T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 開始点、終了点、中心から作成（角度自動計算）
    fn from_center_and_endpoints(
        center: (T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_point: (T, T),
        end_point: (T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    /// 3点から楕円弧を構築（弧上の3点）
    fn from_three_points(p1: (T, T), p2: (T, T), p3: (T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 完全な楕円から楕円弧を作成
    fn from_full_ellipse(
        center: (T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_angle: T,
        sweep_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 接線と点から楕円弧を作成
    fn from_tangent_and_point(
        start_point: (T, T),
        start_tangent: (T, T),
        end_point: (T, T),
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 単位楕円弧作成（原点中心、a=1, b=1, 回転なし、0度から90度）
    fn unit_ellipse_arc() -> Self
    where
        Self: Sized;
}

/// EllipseArc3D生成のためのConstructorトレイト
pub trait EllipseArc3DConstructor<T: Scalar> {
    /// 基本コンストラクタ（中心点、平面法線、長軸半径、短軸半径、長軸方向、角度範囲）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        major_axis_direction: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 完全な座標系で作成（minor_axis_directionは自動計算）
    fn new_with_coordinate_system(
        center: (T, T, T),
        normal: (T, T, T),
        major_axis_direction: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の楕円弧作成
    fn new_xy_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XZ平面上の楕円弧作成
    fn new_xz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// YZ平面上の楕円弧作成
    fn new_yz_plane(
        center: (T, T, T),
        semi_major_axis: T,
        semi_minor_axis: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 3D空間の3点から楕円弧を構築
    fn from_three_points_3d(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// XY平面単位楕円弧（0度から90度）
    fn unit_ellipse_arc_xy() -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - EllipseArc基本情報アクセス
// ============================================================================

/// EllipseArc2D基本プロパティアクセス
pub trait EllipseArc2DProperties<T: Scalar> {
    /// 楕円弧の中心座標を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 楕円の回転角を取得（ラジアン）
    fn rotation(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    /// 角度範囲（sweep angle）を取得
    fn sweep_angle(&self) -> T {
        let mut sweep = self.end_angle() - self.start_angle();
        // 常に正の角度範囲を返す
        if sweep < T::ZERO {
            sweep += T::TAU;
        }
        sweep
    }

    /// 開始点の座標を取得
    fn start_point(&self) -> (T, T);

    /// 終了点の座標を取得
    fn end_point(&self) -> (T, T);

    /// 中点の座標を取得
    fn mid_point(&self) -> (T, T);

    /// 楕円弧が閉じているか（完全な楕円）
    fn is_closed(&self) -> bool {
        (self.sweep_angle() - T::TAU).abs() < T::EPSILON
    }

    /// 楕円弧が半円以上か
    fn is_major_arc(&self) -> bool {
        self.sweep_angle() > T::PI
    }

    /// Analysis層互換の座標変換
    fn to_analysis_vector(&self) -> Vector2<T>;

    /// 中心点をタプルとして取得
    fn center_tuple(&self) -> (T, T) {
        self.center()
    }
}

/// EllipseArc3D基本プロパティアクセス
pub trait EllipseArc3DProperties<T: Scalar>: EllipseArc2DProperties<T> {
    /// 楕円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> (T, T, T);

    /// 楕円の長軸方向ベクトルを取得
    fn major_axis_direction(&self) -> (T, T, T);

    /// 楕円の短軸方向ベクトルを取得
    fn minor_axis_direction(&self) -> (T, T, T);

    /// 3D中心座標を取得
    fn center_3d(&self) -> (T, T, T);

    /// 3D開始点の座標を取得
    fn start_point_3d(&self) -> (T, T, T);

    /// 3D終了点の座標を取得
    fn end_point_3d(&self) -> (T, T, T);

    /// 3D中点の座標を取得
    fn mid_point_3d(&self) -> (T, T, T);

    /// Analysis層互換の3D座標変換
    fn to_analysis_vector_3d(&self) -> Vector3<T>;

    /// 3D中心点をタプルとして取得
    fn center_3d_tuple(&self) -> (T, T, T) {
        self.center_3d()
    }
}

// ============================================================================
// 3. Measure Traits - EllipseArc計量・関係演算
// ============================================================================

/// EllipseArc2D計量機能
pub trait EllipseArc2DMeasure<T: Scalar> {
    /// 楕円弧の弧長を計算
    fn arc_length(&self) -> T;

    /// 楕円弧の弧長を高精度計算（数値積分）
    fn arc_length_precise(&self, tolerance: T) -> T;

    /// 楕円弧の包絡面積を計算（扇形の面積）
    fn sector_area(&self) -> T;

    /// 楕円弧セグメントの面積を計算（弦で区切られた領域）
    fn segment_area(&self) -> T;

    /// 指定したパラメータでの楕円弧上の点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);

    /// 指定した角度での楕円弧上の点を取得
    fn point_at_angle(&self, angle: T) -> (T, T);

    /// 指定した弧長位置での楕円弧上の点を取得
    fn point_at_arc_length(&self, arc_length: T) -> Option<(T, T)>;

    /// 楕円弧上の指定点での接線方向を取得
    fn tangent_at_parameter(&self, t: T) -> (T, T);

    /// 楕円弧上の指定点での法線方向を取得
    fn normal_at_parameter(&self, t: T) -> (T, T);

    /// 楕円弧上の指定点での曲率を取得
    fn curvature_at_parameter(&self, t: T) -> T;

    /// 点が楕円弧上にあるかを判定
    fn contains_point(&self, point: (T, T), tolerance: T) -> bool;

    /// 点から楕円弧への最短距離を計算
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 楕円弧上の点で点に最も近い点を取得
    fn closest_point_on_arc(&self, point: (T, T)) -> (T, T);

    /// 他の楕円弧との交点を計算
    fn intersection_with_arc(&self, other: &Self) -> Vec<(T, T)>;

    /// 直線との交点を計算
    fn intersection_with_line(&self, line_point: (T, T), line_direction: (T, T)) -> Vec<(T, T)>;

    /// 楕円弧の境界ボックスを取得
    fn bounding_box(&self) -> ((T, T), (T, T));

    /// 楕円弧が凸形状か判定
    fn is_convex(&self) -> bool;

    /// 楕円弧の曲率中心を計算
    fn center_of_curvature_at_parameter(&self, t: T) -> (T, T);
}

/// EllipseArc3D計量機能
pub trait EllipseArc3DMeasure<T: Scalar>: EllipseArc2DMeasure<T> {
    /// 3D空間での楕円弧の弧長を計算
    fn arc_length_3d(&self) -> T;

    /// 3D空間での指定パラメータでの楕円弧上の点を取得
    fn point_at_parameter_3d(&self, t: T) -> (T, T, T);

    /// 3D空間での指定角度での楕円弧上の点を取得
    fn point_at_angle_3d(&self, angle: T) -> (T, T, T);

    /// 3D空間での楕円弧上の指定点での接線方向を取得
    fn tangent_at_parameter_3d(&self, t: T) -> (T, T, T);

    /// 3D空間での楕円弧上の指定点での法線方向を取得（平面内法線）
    fn normal_at_parameter_3d(&self, t: T) -> (T, T, T);

    /// 3D空間での楕円弧上の指定点での双法線方向を取得（平面法線）
    fn binormal_at_parameter_3d(&self, t: T) -> (T, T, T);

    /// 3D空間での点が楕円弧上にあるかを判定
    fn contains_point_3d(&self, point: (T, T, T), tolerance: T) -> bool;

    /// 3D空間での点から楕円弧への最短距離を計算
    fn distance_to_point_3d(&self, point: (T, T, T)) -> T;

    /// 3D空間での楕円弧上の点で点に最も近い点を取得
    fn closest_point_on_arc_3d(&self, point: (T, T, T)) -> (T, T, T);

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

    /// 3D空間での楕円弧の境界ボックスを取得
    fn bounding_box_3d(&self) -> ((T, T, T), (T, T, T));

    /// 楕円弧が平面内に含まれるか判定
    fn is_planar(&self, tolerance: T) -> bool;

    /// 楕円弧の投影を指定平面に計算
    fn project_to_plane(&self, plane_point: (T, T, T), plane_normal: (T, T, T)) -> Vec<(T, T, T)>;
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// EllipseArc2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait EllipseArc2DCore<T: Scalar>:
    EllipseArc2DConstructor<T> + EllipseArc2DProperties<T> + EllipseArc2DMeasure<T>
{
}

/// EllipseArc3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait EllipseArc3DCore<T: Scalar>:
    EllipseArc3DConstructor<T> + EllipseArc3DProperties<T> + EllipseArc3DMeasure<T>
{
}

// ============================================================================
// パラメトリック表現トレイト（高度な機能）
// ============================================================================

/// 楕円弧のパラメトリック表現機能
pub trait EllipseArcParametric<T: Scalar> {
    /// パラメータの有効範囲を取得
    fn parameter_range(&self) -> (T, T);

    /// パラメータを正規化（0.0 ～ 1.0 の範囲に変換）
    fn normalize_parameter(&self, t: T) -> T;

    /// 正規化されたパラメータを元の範囲に復元
    fn denormalize_parameter(&self, normalized_t: T) -> T;

    /// パラメータでの1次微分（速度ベクトル）
    fn first_derivative_at(&self, t: T) -> (T, T);

    /// パラメータでの2次微分（加速度ベクトル）
    fn second_derivative_at(&self, t: T) -> (T, T);

    /// パラメータでの曲率計算
    fn curvature_at(&self, t: T) -> T;

    /// パラメータでの曲率半径計算
    fn radius_of_curvature_at(&self, t: T) -> T {
        let curvature = self.curvature_at(t);
        if curvature.abs() < T::EPSILON {
            T::INFINITY
        } else {
            T::ONE / curvature.abs()
        }
    }
}

/// 楕円弧の分割・サンプリング機能
pub trait EllipseArcSampling<T: Scalar> {
    /// 楕円弧を等間隔のパラメータで分割
    fn sample_by_parameter(&self, num_points: usize) -> Vec<(T, T)>;

    /// 楕円弧を等弧長で分割
    fn sample_by_arc_length(&self, num_points: usize) -> Vec<(T, T)>;

    /// 楕円弧を指定した角度間隔で分割
    fn sample_by_angle(&self, angle_step: T) -> Vec<(T, T)>;

    /// 楕円弧を適応的にサンプリング（曲率に基づく）
    fn adaptive_sample(&self, tolerance: T, max_points: usize) -> Vec<(T, T)>;
}
