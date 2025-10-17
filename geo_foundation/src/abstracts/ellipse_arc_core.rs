//! EllipseArc Core Foundation - 楕円弧Core Foundation統一システム
//!
//! 楕円弧のCore Foundation パターン実装
//! 統一インターフェースによる楕円弧の基本トレイト群
//! Transform, Collision, Intersection Foundation と統合可能

use crate::Scalar;
use std::fmt::Debug;

// ============================================================================
// Ellipse Arc Core Foundation - 統一基盤システム
// ============================================================================

/// 楕円円弧Core Foundation トレイト
///
/// 楕円弧の Foundation 統一システム基盤
pub trait EllipseArcCore<T: Scalar>: Debug + Clone {
    /// 点の型
    type Point;
    /// 角度の型
    type Angle;

    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 長軸の半径を取得
    fn major_radius(&self) -> T;

    /// 短軸の半径を取得
    fn minor_radius(&self) -> T;

    /// 回転角度を取得（長軸のX軸からの角度）
    fn rotation(&self) -> Self::Angle;

    /// 開始角度を取得（楕円のパラメータ角度）
    fn start_angle(&self) -> Self::Angle;

    /// 終了角度を取得（楕円のパラメータ角度）
    fn end_angle(&self) -> Self::Angle;

    /// 完全な楕円かどうかを判定
    fn is_full_ellipse(&self) -> bool;

    /// 指定パラメータ角度での点を取得
    fn point_at_angle(&self, angle: Self::Angle) -> Self::Point;
}

/// 楕円弧メトリクス Foundation トレイト
///
/// 楕円弧の計測機能を統一インターフェースで提供
pub trait EllipseArcMetrics<T: Scalar>: EllipseArcCore<T> {
    /// 楕円弧の長さを取得（楕円積分による近似計算）
    fn arc_length(&self) -> T;

    /// 楕円扇形の面積を取得
    fn sector_area(&self) -> T;

    /// 角度スパンを取得
    fn angle_span(&self) -> Self::Angle;

    /// 中点角度を取得
    fn mid_angle(&self) -> Self::Angle;

    /// 中点を取得
    fn mid_point(&self) -> Self::Point;

    /// 離心率を計算
    fn eccentricity(&self) -> T;

    /// 楕円が円に近いかを判定
    fn is_nearly_circular(&self, tolerance: T) -> bool;
}

/// 統一EllipseArc Foundation トレイト
///
/// 全ての Foundation システムを統合する統一インターフェース
/// Core + Metrics + Transform + Collision + Intersection
pub trait UnifiedEllipseArcFoundation<T: Scalar>: EllipseArcCore<T> + EllipseArcMetrics<T> {
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
// 3D Ellipse Arc Core Foundation
// ============================================================================

/// 3D楕円円弧Core Foundation トレイト
pub trait EllipseArc3DCore<T: Scalar>: EllipseArcCore<T> {
    /// ベクトル型
    type Vector;

    /// 楕円円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 楕円円弧の平面上でのU軸ベクトルを取得（長軸方向）
    fn u_axis(&self) -> Self::Vector;

    /// 楕円円弧の平面上でのV軸ベクトルを取得（短軸方向）
    fn v_axis(&self) -> Self::Vector;
}
