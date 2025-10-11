//! 幾何トレイト用のヘルパー関数
//!
//! geo_foundationに**特化した**ヘルパー関数のみを提供
//!
//! ## 設計原則
//! - geo_foundation固有のロジックのみ
//! - analysis クレートへの依存は最小限
//! - 単一責任の原則を遵守

use crate::Scalar;

// =============================================================================
// パラメータ操作（geo_foundation固有）
// =============================================================================

/// パラメータ値の正規化（0.0 <= t <= 1.0）
///
/// 幾何形状のパラメトリック操作で使用
pub fn normalize_parameter<T: Scalar>(t: T) -> T {
    if t < T::ZERO {
        T::ZERO
    } else if t > T::ONE {
        T::ONE
    } else {
        t
    }
}

/// パラメータ値の範囲チェック
///
/// BasicParametric トレイトの実装で使用
pub fn parameter_in_range<T: Scalar>(t: T, min: T, max: T) -> bool {
    t >= min && t <= max
}

/// 線形補間
///
/// 幾何形状間の補間計算で使用
pub fn lerp<T: Scalar>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

/// 逆線形補間（パラメータ値を求める）
///
/// 点からパラメータ値への逆算で使用
pub fn inverse_lerp<T: Scalar>(a: T, b: T, value: T) -> Option<T> {
    let diff = b - a;
    if diff == T::ZERO {
        None
    } else {
        Some((value - a) / diff)
    }
}

// =============================================================================
// 幾何学的変換（geo_foundation固有）
// =============================================================================

/// 角度を正規化されたパラメータに変換
///
/// 円・円弧のパラメータ化で使用
pub fn angle_to_parameter<T: Scalar>(angle: T, start_angle: T, end_angle: T) -> T {
    let range = end_angle - start_angle;
    if range == T::ZERO {
        T::ZERO
    } else {
        (angle - start_angle) / range
    }
}

/// パラメータを角度に変換
///
/// 円・円弧のパラメータ逆変換で使用
pub fn parameter_to_angle<T: Scalar>(t: T, start_angle: T, end_angle: T) -> T {
    start_angle + t * (end_angle - start_angle)
}
