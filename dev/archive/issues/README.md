# Archived Issue Documents

**作成日**: 2026年2月27日  
**目的**: クローズ済み Issue の設計/実装ドキュメントを保管し、機能単位で追跡しやすくする

---

## 運用ルール

- 本ディレクトリは、過去に作成された Issue 関連文書の保管場所として扱う
- 新規運用では、closed/open と連動した archive 移動を必須にしない
- 設計文書は archive 前提で命名せず、用途と内容が分かる名前で管理する

---

## 機能カテゴリ別インデックス

### Geometry / Solver

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #248 | CLOSED | primitive_nurbs の Newton 法重複実装の統合 | `dev/archive/issues/architecture/issue-248-newton-consolidation-design-archive-note.md` |

### View Refactoring（app/stage/render）

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #258 | CLOSED | View層の責務境界再整理（Phase 4-12） | `dev/archive/issues/architecture/issue-258-phase4-app-factory-facade-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 5 Debug） | `dev/archive/issues/architecture/issue-258-phase5-debug-scene-split-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 5 Manager） | `dev/archive/issues/architecture/issue-258-phase5-manager-loader-policy-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 6 Naming） | `dev/archive/issues/architecture/issue-258-phase6-naming-refinement-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 7 Render重複削減） | `dev/archive/issues/architecture/issue-258-phase7-render-duplication-reduction-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 8 Resource Factory） | `dev/archive/issues/architecture/issue-258-phase8-render-resource-factory-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 9 Quality） | `dev/archive/issues/architecture/issue-258-phase9-render-quality-hardening-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 10 Common Infra） | `dev/archive/issues/architecture/issue-258-phase10-stage-common-infra-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 11 Boundary） | `dev/archive/issues/architecture/issue-258-phase11-stage-boundary-factory-archive-note.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 12 Quality/Logging） | `dev/archive/issues/architecture/issue-258-phase12-stage-quality-logging-archive-note.md` |

### ViewModel Naming

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #281 | CLOSED | debug/sample 接頭辞ポリシー適用（NURBS） | `dev/archive/issues/architecture/issue-281-phase1-nurbs-naming-archive-note.md` |
| #281 | CLOSED | debug/sample 接頭辞ポリシー適用（Octree設定） | `dev/archive/issues/architecture/issue-281-phase2-octree-settings-naming-archive-note.md` |

### ViewModel Camera Split

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #284 | CLOSED | camera.rs 責務分割（Phase 1 Math） | `dev/archive/issues/architecture/issue-284-phase1-camera-math-extraction-archive-note.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 2 Projection） | `dev/archive/issues/architecture/issue-284-phase2-camera-projection-split-archive-note.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 3 Navigation） | `dev/archive/issues/architecture/issue-284-phase3-camera-navigation-split-archive-note.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 4 Presets） | `dev/archive/issues/architecture/issue-284-phase4-camera-presets-split-archive-note.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 5 Transition） | `dev/archive/issues/architecture/issue-284-phase5-camera-transition-split-archive-note.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 6 Tests） | `dev/archive/issues/architecture/issue-284-phase6-camera-tests-split-archive-note.md` |

### Foundation

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #170 | CLOSED | 衝突判定・交差判定の数値計算精度改善 | `dev/archive/issues/foundation/issue-170-collision-numerical-precision-archive-note.md` |
| #320 | CLOSED | geo_contracts 正規化と geo_foundation::commons 重複解消方針 | `dev/archive/issues/architecture/issue-320-contracts-commons-normalization-archive-note.md` |
