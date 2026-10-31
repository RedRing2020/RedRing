//! 切削シミュレーションの実行前の Preflight 判定（設計: CUTTING_SIMULATION_DESIGN.md「本番導線の入力前提条件」）
//!
//! 判定は `ReadyStrict`（`Ok`）・`NotReady`・`InvalidInput` の 3 値とする。機械制約とポスト由来の加工条件
//! （回避高さ・速度）は、設定する経路ができた時点で必須の受理条件に加える。
//! ツールパスによらず入力だけで決まる判定（`cutting_simulation_input_errors`）は、投入の前の確認にも使う。

use std::fmt::{Display, Formatter};

use cam_core::ToolPath;

use crate::constants::MAX_DEXEL_LINES;
use crate::simulation_input::CuttingSimulationInput;
use crate::stock_model::SweptToolShape;
use crate::tri_dexel::{dexel_line_count, dexel_spacing};

/// Preflight 判定で実行を拒否する理由
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PreflightFailure {
    /// 必須の条件が満たされていない（設定を補って再実行する）
    NotReady(Vec<NotReadyReason>),
    /// 入力の値が契約に違反している
    InvalidInput(CuttingSimulationInputError),
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
    /// 精度から決めた格子の線の本数が上限を超える（線の本数と、上限に収まる最も細かい精度）
    TooManyDexelLines {
        lines: usize,
        finest_precision: f64,
    },
}

/// ツールパスによらず入力だけで決まる、切削シミュレーションの入力の誤り
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CuttingSimulationInputError {
    /// 素材が有限で体積を持つ直方体でない
    Stock,
    /// 工具のパラメータが不正
    Tool,
    /// 精度が正の有限値でない
    Precision,
    /// 記録の間隔が不正
    SnapshotInterval,
    /// 精度から決めた格子の線の本数が上限を超える（線の本数と、上限に収まる最も細かい精度）
    TooManyDexelLines { lines: usize, finest_precision: f64 },
}

/// 切削シミュレーションの入力の誤りをすべて返す（線の本数は、素材・工具・精度が正しい場合だけ判定する）
pub fn cutting_simulation_input_errors(
    input: &CuttingSimulationInput,
) -> Vec<CuttingSimulationInputError> {
    let mut errors = Vec::new();
    let stock = &input.stock;
    let stock_is_finite = [stock.min(), stock.max()]
        .iter()
        .all(|point| point.x().is_finite() && point.y().is_finite() && point.z().is_finite());
    let stock_has_volume = stock.width() > 0.0 && stock.height() > 0.0 && stock.depth() > 0.0;
    if !stock_is_finite || !stock_has_volume {
        errors.push(CuttingSimulationInputError::Stock);
    }
    let tool = &input.tool;
    let positive = |value: f64| value.is_finite() && value > 0.0;
    if !positive(tool.radius())
        || !tool.cutting_length.is_none_or(positive)
        || !tool.validate_parameters()
    {
        errors.push(CuttingSimulationInputError::Tool);
    }
    if !positive(input.precision) {
        errors.push(CuttingSimulationInputError::Precision);
    }
    let values_are_valid = errors.is_empty();
    if !input.snapshot_interval.is_valid() {
        errors.push(CuttingSimulationInputError::SnapshotInterval);
    }
    if values_are_valid {
        let lines = dexel_line_count(stock, dexel_spacing(tool.radius(), input.precision));
        if lines > MAX_DEXEL_LINES {
            errors.push(CuttingSimulationInputError::TooManyDexelLines {
                lines,
                finest_precision: finest_precision(input),
            });
        }
    }
    errors
}

/// ツールパスと入力が実行してよい状態かを判定する。
pub(crate) fn preflight_cutting_simulation(
    toolpath: &ToolPath<f64>,
    input: &CuttingSimulationInput,
) -> Result<(), PreflightFailure> {
    let input_errors = cutting_simulation_input_errors(input);
    // 線の本数の上限は設定を補えば満たせるため NotReady とし、ほかは InvalidInput とする
    if let Some(&error) = input_errors
        .iter()
        .find(|error| !matches!(error, CuttingSimulationInputError::TooManyDexelLines { .. }))
    {
        return Err(PreflightFailure::InvalidInput(error));
    }
    let tool = &input.tool;

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
    // 工具の種類だけを確かめる（工具長を指定しない工具の長さは使わない）
    if SweptToolShape::from_tool(tool, 0.0).is_none() {
        reasons.push(NotReadyReason::UnsupportedTool);
    }
    reasons.extend(input_errors.into_iter().filter_map(|error| match error {
        CuttingSimulationInputError::TooManyDexelLines {
            lines,
            finest_precision,
        } => Some(NotReadyReason::TooManyDexelLines {
            lines,
            finest_precision,
        }),
        _ => None,
    }));

    if reasons.is_empty() {
        Ok(())
    } else {
        Err(PreflightFailure::NotReady(reasons))
    }
}

