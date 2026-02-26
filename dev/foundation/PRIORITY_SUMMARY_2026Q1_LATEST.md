# 優先順位整理サマリー（2026年2月13日版）

**作成日**: 2026年2月13日  
**対象**: Loadmap + GitHub Issues から再整理

---

## 📊 現在の状況

### ✅ 完了済み (Tier 1-2)
| Issue | タイトル | 完了日 | 工数 | 効果 |
|-------|---------|--------|------|------|
| #204 | 形状可視化システム完成（15形状） | 2026/02/08 | 2週間 | GPU変換実装、アーキテクチャ改善（-281行） |
| #222 | geo_commons Foundation準拠 | 2026/02/13 | 3日 | LineSegment3D衝突検出、循環依存解消 |
| #202 | レガシーAPI移行（Phase 2） | 2026/02/13 | 2-3日 | 13形状Foundation準拠完了 |
| #224 | CAM工具のデータ駆動設計 | 2026/02/13 | 3日 | Tool構造体データ駆動設計、検証ロジック改善 |

---

## 🎯 実施順序（Tier別）

### 🔴 **Tier 1: 最優先** (Week 3-5)

#### 1.1 Issue #203: CAM可視化基礎版
**概要**: 工具経路の基本表示機能  
**工数**: 1週間  
**依存**: Issue #204 (✅ 完了)  
**ステータス**: 🟡 準備中  
**実施時期**: Week 3  
**成果物**:
- ToolPath構造体と表示機能
- 色分け表示（早送り/切削/アプローチ）
- 工具経路レンダリング

#### 1.2 Issue #210: NURBS GPU描画実装
**概要**: NURBS曲線/曲面の高速描画  
**工数**: 3-4週間  
**依存**: Issue #204 (✅ 完了)  
**ステータス**: 📝 計画策定済  
**実施時期**: Week 6-9  
**成果物**:
- BezierCurve2D/3DのGPU描画
- BezierSurface3DのGPU描画
- テセレーション最適化

---

### 🟠 **Tier 2: 高優先** (Week 4-10)

#### 2.1 Issue #206: Octree空間分割実装
**概要**: 3D空間を効率的に分割するデータ構造  
**工数**: 2週間  
**依存**: Issue #204 (✅ 完了)  
**ステータス**: 📝 設計完了  
**実施時期**: Week 4-5  
**設計文書**: `dev/architecture/OCTREE_DESIGN.md`  
**成果物**:
- **Phase 1**: 基本Octree（挿入、検索、最近傍）
- **Phase 2**: ボクセルOctree（材料除去シミュレーション用）

#### 2.2 Issue #207: Octree可視化（デバッグ用）
**概要**: Octreeの空間分割を可視化  
**工数**: 1週間  
**依存**: Issue #206  
**ステータス**: 📝 計画済  
**実施時期**: Week 6  
**成果物**:
- OctreeNodeのワイヤーフレーム表示
- ボクセル層の色分け表示
- デバッグUI統合

#### 2.3 Issue #208: エンティティ層基礎（Phase 4.0）
**概要**: 幾何形状に属性・ID管理を追加  
**工数**: 3.5週間  
**依存**: Issue #204, #203  
**ステータス**: 📝 設計完了  
**実施時期**: Week 7-10  
**設計文書**: `dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md`  
**成果物**:
- EntityId (UUID) 管理
- DisplayAttributes (色・線種・表示/非表示)
- Metadata (名前・説明・タグ)
- GeometricEntity / CAMEntity型定義

#### 2.4 Issue #214: 切削シミュレーション
**概要**: 工具による材料除去シミュレーション  
**工数**: 4-5週間  
**依存**: Issue #206  
**ステータス**: ⏸ 待機中  
**実施時期**: Week 7-11  
**成果物**:
- ボクセルベースの材料除去計算
- 刃物接触判定
- 加工面の可視化

#### 2.5 Issue #215: ECSプロトタイプ・評価
**概要**: Entity Component System によるシステム評価  
**工数**: 1週間  
**依存**: Issue #208 Phase 1  
**ステータス**: 📝 計画中  
**実施時期**: Week 11  
**成果物**:
- ECS設計とプロトタイプ実装
- パフォーマンス評価
- 採用判断

---

### 🟡 **Tier 3: 中優先** (Week 12+)

#### 3.1 Issue #42: トレランス指定テセレーション
**概要**: 曲線・曲面を精度指定で多角形化  
**工数**: 1-2週間  
**依存**: Issue #204, #208  
**ステータス**: ⏸ on-hold  
**実施時期**: Week 12-13  
**関連ドキュメント**: `dev/architecture/SHAPE_TESSELLATION_DESIGN.md`

#### 3.2 Issue #40: 2D輪郭線オフセット
**概要**: 2D図形の平行移動（オフセット）計算  
**工数**: 1週間  
**依存**: Issue #42  
**ステータス**: 📝 待機中  
**実施時期**: Week 14

#### 3.3 Issue #41: 3Dメッシュオフセット
**概要**: 3D メッシュの段差加工  
**工数**: 1-2週間  
**依存**: Issue #42  
**ステータス**: 📝 待機中  
**実施時期**: Week 15-16

