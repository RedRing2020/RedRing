//! CAM機能の基盤クレート
//!
//! このクレートは、CAM（Computer-Aided Manufacturing）機能の中核となる
//! データ構造とアルゴリズムを提供します。
//!
//! # 主要モジュール
//!
//! - [`tolerance`][]: CAM用トレランス管理
//! - [`tool`][]: 工具定義（工具径、長さ、種別）
//! - [`toolset`][]: ツールセット定義（工具＋ホルダー、干渉距離）
//! - [`toolpath`][]: 工具経路データ構造（セグメント、経路全体）
//! - [`validation`][]: CAM用検証機能（2D輪郭閉判定、トレランスチェック）
//!
//! # 設計方針
//!
//! - **CAD/CAM境界の明確化**: 形状演算は `geo_algorithms`、工具経路生成は `cam_core`
//! - **3軸加工を前提**: Phase 1では3軸加工（2D CAM/3D CAM）のみサポート
//! - **トレランス管理**: 加工精度要件に基づくトレランス設定
//! - **将来拡張**: `cam_algorithms`, `cam_sim` 等が依存する基盤として設計
//!
//! # 依存関係
//!
//! ```text
//! cam_core
//!   ↓
//! geo_algorithms (オフセット、閉曲線判定)
//!   ↓
//! geo_primitives (幾何プリミティブ)
//!   ↓
//! geo_foundation (Foundation Pattern)
//!   ↓
//! analysis (数値計算基盤)
//! ```

pub mod tolerance;
pub mod tool;
pub mod toolset;
pub mod toolpath;
pub mod validation;

// 主要型の再エクスポート
pub use tolerance::CamTolerance;
pub use tool::{Tool, ToolType};
pub use toolset::{
    Holder, HolderInterferenceOffset, HolderSegment, HolderSegmentKind, ToolSet,
    ToolSetReferencePoint,
};
pub use toolpath::{
    ArcDirection, ContourLevelPath, CuttingDirection, PathGeometry, PathSegment, SegmentType,
    ToolPath,
};
pub use validation::{ValidationError, validate_2d_contour};
