//! 点の基本トレイト
//!
//! 点の基本的な属性アクセスとデータ構造に直接関連する計算

use crate::Scalar;
use super::foundation::GeometryFoundation;

// =============================================================================
// 点 (Point)
// =============================================================================

/// 点の基本トレイト
pub trait PointCore<T: Scalar>: GeometryFoundation<T> {
    /// 座標成分を取得
    fn coordinates(&self) -> Vec<T>;
    
    /// 次元数を取得
    fn dimension(&self) -> usize;
    
    /// 原点からの距離を取得
    fn distance_from_origin(&self) -> T {
        let coords = self.coordinates();
        let sum_of_squares: T = coords.iter()
            .map(|&c| c * c)
            .fold(T::ZERO, |acc, x| acc + x);
        sum_of_squares.sqrt()
    }
}

/// 2D点の基本トレイト
pub trait Point2DCore<T: Scalar>: PointCore<T> {
    /// X座標を取得
    fn x(&self) -> T;
    
    /// Y座標を取得  
    fn y(&self) -> T;
    
    /// 極座標での半径を取得
    fn polar_radius(&self) -> T {
        (self.x() * self.x() + self.y() * self.y()).sqrt()
    }
    
    /// 極座標での角度を取得（ラジアン）
    fn polar_angle(&self) -> T {
        self.y().atan2(self.x())
    }
}

/// 3D点の基本トレイト
pub trait Point3DCore<T: Scalar>: Point2DCore<T> {
    /// Z座標を取得
    fn z(&self) -> T;
    
    /// 球座標での半径を取得
    fn spherical_radius(&self) -> T {
        (self.x() * self.x() + self.y() * self.y() + self.z() * self.z()).sqrt()
    }
    
    /// 球座標での極角を取得（ラジアン、0 <= theta <= π）
    fn spherical_theta(&self) -> T {
        let r = self.spherical_radius();
        if r == T::ZERO {
            T::ZERO
        } else {
            (self.z() / r).acos()
        }
    }
    
    /// 球座標での方位角を取得（ラジアン、-π <= phi <= π）
    fn spherical_phi(&self) -> T {
        self.y().atan2(self.x())
    }
}