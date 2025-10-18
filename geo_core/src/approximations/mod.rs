//! 幾何図形の近似計算モジュール
//!
//! このモジュールは各種幾何図形の数値近似計算を提供します。
//! 主に周長・弧長・面積などの解析的に計算困難な値の近似計算を担当します。

pub mod ellipse;
pub mod curves;

#[cfg(test)]
mod tests;

pub use ellipse::*;
pub use curves::*;