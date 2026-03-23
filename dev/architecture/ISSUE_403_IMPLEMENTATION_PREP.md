# Issue #403 実装手順ドキュメント

**Issue**: [#403: [Distance][Phase2] NURBS × Primitives 距離演算の拡張集約（geo_algorithms）](https://github.com/RedRing2020/RedRing/issues/403)

**対応開始日**: 2026年3月23日

**最終更新**: 2026年3月23日

---

## 📋 目的再確認

#342 Phase1 で line/line と NurbsCurve3D-point の distance 実装を `geo_algorithms::distance` モジュールへ集約した。
Phase2 では、NURBS × Primitives の distance 対象範囲を拡大し、**対称性の統一** と **fallible API の明示** を通じて、distance 公開面の責務を整理する。

---

## 🎯 スコープ確定

### 優先実装対象: NurbsCurve3D × 8種 Primitive

| 対象ペア | 種類 | 既存実装 | ファイル | 説明 |
|---------|------|--------|---------|-----|
| NurbsCurve3D - LineSegment3D | Line | ✓ | collision/primitive_nurbs.rs | 線分への距離 |
| NurbsCurve3D - Ray3D | Line | ✓ | collision/primitive_nurbs.rs | 半直線への距離 |
| NurbsCurve3D - InfiniteLine3D | Line | ✓ | collision/primitive_nurbs.rs | 無限直線への距離 |
| NurbsCurve3D - Circle3D | Circle | ✓ | collision/primitive_nurbs.rs | 円への距離 |
| NurbsCurve3D - Plane3D | Plane | ✓ | collision/primitive_nurbs.rs | 平面への距離 |
| NurbsCurve3D - SphericalSolid3D | Solid | ✓ | collision/primitive_nurbs.rs | 球への距離 |
| NurbsCurve3D - EllipsoidalSolid3D | Solid | ✓ | collision/primitive_nurbs.rs | 楕円体への距離 |
| NurbsCurve3D - CylindricalSolid3D | Solid | ✓ | collision/primitive_nurbs.rs | 円柱への距離 |

### 次点実装対象: NurbsSurface3D × 9種 Primitive

| 対象ペア | 種類 | 既存実装 | ファイル | 説明 |
|---------|------|--------|---------|-----|
| NurbsSurface3D - Point3D | Point | ✓ | collision/primitive_nurbs_surface.rs | 点への距離 |
| NurbsSurface3D - LineSegment3D | Line | ✓ | collision/primitive_nurbs_surface.rs | 線分への距離 |
| NurbsSurface3D - Ray3D | Line | ✓ | collision/primitive_nurbs_surface.rs | 半直線への距離 |
| NurbsSurface3D - InfiniteLine3D | Line | ✓ | collision/primitive_nurbs_surface.rs | 無限直線への距離 |
| NurbsSurface3D - Circle3D | Circle | ✓ | collision/primitive_nurbs_surface.rs | 円への距離 |
| NurbsSurface3D - Plane3D | Plane | ✓ | collision/primitive_nurbs_surface.rs | 平面への距離 |
| NurbsSurface3D - SphericalSolid3D | Solid | ✓ | collision/primitive_nurbs_surface.rs | 球への距離 |
| NurbsSurface3D - EllipsoidalSolid3D | Solid | ✓ | collision/primitive_nurbs_surface.rs | 楕円体への距離 |
| NurbsSurface3D - CylindricalSolid3D | Solid | ✓ | collision/primitive_nurbs_surface.rs | 円柱への距離 |

**計**: NurbsCurve3D 8 pairs + NurbsSurface3D 9 pairs = **17 pairs**

---

## 📌 着手前チェック（必須）

実装開始 **前に** 以下を確認し、全て ✓ チェック完了時点で開始。

### P2-1: #342 Phase1 が develop に反映済み

**確認項目**:
- [x] `model/geo_algorithms/src/distance/primitive_3d.rs` に `infinite_line3d_infinite_line3d_distance`, `nurbscurve3d_point3d_distance`, `nurbscurve3d_point3d_try_distance` が存在
- [x] `model/geo_algorithms/tests/coverage_matrix_engine.rs` に Phase1 対応の 3 MatrixEntry が存在
- [x] `cargo test -p geo_algorithms` が成功（169 tests 以上）

**確認結果** (2026年3月23日実施):
- ✅ `nurbscurve3d_point3d`: 8 matches (関数定義+テスト)
- ✅ `3d:nurbscurve-point:distance`: coverage matrix 確認
- ✅ `infinite_line3d_infinite_line3d_distance`: 3 matches
- ✅ `cargo test -p geo_algorithms`: 169 tests passed

**確認コマンド**:
```bash
cargo test -p geo_algorithms --quiet
grep "nurbscurve3d_point3d" model/geo_algorithms/src/distance/primitive_3d.rs
grep "3d:nurbscurve-point:distance" model/geo_algorithms/tests/coverage_matrix_engine.rs
```

### P2-2: 対称性ルールが明文化済み

**確認項目**:
- [x] 実装ガイド（本ドキュメント「実装ガイド」節）が対称性パターンを明示
- [x] 正規方向の完全実装 ← 既存 collision 実装を参照または再利用
- [x] 逆方向は `pub fn typeB_typeA_distance(...) { typeA_typeB_distance(...) }` で委譲

**確認結果** (2026年3月23日実施):
- ✅ 実装ガイド節 2「対称性パターン」に A/B/C の3パターンを明文化
- ✅ Phase1 実装で `point3d_nurbscurve3d_distance` が委譲パターンを実装

**パターン例** (Phase1 から継承):
```rust
// 正規方向: 完全実装
pub fn nurbscurve3d_point3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    point: &Point3D<T>,
) -> T { /* ... */ }

// 逆方向: 委譲
pub fn point3d_nurbscurve3d_distance<T: Scalar>(
    point: &Point3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_point3d_distance(curve, point)
}
```

### P2-3: fallible API シグネチャ方針が決定済み

**確認項目**:
- [x] `DistanceConvergenceError<T>` 列挙値 (InvalidInitialization, NotConverged, NumericalFailure) が定義済み
- [x] `nurbscurve3d_point3d_try_distance` 形式（`try_` prefix + Result 返却）が Phase1 で実装済み
- [x] サンプル数既定値（64 など）が決定済み
- [x] 失敗条件（サンプル数 0 → InvalidInitialization など）が明文化済み

**確認結果** (2026年3月23日実施):
- ✅ `DistanceConvergenceError<T>` が geo_contracts で定義済み
- ✅ `nurbscurve3d_point3d_try_distance` が primitive_3d.rs に実装済み
- ✅ max_samples デフォルト値 64 を使用
- ✅ 失敗条件テストが 5 個実装済み

**既定値の確認**:
```bash
grep -A 5 "nurbscurve3d_point3d_try_distance<" model/geo_algorithms/src/distance/primitive_3d.rs
```

### P2-4: coverage matrix 追加方針が決定済み

**確認項目**:
- [x] Phase2 で追加する 17 MatrixEntry の命名規則が決定済み
- [x] 対称ペア（A-B, B-A）の delegation パターンが記述済み
- [x] `tests/coverage_matrix_engine.rs` への登録箇所が特定済み

**確認結果** (2026年3月23日実施):
- ✅ 命名規則確定: `3d:{shape}-{shape}:distance` (Phase1 実装例参照)
- ✅ Delegation パターン: reverse 関数が regular 関数を呼び出し
- ✅ MatrixEntry 登録位置: coverage_matrix_engine.rs 内に P1 entries が存在

**命名規則**（Phase1 から継承）:
- Regular: `3d:nurbscurve-linesegment:distance`
- Reverse: `3d:linesegment-nurbscurve:distance`
- 逆方向は `delegate_to: "3d:nurbscurve-linesegment:distance"` で参照

### P2-5: アーキテクチャ依存性が問題なし

**確認項目**:
- [x] `geo_algorithms` → `geo_nurbs`, `geo_primitives`, `geo_contracts` への依存が許可範囲内
- [x] `geo_algorithms` 実装ファイル内での `use geo_primitives::*` 直接 import が禁止範囲でない（再エクスポート経由を使用）
- [x] `./scripts/check_architecture_dependencies_simple.ps1` が現状で PASS

**確認結果** (2026年3月23日実施):
- ✅ Dependency check: SUCCESS
  - View -> ViewModel -> Model direction maintained
  - Model layer naming rules followed
  - No forbidden dependencies detected

**確認コマンド**:
```bash
./scripts/check_architecture_dependencies_simple.ps1
```

---

## 🛠️ 実装ガイド

### 1. 実装ファイル構成

```
model/geo_algorithms/src/distance/
  ├── mod.rs                 (public entrypoint 宣言)
  ├── primitive_3d.rs        (Phase1 + Phase2 NurbsCurve3D/NurbsSurface3D entrypoint)
  └── (既存: primitive.rs などと分離)

model/geo_algorithms/tests/
  └── coverage_matrix_engine.rs (MatrixEntry追加: 17 entries)
```

### 2. 対称性パターン（重要）

**A. 計算対象が NURBS（正規方向）**

```rust
pub fn nurbscurve3d_linesegment3d_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
) -> T {
    // 既存 collision::primitive_nurbs ロジックを呼び出すか、
    // または新たに実装
    // ...
}
```

**B. 計算対象が Primitive（逆方向）**

```rust
pub fn linesegment3d_nurbscurve3d_distance<T: Scalar>(
    segment: &LineSegment3D<T>,
    curve: &NurbsCurve3D<T>,
) -> T {
    nurbscurve3d_linesegment3d_distance(curve, segment)
}
```

**C. Fallible variant (収束計算が必要な場合)**

```rust
pub fn nurbscurve3d_linesegment3d_try_distance<T: Scalar>(
    curve: &NurbsCurve3D<T>,
    segment: &LineSegment3D<T>,
    max_samples: usize,
) -> Result<T, DistanceConvergenceError<T>> {
    if max_samples == 0 {
        return Err(DistanceConvergenceError::InvalidInitialization);
    }
    // ... 収束計算実装
}
```

### 3. 各ペア実装順序（提案）

**Phase2-1（最初）: NurbsCurve3D - LineSegment3D**
- 理由: 実装が最も単純、既存 collision ロジック明確
- ファイル: `model/geo_algorithms/src/distance/primitive_3d.rs`
- 新規関数: 3 個（regular, reverse, try-variant）
- 新規テスト: 5～10 個（Normal, Parallel, Degenerate など）

**Phase2-2: NurbsCurve3D - Ray3D**

**Phase2-3: NurbsCurve3D - InfiniteLine3D**

**Phase2-4: NurbsCurve3D - Circle3D**

... 以降、Plane, Solids と進める

**Phase2-Nx: NurbsSurface3D × 9 pairs**
- NurbsCurve3D 系の実装後に开始

### 4. 既存実装の活用戦略

**A. collision モジュールの distance ロジックを参照**

```java
// collision/primitive_nurbs.rs の約19個の distance関数から
// 計算コア（サンプリング・最少距離検索など）を理解してコピーまたは引用
```

**B. 完全移植か参照か判断**

| 選択肢 | 利点 | 欠点 |
|------|-----|-----|
| 移植: distance → collision 委譲 | 責務明確、DRY | collision との往来 |
| 参照: distance で独立実装 | 完全独立、責務完全分離 | コード重複 |
| ハイブリッド | 共有ユーティリティ化 | 実装複雑 |

**現段階での推奨**: 参照して distance 側で独立実装し、長期的には共有ユーティリティ化を検討

### 5. テスト方針

**各ペア対応時の最小テスト**:

```rust
#[test]
fn test_nurbscurve3d_linesegment3d_distance_normal() {
    // Normal case: 曲線と線分が通常配置
}

#[test]
fn test_nurbscurve3d_linesegment3d_distance_zero() {
    // Zero distance: 交差・接触する場合
}

#[test]
fn test_nurbscurve3d_linesegment3d_distance_symmetric() {
    // Symmetry: reverse 関数と値が一致することを確認
}

#[test]
fn test_nurbscurve3d_linesegment3d_try_distance_convergence() {
    // Fallible: max_samples=0 で InvalidInitialization
    // max_samples=1 で NotConverged （または成功）
    // max_samples≥2 で成功
}
```

---

## ✅ 完了条件

Phase2 実装が以下を全て満たすことで完了と判定：

1. **実装箇所**:
   - [ ] `model/geo_algorithms/src/distance/primitive_3d.rs` に NurbsCurve3D 系 8 pairs の entrypoint（正規+逆+try）が追加
   - [ ] NurbsSurface3D 系については最低でも Point 対応が追加（他は次フェーズ許容）

2. **対称性の統一**:
   - [ ] 全 reverse 関数が正規方向へ委譲
   - [ ] coverage matrix に 17 entries（または実装分）が追加
   - [ ] matrix 検証テスト `cargo test coverage_matrix` が PASS

3. **fallible API と失敗モデル**:
   - [ ] `nurbscurve3d_*_try_distance` 形式の try-variant が最低 2～3 個以上
   - [ ] テストで InvalidInitialization, NotConverged, (NumericalFailure) の失敗条件が明示

4. **コード品質**:
   - [ ] `cargo clippy -p geo_algorithms -- -D warnings` → 0 warnings
   - [ ] `cargo fmt --all` → フォーマット完全準拠
   - [ ] `cargo test -p geo_algorithms` → 新規テスト全 PASS（3 before の 169 cases + Alpha新規追加）

5. **アーキテクチャ整合性**:
   - [ ] `./scripts/check_architecture_dependencies_simple.ps1` → SUCCESS
   - [ ] `geo_algorithms` import ルール準守（geo_primitives 再エクスポート経由のみ）

6. **ドキュメント更新**:
   - [ ] 本ドキュメントを archived に移動
   - [ ] Issue #403 body を最終確認チェック済みに更新（P2-1～P2-5 全チェック）
   - [ ] PR body に実装ペア一覧と coverage matrix 新規 entries を記載

---

## 📝 注意点・ガイダンス

### A. Generics 制約

NURBS distance はほぼ全て `T: Scalar` を必須とする。Curve/Surface の位置・方向計算が浮動小数演算であるため。

```rust
// ✓ OK
pub fn nurbscurve3d_point3d_distance<T: Scalar>(..

// ✗ NG (Scalar 不足)
pub fn nurbscurve3d_point3d_distance<T: Copy>(..
```

### B. Sample 数既定値

fallible API の `max_samples` 既定値を統一（Phase1 では 64）。テストで 0, 1, 2, 64 などを明示的に試すこと。

```rust
const DEFAULT_NURBS_DISTANCE_SAMPLES: usize = 64;
```

### C. 既存 collision::primitive_nurbs との重複

collision 側で同じ機能が残る場合:
- distance モジュール: 公開 entrypoint 層
- collision モジュール: 内部実装・テスト層

長期的には collision が distance を呼び出す方向へリファクタリングすることを検討。

### D. Error 型の一貫性

`DistanceConvergenceError` の定義を collision/distance で重複させない。
`geo_contracts::operations::DistanceConvergenceError` から import して使用。

```bash
grep -r "DistanceConvergenceError" geo_contracts/src/
```

---

## 📊 進捗トラッキング

本ドキュメント完成後、以下ステップで進行：

| Step | 作業内容 | 確認 |
|------|---------|-----|
| S1 | 着手前チェック P2-1～P2-5 全完了 | [x] 2026-03-23 完了 |
| S2 | Feature branch 作成: `feature/issue-403-phase2-nurbs-distance` | [x] 2026-03-23 完了 |
| S3 | NurbsCurve3D-LineSegment3D 実装+テスト | [x] 2026-03-23 完了 |
| S4 | cargo clippy/fmt/test 全パス | [x] 2026-03-23 完了 |
| S5 | coverage matrix 3 entries (regular/reverse/try) 追加 | [x] 2026-03-23 完了 |
| S6 | 以降ペア追加していく（S3～S5 繰り返し） | [ ] |
| S7 | architecture 依存チェック PASS | [x] 2026-03-23 完了 |
| S8 | PR 作成・マージ | [ ] |
| S9 | Issue #403 完了チェック・クローズ | [ ] |

---

## 🔗 参考資料

- Issue #403: https://github.com/RedRing2020/RedRing/issues/403
- Issue #342 (Phase1): https://github.com/RedRing2020/RedRing/issues/342
- Phase1 Prep Doc: `dev/architecture/ISSUE_342_IMPLEMENTATION_PREP.md`
- Distance Module: `model/geo_algorithms/src/distance/`
- Collision Module: `model/geo_algorithms/src/collision/primitive_nurbs.rs`
- Coverage Matrix: `model/geo_algorithms/tests/coverage_matrix_engine.rs`
- Module Structure Rules: `dev/architecture/GEO_ALGORITHMS_MODULE_STRUCTURE_RULES.md`

---

**ドキュメント作成日**: 2026年3月23日
**管理責任者**: RedRing開発チーム
**ステータス**: Phase2-4 完了（NurbsCurve3D-Circle3D 追加済み）- 次ステップ: S6 次ペア実装（NurbsCurve3D-Plane3D）
