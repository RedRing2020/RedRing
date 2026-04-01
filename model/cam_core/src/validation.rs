//! CAM用検証機能
//!
//! このモジュールは、CAM処理における幾何データの検証を提供します。
//!
//! # 概要
//!
//! Phase 1では以下の検証機能を実装します：
//!
//! - **2D輪郭の閉曲線判定**: 始点と終点の距離がトレランス以内か検証
//! - **機械制約チェック**: 送り速度と機械軸値が制約内か検証
//!
//! # 例
//!
//! ```
//! use cam_core::{validate_2d_contour, CamTolerance};
//! use geo_algorithms::Point2D;
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
use crate::{
    LinearAxisLabel, MachineAxisKind, MachineAxisValue, MachineConstraint, PathSegment,
    PoseAnnotatedSegment, RotaryAxisLabel, SegmentType, ToolPath,
};
use analysis::Scalar;
use geo_algorithms::{
    Point2D, validate_linear_acceleration_mm_per_s2, validate_linear_speed_mm_per_min,
    validate_linear_travel_mm, validate_rotary_acceleration_deg_per_s2, validate_rotary_angle_deg,
};

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

    /// セグメント送り速度が機械制約を超過
    FeedRateLimitExceeded {
        /// ToolPath内のセグメントインデックス
        segment_index: usize,
        /// 要求送り速度 [mm/min]
        feed_rate: f64,
        /// 上限 [mm/min]
        max_feed_rate: f64,
    },

    /// 線形加速度が機械制約を超過
    LinearAccelerationLimitExceeded {
        /// ToolPath内のセグメントインデックス
        segment_index: usize,
        /// 要求加速度 [mm/s^2]
        acceleration_mm_per_sec2: f64,
        /// 上限 [mm/s^2]
        max_acceleration_mm_per_sec2: f64,
    },

    /// 回転加速度が機械制約を超過
    RotaryAccelerationLimitExceeded {
        /// ToolPath内のセグメントインデックス
        segment_index: usize,
        /// 要求加速度 [deg/s^2]
        acceleration_deg_per_sec2: f64,
        /// 上限 [deg/s^2]
        max_acceleration_deg_per_sec2: f64,
    },

    /// 直動軸値が移動範囲を超過
    LinearAxisLimitExceeded {
        /// 姿勢セグメント配列内のインデックス
        segment_index: usize,
        /// どちらの姿勢か（start / end）
        pose_endpoint: &'static str,
        /// 軸名
        axis_name: String,
        /// 現在値 [mm]
        value_mm: f64,
        /// 最小値 [mm]
        min_mm: f64,
        /// 最大値 [mm]
        max_mm: f64,
    },

    /// 回転軸値が旋回範囲を超過
    RotaryAxisLimitExceeded {
        /// 姿勢セグメント配列内のインデックス
        segment_index: usize,
        /// どちらの姿勢か（start / end）
        pose_endpoint: &'static str,
        /// 軸名
        axis_name: String,
        /// 現在値 [deg]
        value_deg: f64,
        /// 最小値 [deg]
        min_deg: f64,
        /// 最大値 [deg]
        max_deg: f64,
    },

    /// サポート外の機械軸名
    UnsupportedMachineAxis {
        /// 姿勢セグメント配列内のインデックス
        segment_index: usize,
        /// どちらの姿勢か（start / end）
        pose_endpoint: &'static str,
        /// 軸名
        axis_name: String,
        /// 軸種別
        axis_kind: MachineAxisKind,
    },
}

