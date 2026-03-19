//! Geometry extension foundation traits.
//!
//! This module defines the minimal extension contracts for geometric primitives:
//! - `ExtensionFoundation<T>`: Primitive kind identification and measurement
//! - `Bounded<T>`: Axis-aligned bounding box (AABB)

use crate::classification::PrimitiveKind;
use analysis::abstract_types::Scalar;

/// Base trait for all geometric primitives (Extension Foundation).
///
/// Every geometric primitive must implement this trait to provide:
/// - Type identification (`primitive_kind()`)
/// - Measurement value access (`measure()`)
pub trait ExtensionFoundation<T: Scalar = f64> {
    /// Returns the primitive type.
    fn primitive_kind(&self) -> PrimitiveKind;

    /// Returns the measurement value (length, area, volume, etc.) if applicable.
    fn measure(&self) -> Option<T>;
}

/// Trait for geometric primitives with axis-aligned bounding boxes (AABB).
///
/// # Implementors
///
/// Primitives with spatial extent (Circle, Triangle, NURBS) implement this trait.
/// Basic elements like Point, Vector, Direction do not need AABB.
///
/// # AABB Type
///
/// The concrete AABB type (`geo_core::Aabb2D`, `geo_core::Aabb3D`) is specified
/// by the implementor. This trait only defines the contract.
pub trait Bounded<T: Scalar = f64>: ExtensionFoundation<T> {
    /// The AABB type (e.g., `geo_core::Aabb2D` or `geo_core::Aabb3D`).
    type Aabb;

    /// Returns the axis-aligned bounding box.
    fn aabb(&self) -> Option<Self::Aabb>;
}
