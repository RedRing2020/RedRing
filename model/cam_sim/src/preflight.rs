//! 切削シミュレーションの実行前の Preflight 判定（設計: CUTTING_SIMULATION_DESIGN.md「本番導線の入力前提条件」）
//!
//! 判定は `ReadyStrict`（`Ok`）・`NotReady`・`InvalidInput` の 3 値とする。機械制約とポスト由来の加工条件
//! （回避高さ・速度）は、設定する経路ができた時点で必須の受理条件に加える。

use std::fmt::{Display, Formatter};

use cam_core::ToolPath;

use crate::constants::STOCK_OCTREE_MAX_DEPTH;
use crate::simulation_input::CuttingSimulationInput;
use crate::simulator::tool_cutting_behavior;

/// Preflight 判定を通過した入力から求めた、実行の設定
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PreflightReady {
    /// 精度から求めた素材の Octree の分割の深さ
    pub(crate) octree_depth: usize,
}

/// Preflight 判定で実行を拒否する理由
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PreflightFailure {
    /// 必須の条件が満たされていない（設定を補って再実行する）
    NotReady(Vec<NotReadyReason>),
    /// 入力の値が契約に違反している
    InvalidInput(InvalidInputReason),
}

/// 必須の条件のうち、満たされていないもの
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NotReadyReason {
    EmptyToolPath,
    NoCuttingSegment,
    ToolMismatch {
        toolpath_tool_id: String,
        tool_id: String,
    },
    UnsupportedTool,
    PrecisionNotAchievable {
        achievable_precision: f64,
    },
}

/// 契約に違反している入力の値
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InvalidInputReason {
    Stock,
    Tool,
    Precision,
    SnapshotInterval,
}

/// ツールパスと入力が実行してよい状態かを判定する。
pub(crate) fn preflight_cutting_simulation(
    toolpath: &ToolPath<f64>,
    input: &CuttingSimulationInput,
) -> Result<PreflightReady, PreflightFailure> {
    let stock = &input.stock;
    let stock_is_finite = [stock.min(), stock.max()]
        .iter()
        .all(|point| point.x().is_finite() && point.y().is_finite() && point.z().is_finite());
    let stock_has_volume = stock.width() > 0.0 && stock.height() > 0.0 && stock.depth() > 0.0;
    if !stock_is_finite || !stock_has_volume {
        return Err(PreflightFailure::InvalidInput(InvalidInputReason::Stock));
    }
    let tool = &input.tool;
    let positive = |value: f64| value.is_finite() && value > 0.0;
    if !positive(tool.radius()) || !positive(tool.cutting_length) || !tool.validate_parameters() {
        return Err(PreflightFailure::InvalidInput(InvalidInputReason::Tool));
    }
    if !(input.precision.is_finite() && input.precision > 0.0) {
        return Err(PreflightFailure::InvalidInput(
            InvalidInputReason::Precision,
        ));
    }
    if !input.snapshot_interval.is_valid() {
        return Err(PreflightFailure::InvalidInput(
            InvalidInputReason::SnapshotInterval,
        ));
    }

    let mut reasons = Vec::new();
    if toolpath.total_segments() == 0 {
        reasons.push(NotReadyReason::EmptyToolPath);
    } else if toolpath
        .contour_levels
        .iter()
        .all(|level| level.cutting_segments().is_empty())
    {
        reasons.push(NotReadyReason::NoCuttingSegment);
    }
    if toolpath.tool_id != tool.id {
        reasons.push(NotReadyReason::ToolMismatch {
            toolpath_tool_id: toolpath.tool_id.clone(),
            tool_id: tool.id.clone(),
        });
    }
    if tool_cutting_behavior(tool).is_err() {
        reasons.push(NotReadyReason::UnsupportedTool);
    }

    let max_extent = stock.width().max(stock.height()).max(stock.depth());
    let octree_depth = required_octree_depth(max_extent, input.precision);
    if octree_depth > STOCK_OCTREE_MAX_DEPTH {
        reasons.push(NotReadyReason::PrecisionNotAchievable {
            achievable_precision: max_extent / f64::from(1_u32 << STOCK_OCTREE_MAX_DEPTH),
        });
    }

    if reasons.is_empty() {
        Ok(PreflightReady { octree_depth })
    } else {
        Err(PreflightFailure::NotReady(reasons))
    }
}

/// 最も深いボクセルの大きさ（最大の辺の長さ / 2^深さ）が精度以下になる最小の深さ
fn required_octree_depth(max_extent: f64, precision: f64) -> usize {
    let mut depth = 0;
    let mut voxel_size = max_extent;
    while voxel_size > precision && depth <= STOCK_OCTREE_MAX_DEPTH {
        voxel_size /= 2.0;
        depth += 1;
    }
    depth
}

impl Display for PreflightFailure {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotReady(reasons) => {
                write!(f, "cutting simulation is not ready:")?;
                for reason in reasons {
                    write!(f, " {reason};")?;
                }
                Ok(())
            }
            Self::InvalidInput(reason) => write!(f, "invalid cutting simulation input: {reason}"),
        }
    }
}

