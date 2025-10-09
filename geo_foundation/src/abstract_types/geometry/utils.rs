//! 幾何計算用ユーティリティ関数
//!
//! ジェネリックな型パラメータを使用した抽象的な幾何計算関数を提供

use crate::abstract_types::scalar::Scalar;

/// 2つのScalar値の距離（ジェネリック版）
pub fn scalar_distance<T: Scalar>(a: T, b: T) -> T {
    (a - b).abs()
}

/// 2つのf64値の最小値
pub fn f64_min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
    }
}

/// 2つのf64値の最大値
pub fn f64_max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}

/// ジェネリックScalar値の最小値
pub fn scalar_min<T: Scalar>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

/// ジェネリックScalar値の最大値
pub fn scalar_max<T: Scalar>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

/// 線形補間（ジェネリック版）
pub fn lerp<T: Scalar>(start: T, end: T, t: T) -> T {
    start + (end - start) * t
}

/// 指定範囲での値のクランプ（ジェネリック版）
pub fn clamp<T: Scalar>(value: T, min: T, max: T) -> T {
    scalar_max(min, scalar_min(max, value))
}

/// 値が指定範囲内にあるかチェック（ジェネリック版）
pub fn in_range<T: Scalar>(value: T, min: T, max: T) -> bool {
    value >= min && value <= max
}
