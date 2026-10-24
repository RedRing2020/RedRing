//! geo_entity - 業務に依らない CAD のエンティティ基盤クレート（Phase 4.0）
//!
//! 業務（CAM 等）の属性・表示の定義は持たず、業務層のクレート（`cam_entity` 等）が持つ。
//!
//! - EntityId: エンティティ同一性ID（決定的生成対応）
//! - FaceId: 面の同一性ID（全体で一意）
//! - AttributeCode: 属性種別コード（負=system, 正=user, 0禁止）
//! - Attributes: 型検証付き属性コンテナ
//! - Group/Layer 関係モデル: group 複数所属 / layer 単一所属の relation 管理

pub mod attribute_code;
pub mod attribute_value;
pub mod attributes;
pub mod display;
pub mod entity_id;
pub mod error;
pub mod face_id;
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
pub use face_id::FaceId;
pub use geo_contracts::StrokePattern;
pub use geometric_entity::GeometricEntity;
pub use metadata::Metadata;
pub use relations::{
    EntityGroupMembership, EntityLayerMembership, GroupEntity, GroupId, LayerEntity, LayerId,
    RelationStore,
};
pub use system_attributes::{
    DIMENSION_LENGTH_VALUE, META_DESCRIPTION, META_NAME, META_TAG, SYSTEM_ATTRIBUTE_DEFS,
    SystemAttributeDef, find_system_attribute_def,
};
