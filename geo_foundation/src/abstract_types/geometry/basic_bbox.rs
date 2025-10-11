//! 境界ボックス（BBox）の基本トレイト
//!
//! BoundingBoxから統一してBBoxに変更

use super::foundation::{BasicContainment, BasicMetrics, GeometryFoundation};
use crate::Scalar;

// =============================================================================
// 境界ボックス (BBox) - BoundingBoxから統一
// =============================================================================

/// 境界ボックスの基本トレイト（BBoxに統一）
pub trait BBoxCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T>
{
    /// 最小点を取得
    fn min_point(&self) -> Self::Point;

    /// 最大点を取得
    fn max_point(&self) -> Self::Point;

    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 境界ボックスが有効かを判定（min <= max）
    fn is_valid(&self) -> bool;

    /// 対角線の長さを取得
    fn diagonal_length(&self) -> T;

    /// 境界ボックスのサイズ（各次元の幅）を取得
    fn size(&self) -> Self::Vector;

    /// 境界ボックスを拡張（マージンを追加）
    fn expand(&self, margin: T) -> Self;

    /// 他の境界ボックスと結合（最小を包含する境界ボックス）
    fn union(&self, other: &Self) -> Self;

    /// 他の境界ボックスとの交差を取得
    fn intersection(&self, other: &Self) -> Option<Self>
    where
        Self: Sized;

    /// 他の境界ボックスと重複するかを判定
    fn intersects(&self, other: &Self) -> bool;

    /// 指定点が境界ボックス内にあるかを判定
    fn contains_point(&self, point: &Self::Point) -> bool;

    /// 他の境界ボックスを完全に含むかを判定
    fn contains_bbox(&self, other: &Self) -> bool;
}

/// BBoxCoreに対するBasicMetricsのデフォルト実装
impl<T: Scalar, B: BBoxCore<T>> BasicMetrics<T> for B {
    fn area(&self) -> Option<T> {
        // 2D/3Dで異なる実装が必要なため、Noneを返す
        // 具体的な実装はBBox2DCore, BBox3DCoreで提供
        None
    }

    fn perimeter(&self) -> Option<T> {
        // 2D/3Dで異なる実装が必要なため、Noneを返す
        None
    }

    fn length(&self) -> Option<T> {
        // 境界ボックスの対角線の長さを返す
        Some(self.diagonal_length())
    }
}

/// 2D境界ボックスの基本トレイト（BBoxに統一）
pub trait BBox2DCore<T: Scalar>: BBoxCore<T> {
    /// 幅を取得（X軸方向のサイズ）
    fn width(&self) -> T;

    /// 高さを取得（Y軸方向のサイズ）
    fn height(&self) -> T;

    /// アスペクト比を取得（幅/高さ）
    fn aspect_ratio(&self) -> T {
        let height = self.height();
        if height != T::zero() {
            self.width() / height
        } else {
            T::INFINITY
        }
    }

    /// 正方形かどうかを判定
    fn is_square(&self) -> bool {
        (self.width() - self.height()).abs() < T::EPSILON
    }
}

/// BBox2DCoreに対するBasicMetricsの具体実装
impl<T: Scalar, B: BBox2DCore<T>> BasicMetrics<T> for B {
    fn area(&self) -> Option<T> {
        Some(self.width() * self.height())
    }

    fn perimeter(&self) -> Option<T> {
        Some(T::from(2.0).unwrap() * (self.width() + self.height()))
    }

    fn length(&self) -> Option<T> {
        // 対角線の長さ
        Some(self.diagonal_length())
    }
}

/// 3D境界ボックスの基本トレイト（BBoxに統一）
pub trait BBox3DCore<T: Scalar>: BBox2DCore<T> {
    /// 奥行きを取得（Z軸方向のサイズ）
    fn depth(&self) -> T;

    /// 体積を取得
    fn volume(&self) -> T {
        self.width() * self.height() * self.depth()
    }

    /// 表面積を取得
    fn surface_area(&self) -> T {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        T::from(2.0).unwrap() * (w * h + w * d + h * d)
    }

    /// 立方体かどうかを判定
    fn is_cube(&self) -> bool {
        let w = self.width();
        let h = self.height();
        let d = self.depth();
        (w - h).abs() < T::EPSILON && (h - d).abs() < T::EPSILON
    }
}

/// BBox3DCoreに対するBasicMetricsの具体実装
impl<T: Scalar, B: BBox3DCore<T>> BasicMetrics<T> for B {
    fn area(&self) -> Option<T> {
        // 3Dの場合、表面積を返す
        Some(self.surface_area())
    }

    fn perimeter(&self) -> Option<T> {
        // 3Dの場合、エッジの総長を返す（12本のエッジ）
        Some(T::from(4.0).unwrap() * (self.width() + self.height() + self.depth()))
    }

    fn length(&self) -> Option<T> {
        // 対角線の長さ
        Some(self.diagonal_length())
    }
}
