//! Entity Core Traits
//!
//! ViewModel 層はこのトレイト経由で Entity を扱い、
//! 具体実装（geo_entity など）への直接依存を避ける。

use crate::Scalar;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrokePattern {
    Solid,
    Dashed,
    Dotted,
    DashDot,
    DashDotDot,
    Hidden,
}

pub trait EntityIdentity {
    type Id: Copy + Eq + std::hash::Hash;

    fn entity_id(&self) -> Self::Id;
}

pub trait EntityDisplayProperties {
    fn entity_visible(&self) -> bool;
    fn entity_color(&self) -> [f32; 4];
}

pub trait StrokeDisplayProperties {
    fn entity_stroke_pattern(&self) -> StrokePattern;
    fn entity_stroke_width(&self) -> f32;
}

pub trait PlanarStroke2DProperties<T: Scalar> {
    fn definition_plane_origin(&self) -> (T, T, T);
    fn definition_plane_normal(&self) -> (T, T, T);
    fn definition_plane_u_axis(&self) -> (T, T, T);
}

pub trait LineEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn line_start(&self) -> (T, T, T);
    fn line_end(&self) -> (T, T, T);
}

pub trait CircleEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn circle_center(&self) -> (T, T, T);
    fn circle_radius(&self) -> T;
    fn circle_axis(&self) -> (T, T, T);
}

pub trait ArcEntity3DProperties<T: Scalar>: EntityIdentity + EntityDisplayProperties {
    fn arc_center(&self) -> (T, T, T);
    fn arc_radius(&self) -> T;
    fn arc_start_angle(&self) -> T;
    fn arc_end_angle(&self) -> T;
    fn arc_normal(&self) -> (T, T, T);
    fn arc_start_direction(&self) -> (T, T, T);
}
