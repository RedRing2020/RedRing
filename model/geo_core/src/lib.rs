//! geo_core - 幾何計算基盤クレート
//!
//! Foundation パターンにおけるブリッジ役として、geo_primitives への
//! アクセスを仲介し、計量計算・近似計算の機能を提供します。
//!
//! ## 主要機能
//! - **metrics**: 面積、体積、距離等の計量計算
//! - **approximations**: 楕円・曲線の近似計算
//! - **bridge**: Foundation パターン準拠のプリミティブアクセス
//!
//! ## Foundation パターンでの役割
//! ```text
//! geo_nurbs → geo_core → geo_primitives
//!           (ブリッジ)
//! ```
//!
//! ---
//! © RedRing Project

// 主要モジュール
pub mod approximations;
pub mod metrics;

// テストモジュール
#[cfg(test)]
mod unit_tests;
