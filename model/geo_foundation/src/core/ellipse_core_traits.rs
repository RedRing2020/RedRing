//! Ellipse Core Traits - Ellipse形状のCore機能統合
//!
//! Foundation Pattern Phase 1 実装
//! 3-5-4 パターン: Constructor(3) + Properties(5) + Measure(4)
//!
//! 作成日: 2025年11月28日

use crate::Scalar;

// ============================================================================
// Ellipse2D Core Traits
// ============================================================================

/// Ellipse2D Constructor トレイト（3メソッド）
pub trait Ellipse2DConstructor<T: Scalar>: Sized {
    /// 中心、長軸、短軸、回転角から楕円を作成
    fn new(center: (T, T), semi_major: T, semi_minor: T, rotation: T) -> Option<Self>;

    /// 単位楕円を作成（原点中心、a=1, b=1、回転なし）
    fn unit_ellipse() -> Self;

    /// 軸に平行な楕円を作成（回転なし）
    fn axis_aligned(center: (T, T), semi_major: T, semi_minor: T) -> Option<Self>;
}

/// Ellipse2D Properties トレイト（5メソッド）
pub trait Ellipse2DProperties<T: Scalar> {
    /// 中心座標を取得
    fn center(&self) -> (T, T);

    /// 長半軸の長さを取得
    fn semi_major_axis(&self) -> T;

    /// 短半軸の長さを取得
    fn semi_minor_axis(&self) -> T;

    /// 回転角を取得（ラジアン）
    fn rotation(&self) -> T;

    /// 離心率を取得
    fn eccentricity(&self) -> T;
}

/// Ellipse2D Measure トレイト（4メソッド）
pub trait Ellipse2DMeasure<T: Scalar> {
    /// 楕円の面積（測度）を計算
    fn measure(&self) -> T;

    /// 楕円の周長を計算（近似）
    fn perimeter(&self) -> T;

    /// 点が楕円内部にあるか判定
    fn contains_point(&self, point: (T, T)) -> bool;

    /// 楕円が円かどうか判定
    fn is_circle(&self) -> bool;
}

/// Ellipse2D Core トレイト（統合インターフェース）
pub trait Ellipse2DCore<T: Scalar>:
    Ellipse2DConstructor<T> + Ellipse2DProperties<T> + Ellipse2DMeasure<T>
{
}

// Note: Ellipse3D は Ellipse2D 実装完了後に Phase 2 で追加予定
