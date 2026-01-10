//! BBox Core Traits - BBox形状の3つのCore機能統合
//!
//! Foundation ハイブリッド実装方針に基づく
//! Core機能（Constructor/Properties/Measure）を形状別に統合
//! Transform機能は共通のAnalysisTransformトレイトを使用

use crate::Scalar;

// ============================================================================
// Type Aliases for Complex Types
// ============================================================================

/// 2つの2D点を表す型エイリアス（BBox2D用）
pub type TwoPoints2D<T> = ((T, T), (T, T));

/// 2つの3D点を表す型エイリアス（BBox3D用）
pub type TwoPoints3D<T> = ((T, T, T), (T, T, T));

/// 8つの3D点を表す型エイリアス（BBox3D頂点用）
pub type EightPoints3D<T> = [(T, T, T); 8];

// ============================================================================
// 1. Constructor Traits - BBox生成機能
// ============================================================================

/// BBox2D生成のためのConstructorトレイト
pub trait BBox2DConstructor<T: Scalar> {
    /// 最小点と最大点から境界ボックスを作成
    fn new(min: (T, T), max: (T, T)) -> Self
    where
        Self: Sized;

    /// 単一点から境界ボックスを作成（点境界ボックス）
    fn from_point(point: (T, T)) -> Self
    where
        Self: Sized;

    /// 複数の点を包含する境界ボックスを作成
    fn from_points(points: &[(T, T)]) -> Option<Self>
    where
        Self: Sized;

    /// 中心点とサイズから境界ボックスを作成
    fn from_center_size(center: (T, T), width: T, height: T) -> Self
    where
        Self: Sized;

    /// 原点中心の単位境界ボックス（-0.5 to +0.5）
    fn unit_box() -> Self
    where
        Self: Sized;

    /// 空の境界ボックス（無効な状態）
    fn empty() -> Self
    where
        Self: Sized;
}

/// BBox3D生成のためのConstructorトレイト
pub trait BBox3DConstructor<T: Scalar> {
    /// 最小点と最大点から3D境界ボックスを作成
    fn new(min: (T, T, T), max: (T, T, T)) -> Self
    where
        Self: Sized;

    /// 単一点から3D境界ボックスを作成
    fn from_point(point: (T, T, T)) -> Self
    where
        Self: Sized;

    /// 複数の点を包含する3D境界ボックスを作成
    fn from_points(points: &[(T, T, T)]) -> Option<Self>
    where
        Self: Sized;

    /// 中心点とサイズから3D境界ボックスを作成
    fn from_center_size(center: (T, T, T), width: T, height: T, depth: T) -> Self
    where
        Self: Sized;

    /// 原点中心の単位3D境界ボックス（-0.5 to +0.5）
    fn unit_box() -> Self
    where
        Self: Sized;

    /// 空の3D境界ボックス
    fn empty() -> Self
    where
        Self: Sized;

    /// XY平面上の2D境界ボックスから3D境界ボックスを作成
    fn from_2d_with_z_range(min_2d: (T, T), max_2d: (T, T), z_min: T, z_max: T) -> Self
    where
        Self: Sized;
}

// ============================================================================
// 2. Properties Traits - BBox基本情報取得
// ============================================================================

/// BBox2D基本プロパティ取得トレイト
pub trait BBox2DProperties<T: Scalar> {
    /// 最小点（左下）を取得
    fn min(&self) -> (T, T);

    /// 最大点（右上）を取得
    fn max(&self) -> (T, T);

    /// 中心点を取得
    fn center(&self) -> (T, T);

    /// 幅を取得
    fn width(&self) -> T;

    /// 高さを取得
    fn height(&self) -> T;

    /// サイズ（幅、高さ）を取得
    fn size(&self) -> (T, T);

    /// 境界ボックスが空かどうか判定
    fn is_empty(&self) -> bool;

    /// 境界ボックスが有効かどうか判定
    fn is_valid(&self) -> bool;

    /// 境界ボックスが点かどうか判定（幅・高さが0）
    fn is_point(&self) -> bool;

    /// 4つの角の点を取得（左下、右下、右上、左上）
    fn corners(&self) -> [(T, T); 4];

    /// 形状の次元数（2）
    fn dimension(&self) -> u32;
}

/// BBox3D基本プロパティ取得トレイト
pub trait BBox3DProperties<T: Scalar> {
    /// 最小点を取得
    fn min(&self) -> (T, T, T);

    /// 最大点を取得
    fn max(&self) -> (T, T, T);

    /// 中心点を取得
    fn center(&self) -> (T, T, T);

    /// 幅（X方向）を取得
    fn width(&self) -> T;

    /// 高さ（Y方向）を取得
    fn height(&self) -> T;

    /// 奥行き（Z方向）を取得
    fn depth(&self) -> T;

    /// サイズ（幅、高さ、奥行き）を取得
    fn size(&self) -> (T, T, T);

    /// 境界ボックスが空かどうか判定
    fn is_empty(&self) -> bool;

    /// 境界ボックスが有効かどうか判定
    fn is_valid(&self) -> bool;

    /// 境界ボックスが点かどうか判定（全次元が0）
    fn is_point(&self) -> bool;

    /// 8つの頂点を取得
    fn vertices(&self) -> EightPoints3D<T>;

    /// XY平面での2D射影を取得
    fn xy_projection(&self) -> TwoPoints2D<T>;

