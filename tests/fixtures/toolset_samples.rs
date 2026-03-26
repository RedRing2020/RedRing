//! CAMツールセットのサンプル定義
//!
//! テストや検証用途で使用できるサンプルツールセットを提供します。

use cam_core::{
    Holder, HolderInterferenceOffset, HolderSegment, Tool, ToolSet, ToolSetReferencePoint,
    ToolType,
};

/// 2段ホルダー + フラットエンドミルのサンプルツールセット
pub fn sample_flat_toolset() -> ToolSet<f64> {
    let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);

    let holder = Holder::new(
        "HOLDER-SAMPLE".to_string(),
        vec![
            HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0),
            HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25),
        ],
    )
    .with_interference_offset(HolderInterferenceOffset::new(10.0, 0.0));

    ToolSet::new(
        "TS-SAMPLE-01".to_string(),
        "Flat 10 Sample".to_string(),
        tool,
        holder,
        75.0,
        40.0,
    )
    .with_shank_diameter(10.0)
    .with_shank_length(25.0)
    .with_reference_point(ToolSetReferencePoint::Tip)
}

/// 2段ホルダー + ボールエンドミルのサンプルツールセット
pub fn sample_ball_toolset() -> ToolSet<f64> {
    let tool = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 3.0, 24.0);

    let holder = Holder::new(
        "HOLDER-SAMPLE-BALL".to_string(),
        vec![
            HolderSegment::cylinder(18.0, 26.0, 0.5, 0.5),
            HolderSegment::taper(12.0, 26.0, 16.0, 0.5, 0.25),
        ],
    )
    .with_interference_offset(HolderInterferenceOffset::new(8.0, 0.0));

    ToolSet::new(
        "TS-SAMPLE-02".to_string(),
        "Ball 6 Sample".to_string(),
        tool,
        holder,
        62.0,
        30.0,
    )
    .with_shank_diameter(6.0)
    .with_shank_length(18.0)
    .with_reference_point(ToolSetReferencePoint::Tip)
}

/// 後方互換のため、既存名はフラットサンプルを返す。
pub fn sample_toolset() -> ToolSet<f64> {
    sample_flat_toolset()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_flat_toolset_is_valid() {
        let toolset = sample_flat_toolset();
        assert!(toolset.validate_parameters());
        assert_eq!(toolset.tool.tool_type, ToolType::FlatEndMill);
    }

    #[test]
    fn sample_ball_toolset_is_valid() {
        let toolset = sample_ball_toolset();
        assert!(toolset.validate_parameters());
        assert_eq!(toolset.tool.tool_type, ToolType::BallEndMill);
    }
}
