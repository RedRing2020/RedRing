# Geometry Core Migration (Vector & Point f64 + Primitive Extraction)# Vector f64 Migration Notes



## 1. BackgroundThis document summarizes the breaking changes introduced during the migration from `Scalar`-centric vector storage to native `f64` storage for `Vector2D` and `Vector3D`.

従来 `Vector2D/3D`, `Point2D/3D` は内部的に `Scalar` を保持していた。浮動小数点変換や `.value()` 呼び出しが多く、ホットパスでのオーバーヘッドとノイズが増大していたため、数値表現の単純化と API 明瞭化を目的に `f64` ベースへ移行した。

## Overview

---- Internal representation changed: `[Scalar; N]` -> `[f64; N]`.

## 2. Phase: Vector Migration to f64- Accessor methods (`x()`, `y()`, `z()`) now return `f64` directly.

- Core algebraic methods (`dot`, `norm`, etc.) now return `f64` instead of `Scalar`.

### Goals- Multiplication is now implemented for `Mul<f64>`; `Mul<Scalar>` is removed for vectors.

- `Vector2D/Vector3D` の内部表現を `Scalar` -> `[f64; N]` へ移行- Reverse scalar multiplication (`f64 * Vector2D/Vector3D`) IMPLEMENTED (2025-10-05) for ergonomic symmetry.

- 算術演算・アクセサの戻り値を `f64` 化- Many call sites no longer need `.value()` unwrap chains; remove them to compile.

- 逆方向スカラー乗算 `f64 * Vector` 追加

## Rationale

### Key Changes- Reduce heapless wrapper overhead in tight numeric loops (render + analysis crates).

- 構造体定義更新: `pub struct Vector3D { pub data: [f64; 3] }` (例)- Simplify interop with algorithms that already operate purely on `f64`.

- `x()/y()/z()` アクセサ: 直接 `f64` を返却- Prepare for future SIMD optimizations (contiguous plain `f64` storage).

- `norm()` / `dot()` / `cross()` が中間 `Scalar` 生成を行わない

- 実装インライン最適化: 多用メソッドに `#[inline(always)]`## API Changes (Breaking)

- 実装互換保持: 旧来 API を参照する下流コードのビルドを壊さないよう、一時的に拡張 trait で代替した箇所を順次除去| Before | After | Notes |

|--------|-------|-------|

### Reverse Scalar Multiplication| `v.x().value()` | `v.x()` | Accessors return `f64`.

```rust| `v.dot(&w).value()` | `v.dot(&w)` | Dot now returns `f64`.

impl Mul<Vector3D> for f64 { /* value * vector */ }| `v.norm().value()` | `v.norm()` | Norm returns `f64`.

```| `Vector::new([Scalar;N])` usage indirectly via `Vector3D::new(Scalar,...)` | `Vector3D::from_f64(x,y,z)` or `Vector3D::new([f64;3])` via trait in scope | Removed multi-Scalar constructor.

`Vector * f64` と対称性を確保し算術記述性を向上。| `v * Scalar::new(s)` | `v * s` | Implemented `Mul<f64>`.

| `Scalar::new(…) * v` (commutative patterns) | `s * v` | Reverse `Mul<f64>` now implemented (owned & &ref).

### Rationale| `CadVector::scale(f)` (Scalar wrapping) | `cad_vec.0.clone() * f` | Wrapper kept deprecated.

- 頻出線形代数カーネルのボイラープレート削減| `Direction3D::dot(&a,&b).value()` | `a.dot(&b)` | Dot returns `f64`.

- SIMD / auto-vectorization 最適化余地拡大

- API の一貫性 (全座標は常に f64)## Traits Updated

- `Vector<const D: usize>`: `dot(&self, &Self) -> f64`, `norm(&self) -> f64`.

---- `TolerantEq` implementation for vectors now compares raw `f64` components against `context.linear`.

## 3. Phase: Point Big Bang Migration

## Removed / Deprecated

### Approach- Implicit `Scalar` returns from vector operations.

ポイントは中間移行レイヤを設けず一括 (big bang) で `f64` 配列へ切り替え。理由:- Deprecated wrappers still present: `CadVector`, `CadDirection` (will be removed or type-aliased in a later sweep).

1. Point は不変参照経由利用が多く、順次移行より一括置換のほうが差分が単純- `Arc2D::new` (Scalar angles) is deprecated; use `Arc2D::new_f64(center, radius_f64, Angle, Angle)`.

2. 座標アクセス `.value()` ノイズを完全除去

