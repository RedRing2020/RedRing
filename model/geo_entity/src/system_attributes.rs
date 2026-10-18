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
