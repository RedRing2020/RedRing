//! CAMツールセットのサンプル定義
//!
//! テストや検証用途で使用できるサンプルツールセットを提供します。

use cam_core::{
    Holder, HolderInterferenceOffset, HolderSegment, Tool, ToolSet, ToolSetReferencePoint,
    ToolType,
};

/// 2段ホルダー + フラットエンドミルのサンプルツールセット
pub fn sample_toolset() -> ToolSet<f64> {
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
        10.0,
    )
    .with_reference_point(ToolSetReferencePoint::Tip)
}
