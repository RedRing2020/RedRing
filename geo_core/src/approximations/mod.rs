//! 幾何図形の近似計算モジュール
//!
//! このモジュールは各種幾何図形の数値近似計算を提供します。
//! 主に周長・弧長・面積などの解析的に計算困難な値の近似計算を担当します。

pub mod curves;
pub mod ellipse;

#[cfg(test)]
mod tests;

pub use curves::*;
pub use ellipse::*;
