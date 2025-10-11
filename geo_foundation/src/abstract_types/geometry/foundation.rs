//! 幾何形状の基盤トレイト
//!
//! 全ての幾何形状が実装すべき最小限の共通インターフェース

use crate::Scalar;

/// 幾何形状の基盤トレイト
///
/// データアクセス層：基本属性の取得のみ
pub trait GeometryFoundation<T: Scalar> {
    /// 点の型
    type Point;
    /// ベクトルの型
    type Vector;
    /// 境界ボックスの型（BBoxに統一）
    type BBox;

    /// 境界ボックスを取得（空間インデックス・レンダリングの基盤として基本機能）
    fn bounding_box(&self) -> Self::BBox;
}

/// 基本計量特性
///
/// データ構造に直接関連する基本計算のみ
pub trait BasicMetrics<T: Scalar> {
    /// 長さを取得（線分、円弧、曲線など）
    fn length(&self) -> Option<T> {
        None
    }

    /// 面積を取得（円、楕円、多角形など）
    fn area(&self) -> Option<T> {
        None
    }

    /// 体積を取得（球、円柱、多面体など）
    fn volume(&self) -> Option<T> {
        None
    }

    /// 周長を取得（閉じた図形）
    fn perimeter(&self) -> Option<T> {
        None
    }
}

/// 基本包含判定
///
/// 点の包含関係の基本判定のみ
pub trait BasicContainment<T: Scalar>: GeometryFoundation<T> {
    /// 点が図形内部に含まれるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 点が図形の境界上にあるかを判定（許容誤差付き）
    fn on_boundary(&self, point: &Self::Point, tolerance: T) -> bool;

    /// 点から図形への最短距離を計算
    fn distance_to_point(&self, point: &Self::Point) -> T;
}

/// 基本パラメータ化
///
/// パラメトリック形状の基本的なパラメータアクセス
pub trait BasicParametric<T: Scalar>: GeometryFoundation<T> {
    /// パラメータ範囲を取得
    fn parameter_range(&self) -> (T, T);

    /// 指定パラメータでの点を取得
    fn point_at_parameter(&self, t: T) -> Self::Point;

    /// 指定パラメータでの接線ベクトルを取得
    fn tangent_at_parameter(&self, t: T) -> Self::Vector;
}

/// 基本方向性
///
/// 方向を持つ幾何要素の基本インターフェース
pub trait BasicDirectional<T: Scalar>: GeometryFoundation<T> {
    /// 方向ベクトルの型
    type Direction;

    /// 主方向を取得
    fn direction(&self) -> Self::Direction;

    /// 方向を反転
    fn reverse_direction(&self) -> Self;
}
