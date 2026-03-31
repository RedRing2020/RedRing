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
        }
    }
}

impl Error for SimulationError {}