impl Display for NotReadyReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyToolPath => write!(f, "toolpath is empty"),
            Self::NoCuttingSegment => write!(f, "toolpath has no cutting segment"),
            Self::ToolMismatch {
                toolpath_tool_id,
                tool_id,
            } => write!(
                f,
                "tool id does not match the toolpath (toolpath: {toolpath_tool_id}, tool: {tool_id})"
            ),
            Self::UnsupportedTool => write!(f, "only flat and ball end mills are supported"),
            Self::PrecisionNotAchievable {
                achievable_precision,
            } => write!(
                f,
                "precision is finer than the stock model supports (achievable: {achievable_precision})"
            ),
        }
    }
}

impl Display for InvalidInputReason {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stock => write!(f, "stock must be a finite, non-empty box"),
            Self::Tool => write!(f, "tool parameters are invalid"),
            Self::Precision => write!(f, "precision must be finite and greater than zero"),
            Self::SnapshotInterval => write!(f, "snapshot interval is invalid"),
        }
    }
}

#[cfg(test)]
mod tests {
    use cam_core::{ContourLevelPath, CuttingDirection, PathSegment, SegmentType, Tool, ToolPath};
    use geo_algorithms::{Aabb3D, Point3D};

    use super::*;
    use crate::SnapshotInterval;

    fn toolpath(tool_id: &str, segment_type: SegmentType<f64>) -> ToolPath<f64> {
        ToolPath::new(
            tool_id.to_string(),
            CuttingDirection::Down,
            vec![],
            vec![ContourLevelPath::new(
                0,
                0.0,
                vec![PathSegment::new_line(
                    Point3D::new(0.0, 0.0, 0.0),
                    Point3D::new(10.0, 0.0, 0.0),
                    segment_type,
                )],
            )],
            vec![],
        )
    }

    fn cutting() -> SegmentType<f64> {
        SegmentType::Cutting { feed_rate: 100.0 }
    }

    /// 最大の辺が 16 の素材を、工具 `EM2` で切削する入力
    fn input(precision: f64) -> CuttingSimulationInput {
        CuttingSimulationInput {
            stock: Aabb3D::new(Point3D::new(0.0, 0.0, -4.0), Point3D::new(16.0, 8.0, 0.0)),
            tool: Tool::flat_end_mill("EM2".to_string(), 2.0, 10.0),
            precision,
            snapshot_interval: SnapshotInterval::default(),
        }
    }

    #[test]
    fn octree_depth_is_the_smallest_depth_finer_than_precision() {
        // 16 / 2^4 = 1.0 ≤ 1.0、16 / 2^3 = 2.0 > 1.5
        assert_eq!(
            preflight_cutting_simulation(&toolpath("EM2", cutting()), &input(1.0)),
            Ok(PreflightReady { octree_depth: 4 })
        );
        assert_eq!(
            preflight_cutting_simulation(&toolpath("EM2", cutting()), &input(1.5)),
            Ok(PreflightReady { octree_depth: 4 })
        );
        // 素材より粗い精度は分割しない
        assert_eq!(
            preflight_cutting_simulation(&toolpath("EM2", cutting()), &input(20.0)),
            Ok(PreflightReady { octree_depth: 0 })
        );
    }

    #[test]
    fn missing_conditions_are_all_reported_as_not_ready() {
        let rapid = SegmentType::Rapid;
        let radius = CuttingSimulationInput {
            tool: Tool::radius_end_mill("R".to_string(), 4.0, 0.5, 10.0),
            ..input(1.0e-6)
        };
        assert_eq!(
            preflight_cutting_simulation(&toolpath("EM2", rapid), &radius),
            Err(PreflightFailure::NotReady(vec![
                NotReadyReason::NoCuttingSegment,
                NotReadyReason::ToolMismatch {
                    toolpath_tool_id: "EM2".to_string(),
                    tool_id: "R".to_string(),
                },
                NotReadyReason::UnsupportedTool,
                NotReadyReason::PrecisionNotAchievable {
                    achievable_precision: 16.0 / 1024.0,
                },
            ]))
        );

        let empty = ToolPath::new(
            "EM2".to_string(),
            CuttingDirection::Down,
            vec![],
            vec![],
            vec![],
        );
        assert_eq!(
            preflight_cutting_simulation(&empty, &input(1.0)),
            Err(PreflightFailure::NotReady(vec![
                NotReadyReason::EmptyToolPath
            ]))
        );
    }

    #[test]
    fn malformed_values_are_invalid_input() {
        let path = toolpath("EM2", cutting());
        let flat_stock = CuttingSimulationInput {
            stock: Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(16.0, 8.0, 0.0)),
            ..input(1.0)
        };
        let invalid_interval = CuttingSimulationInput {
            snapshot_interval: SnapshotInterval::ByDistance {
                interval_mm: f64::NAN,
                include_segment_endpoints: true,
            },
            ..input(1.0)
        };
        let no_cutting_length = CuttingSimulationInput {
            tool: Tool::flat_end_mill("EM2".to_string(), 2.0, 0.0),
            ..input(1.0)
        };
        for (value, reason) in [
            (flat_stock, InvalidInputReason::Stock),
            (no_cutting_length, InvalidInputReason::Tool),
            (input(0.0), InvalidInputReason::Precision),
            (input(f64::INFINITY), InvalidInputReason::Precision),
            (invalid_interval, InvalidInputReason::SnapshotInterval),
        ] {
            assert_eq!(
                preflight_cutting_simulation(&path, &value),
                Err(PreflightFailure::InvalidInput(reason))
            );
        }
    }
}