#### 3.4 Issue #205: Phase 4完全版（B-Rep トポロジー）
**概要**: 完全なトポロジー管理（Vertex/Edge/Face/Solid）  
**工数**: 6週間  
**依存**: Issue #208  
**ステータス**: 📝 計画済  
**実施時期**: Week 19-24  
**設計文書**: `dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md`

---

## 🚦 依存関係マップ

```
🔴 Tier 1
├─ Issue #203 (CAM可視化)
│  └─ depends on: #204 ✅
└─ Issue #210 (NURBS GPU)
   └─ depends on: #204 ✅

🟠 Tier 2
├─ Issue #206 (Octree)
│  ├─ depends on: #204 ✅
│  ├─ required by: #207, #214, #208
│  └─ blocks: Tier 3 CAM演算
│
├─ Issue #207 (Octree可視化)
│  └─ depends on: #206
│
├─ Issue #208 (エンティティ層)
│  ├─ depends on: #204 ✅, #203
│  ├─ required by: #214, #215
│  └─ blocks: #205 (Phase 4完全版)
│
├─ Issue #214 (切削シミュレーション)
│  ├─ depends on: #206
│  └─ produces: 加工検証データ
│
└─ Issue #215 (ECS評価)
   └─ depends on: #208 Phase 1

🟡 Tier 3  
├─ Issue #42 (テセレーション)
│  ├─ depends on: #204 ✅, #208
│  └─ blocks: #40, #41
│
├─ Issue #40 (2Dオフセット)
│  └─ depends on: #42
│
├─ Issue #41 (3Dオフセット)
│  └─ depends on: #42
│
└─ Issue #205 (Phase 4完全版)
   └─ depends on: #208
```

---

## 📅 推奨スケジュール（Week単位）

| Week | Tier | Issues | 並行可能 |
|------|------|--------|---------|
| W3 | 🔴 1 | #203 CAM可視化 | #202移行 ✅ |
| W4-5 | 🟠 2 | #206 Octree | #203 並行可能 |
| W6 | 🟠 2 | #207 可視化 + #210 NURBS開始 | - |
| W7-10 | 🟠 2 | #208 エンティティ + #214 切削 | #210 並行 |
| W11 | 🟠 2 | #215 ECS評価 | - |
| W12-13 | 🟡 3 | #42 テセレーション | - |
| W14 | 🟡 3 | #40 2Dオフセット | - |
| W15-16 | 🟡 3 | #41 3Dオフセット | - |
| W17-18 | 🔄 統合 | 統合テスト・ドキュメント | - |
| W19-24 | 🟡 3 | #205 Phase 4完全版 | - |

---

## 🎯 今週末（Week 3）の目標

### 実施予定: Issue #203 CAM可視化基礎版

**チェックリスト**:
- [ ] CAM_VISUALIZATION_REQUIREMENTS.md の確認
- [ ] ToolPath データ構造の設計
- [ ] 工具経路レンダリングパイプライン設計
- [ ] 色分け表示ロジック設計
- [ ] 実装開始

**成功条件**:
- 工具経路が3D画面に表示される
- 工具経路セグメントが色分け表示される
- 基本的なデバッグ出力が確認できる

---

## 📋 設計ドキュメント参照

| Issue | 設計ドキュメント | 最終更新 |
|-------|----------------|---------|
| #203 | `CAM_VISUALIZATION_REQUIREMENTS.md` | 2026/02/11 |
| #206 | `OCTREE_DESIGN.md` | 2026/02/07 |
| #207 | `OCTREE_VISUALIZATION_DESIGN.md` | 2026/02/07 |
| #208 | `PHASE4_TOPOLOGY_ENTITY_DESIGN.md` | 2026/01/31 |
| #210 | `SHAPE_VISUALIZATION_DESIGN.md` (Part 2) | 2026/01/28 |
| #214 | `CUTTING_SIMULATION_DESIGN.md` | 2026/02/04 |
| #42 | `SHAPE_TESSELLATION_DESIGN.md` | 2025/12/22 |
| #205 | `PHASE4_TOPOLOGY_ENTITY_DESIGN.md` | 2026/01/31 |

---

## 💡 優先順位決定ロジック

### Tier 1（最優先）
- ✅ 基盤技術：デバッグ・可視化システムの充実
- ✅ 依存関係：他のTierのブロッカーを解消

### Tier 2（高優先）
- ✅ 次のフェーズへの基盤：エンティティ管理、空間データ構造
- ✅ CAM機能：切削シミュレーション対応
- ✅ 並行実施可能：依存関係が少ない

### Tier 3（中優先）
- ✅ CAM演算：テセレーション、オフセット計算
- ✅ 高度機能：B-Repトポロジー
- ✅ 実施時期：Tier 1-2完了後

---

## 🔄 更新履歴

| 日付 | 変更 |
|------|------|
| 2026/02/13 | Issue #224 PR #226作成。優先順位整理ドキュメント新規作成 |
| 2026/02/13 | Issue #202, #222 実装完了による優先順位更新 |
| 2026/02/08 | 初版作成（Loadmap整理） |

---

## 関連ドキュメント

- [ISSUE_PRIORITY_MATRIX_2026Q1.md](ISSUE_PRIORITY_MATRIX_2026Q1.md)
- [ROADMAP_2026_Q1_Q3_STAGED.md](ROADMAP_2026_Q1_Q3_STAGED.md)
- [PHASE3_COMPLETION_REPORT.md](PHASE3_COMPLETION_REPORT.md)
