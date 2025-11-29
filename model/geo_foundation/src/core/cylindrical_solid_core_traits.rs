//! CylindricalSolid Core Traits - 円柱ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1: 最小限のメソッド
//! - Constructor: 3メソッド（new, new_standard, unit_cylinder）
//! - Properties: 6メソッド（center, radius, height, axis, ref_direction, diameter）
//! - Measure: 4メソッド（volume, surface_area, contains_point, distance_to_point）
//!
//! ## Phase 2: 標準機能追加
//! - Constructor: +3メソッド（from_axis_and_radius, from_diameter, from_two_points_and_radius）
//! - Properties: +3メソッド（top_center, lateral_surface_area, base_area）
//! - Measure: +4メソッド（point_at_cylindrical, bounding_box, closest_point_on_surface, intersects_line）

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - CylindricalSolid生成機能（Phase 1: 最小限）
// ============================================================================

/// CylindricalSolid3D生成のためのConstructorトレイト
pub trait CylindricalSolid3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式で円柱ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 円柱底面の中心点（x, y, z）
    /// * `axis` - 円柱の軸方向ベクトル（Z軸、高さ方向）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `radius` - 底面の半径（正の値）
    /// * `height` - 円柱の高さ（正の値）
    fn new(
        center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の円柱ソリッドを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位円柱（半径1、高さ2）
    fn unit_cylinder() -> Self
    where
        Self: Sized;

    // Phase 2: 追加コンストラクタ（3メソッド）

    /// 軸線の始点・終点と半径から円柱を作成
    fn from_axis_and_radius(start_point: (T, T, T), end_point: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 直径指定で円柱を作成
    fn from_diameter(center: (T, T, T), axis: (T, T, T), diameter: T, height: T) -> Option<Self>
    where
        Self: Sized;

    /// 2点と半径から円柱を作成（軸方向は2点を結ぶベクトル）
    fn from_two_points_and_radius(p1: (T, T, T), p2: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - CylindricalSolid基本情報取得（Phase 1: 最小限）
// ============================================================================

/// CylindricalSolid3D基本プロパティ取得トレイト
pub trait CylindricalSolid3DProperties<T: Scalar> {
    /// 円柱底面の中心点取得
    fn center(&self) -> (T, T, T);

    /// 円柱底面の半径取得
    fn radius(&self) -> T;

    /// 円柱の高さ取得
    fn height(&self) -> T;

    /// 軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 円柱底面の直径取得
    fn diameter(&self) -> T;

    // Phase 2: 追加プロパティ（3メソッド）

    /// 円柱上面の中心点取得
    fn top_center(&self) -> (T, T, T);

    /// 側面積のみを取得（2πrh）
    fn lateral_surface_area(&self) -> T;

    /// 底面積を取得（πr²）
    fn base_area(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - CylindricalSolid測定機能（Phase 1: 最小限）
// ============================================================================

/// CylindricalSolid3D測定機能トレイト
pub trait CylindricalSolid3DMeasure<T: Scalar> {
    /// 円柱の体積を計算
    ///
    /// 体積 = π × r² × h
    fn volume(&self) -> T;

    /// 円柱の表面積を計算
    ///
    /// 表面積 = 2π × r × (r + h)
    fn surface_area(&self) -> T;

    /// 点が円柱内部に含まれるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点と円柱との最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    // Phase 2: 追加測定（4メソッド）

    /// 円柱座標系での点を取得
    ///
    /// r: 半径方向距離, theta: 角度, z: 高さ方向距離
    fn point_at_cylindrical(&self, r: T, theta: T, z: T) -> (T, T, T);

    /// 円柱の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));

    /// 指定点に最も近い円柱表面上の点を取得
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);

    /// 直線との交点を計算
    fn intersects_line(&self, line_point: (T, T, T), line_dir: (T, T, T)) -> Option<(T, T, T)>;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// CylindricalSolid3DのCore機能を統合するトレイト
pub trait CylindricalSolid3DCore<T: Scalar>:
    CylindricalSolid3DConstructor<T> + CylindricalSolid3DProperties<T> + CylindricalSolid3DMeasure<T>
{
}