/// 格子の線の本数が上限に収まる、最も細かい精度を有効数字 2 桁に切り上げた値（`input.precision` では上限を超える
/// 場合）
fn finest_precision(input: &CuttingSimulationInput) -> f64 {
    let fits = |precision: f64| {
        dexel_line_count(&input.stock, dexel_spacing(input.tool.radius(), precision))
            <= MAX_DEXEL_LINES
    };
    // 各方向に 1 本ずつになる精度では必ず収まる。収まる側を保ったまま二分する
    let longest = input
        .stock
        .width()
        .max(input.stock.height())
        .max(input.stock.depth());
    let (mut coarse, mut fine) = (
        longest * longest / (8.0 * input.tool.radius()),
        input.precision,
    );
    for _ in 0..64 {
        let middle = 0.5 * (coarse + fine);
        if fits(middle) {
            coarse = middle;
        } else {
            fine = middle;
        }
    }
    // 切り上げた値は、二分で求めた値より粗いため収まる
    let digits = 1 - coarse.log10().floor() as i32;
    let scale = 10_f64.powi(digits);
    (coarse * scale).ceil() / scale
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
            Self::TooManyDexelLines {
                lines,
                finest_precision,
            } => CuttingSimulationInputError::TooManyDexelLines {
                lines: *lines,
                finest_precision: *finest_precision,
            }
            .fmt(f),
        }
    }
}

impl Display for CuttingSimulationInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stock => write!(f, "stock must be a finite, non-empty box"),
            Self::Tool => write!(f, "tool parameters are invalid"),
            Self::Precision => write!(f, "precision must be finite and greater than zero"),
            Self::SnapshotInterval => write!(f, "snapshot interval is invalid"),
            Self::TooManyDexelLines {
                lines,
                finest_precision,
            } => write!(
                f,
                "precision is too fine for the stock size ({lines} dexel lines exceed {MAX_DEXEL_LINES}; the finest precision is {finest_precision:.4})"
            ),
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
    fn ready_input_passes() {
        assert_eq!(
            preflight_cutting_simulation(&toolpath("EM2", cutting()), &input(0.01)),
            Ok(())
        );
    }

    #[test]
    fn missing_conditions_are_all_reported_as_not_ready() {
        let rapid = SegmentType::Rapid;
        let radius = CuttingSimulationInput {
            tool: Tool::radius_end_mill("R".to_string(), 4.0, 0.5, 10.0),
            ..input(0.01)
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
    fn too_fine_precision_reports_the_finest_precision_within_the_line_limit() {
        let path = toolpath("EM2", cutting());
        let Err(PreflightFailure::NotReady(reasons)) =
            preflight_cutting_simulation(&path, &input(1.0e-6))
        else {
            panic!("too many dexel lines must be not ready");
        };
        let [
            NotReadyReason::TooManyDexelLines {
                lines,
                finest_precision,
            },
        ] = reasons[..]
        else {
            panic!("unexpected reasons: {reasons:?}");
        };
        assert!(lines > MAX_DEXEL_LINES);
        assert_eq!(
            preflight_cutting_simulation(&path, &input(finest_precision)),
            Ok(())
        );
        // 有効数字 2 桁で 1 つ細かい精度では収まらない
        let digits = 1 - finest_precision.log10().floor() as i32;
        let finer = finest_precision - 10_f64.powi(-digits);
        assert!(preflight_cutting_simulation(&path, &input(finer)).is_err());
    }

    #[test]
    fn input_errors_are_all_reported_without_the_toolpath() {
        let malformed = CuttingSimulationInput {
            tool: Tool::flat_end_mill("EM2".to_string(), 2.0, 0.0),
            snapshot_interval: SnapshotInterval::ByDistance {
                interval_mm: f64::NAN,
                include_segment_endpoints: true,
            },
            ..input(0.0)
        };
        assert_eq!(
            cutting_simulation_input_errors(&malformed),
            vec![
                CuttingSimulationInputError::Tool,
                CuttingSimulationInputError::Precision,
                CuttingSimulationInputError::SnapshotInterval,
            ]
        );
        assert!(matches!(
            cutting_simulation_input_errors(&input(1.0e-6))[..],
            [CuttingSimulationInputError::TooManyDexelLines { .. }]
        ));
        assert_eq!(cutting_simulation_input_errors(&input(0.01)), vec![]);
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
            (flat_stock, CuttingSimulationInputError::Stock),
            (no_cutting_length, CuttingSimulationInputError::Tool),
            (input(0.0), CuttingSimulationInputError::Precision),
            (input(f64::INFINITY), CuttingSimulationInputError::Precision),
            (
                invalid_interval,
                CuttingSimulationInputError::SnapshotInterval,
            ),
        ] {
            assert_eq!(
                preflight_cutting_simulation(&path, &value),
                Err(PreflightFailure::InvalidInput(reason))
            );
        }
    }
}
