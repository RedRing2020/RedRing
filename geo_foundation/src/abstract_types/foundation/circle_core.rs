//! Circle Core Foundation - 円Core Foundation統一システム
//!
//! 円のCore Foundation パターン実装
//! 統一インターフェースによる円の基本トレイト群
//! Transform, Collision, Intersection Foundation と統合可能

use crate::Scalar;
use std::fmt::Debug;

// ============================================================================
// Circle Core Foundation - 統一基盤システム
// ============================================================================

/// 円Core Foundation トレイト
///
/// 円の Foundation 統一システム基盤
pub trait CircleCore<T: Scalar>: Debug + Clone {
    /// 点の型
    type Point;

    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 半径を取得
    fn radius(&self) -> T;
}

/// 円メトリクス Foundation トレイト
///
/// 円の計測機能を統一インターフェースで提供
pub trait CircleMetrics<T: Scalar>: CircleCore<T> {
    /// 円の面積を取得
    fn area(&self) -> T;

    /// 円の周長を取得
    fn circumference(&self) -> T;

    /// 円の直径を取得
    fn diameter(&self) -> T {
        self.radius() + self.radius() // 2 * radius
    }
}

/// 統一Circle Foundation トレイト
///
/// 全ての Foundation システムを統合する統一インターフェース
/// Core + Metrics + Transform + Collision + Intersection
pub trait UnifiedCircleFoundation<T: Scalar>: CircleCore<T> + CircleMetrics<T> {
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
// 3D Circle Core Foundation
// ============================================================================

/// 3D円Core Foundation トレイト
pub trait Circle3DCore<T: Scalar>: CircleCore<T> {
    /// ベクトル型
    type Vector;

    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 円の平面上でのU軸ベクトルを取得
    fn u_axis(&self) -> Self::Vector;

    /// 円の平面上でのV軸ベクトルを取得
    fn v_axis(&self) -> Self::Vector;
}