impl ValidationError {
    /// ValidationError をロケール非依存の安定キーへ変換する。
    pub fn message_key(&self) -> &'static str {
        match self {
            Self::ContourNotClosed { .. } => "validation.contour.not_closed",
            Self::InsufficientPoints { .. } => "validation.contour.insufficient_points",
            Self::EmptyContour => "validation.contour.empty",
            Self::FeedRateLimitExceeded { .. } => "validation.machine.feed_rate_limit_exceeded",
            Self::LinearAccelerationLimitExceeded { .. } => {
                "validation.machine.linear_acceleration_limit_exceeded"
            }
            Self::RotaryAccelerationLimitExceeded { .. } => {
                "validation.machine.rotary_acceleration_limit_exceeded"
            }
            Self::LinearAxisLimitExceeded { .. } => "validation.machine.linear_axis_limit_exceeded",
            Self::RotaryAxisLimitExceeded { .. } => "validation.machine.rotary_axis_limit_exceeded",
            Self::UnsupportedMachineAxis { .. } => "validation.machine.unsupported_axis",
        }
    }
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
            ValidationError::FeedRateLimitExceeded {
                segment_index,
                feed_rate,
                max_feed_rate,
            } => write!(
                f,
                "Feed rate exceeds machine limit at segment {}: feed_rate={:.6}, max_feed_rate={:.6}",
                segment_index, feed_rate, max_feed_rate
            ),
            ValidationError::LinearAccelerationLimitExceeded {
                segment_index,
                acceleration_mm_per_sec2,
                max_acceleration_mm_per_sec2,
            } => write!(
                f,
                "Linear acceleration exceeds machine limit at segment {}: acceleration_mm_per_sec2={:.6}, max_acceleration_mm_per_sec2={:.6}",
                segment_index, acceleration_mm_per_sec2, max_acceleration_mm_per_sec2
            ),
            ValidationError::RotaryAccelerationLimitExceeded {
                segment_index,
                acceleration_deg_per_sec2,
                max_acceleration_deg_per_sec2,
            } => write!(
                f,
                "Rotary acceleration exceeds machine limit at segment {}: acceleration_deg_per_sec2={:.6}, max_acceleration_deg_per_sec2={:.6}",
                segment_index, acceleration_deg_per_sec2, max_acceleration_deg_per_sec2
            ),
            ValidationError::LinearAxisLimitExceeded {
                segment_index,
                pose_endpoint,
                axis_name,
                value_mm,
                min_mm,
                max_mm,
            } => write!(
                f,
                "Linear axis out of range at segment {} ({}) axis {}: value_mm={:.6}, range=[{:.6}, {:.6}]",
                segment_index, pose_endpoint, axis_name, value_mm, min_mm, max_mm
            ),
            ValidationError::RotaryAxisLimitExceeded {
                segment_index,
                pose_endpoint,
                axis_name,
                value_deg,
                min_deg,
                max_deg,
            } => write!(
                f,
                "Rotary axis out of range at segment {} ({}) axis {}: value_deg={:.6}, range=[{:.6}, {:.6}]",
                segment_index, pose_endpoint, axis_name, value_deg, min_deg, max_deg
            ),
            ValidationError::UnsupportedMachineAxis {
                segment_index,
                pose_endpoint,
                axis_name,
                axis_kind,
            } => write!(
                f,
                "Unsupported machine axis at segment {} ({}) axis {} ({:?})",
                segment_index, pose_endpoint, axis_name, axis_kind
            ),
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
/// use geo_algorithms::Point2D;
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

    let distance = start.distance_to(end);

    // トレランス比較
    if distance > tolerance.closure_tolerance {
        return Err(ValidationError::ContourNotClosed {
            distance: distance.to_f64(),
            tolerance: tolerance.closure_tolerance.to_f64(),
        });
    }

    Ok(())
}

