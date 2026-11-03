use std::error::Error;
use std::fmt::{Display, Formatter};

/// `cam_sim` の実行時エラー。
/// Phase 1aで想定する失敗条件を最小セットで表現する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimulationError {
    EmptyToolpath,
    UnsupportedToolType,
    UnsupportedGeometry,
    InvalidInterval,
    /// 成果物の時点が差し引く掃引体の数（`artifact`）が、ツールパスと工具から作った掃引体の数（`toolpath`）を超える
    PrimitiveCountMismatch {
        artifact: usize,
        toolpath: usize,
    },
}

impl Display for SimulationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SimulationError::EmptyToolpath => write!(f, "toolpath is empty"),
            SimulationError::UnsupportedToolType => {
                write!(f, "only flat and ball end mills are supported")
            }
            SimulationError::UnsupportedGeometry => {
                write!(f, "unsupported geometry type in toolpath")
            }
            SimulationError::InvalidInterval => write!(f, "snapshot interval is invalid"),
            SimulationError::PrimitiveCountMismatch { artifact, toolpath } => write!(
                f,
                "the artifact applies {artifact} swept primitives, but the toolpath has {toolpath}"
            ),
        }
    }
}

impl Error for SimulationError {}
