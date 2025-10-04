# Vector f64 Migration Notes

This document summarizes the breaking changes introduced during the migration from `Scalar`-centric vector storage to native `f64` storage for `Vector2D` and `Vector3D`.

## Overview
- Internal representation changed: `[Scalar; N]` -> `[f64; N]`.
- Accessor methods (`x()`, `y()`, `z()`) now return `f64` directly.
- Core algebraic methods (`dot`, `norm`, etc.) now return `f64` instead of `Scalar`.
- Multiplication is now implemented for `Mul<f64>`; `Mul<Scalar>` is removed for vectors.
- Reverse scalar multiplication (`f64 * Vector2D/Vector3D`) IMPLEMENTED (2025-10-05) for ergonomic symmetry.
- Many call sites no longer need `.value()` unwrap chains; remove them to compile.

## Rationale
- Reduce heapless wrapper overhead in tight numeric loops (render + analysis crates).
- Simplify interop with algorithms that already operate purely on `f64`.
- Prepare for future SIMD optimizations (contiguous plain `f64` storage).

## API Changes (Breaking)
| Before | After | Notes |
|--------|-------|-------|
| `v.x().value()` | `v.x()` | Accessors return `f64`.
| `v.dot(&w).value()` | `v.dot(&w)` | Dot now returns `f64`.
| `v.norm().value()` | `v.norm()` | Norm returns `f64`.
| `Vector::new([Scalar;N])` usage indirectly via `Vector3D::new(Scalar,...)` | `Vector3D::from_f64(x,y,z)` or `Vector3D::new([f64;3])` via trait in scope | Removed multi-Scalar constructor.
| `v * Scalar::new(s)` | `v * s` | Implemented `Mul<f64>`.
| `Scalar::new(…) * v` (commutative patterns) | `s * v` | Reverse `Mul<f64>` now implemented (owned & &ref).
| `CadVector::scale(f)` (Scalar wrapping) | `cad_vec.0.clone() * f` | Wrapper kept deprecated.
| `Direction3D::dot(&a,&b).value()` | `a.dot(&b)` | Dot returns `f64`.

## Traits Updated
- `Vector<const D: usize>`: `dot(&self, &Self) -> f64`, `norm(&self) -> f64`.
- `TolerantEq` implementation for vectors now compares raw `f64` components against `context.linear`.

## Removed / Deprecated
- Implicit `Scalar` returns from vector operations.
- Deprecated wrappers still present: `CadVector`, `CadDirection` (will be removed or type-aliased in a later sweep).
- `Arc2D::new` (Scalar angles) is deprecated; use `Arc2D::new_f64(center, radius_f64, Angle, Angle)`.

## Mechanical Migration Steps
1. Remove every `.value()` chained off vector accessors and algebraic results.
2. Replace `Vector3D::new(Scalar::new(x), …)` with `Vector3D::from_f64(x, …)`.
3. Change any `let s = v.dot(&w); s.value()` to just `let s = v.dot(&w);`.
4. Replace `v * Scalar::new(k)` with `v * k` (ensure `k: f64`).
5. For normalization comparisons, adjust expectations: you now receive pure `f64`.
6. Update tests: assertions no longer call `.value()`.
7. (New) Use either `v * s` or `s * v` freely; both directions supported.

## Example Before / After
```rust
// Before
let v = Vector3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));
let len = v.norm().value();
let scaled = v * Scalar::new(0.5);
assert_eq!(v.x().value(), 1.0);

// After
let v = Vector3D::from_f64(1.0, 2.0, 3.0);
let len = v.norm();
let scaled = 0.5 * v.clone(); // reverse mul also allowed
assert_eq!(v.x(), 1.0);
```

## Performance Considerations
- Fewer temporary `Scalar` objects created in hot loops.
- Potential for future SIMD: contiguous `[f64;N]` layout.
- Eliminates virtual dispatch or wrapper unwrapping in simple arithmetic.

## Downstream Effects
- Any crate depending on previous `Scalar` returns must be recompiled and updated.
- Generic code relying on `Mul<Scalar>` will need a bound change or conversion to `Mul<f64>`.
- Reverse multiplication simplifies generic numeric expressions (e.g., scalars on left produced by literals or function returns).

## Direction & Future Cleanup
- Plan to remove `CadVector` & related deprecated APIs once external codebases migrate.
- Introduce optional reverse multiplication (`f64 * Vector3D`) if ergonomics needed. (DONE)
- Audit tolerance logic for potential `f64` inlining and constexpr thresholds.

## Test Updates
- Updated all vector and primitive tests to assert on raw `f64`.
- Ensured `geo_core` tests pass (44 tests) post-migration.
- Added tests for reverse scalar multiplication (owned and reference forms; zero vector cases) on 2025-10-05.

