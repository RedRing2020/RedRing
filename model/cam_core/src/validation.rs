//! CAM用検証機能
//!
//! このモジュールは、CAM処理における幾何データの検証を提供します。
//!
//! # 概要
//!
//! Phase 1では以下の検証機能を実装します：
//!
//! - **2D輪郭の閉曲線判定**: 始点と終点の距離がトレランス以内か検証
//!
//! # 例
//!
//! ```
//! use cam_core::{validate_2d_contour, CamTolerance};
//! use geo_primitives::Point2D;
//!
//! let points = vec![
//!     Point2D::new(0.0, 0.0),
//!     Point2D::new(10.0, 0.0),
//!     Point2D::new(10.0, 10.0),
//!     Point2D::new(0.0, 10.0),
//!     Point2D::new(0.0, 0.0001),  // ほぼ閉じている
//! ];
//!
//! let tolerance = CamTolerance::default();
//! let result = validate_2d_contour(&points, &tolerance);
//! assert!(result.is_ok());
//! ```

use crate::tolerance::CamTolerance;
use analysis::Scalar;
use geo_primitives::Point2D;

/// 検証エラー
///
/// CAM検証で発生する可能性のあるエラーを定義します。
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// 2D輪郭が閉じていない
    ///
    /// 始点と終点の距離がトレランスを超えています。
    ContourNotClosed {
        /// 始点と終点の距離
        distance: f64,
        /// 許容トレランス
        tolerance: f64,
    },

    /// 2D輪郭の点数が不足
    ///
    /// 閉曲線を構成するには最低3点必要です。
    InsufficientPoints {
        /// 実際の点数
        point_count: usize,
    },

    /// 2D輪郭が空
    EmptyContour,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::ContourNotClosed {
                distance,
                tolerance,
            } => write!(
                f,
                "Contour is not closed: distance={:.6}, tolerance={:.6}",
                distance, tolerance
            ),
            ValidationError::InsufficientPoints { point_count } => write!(
                f,
                "Insufficient points for closed contour: {} points (minimum 3 required)",
                point_count
            ),
            ValidationError::EmptyContour => write!(f, "Contour is empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

/// 2D輪郭が閉曲線かどうか検証
///
/// # 引数
///
/// - `points`: 2D点列
/// - `tolerance`: トレランス設定
///
/// # 戻り値
///
/// - `Ok(())`: 輪郭が閉じている
/// - `Err(ValidationError)`: 閉じていない、または点数不足
///
/// # エラー
///
/// - `ValidationError::EmptyContour`: 点列が空
/// - `ValidationError::InsufficientPoints`: 点数が3未満
/// - `ValidationError::ContourNotClosed`: 始点と終点の距離がトレランス超過
///
/// # 例
///
/// ```
/// use cam_core::{validate_2d_contour, CamTolerance};
/// use geo_primitives::Point2D;
///
/// let points = vec![
///     Point2D::new(0.0, 0.0),
///     Point2D::new(10.0, 0.0),
///     Point2D::new(10.0, 10.0),
///     Point2D::new(0.0, 0.0),
/// ];
///
/// let tolerance = CamTolerance::default();
/// assert!(validate_2d_contour(&points, &tolerance).is_ok());
/// ```
pub fn validate_2d_contour<T: Scalar>(
    points: &[Point2D<T>],
    tolerance: &CamTolerance<T>,
) -> Result<(), ValidationError> {
    // 空チェック
    if points.is_empty() {
        return Err(ValidationError::EmptyContour);
    }

    // 点数チェック
    if points.len() < 3 {
        return Err(ValidationError::InsufficientPoints {
            point_count: points.len(),
        });
    }

    // 始点と終点の距離計算
    let start = &points[0];
    let end = &points[points.len() - 1];

    let dx = end.x() - start.x();
    let dy = end.y() - start.y();
    let distance = (dx * dx + dy * dy).sqrt();

    // トレランス比較
    if distance > tolerance.closure_tolerance {
        return Err(ValidationError::ContourNotClosed {
            distance: distance.to_f64(),
            tolerance: tolerance.closure_tolerance.to_f64(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerance::CAM_DEFAULT_CLOSURE_TOLERANCE_F64;

    #[test]
    fn test_validate_closed_contour() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
            Point2D::new(0.0, 0.0),
        ];

        let tolerance = CamTolerance::default();
        assert!(validate_2d_contour(&points, &tolerance).is_ok());
    }

    #[test]
    fn test_validate_almost_closed_contour() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
            Point2D::new(0.0, 0.0001), // 0.0001mm の誤差（デフォルトトレランス 0.001mm 以内）
        ];

        let tolerance = CamTolerance::default();
        assert!(validate_2d_contour(&points, &tolerance).is_ok());
    }

    #[test]
    fn test_validate_not_closed_contour() {
        let points = vec![
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(10.0, 10.0),
            Point2D::new(0.0, 10.0),
            Point2D::new(0.0, 1.0), // 1mm の誤差（トレランス 0.001mm を超過）
        ];

        let tolerance = CamTolerance::default();
        let result = validate_2d_contour(&points, &tolerance);
        assert!(result.is_err());

        if let Err(ValidationError::ContourNotClosed {
            distance,
            tolerance: tol,
        }) = result
        {
            assert_eq!(distance, 1.0);
            assert_eq!(tol, CAM_DEFAULT_CLOSURE_TOLERANCE_F64);
        } else {
            panic!("Expected ContourNotClosed error");
        }
    }

    #[test]
    fn test_validate_empty_contour() {
        let points: Vec<Point2D<f64>> = vec![];
        let tolerance = CamTolerance::default();
        let result = validate_2d_contour(&points, &tolerance);

        assert!(result.is_err());
        assert!(matches!(result, Err(ValidationError::EmptyContour)));
    }

    #[test]
    fn test_validate_insufficient_points() {
        let points = vec![Point2D::new(0.0, 0.0), Point2D::new(10.0, 0.0)];

        let tolerance = CamTolerance::default();
        let result = validate_2d_contour(&points, &tolerance);

        assert!(result.is_err());
        if let Err(ValidationError::InsufficientPoints { point_count }) = result {
            assert_eq!(point_count, 2);
        } else {
            panic!("Expected InsufficientPoints error");
        }
    }
}