/// ToolPathの送り速度が機械制約内かを検証
pub fn validate_toolpath_machine_constraints<T: Scalar + std::iter::Sum>(
    toolpath: &ToolPath<T>,
    machine_constraint: &MachineConstraint<T>,
    tolerance: &CamTolerance<T>,
) -> Result<(), ValidationError> {
    let Some(linear_speed_limit) = machine_constraint.linear_speed_limit else {
        return Ok(());
    };

    let max_feed = linear_speed_limit.limit_mm_per_min.to_f64();
    let tol = tolerance.machine_accuracy.to_f64();
    let linear_accel_limit = machine_constraint
        .linear_acceleration_limit
        .map(|limit| limit.limit_mm_per_sec2.to_f64());
    let rotary_accel_limit = machine_constraint
        .rotary_acceleration_limit
        .map(|limit| limit.limit_deg_per_sec2.to_f64());

    for (segment_index, segment) in collect_toolpath_segments(toolpath).into_iter().enumerate() {
        let Some(feed_rate) = segment_feed_rate(segment) else {
            continue;
        };

        let feed = feed_rate.to_f64();
        let result = validate_linear_speed_mm_per_min(feed, 0.0, max_feed, tol);
        if !result.is_valid() {
            return Err(ValidationError::FeedRateLimitExceeded {
                segment_index,
                feed_rate: feed,
                max_feed_rate: max_feed,
            });
        }

        if let (Some(max_linear_accel), Some(requested_linear_accel)) = (
            linear_accel_limit,
            segment.linear_acceleration_hint_mm_per_sec2(),
        ) {
            let result = validate_linear_acceleration_mm_per_s2(
                requested_linear_accel,
                0.0,
                max_linear_accel,
                tol,
            );
            if !result.is_valid() {
                return Err(ValidationError::LinearAccelerationLimitExceeded {
                    segment_index,
                    acceleration_mm_per_sec2: requested_linear_accel,
                    max_acceleration_mm_per_sec2: max_linear_accel,
                });
            }
        }

        if let (Some(max_rotary_accel), Some(requested_rotary_accel)) = (
            rotary_accel_limit,
            segment.rotary_acceleration_hint_deg_per_sec2(),
        ) {
            let result = validate_rotary_acceleration_deg_per_s2(
                requested_rotary_accel,
                0.0,
                max_rotary_accel,
                tol,
            );
            if !result.is_valid() {
                return Err(ValidationError::RotaryAccelerationLimitExceeded {
                    segment_index,
                    acceleration_deg_per_sec2: requested_rotary_accel,
                    max_acceleration_deg_per_sec2: max_rotary_accel,
                });
            }
        }
    }

    Ok(())
}

/// 姿勢付きセグメント列の機械軸値が制約内かを検証
pub fn validate_pose_segments_machine_constraints<T: Scalar>(
    segments: &[PoseAnnotatedSegment<T>],
    machine_constraint: &MachineConstraint<T>,
    tolerance: &CamTolerance<T>,
) -> Result<(), ValidationError> {
    let linear_tol_mm = tolerance.machine_accuracy.to_f64();
    let rotary_tol_deg = tolerance.angle_tolerance_deg.to_f64();

    for (segment_index, segment) in segments.iter().enumerate() {
        validate_axis_values(
            segment.pose_span.start_pose.machine_axes.as_deref(),
            segment_index,
            "start",
            machine_constraint,
            linear_tol_mm,
            rotary_tol_deg,
        )?;
        validate_axis_values(
            segment.pose_span.end_pose.machine_axes.as_deref(),
            segment_index,
            "end",
            machine_constraint,
            linear_tol_mm,
            rotary_tol_deg,
        )?;
    }

    Ok(())
}

fn collect_toolpath_segments<T: Scalar + std::iter::Sum>(
    toolpath: &ToolPath<T>,
) -> Vec<&PathSegment<T>> {
    let mut segments = Vec::new();
    segments.extend(toolpath.approach_segments.iter());
    for contour in &toolpath.contour_levels {
        segments.extend(contour.segments.iter());
    }
    segments.extend(toolpath.retract_segments.iter());
    segments
}

fn segment_feed_rate<T: Scalar>(segment: &PathSegment<T>) -> Option<T> {
    match segment.segment_type {
        SegmentType::Cutting { feed_rate }
        | SegmentType::Approach { feed_rate }
        | SegmentType::Retract { feed_rate }
        | SegmentType::PassRetract { feed_rate } => Some(feed_rate),
        SegmentType::Rapid => None,
    }
}

