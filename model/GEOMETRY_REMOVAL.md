# Geometry Module Removal

The legacy `model::geometry` (geometry2d/geometry3d) implementation has been removed.

## Rationale

- Functionality superseded by `geo_core` (numerics, scalar, tolerance) and `geo_primitives` (actual Point/Vector/Curve primitives).
- No remaining external crate imports of `model::geometry::*`.
- Simplifies dependency graph; `model` now focuses on abstract traits (`geometry_trait`, `geometry_kind`, `geometry_common`).

## Timeline

1. Dry-run: `pub mod geometry;` commented out, build confirmed no external references (only internal adapters broke).
2. Traits (`Curve2D`, `Curve3D`) refactored to use associated types instead of concrete Point/Vector from legacy module.
3. Adapter `geometry_simple_adapter` disabled (kept file stub for history).
4. Directory physically removed.

## What Changed

- Removed: `model/src/geometry/` directory and exports.
- Updated: `curve2d.rs`, `curve3d.rs` now expose associated types `Point`, `Vector`.
- Disabled: `geometry_simple_adapter.rs` (deprecated bridge layer).

## Migration Mapping (Old -> New)

| Old (model::geometry)                                    | New Location                        |
| -------------------------------------------------------- | ----------------------------------- |
| geometry2d::Point                                        | geo_foundation::Point2D             |
| geometry2d::Vector                                       | geo_foundation::Vector2D            |
| geometry2d::{Line,Circle,Arc,Ellipse,Ray,InfiniteLine}   | geo_foundation::\* 2D               |
| geometry3d::Point                                        | geo_foundation::Point3D             |
| geometry3d::Vector                                       | geo_foundation::Vector3D            |
| geometry3d::{Line,Circle,Arc,Ellipse,Plane,InfiniteLine} | geo_foundation::\* 3D               |
| geometry3d::NurbsCurve (stub)                            | (Planned) geo_foundation::nurbs::\* |
| SimpleAdaptedLine adapter                                | (Deprecated) Removed use            |

## Implementing New Curves

Implementors should supply associated types:

```rust
impl Curve2D for MyCurve2D {
    type Point = geo_foundation::Point2D;
    type Vector = geo_foundation::Vector2D;
    // ... methods ...
}
```

## Future Work

- Move any still-useful math utilities from disabled adapter (if needed) into `geo_core`.
- Add documentation examples for `Curve2D` / `Curve3D` implementations.
- Introduce a lightweight adapter crate only if cross-crate coercions become necessary.

## Verification

- Workspace build & tests passed after removal (no new failures introduced).

---

This document serves as a historical record of the geometry layer consolidation.
