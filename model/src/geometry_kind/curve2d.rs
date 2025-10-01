//! CurveKind2D: 2次元曲線の分類
//!
//! `geometry::geometry2d` 配下の構造体に対応する分類を網羅的に定義。
//! 実装との整合性、語義的明快さ、将来的な拡張性を考慮。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveKind2D {
    /// 無限直線（CAD的表現）
    InfiniteLine,
    /// 半直線（方向付き）
    Ray,
    /// 有限直線
    Line,
    /// 円
    Circle,
    /// 円弧
    Arc,
    /// 楕円
    Ellipse,
    /// 楕円弧
    EllipseArc,
    /// NURBS曲線
    NurbsCurve,
    /// 未分類・不明な曲線
    Unknown,
}