fn validate_axis_values<T: Scalar>(
    machine_axes: Option<&[MachineAxisValue<T>]>,
    segment_index: usize,
    pose_endpoint: &'static str,
    machine_constraint: &MachineConstraint<T>,
    linear_tol_mm: f64,
    rotary_tol_deg: f64,
) -> Result<(), ValidationError> {
    let Some(machine_axes) = machine_axes else {
        return Ok(());
    };

    for axis in machine_axes {
        match axis.axis_kind {
            MachineAxisKind::Linear => {
                let Some(axis_label) = parse_linear_axis_label(&axis.axis_name) else {
                    return Err(ValidationError::UnsupportedMachineAxis {
                        segment_index,
                        pose_endpoint,
                        axis_name: axis.axis_name.clone(),
                        axis_kind: axis.axis_kind,
                    });
                };

                let Some(limit) = machine_constraint.linear_limit_for_axis(axis_label) else {
                    continue;
                };

                let value_mm = axis.value.to_f64();
                let min_mm = limit.min_mm.to_f64();
                let max_mm = limit.max_mm.to_f64();
                let result = validate_linear_travel_mm(value_mm, min_mm, max_mm, linear_tol_mm);
                if !result.is_valid() {
                    return Err(ValidationError::LinearAxisLimitExceeded {
                        segment_index,
                        pose_endpoint,
                        axis_name: axis.axis_name.clone(),
                        value_mm,
                        min_mm,
                        max_mm,
                    });
                }
            }
            MachineAxisKind::Rotary => {
                let Some(axis_label) = parse_rotary_axis_label(&axis.axis_name) else {
                    return Err(ValidationError::UnsupportedMachineAxis {
                        segment_index,
                        pose_endpoint,
                        axis_name: axis.axis_name.clone(),
                        axis_kind: axis.axis_kind,
                    });
                };

                let Some(limit) = machine_constraint.rotary_limit_for_axis(axis_label) else {
                    continue;
                };

                let value_deg = axis.value.to_f64();
                let min_deg = limit.min_deg.to_f64();
                let max_deg = limit.max_deg.to_f64();
                let result = validate_rotary_angle_deg(value_deg, min_deg, max_deg, rotary_tol_deg);
                if !result.is_valid() {
                    return Err(ValidationError::RotaryAxisLimitExceeded {
                        segment_index,
                        pose_endpoint,
                        axis_name: axis.axis_name.clone(),
                        value_deg,
                        min_deg,
                        max_deg,
                    });
                }
            }
        }
    }

    Ok(())
}

fn parse_linear_axis_label(axis_name: &str) -> Option<LinearAxisLabel> {
    match axis_name.trim().to_ascii_uppercase().as_str() {
        "X" => Some(LinearAxisLabel::X),
        "Y" => Some(LinearAxisLabel::Y),
        "Z" => Some(LinearAxisLabel::Z),
        "U" => Some(LinearAxisLabel::U),
        "V" => Some(LinearAxisLabel::V),
        "W" => Some(LinearAxisLabel::W),
        _ => None,
    }
}

