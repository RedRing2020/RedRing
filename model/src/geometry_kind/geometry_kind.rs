//! GeometryKind: 幾何要素のトップレベル分類
//!
//! CurveKind2D, CurveKind3D, SurfaceKind などを包含する抽象分類。
//! geometry2d, geometry3d, surface モジュールとの整合性を重視。

use crate::geometry_kind::CurveKind2D;
use crate::geometry_kind::CurveKind3D;
use crate::geometry_kind::SurfaceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryKind {
    /// 2次元曲線
    Curve2D(CurveKind2D),
    /// 3次元曲線
    Curve3D(CurveKind3D),
    /// サーフェス
    Surface(SurfaceKind),
    /// 未分類・不明な幾何要素
    Unknown,
}
