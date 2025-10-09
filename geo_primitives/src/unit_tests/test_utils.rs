//! Test utilities
//! テスト共通のユーティリティとエイリアス定義
//!
//! このモジュールは実装コードとテストコードの分離を支援し、
//! テストでのみ必要な型インポートを提供します。

// テストユーティリティでは全ての型を提供するため、未使用要素警告を抑制
#![allow(unused_imports)]
#![allow(dead_code)]

// 2D 幾何要素のテスト用エイリアス
pub use crate::geometry2d::{
    // Arc, BBox2D, Circle, Direction2D, Ellipse, EllipseArc, InfiniteLine2D, Point2D, Point2DF32,  // 一時的にコメントアウト（Direction2D整理中）
    BBox2D,
    Point2D,
    Point2DF32,
    Point2DF64,
    Vector,
    Vector2D,
};

// 3D 幾何要素のテスト用エイリアス（将来の拡張用）
pub use crate::geometry3d::{Point3DF64, Vector3D};

// Surface要素のテスト用エイリアス
// pub use crate::surface::{SphereF32, SphereF64};  // 一時的にコメントアウト

// 型エイリアス（テスト用）

// テスト用の便利な型エイリアス
pub type TestPoint2D = Point2D<f64>;
pub type TestPoint2DF32 = Point2D<f32>;
pub type TestPoint3D = Point3DF64;
pub type TestVector2D = Vector2D; // Vector2D = Vector<f64> のエイリアス
pub type TestVector2DF32 = Vector<f32>;
pub type TestVector3D = Vector3D;
// pub type TestCircle = Circle<f64>;  // 一時的にコメントアウト（Circle依存のため）
// pub type TestCircleF32 = Circle<f32>;  // 一時的にコメントアウト（Circle依存のため）
// pub type TestSphereF64 = SphereF64;  // 一時的にコメントアウト
// pub type TestSphereF32 = SphereF32;  // 一時的にコメントアウト

// テスト用定数
pub const TEST_TOLERANCE: f64 = 1e-10;
pub const TEST_TOLERANCE_F32: f32 = 1e-6;

// テスト用ヘルパー関数
pub mod helpers {
    use super::*;
    use std::f64::consts::PI;

    /// テスト用の基本的な点を作成
    pub fn test_points_2d() -> Vec<TestPoint2D> {
        vec![
            TestPoint2D::origin(),
            TestPoint2D::new(1.0, 0.0),
            TestPoint2D::new(0.0, 1.0),
            TestPoint2D::new(1.0, 1.0),
            TestPoint2D::new(-1.0, -1.0),
        ]
    }

    /// テスト用の基本的な円を作成
    // pub fn test_circles() -> Vec<TestCircle> {  // 一時的にコメントアウト（Circle依存のため）
    //     vec![
    //         TestCircle::unit_circle(),
    //         TestCircle::new(TestPoint2D::origin(), 2.0),
    //         TestCircle::new(TestPoint2D::new(1.0, 1.0), 1.5),
    //     ]
    // }

    /// 角度の配列（0, π/4, π/2, π, 3π/2, 2π）
    pub fn test_angles() -> Vec<f64> {
        vec![0.0, PI / 4.0, PI / 2.0, PI, 3.0 * PI / 2.0, 2.0 * PI]
    }

    /// 浮動小数点の近似比較（f64）- デフォルトのtolerance使用
    pub fn approx_eq_f64(a: f64, b: f64) -> bool {
        (a - b).abs() < TEST_TOLERANCE
    }

    /// 浮動小数点の近似比較（f64）- tolerance指定版
    pub fn approx_eq_f64_with_tolerance(a: f64, b: f64, tolerance: f64) -> bool {
        (a - b).abs() < tolerance
    }

    /// 浮動小数点の近似比較（f32）
    pub fn approx_eq_f32(a: f32, b: f32, tolerance: f32) -> bool {
        (a - b).abs() < tolerance
    }

    /// 点の近似比較
    pub fn approx_eq_point2d(a: &TestPoint2D, b: &TestPoint2D) -> bool {
        approx_eq_f64_with_tolerance(a.x(), b.x(), TEST_TOLERANCE)
            && approx_eq_f64_with_tolerance(a.y(), b.y(), TEST_TOLERANCE)
    }

    /// ベクトルの近似比較
    pub fn approx_eq_vector2d(a: &TestVector2D, b: &TestVector2D) -> bool {
        approx_eq_f64_with_tolerance(a.x(), b.x(), TEST_TOLERANCE)
            && approx_eq_f64_with_tolerance(a.y(), b.y(), TEST_TOLERANCE)
    }
}

// テスト用マクロ（オプション）
#[macro_export]
macro_rules! assert_approx_eq {
    ($a:expr, $b:expr) => {
        assert!(
            crate::unit_tests::test_utils::helpers::approx_eq_f64($a, $b),
            "Expected approximately equal: {} ≈ {}",
            $a,
            $b
        );
    };
    ($a:expr, $b:expr, $tolerance:expr) => {
        assert!(
            ($a - $b).abs() < $tolerance,
            "Expected approximately equal within {}: {} ≈ {}",
            $tolerance,
            $a,
            $b
        );
    };
}

#[macro_export]
macro_rules! assert_point_approx_eq {
    ($a:expr, $b:expr) => {
        assert!(
            crate::unit_tests::test_utils::helpers::approx_eq_point2d($a, $b),
            "Expected approximately equal points: {:?} ≈ {:?}",
            $a,
            $b
        );
    };
}