fn parse_rotary_axis_label(axis_name: &str) -> Option<RotaryAxisLabel> {
    match axis_name.trim().to_ascii_uppercase().as_str() {
        "A" => Some(RotaryAxisLabel::A),
        "B" => Some(RotaryAxisLabel::B),
        "C" => Some(RotaryAxisLabel::C),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tolerance::CAM_DEFAULT_CLOSURE_TOLERANCE_F64;
    use crate::{
        ArcDirection, ContourLevelPath, CuttingDirection, LinearAxisLimit, PathSegment,
        PoseInterpolationPolicy, RotaryAxisLimit, ToolPose, ToolPoseSpan,
    };
    use geo_algorithms::{Point3D, Vector3D};

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

    #[test]
    fn test_validate_toolpath_machine_constraints_ok() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        );
        let toolpath = ToolPath::new(
            "tool1".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(0, -1.0, vec![segment])],
            vec![],
        );

        let mut constraint = MachineConstraint::empty();
        constraint.linear_speed_limit = Some(crate::LinearSpeedLimit::new(1000.0));

        let tolerance = CamTolerance::default();
        assert!(validate_toolpath_machine_constraints(&toolpath, &constraint, &tolerance).is_ok());
    }

    #[test]
    fn test_validate_toolpath_machine_constraints_feed_rate_exceeded() {
        let segment = PathSegment::new_arc(
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 0.0),
            ArcDirection::CounterClockwise,
            SegmentType::Approach { feed_rate: 1500.0 },
        );
        let toolpath = ToolPath::new(
            "tool2".to_string(),
            CuttingDirection::Up,
            vec![segment],
            vec![],
            vec![],
        );

        let mut constraint = MachineConstraint::empty();
        constraint.linear_speed_limit = Some(crate::LinearSpeedLimit::new(1000.0));

        let tolerance = CamTolerance::default();
        let result = validate_toolpath_machine_constraints(&toolpath, &constraint, &tolerance);
        assert!(matches!(
            result,
            Err(ValidationError::FeedRateLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_validate_toolpath_machine_constraints_linear_acceleration_exceeded() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(5.0, 0.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        )
        .with_linear_acceleration_hint_mm_per_sec2(80.0);

        let toolpath = ToolPath::new(
            "tool3".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(0, -1.0, vec![segment])],
            vec![],
        );

        let mut constraint = MachineConstraint::empty();
        constraint.linear_speed_limit = Some(crate::LinearSpeedLimit::new(1000.0));
        constraint.linear_acceleration_limit = Some(crate::LinearAccelerationLimit::new(50.0));

        let tolerance = CamTolerance::default();
        let result = validate_toolpath_machine_constraints(&toolpath, &constraint, &tolerance);
        assert!(matches!(
            result,
            Err(ValidationError::LinearAccelerationLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_validate_toolpath_machine_constraints_rotary_acceleration_exceeded() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(5.0, 0.0, 0.0),
            SegmentType::Cutting { feed_rate: 500.0 },
        )
        .with_rotary_acceleration_hint_deg_per_sec2(180.0);

        let toolpath = ToolPath::new(
            "tool4".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(0, -1.0, vec![segment])],
            vec![],
        );

        let mut constraint = MachineConstraint::empty();
        constraint.linear_speed_limit = Some(crate::LinearSpeedLimit::new(1000.0));
        constraint.rotary_acceleration_limit = Some(crate::RotaryAccelerationLimit::new(120.0));

        let tolerance = CamTolerance::default();
        let result = validate_toolpath_machine_constraints(&toolpath, &constraint, &tolerance);
        assert!(matches!(
            result,
            Err(ValidationError::RotaryAccelerationLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_validate_pose_segments_machine_constraints_ok_with_wrapped_rotary_range() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            SegmentType::Rapid,
        );
        let start_pose = ToolPose::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, -1.0),
            Some(vec![
                MachineAxisValue::new("X".to_string(), MachineAxisKind::Linear, 10.0),
                MachineAxisValue::new("A".to_string(), MachineAxisKind::Rotary, 350.0),
            ]),
        );
        let end_pose = ToolPose::new(
            Point3D::new(1.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, -1.0),
            Some(vec![
                MachineAxisValue::new("X".to_string(), MachineAxisKind::Linear, 15.0),
                MachineAxisValue::new("A".to_string(), MachineAxisKind::Rotary, 5.0),
            ]),
        );
        let pose_span = ToolPoseSpan::new(
            start_pose,
            end_pose,
            PoseInterpolationPolicy::MachineConstrained,
        );
        let pose_segments = vec![PoseAnnotatedSegment::new(segment, pose_span)];

        let constraint = MachineConstraint {
            linear_axis_limits: vec![LinearAxisLimit::new(LinearAxisLabel::X, 0.0, 100.0)],
            rotary_axis_limits: vec![RotaryAxisLimit::new(RotaryAxisLabel::A, 300.0, 30.0)],
            linear_speed_limit: None,
            rotary_speed_limit: None,
            linear_acceleration_limit: None,
            rotary_acceleration_limit: None,
        };

        let tolerance = CamTolerance::default();
        assert!(
            validate_pose_segments_machine_constraints(&pose_segments, &constraint, &tolerance)
                .is_ok()
        );
    }

    #[test]
    fn test_validate_pose_segments_machine_constraints_axis_range_error() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            SegmentType::Rapid,
        );
        let pose = ToolPose::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, -1.0),
            Some(vec![MachineAxisValue::new(
                "X".to_string(),
                MachineAxisKind::Linear,
                120.0,
            )]),
        );
        let pose_span = ToolPoseSpan::new(
            pose.clone(),
            pose,
            PoseInterpolationPolicy::MachineConstrained,
        );
        let pose_segments = vec![PoseAnnotatedSegment::new(segment, pose_span)];

        let constraint = MachineConstraint {
            linear_axis_limits: vec![LinearAxisLimit::new(LinearAxisLabel::X, 0.0, 100.0)],
            rotary_axis_limits: vec![],
            linear_speed_limit: None,
            rotary_speed_limit: None,
            linear_acceleration_limit: None,
            rotary_acceleration_limit: None,
        };

        let tolerance = CamTolerance::default();
        let result =
            validate_pose_segments_machine_constraints(&pose_segments, &constraint, &tolerance);
        assert!(matches!(
            result,
            Err(ValidationError::LinearAxisLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_validate_pose_segments_machine_constraints_unsupported_axis_name() {
        let segment = PathSegment::new_line(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            SegmentType::Rapid,
        );
        let pose = ToolPose::new(
            Point3D::new(0.0, 0.0, 0.0),
            Vector3D::new(0.0, 0.0, -1.0),
            Some(vec![MachineAxisValue::new(
                "Q".to_string(),
                MachineAxisKind::Linear,
                10.0,
            )]),
        );
        let pose_span = ToolPoseSpan::new(
            pose.clone(),
            pose,
            PoseInterpolationPolicy::MachineConstrained,
        );
        let pose_segments = vec![PoseAnnotatedSegment::new(segment, pose_span)];

        let constraint = MachineConstraint::empty();
        let tolerance = CamTolerance::default();

        let result =
            validate_pose_segments_machine_constraints(&pose_segments, &constraint, &tolerance);
        assert!(matches!(
            result,
            Err(ValidationError::UnsupportedMachineAxis { .. })
        ));
    }

    #[test]
    fn test_validation_error_message_key_mapping() {
        assert_eq!(
            ValidationError::ContourNotClosed {
                distance: 1.0,
                tolerance: 0.001
            }
            .message_key(),
            "validation.contour.not_closed"
        );
        assert_eq!(
            ValidationError::InsufficientPoints { point_count: 2 }.message_key(),
            "validation.contour.insufficient_points"
        );
        assert_eq!(
            ValidationError::EmptyContour.message_key(),
            "validation.contour.empty"
        );
        assert_eq!(
            ValidationError::FeedRateLimitExceeded {
                segment_index: 0,
                feed_rate: 1000.0,
                max_feed_rate: 500.0
            }
            .message_key(),
            "validation.machine.feed_rate_limit_exceeded"
        );
        assert_eq!(
            ValidationError::LinearAccelerationLimitExceeded {
                segment_index: 0,
                acceleration_mm_per_sec2: 80.0,
                max_acceleration_mm_per_sec2: 50.0
            }
            .message_key(),
            "validation.machine.linear_acceleration_limit_exceeded"
        );
        assert_eq!(
            ValidationError::RotaryAccelerationLimitExceeded {
                segment_index: 0,
                acceleration_deg_per_sec2: 180.0,
                max_acceleration_deg_per_sec2: 120.0
            }
            .message_key(),
            "validation.machine.rotary_acceleration_limit_exceeded"
        );
        assert_eq!(
            ValidationError::LinearAxisLimitExceeded {
                segment_index: 0,
                pose_endpoint: "start",
                axis_name: "X".to_string(),
                value_mm: 120.0,
                min_mm: 0.0,
                max_mm: 100.0
            }
            .message_key(),
            "validation.machine.linear_axis_limit_exceeded"
        );
        assert_eq!(
            ValidationError::RotaryAxisLimitExceeded {
                segment_index: 0,
                pose_endpoint: "end",
                axis_name: "A".to_string(),
                value_deg: 200.0,
                min_deg: -120.0,
                max_deg: 120.0
            }
            .message_key(),
            "validation.machine.rotary_axis_limit_exceeded"
        );
        assert_eq!(
            ValidationError::UnsupportedMachineAxis {
                segment_index: 0,
                pose_endpoint: "start",
                axis_name: "Q".to_string(),
                axis_kind: MachineAxisKind::Linear,
            }
            .message_key(),
            "validation.machine.unsupported_axis"
        );
    }
}
