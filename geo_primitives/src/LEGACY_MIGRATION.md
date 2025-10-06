# Legacy Primitives Migration Guide

The legacy `geometry3d` module has been removed. All Legacy* types have been replaced with f64 canonical types.

## Migration Mapping

| Old Legacy Type | New f64 Canonical Type | Alias |
|----------------|----------------------|-------|
| `LegacyDirection3D` | `FDirection3` | `Direction3D` |
| `LegacyLineSegment3D` | `FLineSegment3` | `LineSegment3D` |
| `LegacyPlane` | `FPlane` | `Plane` |
| `LegacyCircle3D` | `FCircle3` | `Circle3D` |

## Code Migration Examples

### Before (Legacy)
```rust
use geo_primitives::geometry3d::{LegacyDirection3D, LegacyPlane};

let dir = LegacyDirection3D::from_vector(vec, &context)?;
let plane = LegacyPlane::from_point_and_normal(origin, dir);
```

### After (f64 Canonical)
```rust
use geo_primitives::{Direction3D, Plane};
// or use geo_primitives::f64geom::{FDirection3, FPlane};

let dir = Direction3D::from_vector(vec)?;  // No tolerance context needed
let plane = Plane::from_point_and_normal(origin, dir);
```

## Key Differences
1. **No ToleranceContext**: f64 types use fixed precision internally
2. **Simplified constructors**: Less boilerplate, more direct
3. **Better performance**: Optimized for f64 without generic overhead
4. **Type aliases**: Use familiar names (`Direction3D`) for new implementations

## Migration Steps
1. Replace `Legacy*` imports with canonical types
2. Remove `ToleranceContext` parameters where no longer needed
3. Update constructor calls to new API
4. Use type aliases for backward compatibility

The legacy types were deprecated and have been completely removed to simplify the codebase and improve performance.