3. 2D/3D 両方を同タイミングで統一し下位互換性の曖昧さを排除## Mechanical Migration Steps

1. Remove every `.value()` chained off vector accessors and algebraic results.

### Changes2. Replace `Vector3D::new(Scalar::new(x), …)` with `Vector3D::from_f64(x, …)`.

- `Point2D { data: [f64;2] }`, `Point3D { data: [f64;3] }`3. Change any `let s = v.dot(&w); s.value()` to just `let s = v.dot(&w);`.

- 生成/変換ヘルパを f64 前提に再定義4. Replace `v * Scalar::new(k)` with `v * k` (ensure `k: f64`).

- AABB / 距離計算コードを `.value()` 除去リライト5. For normalization comparisons, adjust expectations: you now receive pure `f64`.

6. Update tests: assertions no longer call `.value()`.

### Performance Note7. (New) Use either `v * s` or `s * v` freely; both directions supported.

中間 `Scalar` オブジェクト削減によりホットループ内割当とコピーが削減 (定量計測 TBD)。

## Example Before / After

---```rust

## 4. Primitive Extraction Phase P1 (2025-10-05)// Before

let v = Vector3D::new(Scalar::new(1.0), Scalar::new(2.0), Scalar::new(3.0));

### Goallet len = v.norm().value();

`geo_core` を最小数学コア (Point / Vector / Tolerance) に漸近させ、形状プリミティブを `geo_primitives` へ段階的に移動する最初のステップ。let scaled = v * Scalar::new(0.5);

assert_eq!(v.x().value(), 1.0);

### Scope (P1)

移動 (正準定義は `geo_primitives` 側):// After

- `LineSegment3D`let v = Vector3D::from_f64(1.0, 2.0, 3.0);

- `Plane`let len = v.norm();

- `Circle3D`let scaled = 0.5 * v.clone(); // reverse mul also allowed

- `Direction3D`assert_eq!(v.x(), 1.0);

```

`geo_core` には後方互換目的の deprecated コピーを一時残置:

- `LineSegment3D`## Performance Considerations

- `Plane`- Fewer temporary `Scalar` objects created in hot loops.

- `Direction3D`- Potential for future SIMD: contiguous `[f64;N]` layout.

- Eliminates virtual dispatch or wrapper unwrapping in simple arithmetic.

### Import Mapping

| Old (deprecated) | New Canonical | Downstream Effects |
|------------------|---------------|-------------------|
| Legacy geo_core imports | Use geo_primitives instead | Any crate depending on previous `Scalar` returns must be recompiled and updated. |
| LineSegment3D from geo_core | LineSegment3D from geo_primitives | Generic code relying on `Mul<Scalar>` will need a bound change or conversion to `Mul<f64>`. |
| Plane from geo_core | Plane from geo_primitives | Reverse multiplication simplifies generic numeric expressions (e.g., scalars on left produced by literals or function returns). |
| Direction3D from geo_core | Direction3D from geo_primitives | Direction is now implemented with proper normalization guarantees. |

## Direction & Future Cleanup

### Rationale- Plan to remove `CadVector` & related deprecated APIs once external codebases migrate.

1. 逆方向依存 (core -> primitives) を避け設計の矢印を単純化- Introduce optional reverse multiplication (`f64 * Vector3D`) if ergonomics needed. (DONE)

2. 一括大規模 rename のリスクを抑え、警告駆動で段階移行- Audit tolerance logic for potential `f64` inlining and constexpr thresholds.

3. 後続の曲面 / トポロジ層導入前に責務境界を鮮明化

## Test Updates

### Executed Actions- Updated all vector and primitive tests to assert on raw `f64`.

1. 3D geometry モジュール群を `geo_primitives/src/geometry3d` に追加- Ensured `geo_core` tests pass (44 tests) post-migration.

2. `lib.rs` で再エクスポート設定- Added tests for reverse scalar multiplication (owned and reference forms; zero vector cases) on 2025-10-05.

3. `geo_core` に deprecated 構造体を復元 (alias 方式は依存反転のため中止)

4. `Direction3D` に単位ベクトルコンストラクタ (unit_x / unit_y / unit_z) を追加## Checklist When Migrating External Code

- [ ] Remove `.value()` after vector method calls

### Deprecation Policy- [ ] Replace Scalar-based constructors with `from_f64`

- 猶予: 2 イテレーション (または次 minor) 後に削除 PR- [ ] Swap `v * Scalar::new(k)` to `v * k`

- CI TODO: 追加で `geo_core::(LineSegment3D|Plane|Direction3D)` を grep し失敗させる lint スクリプト導入予定- [ ] Adjust trait bounds expecting `Mul<Scalar>`

