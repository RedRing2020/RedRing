//! SurfaceKind: 3次元曲面の分類
//!
//! `geometry::geometry3d::Surface` 配下の構造体に対応する分類を網羅的に定義。
//! 実装との整合性、語義的明快さ、将来的な拡張性を考慮。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    /// 未分類・不明な曲線
    Unknown,
}
