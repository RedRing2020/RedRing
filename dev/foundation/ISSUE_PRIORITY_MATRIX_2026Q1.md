# Issue優先順位マトリクス（2026年Q1）

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**対象期間**: 2026年2月〜4月  

---

## 📊 優先順位サマリー

### ✅ 完了済み（Tier 1）
- **Issue #204**: 形状可視化システム完成（15形状）✅ マージ完了（2026/02/08）
  - 実績: 2週間（計画通り）
  - 成果: 15形状GPU変換実装、ViewModelアーキテクチャ修正（-281行）

### 🔴 Tier 1: 最優先（実施中・次期）

| 優先度 | Issue | タイトル | 工数 | 実施時期 | 依存 | ステータス |
|--------|-------|---------|------|----------|------|-----------|
| 🔴 1.1 | [#203](https://github.com/RedRing2020/RedRing/issues/203) | CAM可視化システム設計・実装 | 1週間 | Week 3 | #204 | 🟡 準備中 |
| 🟡 1.2 | [#210](https://github.com/RedRing2020/RedRing/issues/210) | NURBS GPU描画実装 | 3-4週間 | Week 6-9 | #204 | 📝 計画策定済 |

**Tier 1の目的**: デバッグ・可視化基盤の確立

---

### 🟠 Tier 2: 高優先（基盤技術）

| 優先度 | Issue | タイトル | 工数 | 実施時期 | 依存 | ステータス |
|--------|-------|---------|------|----------|------|-----------|
| 🟠 2.1 | [#206](https://github.com/RedRing2020/RedRing/issues/206) | Octree空間分割実装 | 2週間 | Week 4-5 | #204, #203 | 📝 設計完了 |
| 🟠 2.2 | [#207](https://github.com/RedRing2020/RedRing/issues/207) | Octree可視化（デバッグ用） | 1週間 | Week 6 | #206 | 📝 計画済 |
| 🟠 2.3 | [#208](https://github.com/RedRing2020/RedRing/issues/208) | エンティティ層基礎（Phase 4.0） | 3.5週間 | Week 7-10 | #204, #203 | 📝 設計完了 |
| 🟠 2.4 | [#202](https://github.com/RedRing2020/RedRing/issues/202) | レガシーAPI移行 | 1週間 | 並行実施 | - | ⏸ 保留中 |

**Tier 2の目的**: 空間データ構造、エンティティ管理基盤の確立

---

### 🟡 Tier 3: 中優先（CAM演算・高度機能）

| 優先度 | Issue | タイトル | 工数 | 実施時期 | 依存 | ステータス |
|--------|-------|---------|------|----------|------|-----------|
| 🟡 3.1 | [#42](https://github.com/RedRing2020/RedRing/issues/42) | トレランス指定テッセレーション | 1-2週間 | Week 12-13 | #204, #208 | ⏸ on-hold |
| 🟡 3.2 | [#40](https://github.com/RedRing2020/RedRing/issues/40) | 2D輪郭線オフセット | 1週間 | Week 14 | #42 | 📝 待機中 |
| 🟡 3.3 | [#41](https://github.com/RedRing2020/RedRing/issues/41) | 3Dメッシュオフセット | 1-2週間 | Week 15-16 | #42 | 📝 待機中 |
| 🟡 3.4 | [#205](https://github.com/RedRing2020/RedRing/issues/205) | Phase 4完全版（B-Rep） | 6週間 | Week 19-24 | #208 | 📝 計画済 |

**Tier 3の目的**: CAM演算基盤、トポロジー層完成

---

## 🔄 古いIssueの扱い

### 📌 Issue #44: 形状モデルのOctree表現機能 ✅ クローズ済み

**ステータス**: `closed` (2026/02/08)  
**処置**: **Issue #206に統合・置き換え完了**

| 項目 | Issue #44（旧） | Issue #206（新） |
|------|----------------|------------------|
| 作成日 | 2025/09/23 | 2026/02/07 |
| アーキテクチャ | 旧設計（Foundation Pattern前） | 現行設計（Foundation Pattern準拠） |
| 実装計画 | 簡易的（OctreeNode定義のみ） | 詳細（Phase 1/2, 設計文書完備） |
| 用途 | 形状モデル表現全般 | CAM切削シミュレーション、衝突判定 |
| 設計文書 | なし | `dev/architecture/OCTREE_DESIGN.md` |
| ステータス | **クローズ済み** | Tier 2（高優先） |

**実施済みアクション** (2026/02/08):
- ✅ Issue #44をクローズ（[#44](https://github.com/RedRing2020/RedRing/issues/44)）
- ✅ Issue #206にコメント追加（統合内容を記録）
- ✅ クローズ理由を明記（Issue #206への統合）

**統合理由**:
1. **アーキテクチャの互換性**: #44は Foundation Pattern 導入前の設計
2. **実装の具体性**: #206 は Phase 1/2 の詳細計画を含む
3. **用途の明確化**: CAM/衝突判定に特化し、実用性を重視
4. **重複の排除**: 同じテーマで2つのIssueを管理する必要がない

---

### 📌 Issue #43: 形状の包含ボックス保持機能 ✅ クローズ済み

**ステータス**: `closed` (2026/02/08)  
**処置**: **Phase 3完了により実装済み**

| 項目 | 計画（Issue #43） | 実装済み（Phase 3） |
|------|------------------|--------------------|
| 作成日 | 2025/09/23 | 2025/12/21（Phase 3完了） |
| 構造体 | BoundingBox（予定） | BBox3D（実装済み） |
| トレイト | bounding_box()（予定） | Bounded::aabb()（実装済み） |
| 対応形状 | 未定 | 15形状完全対応 |
| 設計 | 簡易計画 | Foundation Pattern準拠 |
| ステータス | **クローズ済み** | Phase 3で完成 |

**実施済みアクション** (2026/02/08):
- ✅ Issue #43をクローズ（[#43](https://github.com/RedRing2020/RedRing/issues/43)）
- ✅ クローズ理由を明記（Phase 3で実装完了）
- ✅ 実装内容の詳細を記録

**実装済み内容**:
1. **BBox3D構造体**: `geo_primitives/src/bbox_3d.rs`
2. **Bounded トレイト**: `geo_foundation/src/extension_foundation.rs`
3. **15形状対応**: Point3D, Line3D, Segment3D, Ray3D, Plane3D, Triangle3D, Rectangle3D, Circle3D, Ellipse3D, Sphere3D, Cylinder3D, Cone3D, Torus3D, Cuboid3D, Capsule3D
4. **用途**: 交差判定高速化、空間インデックス（Octree）、GPU描画最適化

---

### 📌 その他の保留Issue（要検討）

| Issue | タイトル | 理由 | 推奨アクション | 優先度 |
|-------|---------|------|---------------|--------|
| [#42](https://github.com/RedRing2020/RedRing/issues/42) | テッセレーション機能 | `on-hold`, `needs-foundation-update` | #210完了後に再検討（Tier 3で継続） | Week 12-13 |
| [#33](https://github.com/RedRing2020/RedRing/issues/33) | 幾何交差・ブール演算 | `on-hold`, `needs-foundation-update` | Phase 3完了で基礎実装済み → サブIssueの整理が必要 | 検討中 |

**Note**: 
- Issue #44（Octree表現）は2026/02/08にクローズ済み（Issue #206に統合）
- Issue #43（包含ボックス）は2026/02/08にクローズ済み（Phase 3で実装完了）

---

## 📅 実施スケジュール（Week 1-10）

### 完了済み
- **Week 1-2**: Issue #204（形状可視化）✅ 完了

### 実施予定

#### Week 3: Tier 1 - CAM可視化
- **Issue #203**: CAM可視化基礎版
  - 工具経路の基本表示機能
  - 色分け（早送り/切削/アプローチ）

#### Week 4-5: Tier 2 - 空間データ構造
- **Issue #206**: Octree実装
  - Phase 1: 基本Octree（挿入、検索、最近傍）
  - Phase 2: ボクセルOctree（材料除去シミュレーション）

#### Week 6: Tier 2 - デバッグ可視化
- **Issue #207**: Octree可視化
  - デバッグ用視覚化
  - 空間分割の確認

#### Week 6-9: Tier 1 - NURBS GPU描画
- **Issue #210**: NURBS適応的テッセレーション
  - Week 6: Phase 1（固定分割実装）
  - Week 7-8: Phase 2（適応的細分化）
  - Week 9: Phase 3（最適化、キャッシング、LOD）

#### Week 7-10: Tier 2 - エンティティ基盤
- **Issue #208**: エンティティ層基礎（Phase 4.0）
  - Entity ID管理、属性システム
  - MVVM統合、永続化基盤

---

## 🎯 マイルストーン

### M1: デバッグ表示完成（Week 2終了時）✅ 達成
- ✅ Issue #204 完了
- ✅ 15形状のGPU描画基盤確立
- ✅ ViewModelアーキテクチャ修正

### M2: CAM・空間分割基盤完成（Week 6終了時）
**目標日**: 2026年3月中旬

**成果物**:
- ✅ CAM可視化基礎（#203）
- ✅ Octree実装（#206）
- ✅ Octree可視化（#207）

**検証項目**:
- 工具経路の基本表示が動作
- 1000要素のOctree検索で99%高速化
- Octreeデバッグ表示の動作確認

### M3: NURBS・エンティティ基盤完成（Week 10終了時）
**目標日**: 2026年4月中旬

**成果物**:
- ✅ NURBS GPU描画（#210）
- ✅ エンティティ層基礎（#208）

**検証項目**:
- NURBS曲線・曲面の適応的テッセレーション動作
- 100制御点の曲線を1ms以下で処理
- EntityID/属性管理の基本機能動作

---

## 📝 優先順位決定の基準

### Tier 1: 最優先
- **基準**: デバッグ・可視化に直結する機能
- **理由**: 開発効率向上、問題の早期発見
- **完了条件**: 実装完了、テスト通過、ドキュメント整備

### Tier 2: 高優先
- **基準**: アーキテクチャ基盤、パフォーマンス向上
- **理由**: 後続機能の前提条件となる技術
- **完了条件**: 設計文書、実装、性能検証、統合テスト

### Tier 3: 中優先
- **基準**: CAM演算、高度機能、完全版実装
- **理由**: Tier 1/2の基盤が整った後に実施可能
- **完了条件**: 精度検証、実用性確認、ドキュメント完備

### 保留（on-hold）
- **基準**: アーキテクチャが古い、依存関係が未整備
- **処置**: クローズ、または新Issue作成で置き換え
- **再検討**: Foundation Pattern完了後、または技術的前提が整った時点

---

## 🔗 関連ドキュメント

### 設計文書
- `dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md` - Phase 4完全版設計
- `dev/architecture/ENTITY_FOUNDATION_DESIGN.md` - Phase 4.0基礎設計
- `dev/architecture/OCTREE_DESIGN.md` - Octree詳細設計
- `dev/architecture/CAM_VISUALIZATION_REQUIREMENTS.md` - CAM可視化要件
- `dev/architecture/SHAPE_TESSELLATION_DESIGN.md` - 形状テッセレーション設計
- `dev/architecture/NURBS_FOUNDATION_PATTERN.md` - NURBS Foundation Pattern

### 進捗管理
- `dev/foundation/ROADMAP_2026_Q1_Q3_STAGED.md` - ロードマップ（マスター）
- `dev/foundation/PHASE3_COMPLETION_REPORT.md` - Phase 3完了報告

### GitHub Projects
- **現在のプロジェクトボード**: 使用していません（Issue/PR中心の管理）
- **推奨**: GitHub Projects v2 の導入検討（Week 10以降）

---

## 📌 アクション項目

### 即時対応（今週中）
- [x] Issue #44をクローズ（Issue #206への統合を明記）✅ 完了（2026/02/08）
- [x] Issue #43をクローズ（Phase 3で実装完了）✅ 完了（2026/02/08）
- [ ] Issue #203の実装開始準備

### Week 3-5
- [ ] Issue #203実装・完了
- [ ] Issue #206実装開始
- [ ] Issue #207の詳細計画策定

### Week 6-10
- [ ] Issue #210実装開始
- [ ] Issue #208実装開始
- [ ] Tier 3タスクの優先順位再評価

---

## 📊 統計情報

### Issue統計（2026/02/08時点）
- **オープンIssue総数**: 42件（#44, #43クローズにより2件減）
- **Tier 1**: 1件（#203のみ、#204完了）
- **Tier 2**: 5件（#210, #208, #207, #206, #202）
- **Tier 3**: 2件（#205, #42）
- **保留中（on-hold）**: 1件（#42の一部）
- **依存関係未整備（needs-foundation-update）**: 2件
- **直近のクローズ**: 
  - #44 → #206に統合（2026/02/08）
  - #43 → Phase 3で実装完了（2026/02/08）

### 完了率（Q1目標に対して）
- **Week 1-2完了**: 1/11タスク（9%）
- **Week 3-10予定**: 10タスク
- **Q1完了目標**: 11タスク（Tier 1-2のみ）
