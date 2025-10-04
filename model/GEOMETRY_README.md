# Geometry Abstraction Layer (EN Placeholder)

This is the English placeholder for the geometry abstraction layer documentation.

The original (Japanese) detailed description is in `GEOMETRY_README.ja.md`.

## Purpose
Provide high level explanation of:
- Remaining role of the `model` crate: abstraction traits (Curve2D / Curve3D / future Surface)
- Delegation of concrete primitives to `geo_primitives`
- Numerical foundation in `geo_core`

## TODO (to be elaborated)
1. Copy core sections from Japanese version and translate.
2. Add implementation example with concrete types.
3. Migration mapping table (old model geometry -> geo_primitives) with brief rationale.
4. Future roadmap (NURBS, surfaces, tolerance strategy, adapters) summarized.

## Quick Mapping
| Old (removed) | New Location |
|---------------|--------------|
| model::geometry2d::* | geo_primitives (2D) |
| model::geometry3d::* | geo_primitives (3D) |
| Tolerance / Scalar | geo_core |

## Associated Types Pattern
Traits now use associated types so implementations can choose Point / Vector concrete representations.

---
(Will be expanded in a future PR.)
