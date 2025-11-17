//! geo_core - 幾何計算基盤クレート
//!
//! Foundation パターンにおける高度な幾何演算を提供します。
//! 交差判定、衝突検出、複数プリミティブ間の演算など。
//!
//! ## 主要機能
//! - **intersections**: 交差判定・交線計算
//! - **collisions**: 衝突検出・距離計算
//! - **operations**: 複数プリミティブ間の演算
//!
//! ## Foundation パターンでの役割
//! ```text
//! geo_algorithms → geo_core (高度な幾何演算)
//! ```
//!
//! ---
//! © RedRing Project

// 基本的な共通計算機能はgeo_commonsに移動
// 高度な幾何演算のみここで実装予定

// テストモジュール
#[cfg(test)]
mod unit_tests;
