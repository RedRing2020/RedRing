# Phase C Initialization Checklist

**Date**: 2026-03-20  
**Branch**: `issue-318-phase-c-collision-intersection`  
**Status**: 初期化完了 / 段階移行開始

## Pre-Start Validation ✅

- [x] Phase B マージ確認 (PR #346)
  - Commit: `42be51f`
  - develop ブランチに統合完了

- [x] Phase C ブランチ作成
  - `issue-318-phase-c-collision-intersection` from develop

- [ ] ビルド・テスト検証
  - [x] `cargo check -p geo_algorithms`
  - [ ] `cargo test --workspace`
  - [ ] `cargo clippy -- -D warnings`

- [ ] アーキテクチャ検証
  - [x] `./scripts/check_architecture_dependencies_simple.ps1`
  - `./scripts/check_issue_doc_archive.ps1`

---

## Codebase Snapshot

### Collision Files (geo_primitives)
```
arc_2d_collision.rs
arc_3d_collision.rs
circle_2d_collision.rs
circle_3d_collision.rs
conical_solid_3d_collision.rs
conical_surface_3d_collision.rs
cylindrical_solid_3d_collision.rs
cylindrical_surface_3d_collision.rs
ellipse_2d_collision.rs
ellipse_3d_collision.rs
ellipse_arc_2d_collision.rs
ellipse_arc_3d_collision.rs
ellipsoidal_solid_3d_collision.rs
line_segment_2d_collision.rs
line_segment_3d_collision.rs
plane_3d_collision.rs
ray_2d_collision.rs
ray_3d_collision.rs
rect_2d_collision.rs
sphere_3d_collision.rs
toroidal_solid_3d_collision.rs
triangular_prism_3d_collision.rs
triangle_2d_collision.rs
triangle_3d_collision.rs
```

### Intersection Files (geo_primitives)
```
arc_2d_intersection.rs
arc_3d_intersection.rs
circle_2d_intersection.rs
circle_3d_intersection.rs
conical_solid_3d_intersection.rs
conical_surface_3d_intersection.rs
cylindrical_solid_3d_intersection.rs
cylindrical_surface_3d_intersection.rs
ellipse_2d_intersection.rs
ellipse_3d_intersection.rs
ellipse_arc_2d_intersection.rs
ellipse_arc_3d_intersection.rs
ellipsoidal_solid_3d_intersection.rs
line_segment_2d_intersection.rs
line_segment_3d_intersection.rs
plane_3d_intersection.rs
ray_2d_intersection.rs
ray_3d_intersection.rs
rect_2d_intersection.rs
sphere_3d_intersection.rs
toroidal_solid_3d_intersection.rs
triangular_prism_3d_intersection.rs
triangle_2d_intersection.rs
triangle_3d_intersection.rs
```

**Total**: 28 collision + 28 intersection = 56 files to be refactored

---

## Trait Locations (geo_contracts)

### Collision Traits
- `Arc2DCollision<T>`
- `Circle2DCollision<T>`
- `Ellipse2DCollision<T>`
- `EllipseArc2DCollision<T>`
- `LineSegment2DCollision<T>`
- `Ray2DCollision<T>`
- `Rect2DCollision<T>`
- `Triangle2DCollision<T>`
- (3D traits: to be confirmed in geo_contracts)

### Intersection Traits
- Similar structure with `Intersection` suffix
- Point-return semantics vs boolean

---

## Step 1 Focus: 2D Collision/Intersection Files

### Source Files to Consolidate
```
model/geo_primitives/src/
  ├── arc_2d_collision.rs
  ├── arc_2d_intersection.rs
  ├── circle_2d_collision.rs
  ├── circle_2d_intersection.rs
  ├── ellipse_2d_collision.rs
  ├── ellipse_2d_intersection.rs
  ├── ellipse_arc_2d_collision.rs
  ├── ellipse_arc_2d_intersection.rs
  ├── line_segment_2d_collision.rs
  ├── line_segment_2d_intersection.rs
  ├── ray_2d_collision.rs
  ├── ray_2d_intersection.rs
  ├── rect_2d_collision.rs
  ├── rect_2d_intersection.rs
  ├── triangle_2d_collision.rs
  └── triangle_2d_intersection.rs
```

### Design for geo_algorithms
```
model/geo_algorithms/src/collision/
  ├── mod.rs (pub mod 2d_primitives; pub mod 3d_primitives; etc.)
  └── 2d_primitives.rs
      ├── Arc2D × {Arc2D, Circle2D, LineSegment2D, Ray2D, ...}
      ├── Circle2D × {...}
      ├── (consolidate all 2D shape-pair collision logic)

model/geo_algorithms/src/intersection/
  ├── mod.rs
  └── 2d_primitives.rs
      ├── Arc2D × {Arc2D, Circle2D, LineSegment2D, Ray2D, ...}
      ├── Circle2D × {...}
      ├── (consolidate all 2D shape-pair intersection logic)
```

---

## Next Action: Step 1 の正面 API 化

段階移行として次を進める:

1. `geo_algorithms` 側から 2D / 3D collision・intersection free-function を再エクスポートし、呼び出し側の参照先を集約する

2. 2D 実装のうち `geo_algorithms` に移せているロジックを優先して呼び出し側を切り替える

3. 3D はラッパー層を維持したまま正面 API だけ先に `geo_algorithms` へ寄せる

4. 互換維持のため `geo_primitives` 側の旧 impl は Cleanup フェーズまで残置する

---

*Created: 2026-03-20*