- [ ] Update normalization & dot product assertions

### Next Phase (P2)- [ ] Replace deprecated `Arc2D::new` with `new_f64` + `Angle`

- 既存コードの import を `geo_primitives` 版へ一括置換- [ ] (Optional) Refactor code to use `s * v` form where it improves readability

- Deprecated 定義削除

- `ParametricCurve3D` / `ParametricSurface` の所在再評価 (必要なら primitives または新 abstraction crate へ)## Open Items

- `Sphere` 他 3D primitive の移行検討- Decide on removal timing for deprecated CAD wrappers

- Provide changelog entry for workspace root (pending)

---- Potential macro helpers for constructing vectors directly from literals

## 5. Direction3D Relocation (Interim Notes)

## File Renames (Geometry2D)

### Canonical DefinitionAs part of the ongoing geometry2d consolidation, the file `geo_primitives/src/geometry2d/circle2d.rs` has been renamed to `circle.rs` and the module path updated:

`geo_primitives::Direction3D` : 正規化済み 3D ベクトルラッパ (内部 f64)。

```rust

### Migration Steps// Before

1. 旧 import を順次 `geo_primitives` に変更pub mod circle2d;

2. 正規化生成ユーティリティ (from_vector) を利用しゼロ長チェック (None) を明示pub use circle2d::Circle2D;

3. Tolerance 文脈 (`ToleranceContext`) 利用箇所はそのまま (契約不変)

// After

### Removal Checklistpub mod circle;

- [ ] 全 crate で `geo_core::Direction3D` import 0 件pub use circle::Circle2D;

- [ ] ビルド警告 0 (deprecated 関連)```

- [ ] ドキュメント更新反映 (本ファイル + API docs)

No public type name change (still `Circle2D`), so downstream code importing `Circle2D` via the crate root or `geometry2d::Circle2D` remains unaffected. Only code referring explicitly to the old module path `geometry2d::circle2d::Circle2D` must update to `geometry2d::circle::Circle2D` (rare; none inside workspace after refactor).

---

## 6. Outstanding / Follow-upsDocumented here to avoid confusion during future diffs and blame history searches.

| Item | Status |

|------|--------|## Geometry2D Primitive F64 Refactor (Line / Ray / InfiniteLine)

| Bulk import rewrite to primitives | Pending |Date: 2025-10-05

| Deprecated core primitives removal | Pending |

| Trait (`ParametricCurve3D/Surface`) relocation decision | Pending |Refactored `geometry2d::{Line2D, Ray2D, InfiniteLine2D}` to construct direction vectors using `Vector2D::new(f64,f64)` directly instead of wrapping `Scalar`.

| CI grep-based lint for deprecated imports | Planned |

| Performance micro-benchmark (pre/post f64) | Planned |Changes:

- Replaced all `Vector2D::new(Scalar::new(dx), Scalar::new(dy))` with `Vector2D::new(dx, dy)`.

---- Removed `.value()` accesses on vector component usage within geometry2d primitives (kept for `Point2D` because points still store `Scalar`).

## 7. Appendix: Design Principles Reinforced- Simplified zero-length direction checks to squared-length comparisons to avoid unnecessary `sqrt`.

- 型安全: Zero-vector 正規化を `Option` 経由で表現- Replaced cross product tolerance check `cross.tolerant_eq(&Scalar::new(0.0), tol)` with `cross.abs() <= tol.linear`.

- 責務分離: Core (最小算術) / Primitives (幾何要素) / Higher-level (analysis, algorithms)- Added a minimal `Direction2D` implementation (unit-length invariant) and exposed via `geometry2d::Direction2D`.

- 移行戦略: Big Bang (Point) vs Incremental (Primitives) の選択基準は影響半径と呼び出し頻度

Backward Compatibility:

End of document.- Deprecated Scalar-parameter methods remain (`evaluate(Scalar)`, etc.) for now; planned removal in next pre-release cycle.

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

---

## Point f64 Big Bang Migration (2025-10-05)

### Summary
`Point2D` / `Point3D` (geo_core) internal storage migrated from `Scalar` fields to plain `[f64; N]`. Wrapper primitives in `geo_primitives::geometry2d` updated (Batch A) to construct points via `Point2D::new(x, y)` directly. No transitional constructors maintained; this is an intentional breaking change to simplify downstream usage and remove pervasive `.value()` chains.