## Checklist When Migrating External Code
- [ ] Remove `.value()` after vector method calls
- [ ] Replace Scalar-based constructors with `from_f64`
- [ ] Swap `v * Scalar::new(k)` to `v * k`
- [ ] Adjust trait bounds expecting `Mul<Scalar>`
- [ ] Update normalization & dot product assertions
- [ ] Replace deprecated `Arc2D::new` with `new_f64` + `Angle`
- [ ] (Optional) Refactor code to use `s * v` form where it improves readability

## Open Items
- Decide on removal timing for deprecated CAD wrappers
- Provide changelog entry for workspace root (pending)
- Potential macro helpers for constructing vectors directly from literals

## File Renames (Geometry2D)
As part of the ongoing geometry2d consolidation, the file `geo_primitives/src/geometry2d/circle2d.rs` has been renamed to `circle.rs` and the module path updated:

```rust
// Before
pub mod circle2d;
pub use circle2d::Circle2D;

// After
pub mod circle;
pub use circle::Circle2D;
```

No public type name change (still `Circle2D`), so downstream code importing `Circle2D` via the crate root or `geometry2d::Circle2D` remains unaffected. Only code referring explicitly to the old module path `geometry2d::circle2d::Circle2D` must update to `geometry2d::circle::Circle2D` (rare; none inside workspace after refactor).

Documented here to avoid confusion during future diffs and blame history searches.

## Geometry2D Primitive F64 Refactor (Line / Ray / InfiniteLine)
Date: 2025-10-05

Refactored `geometry2d::{Line2D, Ray2D, InfiniteLine2D}` to construct direction vectors using `Vector2D::new(f64,f64)` directly instead of wrapping `Scalar`.

Changes:
- Replaced all `Vector2D::new(Scalar::new(dx), Scalar::new(dy))` with `Vector2D::new(dx, dy)`.
- Removed `.value()` accesses on vector component usage within geometry2d primitives (kept for `Point2D` because points still store `Scalar`).
- Simplified zero-length direction checks to squared-length comparisons to avoid unnecessary `sqrt`.
- Replaced cross product tolerance check `cross.tolerant_eq(&Scalar::new(0.0), tol)` with `cross.abs() <= tol.linear`.
- Added a minimal `Direction2D` implementation (unit-length invariant) and exposed via `geometry2d::Direction2D`.

Backward Compatibility:
- Deprecated Scalar-parameter methods remain (`evaluate(Scalar)`, etc.) for now; planned removal in next pre-release cycle.
- Public type names unchanged; only internal construction patterns differ.

Action for Downstream Code:
- If constructing direction vectors manually, prefer `Vector2D::new(x,y)` and `Direction2D::from_f64(x,y)`.
- Remove obsolete `.value()` after vector accessor or dot/cross results.

## Geometry3D Primitive F64 Refactor (Line3D / Ray3D / InfiniteLine3D)
Date: 2025-10-05

Analogous changes applied to 3D counterparts:
- Replaced `Vector3D::new(Scalar::new(x), ...)` with `Vector3D::new(x, y, z)` in constructors and directional computations.
- Eliminated `.value()` calls on vector components; kept for point coordinate extraction only.
- Zero-length direction checks now use squared length vs `tolerance.linear^2`.
- Cross product magnitude checks converted to squared magnitude to avoid `sqrt` in Line3D containment tests.
- No public API name changes; deprecated Scalar-parameter methods remain temporarily.

Downstream Migration Steps:
- Search for any remaining `Vector3D::new(Scalar::new(` patterns and replace with direct f64 constructors.
- Remove `.value()` on vector component access in custom code.

Performance Note: Avoiding square roots in zero / perpendicular checks reduces overhead in tight intersection loops.

## Reverse Scalar Multiplication Addition
Date: 2025-10-05

Implemented reverse scalar multiplication for both 2D and 3D vectors:
```rust
impl std::ops::Mul<Vector2D> for f64 { /* ... */ }
impl<'a> std::ops::Mul<&'a Vector2D> for f64 { /* ... */ }
impl std::ops::Mul<Vector3D> for f64 { /* ... */ }
impl<'a> std::ops::Mul<&'a Vector3D> for f64 { /* ... */ }
```

Motivation:
- Allow symmetric expressions (`k * v` and `v * k`) especially in generic numeric code and builder-style APIs.
- Improve readability when literal or computed scalar naturally precedes the vector.

Testing:
- Added unit tests covering owned vectors, reference forms, and zero-vector scaling for both 2D and 3D.

Backward Compatibility:
- Pure additive ergonomic feature; no breaking change.
- Encourages downstream code simplification without forcing edits.

---
For questions or to propose additional helpers for migration, open an issue referencing this file.
