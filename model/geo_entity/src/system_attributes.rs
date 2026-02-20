use crate::{AttributeCode, AttributeValueKind};

#[derive(Debug, Clone, Copy)]
pub struct SystemAttributeDef {
    pub code: AttributeCode,
    pub name: &'static str,
    pub value_kind: AttributeValueKind,
    pub description: &'static str,
}

pub const META_NAME: AttributeCode = AttributeCode::new_unchecked(-1_000_001);
pub const META_DESCRIPTION: AttributeCode = AttributeCode::new_unchecked(-1_000_002);
pub const META_TAG: AttributeCode = AttributeCode::new_unchecked(-1_000_003);

pub const CAM_PATH_DRILL_CYCLE: AttributeCode = AttributeCode::new_unchecked(-10_01_02_01);
pub const CAM_PATH_DRILL_FEED_RATE: AttributeCode = AttributeCode::new_unchecked(-10_01_02_02);
pub const CAM_PATH_DRILL_SPINDLE_SPEED: AttributeCode = AttributeCode::new_unchecked(-10_01_02_03);
pub const CAM_PATH_DRILL_PECK_STEP: AttributeCode = AttributeCode::new_unchecked(-10_01_02_04);
pub const CAM_PATH_DRILL_DWELL: AttributeCode = AttributeCode::new_unchecked(-10_01_02_05);

pub const DIMENSION_LENGTH_VALUE: AttributeCode = AttributeCode::new_unchecked(-20_01_01_01);

pub const SYSTEM_ATTRIBUTE_DEFS: &[SystemAttributeDef] = &[
    SystemAttributeDef {
        code: META_NAME,
        name: "meta.name",
        value_kind: AttributeValueKind::String,
        description: "エンティティ名",
    },
    SystemAttributeDef {
        code: META_DESCRIPTION,
        name: "meta.description",
        value_kind: AttributeValueKind::String,
        description: "説明",
    },
    SystemAttributeDef {
        code: META_TAG,
        name: "meta.tag",
        value_kind: AttributeValueKind::String,
        description: "タグ",
    },
    SystemAttributeDef {
        code: CAM_PATH_DRILL_CYCLE,
        name: "cam.path.drill.cycle",
        value_kind: AttributeValueKind::String,
        description: "穴あけサイクル種別",
    },
    SystemAttributeDef {
        code: CAM_PATH_DRILL_FEED_RATE,
        name: "cam.path.drill.feed_rate",
        value_kind: AttributeValueKind::Float,
        description: "穴あけ送り速度",
    },
    SystemAttributeDef {
        code: CAM_PATH_DRILL_SPINDLE_SPEED,
        name: "cam.path.drill.spindle_speed",
        value_kind: AttributeValueKind::Float,
        description: "穴あけ主軸回転数",
    },
    SystemAttributeDef {
        code: CAM_PATH_DRILL_PECK_STEP,
        name: "cam.path.drill.peck_step",
        value_kind: AttributeValueKind::Float,
        description: "穴あけステップ量",
    },
    SystemAttributeDef {
        code: CAM_PATH_DRILL_DWELL,
        name: "cam.path.drill.dwell",
        value_kind: AttributeValueKind::Boolean,
        description: "穴底dwell有効化",
    },
    SystemAttributeDef {
        code: DIMENSION_LENGTH_VALUE,
        name: "dimension.length.value",
        value_kind: AttributeValueKind::Float,
        description: "寸法長さ値",
    },
];

pub fn find_system_attribute_def(code: AttributeCode) -> Option<&'static SystemAttributeDef> {
    SYSTEM_ATTRIBUTE_DEFS
        .iter()
        .find(|definition| definition.code == code)
}