    /// XZ平面での2D射影を取得
    fn xz_projection(&self) -> TwoPoints2D<T>;

    /// YZ平面での2D射影を取得
    fn yz_projection(&self) -> TwoPoints2D<T>;

    /// 形状の次元数（3）
    fn dimension(&self) -> u32;
}

// ============================================================================
// 3. Measure Traits - BBox計量・関係演算機能
// ============================================================================

/// BBox2D計量・関係演算機能トレイト
pub trait BBox2DMeasure<T: Scalar> {
    /// 面積を計算
    fn area(&self) -> T;

    /// 周囲長を計算
    fn perimeter(&self) -> T;

    /// 対角線の長さを計算
    fn diagonal_length(&self) -> T;

    /// 点が境界ボックス内にあるかを判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 点が境界ボックスの境界上にあるかを判定
    fn point_on_boundary(&self, point: (T, T)) -> bool;

    /// 他の境界ボックスと交差するかを判定
    fn intersects(&self, other: &Self) -> bool;

    /// 他の境界ボックスを完全に含むかを判定
    fn contains_bbox(&self, other: &Self) -> bool;

    /// 他の境界ボックスとの交集合を計算
    fn intersection(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    /// 他の境界ボックスとの和集合を計算
    fn union(&self, other: &Self) -> Self
    where
        Self: Sized;

    /// 点までの距離を計算（境界ボックス外の場合）
    fn distance_to_point(&self, point: (T, T)) -> T;

    /// 点に最も近い境界ボックス上の点を取得
    fn closest_point_to(&self, point: (T, T)) -> (T, T);

    /// 他の境界ボックスとの距離を計算
    fn distance_to_bbox(&self, other: &Self) -> T;

    /// 境界ボックスを拡張
    fn expand(&self, margin: T) -> Self
    where
        Self: Sized;

    /// 境界ボックスを縮小
    fn shrink(&self, margin: T) -> Option<Self>
    where
        Self: Sized;

    /// 境界ボックスに点を追加して拡張
    fn extend_to_include_point(&self, point: (T, T)) -> Self
    where
        Self: Sized;

    /// 境界ボックスに他の境界ボックスを追加して拡張
    fn extend_to_include_bbox(&self, other: &Self) -> Self
    where
        Self: Sized;
}

/// BBox3D計量・関係演算機能トレイト
pub trait BBox3DMeasure<T: Scalar> {
    /// 体積を計算
    fn volume(&self) -> T;

    /// 表面積を計算
    fn surface_area(&self) -> T;

    /// 対角線の長さを計算
    fn diagonal_length(&self) -> T;

    /// 点が境界ボックス内にあるかを判定
    fn contains_point(&self, point: (T, T, T)) -> bool;

    /// 点が境界ボックスの表面上にあるかを判定
    fn point_on_surface(&self, point: (T, T, T)) -> bool;

    /// 他の境界ボックスと交差するかを判定
    fn intersects(&self, other: &Self) -> bool;

    /// 他の境界ボックスを完全に含むかを判定
    fn contains_bbox(&self, other: &Self) -> bool;

    /// 他の境界ボックスとの交集合を計算
    fn intersection(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    /// 他の境界ボックスとの和集合を計算
    fn union(&self, other: &Self) -> Self
    where
        Self: Sized;

    /// 点までの距離を計算
    fn distance_to_point(&self, point: (T, T, T)) -> T;

    /// 点に最も近い境界ボックス上の点を取得
    fn closest_point_to(&self, point: (T, T, T)) -> (T, T, T);

    /// 他の境界ボックスとの距離を計算
    fn distance_to_bbox(&self, other: &Self) -> T;

    /// 境界ボックスを拡張
    fn expand(&self, margin: T) -> Self
    where
        Self: Sized;

    /// 境界ボックスを縮小
    fn shrink(&self, margin: T) -> Option<Self>
    where
        Self: Sized;

    /// 境界ボックスに点を追加して拡張
    fn extend_to_include_point(&self, point: (T, T, T)) -> Self
    where
        Self: Sized;

    /// 境界ボックスに他の境界ボックスを追加して拡張
    fn extend_to_include_bbox(&self, other: &Self) -> Self
    where
        Self: Sized;

    /// 指定された軸での射影区間を取得
    fn projection_interval(&self, axis: (T, T, T)) -> (T, T);
}

// ============================================================================
// 統合Traitバンドル（利便性向上）
// ============================================================================

/// BBox2Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform2D<T>を別途使用
pub trait BBox2DCore<T: Scalar>:
    BBox2DConstructor<T> + BBox2DProperties<T> + BBox2DMeasure<T>
{
}

/// BBox3Dの3つのCore機能統合トレイト
/// Transform機能はAnalysisTransform3D<T>を別途使用
pub trait BBox3DCore<T: Scalar>:
    BBox3DConstructor<T> + BBox3DProperties<T> + BBox3DMeasure<T>
{
}

// ============================================================================
// Blanket implementations for Core traits
// ============================================================================

impl<T: Scalar, B> BBox2DCore<T> for B where
    B: BBox2DConstructor<T> + BBox2DProperties<T> + BBox2DMeasure<T>
{
}

impl<T: Scalar, B> BBox3DCore<T> for B where
    B: BBox3DConstructor<T> + BBox3DProperties<T> + BBox3DMeasure<T>
{
}
