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
//! - [`machine_constraint`][]: 機械の最小制約定義（軸ラベル、回転軸旋回範囲）
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

pub mod artifact_binary;
pub mod machine_constraint;
pub mod tolerance;
pub mod tool;
pub mod toolpath;
pub mod toolset;
pub mod validation;

// 主要型の再エクスポート
pub use artifact_binary::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, BinaryFormatError, CompatibilityDecision,
    CoordinateFrame, ExtAttribute, FORMAT_VERSION_MAJOR_V1, FORMAT_VERSION_MINOR_V1,
    INTERFERENCE_MAGIC, InterferenceEvent, InterferenceKind, InterferencePayload,
    KNOWN_MINOR_VERSIONS_V0, LengthUnit, TOOLPATH_MAGIC, read_artifact_v1,
    read_interference_artifact_v1, read_interference_payload_v1, read_toolpath_artifact_v1,
    read_toolpath_payload_v1, write_interference_payload_v1, write_toolpath_payload_v1,
};
pub use machine_constraint::{
    MachineAxisLabel, MachineConstraint, RotaryAxisLabel, RotaryAxisLimit,
};
pub use tolerance::CamTolerance;
pub use tool::{Tool, ToolType};
pub use toolpath::{
    ArcDirection, ContourLevelPath, CuttingDirection, KinematicMode, MachineAxisKind,
    MachineAxisValue, MachineConfigurationClass, PathGeometry, PathSegment, PoseAnnotatedSegment,
    PoseDataPolicy, PoseInterpolationPolicy, SegmentType, TOOLPATH_SCHEMA_VERSION_V0_1, ToolPath,
    ToolPathKinematicMeta, ToolPose, ToolPoseSpan,
};
pub use toolset::{
    Holder, HolderInterferenceOffset, HolderSegment, HolderSegmentKind, ToolSet,
    ToolSetReferencePoint,
};
pub use validation::{ValidationError, validate_2d_contour};
