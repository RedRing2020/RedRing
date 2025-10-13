//! Arc Extension Foundation - 円弧拡張機能統一システム
//!
//! 円弧の拡張機能（Metrics、Containment、Transform、Sampling）を
//! Foundation統一パターンで提供する共通抽象化システム

use crate::Scalar;
use std::fmt::Debug;

// ============================================================================
// Arc Metrics Foundation
// ============================================================================

/// Arc 計測機能拡張Foundation
///
/// Core計測機能を超える詳細な解析とサンプリング機能
pub trait ArcExtensionMetrics<T: Scalar>: Debug {
    /// 弧長を計算
    fn arc_length(&self) -> T;

    /// 扇形の面積を計算
    fn sector_area(&self) -> T;

    /// 弦の長さを計算
    fn chord_length(&self) -> T;

    /// 矢高（sagitta）を計算
    fn sagitta(&self) -> T;

    /// 扇形の周長を計算（弧長 + 2 × 半径）
    fn sector_perimeter(&self) -> T;
}

// ============================================================================
// Arc Containment Foundation
// ============================================================================

/// 円弧包含判定の Foundation トレイト
///
/// 点と円弧の位置関係や角度範囲判定を提供
pub trait ArcContainment<T: Scalar>: Debug {
    /// 点の型
    type Point;

    /// 角度の型
    type Angle;

    /// 指定した点が円弧上にあるかを判定
    fn contains_point(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 指定した角度が円弧の範囲内にあるかを判定
    fn contains_angle(&self, angle: Self::Angle) -> bool;

    /// 指定した角度での円弧上の点を取得
    fn point_at_angle(&self, angle: T) -> Self::Point;

    /// 円弧上の点から角度を逆算
    fn angle_at_point(&self, point: &Self::Point) -> Option<Self::Angle>;
}

// ============================================================================
// Arc Sampling Foundation
// ============================================================================

/// 円弧サンプリングの Foundation トレイト
///
/// 円弧の分割や点列生成機能を提供
pub trait ArcSampling<T: Scalar>: Debug {
    /// 点の型
    type Point;

    /// 円弧を指定数の点に分割
    fn sample_points(&self, num_points: usize) -> Vec<Self::Point>;

    /// 指定された弧長間隔で点列を生成
    fn sample_by_arc_length(&self, arc_length_step: T) -> Vec<Self::Point>;

    /// 指定された角度間隔で点列を生成
    fn sample_by_angle(&self, angle_step: T) -> Vec<Self::Point>;

    /// 適応的サンプリング（弦の最大長に基づく）
    fn adaptive_sample(&self, max_chord_length: T) -> Vec<Self::Point>;
}

// ============================================================================
// Unified Arc Extensions Foundation
// ============================================================================

/// 統合円弧拡張 Foundation トレイト
///
/// 全ての円弧拡張機能を統一したインターフェース
pub trait UnifiedArcExtensions<T: Scalar>:
    ArcExtensionMetrics<T> + ArcContainment<T> + ArcSampling<T> + Debug
{
    /// 円弧の包括的な情報を取得
    fn arc_info(&self) -> ArcInfo<T> {
        ArcInfo {
            arc_length: self.arc_length(),
            sector_area: self.sector_area(),
            chord_length: self.chord_length(),
            sagitta: self.sagitta(),
        }
    }
}

/// 円弧の包括的情報構造体
#[derive(Debug, Clone, PartialEq)]
pub struct ArcInfo<T: Scalar> {
    /// 弧長
    pub arc_length: T,
    /// 扇形面積
    pub sector_area: T,
    /// 弦長
    pub chord_length: T,
    /// 矢高
    pub sagitta: T,
}

// ============================================================================
// Auto-implementation for Unified Extensions
// ============================================================================

/// 全ての個別トレイトを実装した型に対して自動的に統合トレイトを実装
impl<T, U> UnifiedArcExtensions<T> for U
where
    T: Scalar,
    U: ArcExtensionMetrics<T> + ArcContainment<T> + ArcSampling<T> + Debug,
{
    // デフォルト実装を使用
}
