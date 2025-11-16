//! 楕円特化数値計算モジュール（統合移行済み）
//!
//! **🚨 重要な変更通知**
//!
//! 楕円周長計算、楕円の幾何学的性質などの機能は、
//! `geo_core::approximations::ellipse` モジュールに統合されました。
//!
//! ## 移行理由
//! - 重複コードの削除
//! - Foundation パターンへの準拠  
//! - ジェネリック型対応（Scalar トレイト）
//! - アーキテクチャの整理
//!
//! ## 新しい使用方法
//! ```ignore
//! use geo_core::approximations::ellipse::*;
//! 
//! // f64 での使用例
//! let circumference = ellipse_circumference_ramanujan(10.0, 6.0);
//! let area = ellipse_area(10.0, 6.0);
//! ```
//!
//! **注意**: `analysis` クレートは純粋な数値計算を担当するため、
//! 幾何学特化機能は `geo_core` に移動しました。

// このファイルは後方互換性のために残されていますが、
// 実際の機能は geo_core に統合されています。
// 新しいコードでは geo_core を直接使用してください。
