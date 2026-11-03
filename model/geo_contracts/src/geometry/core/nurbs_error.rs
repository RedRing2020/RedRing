//! NURBS 形状の生成の失敗
//!
//! NURBS 曲線・曲面の Constructor の trait定義が返すエラー。ファイルの読み込み等で作る大きなデータのため、
//! どの入力が不正かを呼び出し元に返す。

use std::fmt;

/// NURBS 形状の生成の失敗
#[derive(Debug, Clone, PartialEq)]
pub enum NurbsError {
    /// 制御点の数が次数 + 1 に満たない
    InsufficientControlPoints {
        actual: usize,
        required: usize,
        degree: usize,
    },
    /// 重みの数が制御点の数と一致しない
    WeightCountMismatch { actual: usize, expected: usize },
    /// 重みが正でない
    NonPositiveWeight { index: usize, weight: f64 },
    /// ノットの数が 制御点の数 + 次数 + 1 と一致しない
    KnotCountMismatch { actual: usize, expected: usize },
    /// ノットが減少している（`knots[index] < knots[index - 1]`）
    DecreasingKnot { index: usize },
    /// 始端のノットの重複度が 次数 + 1 に満たない
    InsufficientStartKnotMultiplicity { actual: usize, required: usize },
    /// 終端のノットの重複度が 次数 + 1 に満たない
    InsufficientEndKnotMultiplicity { actual: usize, required: usize },
    /// 曲面の制御点の格子が空
    EmptyControlGrid,
    /// 曲面の制御点の格子の行の長さが揃っていない
    ControlGridRowMismatch {
        row: usize,
        actual: usize,
        expected: usize,
    },
}

impl fmt::Display for NurbsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InsufficientControlPoints {
                actual,
                required,
                degree,
            } => write!(
                f,
                "Insufficient control points: {actual} (degree {degree} requires at least {required})"
            ),
            Self::WeightCountMismatch { actual, expected } => write!(
                f,
                "Weight count mismatch: {actual} (must match control point count {expected})"
            ),
            Self::NonPositiveWeight { index, weight } => {
                write!(f, "Weight {index} must be positive: {weight}")
            }
            Self::KnotCountMismatch { actual, expected } => {
                write!(f, "Knot count mismatch: {actual} (expected {expected})")
            }
            Self::DecreasingKnot { index } => {
                write!(f, "Knot {index} is smaller than the previous knot")
            }
            Self::InsufficientStartKnotMultiplicity { actual, required } => write!(
                f,
                "Start knot multiplicity is {actual} (at least {required} required)"
            ),
            Self::InsufficientEndKnotMultiplicity { actual, required } => write!(
                f,
                "End knot multiplicity is {actual} (at least {required} required)"
            ),
            Self::EmptyControlGrid => write!(f, "Control point grid is empty"),
            Self::ControlGridRowMismatch {
                row,
                actual,
                expected,
            } => write!(
                f,
                "Control point grid row {row} has {actual} points (expected {expected})"
            ),
        }
    }
}

impl std::error::Error for NurbsError {}
