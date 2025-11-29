//! SphericalSolid Core Traits - 球ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1: 最小限のメソッド
//! - Constructor: 3メソッド（new, new_standard, unit_sphere）
//! - Properties: 5メソッド（center, radius, axis, ref_direction, diameter）
//! - Measure: 4メソッド（volume, surface_area, contains_point, distance_to_point）
//!
//! ## Phase 2: 標準機能追加
//! - Constructor: +3メソッド（from_diameter, from_bounding_box, from_four_points）
//! - Properties: +3メソッド（is_unit_sphere, circumference, is_centered_at_origin）
//! - Measure: +4メソッド（point_at_latlong, bounding_box, closest_point_on_surface, intersects_sphere）

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - SphericalSolid生成機能（Phase 1: 最小限）
// ============================================================================

/// SphericalSolid3D生成のためのConstructorトレイト
pub trait SphericalSolid3DConstructor<T: Scalar> {
    // Phase 1: 基本コンストラクタ（3メソッド）

    /// STEP準拠のAXIS2_PLACEMENT_3D形式で球ソリッドを作成
    ///
    /// # Arguments
    /// * `center` - 球の中心点（x, y, z）
    /// * `axis` - 参照軸ベクトル（Z軸）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `radius` - 半径（正の値）
    fn new(center: (T, T, T), axis: (T, T, T), ref_direction: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の球ソリッドを作成（簡易コンストラクタ）
    ///
    /// axis = (0, 0, 1), ref_direction = (1, 0, 0)
    fn new_standard(center: (T, T, T), radius: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点中心の単位球ソリッド（半径1）
    fn unit_sphere() -> Self
    where
        Self: Sized;

    // Phase 2: 追加コンストラクタ（3メソッド）

    /// 直径指定で球ソリッドを作成
    fn from_diameter(center: (T, T, T), diameter: T) -> Option<Self>
    where
        Self: Sized;

    /// 境界ボックスに内接する球ソリッドを作成
    fn from_bounding_box(min: (T, T, T), max: (T, T, T)) -> Option<Self>
    where
        Self: Sized;

    /// 4点を通る球（外接球）を作成
    fn from_four_points(p1: (T, T, T), p2: (T, T, T), p3: (T, T, T), p4: (T, T, T)) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - SphericalSolid基本情報取得（Phase 1: 最小限）
// ============================================================================

/// SphericalSolid3D基本プロパティ取得トレイト
pub trait SphericalSolid3DProperties<T: Scalar> {
    // Phase 1: 基本プロパティ（5メソッド）

    /// 球の中心点取得
    fn center(&self) -> (T, T, T);

    /// 球の半径取得
    fn radius(&self) -> T;

    /// 参照軸方向取得（Z軸、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 球の直径取得
    fn diameter(&self) -> T;

    // Phase 2: 追加プロパティ（3メソッド）

    /// 単位球（半径1）かどうか判定
    fn is_unit_sphere(&self) -> bool;

    /// 大円の円周を取得（2πr）
    fn circumference(&self) -> T;

    /// 原点中心かどうか判定
    fn is_centered_at_origin(&self) -> bool;
}

// ============================================================================
// 3. Measure Traits - SphericalSolid測定機能（Phase 1: 最小限）
// ============================================================================

/// SphericalSolid3D測定機能トレイト
pub trait SphericalSolid3DMeasure<T: Scalar> {
    // Phase 1: 基本測定（4メソッド）

    /// 球ソリッドの体積を計算
    ///
    /// 体積 = (4/3)π × r³
    fn volume(&self) -> T;

    /// 球ソリッドの表面積を計算
    ///
    /// 表面積 = 4π × r²
    fn surface_area(&self) -> T;

    /// 点が球ソリッド内部に含まれるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点と球ソリッド表面との距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    // Phase 2: 追加測定（4メソッド）

    /// 緯度経度指定で表面上の点を取得
    ///
    /// latitude ∈ [-π/2, π/2]: 緯度（-90° to 90°）
    /// longitude ∈ [0, 2π]: 経度（0° to 360°）
    fn point_at_latlong(&self, latitude: T, longitude: T) -> (T, T, T);

    /// 球の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));

    /// 指定点に最も近い表面上の点を取得
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);

    /// 他の球との交差判定
    fn intersects_sphere(&self, other_center: (T, T, T), other_radius: T) -> bool;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// SphericalSolid3DのCore機能を統合するトレイト
///
/// Constructor/Properties/Measureの全機能を統合
pub trait SphericalSolid3DCore<T: Scalar>:
    SphericalSolid3DConstructor<T> + SphericalSolid3DProperties<T> + SphericalSolid3DMeasure<T>
{
}
