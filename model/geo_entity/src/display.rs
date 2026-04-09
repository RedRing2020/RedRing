use geo_contracts::StrokePattern;
use serde::{Deserialize, Serialize};

pub type LineStyle = StrokePattern;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayAttributes {
    pub color: [f32; 4],
    pub line_style: StrokePattern,
    pub line_width: f32,
    pub layer: String,
    pub visible: bool,
}

impl Default for DisplayAttributes {
    fn default() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            line_style: StrokePattern::Solid,
            line_width: 1.0,
            layer: "0".to_string(),
            visible: true,
        }
    }
}

impl DisplayAttributes {
    pub fn cad_red() -> Self {
        Self {
            color: [1.0, 0.0, 0.0, 1.0],
            ..Default::default()
        }
    }

    pub fn cad_green() -> Self {
        Self {
            color: [0.0, 1.0, 0.0, 1.0],
            ..Default::default()
        }
    }

    pub fn cam_cutting() -> Self {
        Self {
            color: [1.0, 1.0, 1.0, 1.0],
            ..Default::default()
        }
    }

    pub fn cam_rapid() -> Self {
        Self {
            color: [0.2, 0.5, 1.0, 1.0],
            ..Default::default()
        }
    }
}
