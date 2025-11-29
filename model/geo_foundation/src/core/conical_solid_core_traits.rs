//! ConicalSolid Core Traits - 円錐ソリッドの3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用
//!
//! ## Phase 1: 最小限のメソッド
//! - Constructor: 3メソッド（new, new_standard, unit_cone）
//! - Properties: 7メソッド（apex, base_center, radius, height, axis, ref_direction, slant_height）
//! - Measure: 4メソッド（volume, surface_area, contains_point, distance_to_point）
//!
//! ## Phase 2: 標準機能追加
//! - Constructor: +3メソッド（from_apex_and_base_circle, from_apex_angle, frustum）
//! - Properties: +3メソッド（half_angle, lateral_surface_area, base_area）
//! - Measure: +4メソッド（point_at_conical, bounding_box, closest_point_on_surface, contains_point_tolerance）

use crate::Scalar;

// ============================================================================
// 1. Constructor Traits - ConicalSolid生成機能（Phase 1: 最小限）
// ============================================================================

/// ConicalSolid3D生成のためのConstructorトレイト
pub trait ConicalSolid3DConstructor<T: Scalar> {
    /// STEP準拠のAXIS2_PLACEMENT_3D形式で円錐ソリッドを作成
    ///
    /// # Arguments
    /// * `apex` - 円錐の頂点（x, y, z）
    /// * `base_center` - 底面の中心点（x, y, z）
    /// * `axis` - 円錐の軸方向ベクトル（頂点から底面へ）
    /// * `ref_direction` - 参照方向ベクトル（X軸）
    /// * `radius` - 底面の半径（正の値）
    /// * `height` - 円錐の高さ（正の値）
    fn new(
        apex: (T, T, T),
        base_center: (T, T, T),
        axis: (T, T, T),
        ref_direction: (T, T, T),
        radius: T,
        height: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// Z軸標準の円錐ソリッドを作成（簡易コンストラクタ）
    ///
    /// 頂点を原点、軸を-Z方向
    fn new_standard(base_center: (T, T, T), radius: T, height: T) -> Option<Self>
    where
        Self: Sized;

    /// 原点頂点の単位円錐（底面半径1、高さ2）
    fn unit_cone() -> Self
    where
        Self: Sized;

    // Phase 2: 追加コンストラクタ（3メソッド）

    /// 頂点と底面円から円錐を作成
    fn from_apex_and_base_circle(
        apex: (T, T, T),
        base_center: (T, T, T),
        base_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 頂点、軸、高さ、頂角から円錐を作成
    fn from_apex_angle(
        apex: (T, T, T),
        axis: (T, T, T),
        height: T,
        half_angle: T,
    ) -> Option<Self>
    where
        Self: Sized;

    /// 円錐台（frustum）を作成
    fn frustum(
        base_center: (T, T, T),
        top_center: (T, T, T),
        base_radius: T,
        top_radius: T,
    ) -> Option<Self>
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - ConicalSolid基本情報取得（Phase 1: 最小限）
// ============================================================================

/// ConicalSolid3D基本プロパティ取得トレイト
pub trait ConicalSolid3DProperties<T: Scalar> {
    /// 円錐の頂点取得
    fn apex(&self) -> (T, T, T);

    /// 円錐底面の中心点取得
    fn base_center(&self) -> (T, T, T);

    /// 円錐底面の半径取得
    fn radius(&self) -> T;

    /// 円錐の高さ取得
    fn height(&self) -> T;

    /// 軸方向取得（頂点から底面へ、正規化済み）
    fn axis(&self) -> (T, T, T);

    /// 参照方向取得（X軸、正規化済み）
    fn ref_direction(&self) -> (T, T, T);

    /// 母線長さ（slant height）取得
    ///
    /// 母線長さ = √(r² + h²)
    fn slant_height(&self) -> T;

    // Phase 2: 追加プロパティ（3メソッド）

    /// 頂角の半分を取得（母線と軸のなす角）
    fn half_angle(&self) -> T;

    /// 側面積のみを取得（πr × slant_height）
    fn lateral_surface_area(&self) -> T;

    /// 底面積を取得（πr²）
    fn base_area(&self) -> T;
}

// ============================================================================
// 3. Measure Traits - ConicalSolid測定機能（Phase 1: 最小限）
// ============================================================================

/// ConicalSolid3D測定機能トレイト
pub trait ConicalSolid3DMeasure<T: Scalar> {
    /// 円錐の体積を計算
    ///
    /// 体積 = (1/3) × π × r² × h
    fn volume(&self) -> T;

    /// 円錐の表面積を計算（底面含む）
    ///
    /// 表面積 = π × r × (r + √(r² + h²))
    fn surface_area(&self) -> T;

    /// 点が円錐内部に含まれるか判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点と円錐との最短距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    // Phase 2: 追加測定（4メソッド）

    /// 円錐座標系での点を取得
    ///
    /// r_ratio: 半径比率 [0, 1], theta: 角度, h_ratio: 高さ比率 [0, 1]
    fn point_at_conical(&self, r_ratio: T, theta: T, h_ratio: T) -> (T, T, T);

    /// 円錐の境界ボックスを取得（最小点、最大点）
    fn bounding_box(&self) -> ((T, T, T), (T, T, T));

    /// 指定点に最も近い円錐表面上の点を取得
    fn closest_point_on_surface(&self, point: (T, T, T)) -> (T, T, T);

    /// 許容誤差付きで点が円錐内部に含まれるか判定
    fn contains_point_tolerance(&self, point: (T, T, T), tolerance: T) -> bool;
}

// ============================================================================
// 4. Core統合トレイト
// ============================================================================

/// ConicalSolid3DのCore機能を統合するトレイト
pub trait ConicalSolid3DCore<T: Scalar>:
    ConicalSolid3DConstructor<T> + ConicalSolid3DProperties<T> + ConicalSolid3DMeasure<T>
{
}
