//! 円弧（Arc）の基本トレイト
//!
//! 円弧は円の一部であり、CircleCoreを継承または包含する

use super::basic_circle::CircleCore;
use super::foundation::{BasicContainment, BasicMetrics, BasicParametric, GeometryFoundation};
use crate::Scalar;
use analysis::Angle;

// =============================================================================
// 円弧 (Arc)
// =============================================================================

/// 円弧の基本トレイト
///
/// 円弧は円の一部として定義され、CircleCoreの機能を含む
pub trait ArcCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T> + BasicParametric<T>
{
    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 半径を取得
    fn radius(&self) -> T;

    /// 開始角度を取得（ラジアン、型安全なAngle型使用）
    fn start_angle(&self) -> Angle<T>;

    /// 終了角度を取得（ラジアン、型安全なAngle型使用）
    fn end_angle(&self) -> Angle<T>;

    /// 角度の範囲を取得（終了角度 - 開始角度）
    fn angle_span(&self) -> Angle<T> {
        self.end_angle() - self.start_angle()
    }

    /// 完全な円かどうかを判定
    fn is_full_circle(&self) -> bool {
        self.angle_span() >= Angle::full_rotation()
    }

    /// 指定角度での点を取得（型安全なAngle型使用）
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;

    /// 指定された角度が弧の範囲内にあるかを判定
    fn contains_angle(&self, angle: Angle<T>) -> bool {
        let normalized_angle = angle.normalize();
        let start = self.start_angle().normalize();
        let end = self.end_angle().normalize();

        if start <= end {
            // 通常の範囲（例：30°から90°）
            normalized_angle >= start && normalized_angle <= end
        } else {
            // 0°をまたぐ範囲（例：300°から60°）
            normalized_angle >= start || normalized_angle <= end
        }
    }

    /// 円弧の中点角度を取得
    fn mid_angle(&self) -> Angle<T> {
        let span = self.angle_span();
        self.start_angle() + span / T::from(2.0).unwrap()
    }

    /// 円弧の中点を取得
    fn mid_point(&self) -> Self::Point {
        self.point_at_angle(self.mid_angle())
    }
}

/// ArcCoreに対するBasicMetricsのデフォルト実装
impl<T: Scalar, A: ArcCore<T>> BasicMetrics<T> for A {
    fn length(&self) -> Option<T> {
        // 円弧長 = 半径 × 角度（ラジアン）
        let angle_radians = self.angle_span().as_radians();
        Some(self.radius() * angle_radians)
    }

    fn area(&self) -> Option<T> {
        // 扇形の面積 = (1/2) × 半径² × 角度（ラジアン）
        let angle_radians = self.angle_span().as_radians();
        let radius = self.radius();
        Some(T::from(0.5).unwrap() * radius * radius * angle_radians)
    }

    fn perimeter(&self) -> Option<T> {
        // 扇形の周長 = 円弧長 + 2 × 半径
        if let Some(arc_length) = self.length() {
            Some(arc_length + T::from(2.0).unwrap() * self.radius())
        } else {
            None
        }
    }
}

/// 3D円弧の基本トレイト（平面円弧）
pub trait Arc3DCore<T: Scalar>: ArcCore<T> {
    /// 円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 円弧の平面上でのU軸ベクトルを取得（開始角度方向）
    fn u_axis(&self) -> Self::Vector;

    /// 円弧の平面上でのV軸ベクトルを取得（U軸に垂直）
    fn v_axis(&self) -> Self::Vector;

    /// 3D空間での開始点を取得
    fn start_point(&self) -> Self::Point {
        self.point_at_angle(self.start_angle())
    }

    /// 3D空間での終了点を取得
    fn end_point(&self) -> Self::Point {
        self.point_at_angle(self.end_angle())
    }
}

// =============================================================================
// 楕円弧 (EllipseArc)
// =============================================================================

/// 楕円円弧の基本トレイト
///
/// 楕円円弧は楕円の一部として定義される
pub trait EllipseArcCore<T: Scalar>:
    GeometryFoundation<T> + BasicMetrics<T> + BasicContainment<T> + BasicParametric<T>
{
    /// 中心点を取得
    fn center(&self) -> Self::Point;

    /// 長軸の半径を取得
    fn major_radius(&self) -> T;

    /// 短軸の半径を取得
    fn minor_radius(&self) -> T;

    /// 回転角度を取得（長軸のX軸からの角度）
    fn rotation(&self) -> Angle<T>;

    /// 開始角度を取得（楕円のパラメータ角度）
    fn start_angle(&self) -> Angle<T>;

    /// 終了角度を取得（楕円のパラメータ角度）
    fn end_angle(&self) -> Angle<T>;

    /// 角度の範囲を取得
    fn angle_span(&self) -> Angle<T> {
        self.end_angle() - self.start_angle()
    }

    /// 完全な楕円かどうかを判定
    fn is_full_ellipse(&self) -> bool {
        self.angle_span() >= Angle::full_rotation()
    }

    /// 指定パラメータ角度での点を取得
    fn point_at_angle(&self, angle: Angle<T>) -> Self::Point;

    /// 楕円円弧が円弧かどうかを判定（長軸と短軸が等しい）
    fn is_circular(&self) -> bool {
        (self.major_radius() - self.minor_radius()).abs() < T::EPSILON
    }
}

/// 3D楕円円弧の基本トレイト
pub trait EllipseArc3DCore<T: Scalar>: EllipseArcCore<T> {
    /// 楕円円弧が存在する平面の法線ベクトルを取得
    fn normal(&self) -> Self::Vector;

    /// 楕円円弧の平面上でのU軸ベクトルを取得（長軸方向）
    fn u_axis(&self) -> Self::Vector;

    /// 楕円円弧の平面上でのV軸ベクトルを取得（短軸方向）
    fn v_axis(&self) -> Self::Vector;
}
