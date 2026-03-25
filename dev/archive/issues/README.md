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
| #248 | CLOSED | primitive_nurbs の Newton 法重複実装の統合 | `dev/archive/issues/architecture/ISSUE_248_NEWTON_CONSOLIDATION_DESIGN.md` |

### View Refactoring（app/stage/render）

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #258 | CLOSED | View層の責務境界再整理（Phase 4-12） | `dev/archive/issues/architecture/ISSUE_258_PHASE4_APP_FACTORY_FACADE_DESIGN.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 5 Debug） | `dev/archive/issues/architecture/ISSUE_258_PHASE5_DEBUG_SCENE_SPLIT_DESIGN.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 5 Manager） | `dev/archive/issues/architecture/ISSUE_258_PHASE5_MANAGER_AND_LOADER_POLICY.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 6 Naming） | `dev/archive/issues/architecture/ISSUE_258_PHASE6_NAMING_REFINEMENT_PREP.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 7 Render重複削減） | `dev/archive/issues/architecture/ISSUE_258_PHASE7_RENDER_DUPLICATION_REDUCTION.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 8 Resource Factory） | `dev/archive/issues/architecture/ISSUE_258_PHASE8_RENDER_RESOURCE_FACTORY.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 9 Quality） | `dev/archive/issues/architecture/ISSUE_258_PHASE9_RENDER_QUALITY_HARDENING.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 10 Common Infra） | `dev/archive/issues/architecture/ISSUE_258_PHASE10_STAGE_COMMON_INFRA.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 11 Boundary） | `dev/archive/issues/architecture/ISSUE_258_PHASE11_STAGE_BOUNDARY_AND_FACTORY.md` |
| #258 | CLOSED | View層の責務境界再整理（Phase 12 Quality/Logging） | `dev/archive/issues/architecture/ISSUE_258_PHASE12_STAGE_QUALITY_AND_LOGGING.md` |

### ViewModel Naming

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #281 | CLOSED | debug/sample 接頭辞ポリシー適用（NURBS） | `dev/archive/issues/architecture/ISSUE_281_PHASE1_NURBS_NAMING.md` |
| #281 | CLOSED | debug/sample 接頭辞ポリシー適用（Octree設定） | `dev/archive/issues/architecture/ISSUE_281_PHASE2_OCTREE_SETTINGS_NAMING.md` |

### ViewModel Camera Split

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #284 | CLOSED | camera.rs 責務分割（Phase 1 Math） | `dev/archive/issues/architecture/ISSUE_284_PHASE1_CAMERA_MATH_EXTRACTION.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 2 Projection） | `dev/archive/issues/architecture/ISSUE_284_PHASE2_CAMERA_PROJECTION_SPLIT.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 3 Navigation） | `dev/archive/issues/architecture/ISSUE_284_PHASE3_CAMERA_NAVIGATION_SPLIT.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 4 Presets） | `dev/archive/issues/architecture/ISSUE_284_PHASE4_CAMERA_PRESETS_SPLIT.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 5 Transition） | `dev/archive/issues/architecture/ISSUE_284_PHASE5_CAMERA_TRANSITION_SPLIT.md` |
| #284 | CLOSED | camera.rs 責務分割（Phase 6 Tests） | `dev/archive/issues/architecture/ISSUE_284_PHASE6_CAMERA_TESTS_SPLIT.md` |

### Foundation

| Issue | 状態 | 機能 | ファイル |
| --- | --- | --- | --- |
| #170 | CLOSED | 衝突判定・交差判定の数値計算精度改善 | `dev/archive/issues/foundation/ISSUE_170_IMPLEMENTATION_REPORT.md` |
| #320 | CLOSED | geo_contracts 正規化と geo_foundation::commons 重複解消方針 | `dev/archive/issues/architecture/ISSUE_320_IMPLEMENTATION_PREP.md` |
