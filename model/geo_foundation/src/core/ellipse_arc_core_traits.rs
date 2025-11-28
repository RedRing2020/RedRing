//! EllipseArc Core Traits - Phase 1 + Phase 2 Implementation
//!
//! Foundation Pattern に基づくCore機能
//! Phase 1: 12メソッド（Constructor 3 + Properties 5 + Measure 4）
//! Phase 2: 10メソッド（Constructor 3 + Properties 3 + Measure 4）
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! 作成日: 2025年11月28日
//! 最終更新日: 2025年11月28日（Phase 2追加）

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - EllipseArc生成機能（3メソッド）
// ============================================================================

/// EllipseArc2D生成のためのConstructorトレイト（Phase 1 + Phase 2）
pub trait EllipseArc2DConstructor<T: Scalar> {
    // ========== Phase 1: 最小限のコンストラクタ（3メソッド） ==========
    
    /// 中心、長軸、短軸、回転角、角度範囲から楕円弧を作成
    fn new(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の単位楕円弧を作成（回転なし、0度から90度）
    fn unit_ellipse_arc() -> Self
    where
        Self: Sized;

    /// 指定した角度範囲の楕円弧を作成（回転なし）
    fn from_ellipse_and_angles(
        center: (T, T),
        semi_major: T,
        semi_minor: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    // ========== Phase 2: 追加コンストラクタ（3メソッド） ==========
    
    /// 円弧から楕円弧を作成（円弧は楕円弧の特殊ケース）
    fn from_circle_arc(
        center: (T, T),
        radius: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 3点を通る楕円弧を作成（開始点、中間点、終了点）
    fn from_three_points(
        start: (T, T),
        mid: (T, T),
        end: (T, T),
    ) -> Option<Self>
    where
        Self: Sized;

    /// 中心と2点から楕円弧を作成（回転角は0）
    fn from_center_and_points(
        center: (T, T),
        start: (T, T),
        end: (T, T),
    ) -> Option<Self>
    where
        Self: Sized;
}

/// EllipseArc3D生成のためのConstructorトレイト（Phase 1 + Phase 2）
pub trait EllipseArc3DConstructor<T: Scalar> {
    // ========== Phase 1: 最小限のコンストラクタ（3メソッド） ==========
    
    /// 3D空間での楕円弧を作成（法線、長軸方向、角度範囲）
    fn new(
        center: (T, T, T),
        normal: (T, T, T),
        semi_major: T,
        semi_minor: T,
        major_direction: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// XY平面上の楕円弧を作成
    fn xy_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 単位楕円弧をXY平面上に作成
    fn unit_ellipse_arc_xy() -> Self
    where
        Self: Sized;

    // ========== Phase 2: 追加コンストラクタ（3メソッド） ==========
    
    /// XZ平面上の楕円弧を作成
    fn xz_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// YZ平面上の楕円弧を作成
    fn yz_plane(
        center: (T, T, T),
        semi_major: T,
        semi_minor: T,
        rotation: T,
        start_angle: T,
        end_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 3点を通る楕円弧を3D空間に作成（開始点、中間点、終了点）
    fn from_three_points(
        start: (T, T, T),
        mid: (T, T, T),
        end: (T, T, T),
    ) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - EllipseArc基本情報取得（5メソッド）
// ============================================================================

/// EllipseArc2D基本プロパティ取得トレイト（Phase 1 + Phase 2）
pub trait EllipseArc2DProperties<T: Scalar> {
    // ========== Phase 1: 最小限のプロパティ（5メソッド） ==========
    
    /// 中心点を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    // ========== Phase 2: 追加プロパティ（3メソッド） ==========
    
    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;

    /// 角度範囲（スイープ角）を取得（ラジアン）
    fn sweep_angle(&self) -> T;

    /// 楕円の離心率を取得（0 <= e < 1）
    fn eccentricity(&self) -> T;
}

/// EllipseArc3D基本プロパティ取得トレイト（Phase 1 + Phase 2）
pub trait EllipseArc3DProperties<T: Scalar> {
    // ========== Phase 1: 最小限のプロパティ（5メソッド） ==========
    
    /// 中心点を取得
    fn center(&self) -> (T, T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 開始角度を取得（ラジアン）
    fn start_angle(&self) -> T;

    /// 終了角度を取得（ラジアン）
    fn end_angle(&self) -> T;

    // ========== Phase 2: 追加プロパティ（3メソッド） ==========
    
    /// 法線ベクトルを取得
    fn normal(&self) -> (T, T, T);

    /// 角度範囲（スイープ角）を取得（ラジアン）
    fn sweep_angle(&self) -> T;

    /// 楕円の離心率を取得（0 <= e < 1）
    fn eccentricity(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - EllipseArc計量・関係演算機能（4メソッド）
// ============================================================================

/// EllipseArc2D計量・関係演算機能トレイト（Phase 1 + Phase 2）
pub trait EllipseArc2DMeasure<T: Scalar> {
    // ========== Phase 1: 最小限の計量（4メソッド） ==========
    
    /// 楕円弧の長さ（測度）
    fn measure(&self) -> T;

    /// 開始点を取得
    fn start_point(&self) -> (T, T);

    /// 終了点を取得
    fn end_point(&self) -> (T, T);

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T);

    // ========== Phase 2: 追加計量（4メソッド） ==========
    
    /// 中点を取得（パラメータt=0.5の点）
    fn mid_point(&self) -> (T, T);

    /// 指定角度での点を取得（ラジアン）
    fn point_at_angle(&self, angle: T) -> Option<(T, T)>;

    /// 点が楕円弧上にあるか判定（許容誤差付き）
    fn contains_point(&self, point: (T, T), tolerance: T) -> bool;

    /// 楕円弧の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T), (T, T));
}

/// EllipseArc3D計量・関係演算機能トレイト（Phase 1 + Phase 2）
pub trait EllipseArc3DMeasure<T: Scalar> {
    // ========== Phase 1: 最小限の計量（4メソッド） ==========
    
    /// 楕円弧の長さ（測度）
    fn measure(&self) -> T;

    /// 開始点を取得
    fn start_point(&self) -> (T, T, T);

    /// 終了点を取得
    fn end_point(&self) -> (T, T, T);

    /// パラメータt（0<=t<=1）での点を取得
    fn point_at_parameter(&self, t: T) -> (T, T, T);

    // ========== Phase 2: 追加計量（4メソッド） ==========
    
    /// 中点を取得（パラメータt=0.5の点）
    fn mid_point(&self) -> (T, T, T);

    /// 指定角度での点を取得（ラジアン）
    fn point_at_angle(&self, angle: T) -> Option<(T, T, T)>;

    /// 点が楕円弧上にあるか判定（許容誤差付き）
    fn contains_point(&self, point: (T, T, T), tolerance: T) -> bool;

    /// 楕円弧の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));
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
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, E> EllipseArc2DCore<T> for E where
    E: EllipseArc2DConstructor<T> + EllipseArc2DProperties<T> + EllipseArc2DMeasure<T>
{
}

impl<T: Scalar, E> EllipseArc3DCore<T> for E where
    E: EllipseArc3DConstructor<T> + EllipseArc3DProperties<T> + EllipseArc3DMeasure<T>
{
}
