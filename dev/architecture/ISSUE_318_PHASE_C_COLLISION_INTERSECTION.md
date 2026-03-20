# Issue #318 Phase C: Collision/Intersection Trait Migration

**Status**: Step 1/2 部分移行中 (2026-03-20)  
**Related Issues**: #318 (Parent), #347 (Phase C Overview), #350, #348, #349, #351  
**Branch**: `issue-318-phase-c-collision-intersection`

## 概要

Phase C は、`geo_primitives` に実装されている衝突判定（collision）・交差判定（intersection）機能を `geo_algorithms` に統合・再構成するフェーズです。

### Phase B の成果（前提条件）
- ✅ 全 shape trait を `geo_contracts` に統一
- ✅ `geo_foundation` 依存を削除（geo_primitives）
- ✅ 定数参照を `analysis` に一本化
- ✅ workspace regression test 全パス- ✅ `geo_foundation` 廃止完了（2026-03-20）
---

## Phase C の目標

### 1. 衝突判定・交差判定実装の責務再整理
- **geo_primitives**: 単一形状（各shape の定義と基本属性）のみに限定
- **geo_algorithms**: 形状ペア操作（collision/intersection）の一元管理

### 2. Foundation Pattern の完全遵守
- Trait 定義: `geo_contracts`
- Trait 実装：`geo_algorithms`（複合形状操作）
- Constants: `analysis`

### 3. 実装の体系化
```
geo_algorithms/
  ├── src/collision/
  │   ├── mod.rs
  │   ├── 2d_primitives.rs      ← 2D shape-pair collision
  │   ├── 3d_primitives.rs      ← 3D shape-pair collision
  │   ├── nurbs_primitives.rs   ← NURBS × Primitive collision
  │   └── ...
  └── src/intersection/
      ├── mod.rs
      ├── 2d_primitives.rs      ← 2D shape-pair intersection
      ├── 3d_primitives.rs      ← 3D shape-pair intersection
      ├── nurbs_primitives.rs   ├── NURBS × Primitive intersection
      └── ...
```

---

## ワークアイテム

### Step 1: 2D Collision/Intersection (#350)
- **Scope**: arc_2d, circle_2d, ellipse_2d, line_segment_2d, ray_2d, triangle_2d
- **Current**: `geo_algorithms` 側に free-function ベースの受け皿を配置済み。今後はこのモジュールを正面 API とし、`geo_primitives` 側は段階的に縮退する。
- **Task**:
  1. geo_algorithms/src/collision/2d_primitives.rs を設計
  2. 既存の geo_primitives/*_collision.rs から実装を抽出
  3. geo_algorithms に trait impl を再配置
  4. geo_primitives から古い実装を削除
  5. `cargo test --workspace` で回帰テスト

### Step 2: 3D Collision/Intersection (#348)
- **Scope**: plane_3d, ray_3d, line_segment_3d, sphere_3d, cone_3d, cylinder_3d, torus_3d, ellipsoid_3d
- **Current**: `geo_algorithms` 側に薄いラッパーを配置済み。3D は当面この層を正面 API とし、実体移設は段階実施とする。
- **Task**:
  1. geo_algorithms/src/collision/3d_primitives.rs を設計
  2. Step 1 のパターンに従う
  3. 複雑な交差判定（ray-sphere など）の実装を確認
  4. workspace 全体の consistency を検証

### Step 3: NURBS/Primitive Mixed (#349)
- **Scope**: NURBS surface × Primitive collision/intersection
- **Task**:
  1. nurbs_primitives.rs の責務境界を明確化
  2. Adaptive tessellation との統合確認
  3. Primitive-only 実装との重複排除

### Step 4: Cleanup & Tests (#351)
- **Scope**: 旧実装削除、test suite 統合
- **Current**: 完了済み。`geo_foundation` 廃止完了（2026-03-20）。`geo_primitives` 东の collision / intersection 実装は削除情場に応じて段阶的に粗処理中。
- **Task**:
  1. geo_primitives から古い collision/intersection ファイルを削除
  2. geo_algorithms に統合テストスイートを作成
  3. 検索確認：orphaned 定義がないか検証
  4. PR review 前の最終 regression test

---

## 実装パターン（Phase B から継承）

### Import 構成（Phase C 確定版）
```rust
// geo_algorithms/src/collision/2d_primitives.rs

use geo_contracts::{
    Arc2DCollision, Circle2DCollision,
    Ellipse2DCollision, // trait definitions
};
use geo_primitives::{Arc2D, Circle2D, Ellipse2D}; // shape impls
use analysis::GEOMETRIC_DISTANCE_TOLERANCE; // constants
```

### Trait Implementation Pattern
```rust
impl<T: Scalar> SomeCollisionTrait<T> for SomeShapePair<T> {
    fn intersects(&self) -> bool {
        // implementation
    }
}
```

---

## 依存関係面での変化

### Before (Current State)
```
geo_primitives → collision/intersection impl stored locally
geo_algorithms → references geo_primitives
```

### After (Target State)
```
geo_algorithms → consolidates all collision/intersection impl
geo_primitives → clean shape definitions only
```

### Transitional State (Current)
```
geo_algorithms → collision/intersection の正面 API
geo_primitives → 互換維持のため旧 impl を一時保持
```

### Allowed Dependency (Confirmed)
- `geo_contracts` → `geo_commons` ✅ (決定済み)

---

## 検証チェックリスト

各ステップ完了時に以下を確認：

- [ ] `cargo check --workspace` 成功
- [ ] `cargo test --workspace` 全パス
- [ ] `cargo clippy -- -D warnings` 違反なし
- [ ] `cargo fmt --check` フォーマット OK
- [ ] `./scripts/check_architecture_dependencies_simple.ps1` 成功
- [ ] `./scripts/check_issue_doc_archive.ps1` 成功

### 2026-03-20 確認済み

- [x] `cargo check -p geo_algorithms` 成功
- [x] `./scripts/check_architecture_dependencies_simple.ps1` 成功
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace`
- [ ] `./scripts/check_issue_doc_archive.ps1`

---

## Notes

### Collision trait locations (geo_contracts 確認)
- Arc2DCollision, Circle2DCollision, Ellipse2DCollision
- Line segment/Ray 3D trait TBD
- Sphere, Cone, Cylinder, Torus, Ellipsoid collision traits in geo_contracts

### Intersection trait locations (geo_contracts 確認)
- Similar structure to collision
- Complex geometries (ray-surface, etc.) may need custom impl pattern

### Test organization
- geo_primitives: shape definition + basic property tests のみ
- geo_algorithms: collision/intersection テスト suite は geo_algorithms/tests/ に統合

---

## Timeline

```
[Phase B - Completed] → [Phase C Start] → Step 1→2→3→4 → [Phase D Planning]
```

**Estimated Duration**: 3-4 weeks (4 sequential subtasks)

---

*Last Updated: 2026-03-20*