### Goals
1. Eliminate boilerplate `Scalar::new(x)` at every construction site.
2. Make point coordinate access symmetrical with vectors (both now yield raw `f64`).
3. Preserve unit / dimensional semantics only on returned metric quantities (distances, lengths) via `Scalar` wrapper, keeping algebraic positions as raw `f64`.

### Breaking API Changes
| Before | After | Notes |
|--------|-------|-------|
| `Point2D::new(Scalar::new(x), Scalar::new(y))` | `Point2D::new(x, y)` | New signature expects raw `f64`.
| `p.x().value()` | `p.x()` | Accessor now returns `f64`.
| `p.y().value()` | `p.y()` | Same as above.
| (geo_primitives) tests using `.value()` for points | Remove `.value()` | Mechanical removal.
| Mixed patterns `Vector2D::new(p.x().value(), p.y().value())` | `Vector2D::new(p.x(), p.y())` | Consistent f64 flow.

### Non-Breaking (Preserved) Semantics
| Aspect | Status | Reason |
|--------|--------|--------|
| Distance / length return type | `Scalar` | Maintain explicit unit semantics and tolerance logic coupling. |
| Angle representation in `Arc2D` | Still `Scalar` | Angular tolerance + future unit tagging. |
| Deprecated Scalar param methods (e.g. `evaluate(Scalar)`) | Temporarily retained | Gradual removal scheduling; callers migrate to `*_f64`. |

### Mechanical Migration Guide (External Code)
1. Grep for `Point2D::new(Scalar::new(` and replace with `Point2D::new(` using raw numbers.
2. Replace every `p.x().value()` / `p.y().value()` with `p.x()` / `p.y()`.
3. Update pattern matches or trait impls that assumed `x(): Scalar` to use raw `f64` (adjust generic bounds if any).
4. Adjust serialization / deserialization: if previously unwrapping `.value()`, remove the unwrap layer.
5. Re-run tests; compile errors will precisely point at any remaining `.value()` misuse.
6. Keep using `Scalar` for distances; do NOT pre-emptively refactor those unless moving to a pure `f64` metric design (not planned yet).

### Rationale
- Reduces allocation and wrapper churn in geometry-heavy loops and tessellation.
- Aligns point ergonomics with vectors after earlier vector refactor.
- Enables clearer future introduction of unit-tagged wrapper types selectively (e.g., `Length`, `Angle`) without blanket wrapping all coordinates.

### Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Silent logic changes if `.value()` removed incorrectly in arithmetic expecting `Scalar` | Potential type inference drift or different trait impl resolution | Compiler errors surface most cases; add targeted tests for critical algorithms. |
| Downstream crates expecting `Point2D::new(Scalar,Scalar)` fail | Hard compile break | Provide concise migration note & search/replace script. |
| Mixing old and new patterns in partially migrated branches | Inconsistent style / confusion | Enforce workspace-wide grep CI step (future) ensuring no `Point2D::new(Scalar::new` remain. |
| Overuse of raw `f64` leading to accidental unit confusion | Subtle correctness bugs | Keep metric-return functions using `Scalar`; consider future `#[must_use]` wrappers or type aliases for distances. |

### Test Strategy Update
- Geometry2D batch adapted: `line`, `ray`, `infinite_line`, `circle`, `point`, `arc` tests rewritten to directly assert on `f64` point accessors.
- Distance / radius / angle assertions still use `.value()` because those remain `Scalar` typed.
- Added reverse-mul vector tests previously; reused tolerance contexts unchanged (tolerance compares raw `f64`).
- Next planned phase: Geometry3D primitives (mirror mechanical changes). Add a smoke test validating cross-2D/3D API consistency (compile-time generic code if any).

### Transitional Helpers / Cleanup
- Verified no alternative legacy `Point2D::from_scalar` or similar constructors remain.
- Deprecated methods for Scalar params (e.g., `translate(Scalar,Scalar)`) intentionally kept; removal ticket to be opened post 0.N+1 pre-release.
- Add CI lint idea: deny `Scalar::new(` usage inside `Point2D::new(` call arguments (pattern no longer valid but future guard).

### Example Before / After (Line2D snippet)
```rust
// Before
let start = Point2D::new(Scalar::new(0.0), Scalar::new(0.0));
let end   = Point2D::new(Scalar::new(2.0), Scalar::new(4.0));
let line = Line2D::new(start, end);
let mid = line.midpoint();
assert_eq!(mid.x().value(), 1.0);

// After
let line = Line2D::new(Point2D::new(0.0, 0.0), Point2D::new(2.0, 4.0));
let mid = line.midpoint();
assert_eq!(mid.x(), 1.0);
```

