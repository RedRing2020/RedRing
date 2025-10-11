//! 円の基本トレイト
//!
//! 円の基本的な属性アクセスとデータ構造に直接関連する計算

use crate::Scalar;
use analysis::abstract_types::Angle;
use super::foundation::{GeometryFoundation, BasicMetrics, BasicContainment, BasicParametric};

// =============================================================================
// 円 (Circle)
// =============================================================================

/// 円の基本トレイト
pub trait CircleCore<T: Scalar>: 
    GeometryFoundation<T> + 
    BasicMetrics<T> + 
    BasicContainment<T> + 
    BasicParametric<T> 
{
    /// 中心点を取得
    fn center(&self) -> Self::Point;
    
    /// 半径を取得
    fn radius(&self) -> T;
    
    /// 直径を取得
    fn diameter(&self) -> T {
        self.radius() + self.radius() // 2 * radius
    }
    
    /// 指定角度での点を取得（型安全なAngle使用）
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;
    
    /// 指定ラジアン値での点を取得（互換性のため）
    fn point_at_radians(&self, radians: T) -> Self::Point {
        self.point_at_angle(Angle::from_radians(radians))
    }
}

/// CircleCoreに対するBasicParametricのデフォルト実装
impl<T: Scalar, C: CircleCore<T>> BasicParametric<T> for C {
    fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU)
    }
    
    fn point_at_parameter(&self, t: T) -> Self::Point {
        // パラメータtを角度として解釈
        self.point_at_radians(t)
    }
    
    fn tangent_at_parameter(&self, _t: T) -> Self::Vector {
        // 円の接線ベクトル（具体的な実装は実装側で定義）
        todo!("具体的な Vector 型に合わせて実装が必要")
    }
}

/// 3D円の基本トレイト（平面円）
pub trait Circle3DCore<T: Scalar>: CircleCore<T> {
    /// 円が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;
    
    /// 円の平面上でのU軸ベクトルを取得
    fn u_axis(&self) -> Self::Vector;
    
    /// 円の平面上でのV軸ベクトルを取得  
    fn v_axis(&self) -> Self::Vector;
    
    /// 局所座標系での指定角度の点を取得
    fn point_at_angle_local(&self, angle: Angle<T>) -> Self::Point;
}