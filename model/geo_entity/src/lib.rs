//! geo_entity - エンティティ基盤クレート（Phase 4.0）
//!
//! - EntityId: エンティティ同一性ID（決定的生成対応）
//! - AttributeCode: 属性種別コード（負=system, 正=user, 0禁止）
//! - Attributes: 型検証付き属性コンテナ
//! - Group/Layer 関係モデル: group 複数所属 / layer 単一所属の relation 管理

pub mod attribute_code;
pub mod attribute_value;
pub mod attributes;
pub mod display;
pub mod entity_id;
pub mod error;
pub mod geometric_entity;
pub mod metadata;
pub mod relations;
pub mod system_attributes;

pub use attribute_code::{AttributeCode, AttributeCodeParts};
pub use attribute_value::{AttributeValue, AttributeValueKind};
pub use attributes::Attributes;
pub use display::{DisplayAttributes, LineStyle};
pub use entity_id::EntityId;
pub use error::{EntityError, EntityResult};
pub use geo_contracts::StrokePattern;
pub use geometric_entity::GeometricEntity;
pub use metadata::Metadata;
pub use relations::{
    EntityGroupMembership, EntityLayerMembership, GroupEntity, GroupId, LayerEntity, LayerId,
    RelationStore,
};
pub use system_attributes::{
    CAM_PATH_DRILL_CYCLE, CAM_PATH_DRILL_DWELL, CAM_PATH_DRILL_FEED_RATE, CAM_PATH_DRILL_PECK_STEP,
    CAM_PATH_DRILL_SPINDLE_SPEED, DIMENSION_LENGTH_VALUE, META_DESCRIPTION, META_NAME, META_TAG,
    SYSTEM_ATTRIBUTE_DEFS, SystemAttributeDef, find_system_attribute_def,
};