### Workspace Status (Post Batch A)
| Area | Status |
|------|--------|
| geo_core Point2D/Point3D | Migrated to `[f64; N]` |
| geo_primitives geometry2d | Updated (all primary primitives) |
| geometry3d | Pending next batch |
| Distances/Angles | Still `Scalar` |
| Deprecated Scalar param methods | Present | 

### Follow-Up Tasks
- [ ] Batch B: geometry3d primitives & tests.
- [ ] Batch C: surface / higher-order primitives.
- [ ] Remove deprecated Scalar parameter overloads (schedule).
- [ ] Introduce compile-time detection (clippy lint / custom script) for obsolete patterns.

### Changelog Entry (Draft)
```
BREAKING: Point2D / Point3D constructors now take raw f64. Accessors return f64 (remove .value()). Distances still return Scalar. Update all call sites accordingly.
```

---

## Migration Quick Reference Table

| Pattern (Old) | Replacement | Notes |
|---------------|-------------|-------|
| `Point2D::new(Scalar::new(x), Scalar::new(y))` | `Point2D::new(x, y)` | Mandatory |
| `pt.x().value()` | `pt.x()` | Mandatory |
| `distance.value()` (where distance: Scalar) | `distance.value()` | No change (intentional) |
| `Vector2D::new(pt.x().value(), pt.y().value())` | `Vector2D::new(pt.x(), pt.y())` | Cleanup |
| `f64 * vector` | (same) | Newly supported reverse mul (already documented) |

---

End of Point Big Bang section.

## Core Role Realignment (2025-10-05)

### Summary
`geo_core` は幾何プリミティブを持たない「幾何計算中核」へ再定義され、f64 ベースの実体型は `geo_primitives` に統合されます。これにより依存方向を単純化し、ロバスト判定や許容誤差計算の再利用性を高めます。

### Layering (Target)
```
model / analysis  ->  geo_primitives  ->  geo_core
```

### Rationale
- 循環依存回避と責務境界の明確化
- f64 正準型の高速化・SIMD 最適化を primitives に集中
- tolerance / robust predicate は軽量核として他層へ提供

### Planned Phases
| Phase | 内容 | 状態 |
|-------|------|------|
| P0 | f64 Vector/Point Big Bang | 完了 |
| P1 | 3D プリミティブ抽出 (LineSegment3D / Plane / Circle3D / Direction3D) | 進行中 |
| P2 | f64 正準層 (f64geom) + 旧型 alias 化 | 着手 |
| P3 | legacy feature を介した旧実装削除 | 予定 |
| P4 | Point/Vector 最終配置 & trait 抽象判断 | 予定 |

### Migration Guidance
旧 geo_core からの LineSegment3D 等を利用している場合:
1. import を `geo_primitives::LineSegment3D` に変更
2. f64 アクセサ (x(), y(), z()) に `.value()` を付けない
3. tolerance 利用箇所はそのまま `geo_core::ToleranceContext`

### Trait Abstraction (Draft)
```rust
pub trait Vec3Like { fn x(&self)->f64; fn y(&self)->f64; fn z(&self)->f64; fn dot(&self,&Self)->f64; }
```
`geo_primitives::f64geom::FVector3` がこれを実装し、`geo_core` のジェネリックロバスト関数適用を検討。

### Deprecation Timeline (Indicative)
| 時期 | アクション |
|------|------------|
| 0.N.0 | f64geom 公開 / 旧 geometry3d 共存 |
| 0.N+1 | 旧型 -> type alias + deprecation (warn) |
| 0.N+2 | CI deny deprecated により旧名禁止 (legacy feature 限定) |
| 0.N+3 | legacy feature & 旧名完全削除 |

#### CI Strict Mode
`ci.yml` に `feature-set: strict` マトリクスを追加し、`RUSTFLAGS=-D deprecated` を設定。これにより将来の削除予定 API 利用を早期に検出可能。通常ビルド (`default`) は警告のみで継続し、移行期間の段階的修正を支援する。

### Risks & Mitigations
| リスク | 対応 |
|--------|------|
| 外部が旧 import 継続 | grep CI + alias deprecation |
| 二重名前空間混乱 (f64geom) | alias テーブル & ドキュメント整理 |
| 多精度要求 | trait 抽象 / 別 backend crate | 
| crate 増殖 | math 抽出は必要性発生時のみ |

### Open Action Items
- [ ] alias 導入と旧構造体 rename
- [ ] CI strict deprecated ジョブ追加
- [ ] trait プロトタイプ実装評価
- [ ] legacy 削除 Issue 作成

---
End of Core Role Realignment Section.
