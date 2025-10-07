//! analysis クレートのユニットテスト
//!
//! このモジュールは、analysisクレートの各機能のテストコードを含む。
//! 元々は各ソースファイル内にあった #[cfg(test)] ブロックを
//! 独立したテストファイルに分離している。
pub mod sampling;
pub mod numerical;
pub mod statistics;
pub mod interpolation;