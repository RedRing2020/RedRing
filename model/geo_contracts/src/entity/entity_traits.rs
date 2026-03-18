//! Entity Core Traits
//!
//! ViewModel 層はこのトレイト経由で Entity を扱い、
//! 具体実装（geo_entity など）への直接依存を避ける。

use crate::Scalar;

pub trait EntityIdentity {
    type Id: Copy + Eq + std::hash::Hash;

    fn entity_id(&self) -> Self::Id;
}

pub trait EntityDisplayProperties {
    fn entity_visible(&self) -> bool;
    fn entity_color(&self) -> [f32; 4];
}

pub trait LineEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn line_start(&self) -> (T, T, T);
    fn line_end(&self) -> (T, T, T);
}
