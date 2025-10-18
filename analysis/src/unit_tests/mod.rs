//! analysis クレートのユニットテスト
//!
//! このモジュールは、analysisクレートの各機能のテストコードを含む。
//! 元々は各ソースファイル内にあった #[cfg(test)] ブロックを
//! 独立したテストファイルに分離している。
pub mod interpolation_additional_tests;
pub mod interpolation_tests;
pub mod linalg;
pub mod numerical_additional_tests;
pub mod numerical_tests;
pub mod sampling_additional_tests;
pub mod sampling_tests;
pub mod scalar_tests;
pub mod statistics_tests;
