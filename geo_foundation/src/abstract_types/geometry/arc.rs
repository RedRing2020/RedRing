//! Arc - 円弧の抽象化トレイト
//!
//! CAD/CAM システムで使用される円弧の抽象化インターフェース

use crate::abstract_types::{Angle, Scalar};
use std::fmt::Debug;

//! Arc - 円弧の抽象化トレイト
//!
//! CAD/CAM システムで使用される円弧の抽象化インターフェース

use crate::abstract_types::{Angle, Scalar};
use super::common::{AngularCurve, CenteredCurve, CurveContainment, CurveMetrics, CurvePoints, CurveTransformation, CurveTypes, RadialCurve};
use std::fmt::Debug;

/// 2D円弧の抽象化トレイト
///
/// 2次元平面上の円弧を表現する共通インターフェース
/// 基本操作は共通トレイトから継承
pub trait Arc2D<T: Scalar>:
    Debug +
    Clone +
    CurveTypes<T> +
    CurvePoints<T> +
    CurveMetrics<T> +
    CurveContainment<T> +
    CurveTransformation<T> +
    AngularCurve<T> +
    CenteredCurve<T> +
    RadialCurve<T>
{
    /// 円の型
    type Circle;

    /// 円弧の基底円を取得
    fn circle(&self) -> &Self::Circle;

    /// 2D固有の平行移動（x, y座標指定）
    fn translate_xy(&self, dx: T, dy: T) -> Self;

    /// 2D固有の回転（原点基準）
    fn rotate_2d(&self, angle: Angle<T>) -> Self;
}

/// 3D円弧の抽象化トレイト
///
/// 3次元空間上の円弧を表現する共通インターフェース
/// 基本操作は共通トレイトから継承
pub trait Arc3D<T: Scalar>:
    Debug +
    Clone +
    CurveTypes<T> +
    CurvePoints<T> +
    CurveMetrics<T> +
    CurveContainment<T> +
    CurveTransformation<T> +
    AngularCurve<T> +
    CenteredCurve<T> +
    RadialCurve<T>
{
    /// 円の型
    type Circle;

    /// 円弧の基底円を取得
    fn circle(&self) -> &Self::Circle;

    /// 3D固有の平行移動（x, y, z座標指定）
    fn translate_xyz(&self, dx: T, dy: T, dz: T) -> Self;

    /// 3D固有の軸を基準とした回転
    fn rotate_about_axis(&self, axis: &Self::Vector, angle: T) -> Self;

    /// 円弧が存在する平面の法線ベクトルを取得
    fn plane_normal(&self) -> Self::Vector;
}
