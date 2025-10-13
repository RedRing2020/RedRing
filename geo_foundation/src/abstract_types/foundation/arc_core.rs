//! Arc Core Foundation - 円弧Core Foundation統一システム
//!
//! 円弧のCore Foundation パターン実装
//! 統一インターフェースによる円弧の基本トレイト群
//! Transform, Collision, Intersection Foundation と統合可能

use crate::Scalar;
use std::fmt::Debug;

// ============================================================================
// Arc Core Foundation - 統一基盤システム
// ============================================================================

/// 円弧Core Foundation統一トレイト
///
/// Foundation統一システムにおける円弧の基本インターフェース
/// Transform, Collision, Intersection Foundation と統合可能
pub trait ArcCore<T: Scalar>: Debug + Clone {
    /// 円の型
    type Circle;
    /// 点の型  
    type Point;
    /// 角度の型
    type Angle;

    /// 基底円を取得
    fn circle(&self) -> &Self::Circle;

    /// 開始角度を取得
    fn start_angle(&self) -> Self::Angle;

    /// 終了角度を取得
    fn end_angle(&self) -> Self::Angle;

    /// 完全な円かどうかを判定
    fn is_full_circle(&self) -> bool;

    /// 開始点を取得
    fn start_point(&self) -> Self::Point;

    /// 終了点を取得
    fn end_point(&self) -> Self::Point;
}

/// Arc メトリクス Foundation トレイト
///
/// 円弧の計測機能を統一インターフェースで提供
pub trait ArcMetrics<T: Scalar>: ArcCore<T> {
    /// 円弧の長さを取得
    fn arc_length(&self) -> T;

    /// 扇形の面積を取得
    fn sector_area(&self) -> T;

    /// 角度スパンを取得
    fn angle_span(&self) -> Self::Angle;

    /// 中点角度を取得
    fn mid_angle(&self) -> Self::Angle;

    /// 中点を取得
    fn mid_point(&self) -> Self::Point;
}

/// 統一Arc Foundation トレイト
///
/// 全ての Foundation システムを統合する統一インターフェース
/// Core + Metrics + Transform + Collision + Intersection
pub trait UnifiedArcFoundation<T: Scalar>: ArcCore<T> + ArcMetrics<T> {
    /// Foundation システム統一メソッド群
    /// Transform, Collision, Intersection Foundation の統一アクセス
    ///
    /// 統一変換メソッド
    fn foundation_transform(&self, operation: &str) -> Option<Self>;

    /// 統一距離計算メソッド  
    fn foundation_distance(&self, other: &Self) -> T;

    /// 統一交点計算メソッド
    fn foundation_intersection(&self, other: &Self) -> Option<Self::Point>;
}

// ============================================================================
// 3D Arc Core Foundation
// ============================================================================

/// 3D円弧 Core Foundation トレイット
pub trait Arc3DCore<T: Scalar>: ArcCore<T> {
    /// ベクトル型
    type Vector;

    /// 円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 円弧の平面上でのU軸ベクトルを取得（開始角度方向）
    fn u_axis(&self) -> Self::Vector;

    /// 円弧の平面上でのV軸ベクトルを取得（U軸に垂直）
    fn v_axis(&self) -> Self::Vector;
}
