//! 純粋数学的幾何図形
//!
//! geo_primitivesのCAD用図形とは区別される、数学計算専用の図形定義

pub mod plane3;

#[cfg(test)]
pub mod plane3_tests;

pub use plane3::Plane3;
