# RedRing 開発ロードマップ 2026 Q1-Q3（段階的実装版）

**作成日**: 2026年2月8日  
**最終更新日**: 2026年2月12日  
**対象期間**: 2026年2月〜2026年8月（28週間）  
**関連ドキュメント**: 
- [PHASE3_COMPLETION_REPORT.md](PHASE3_COMPLETION_REPORT.md)
- [PHASE4_TOPOLOGY_ENTITY_DESIGN.md](../architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md)
- [OCTREE_DESIGN.md](../architecture/OCTREE_DESIGN.md)
- [ENTITY_FOUNDATION_DESIGN.md](../architecture/ENTITY_FOUNDATION_DESIGN.md)
- [CAM_VISUALIZATION_REQUIREMENTS.md](../architecture/CAM_VISUALIZATION_REQUIREMENTS.md)

---

## 📋 情報管理方針（役割分離）

### ドキュメント階層と役割

| ツール | 役割 | 記載内容 |
|--------|------|---------|
| **GitHub Issue** | タスク管理・進捗追跡 | 概要、チェックリスト、依存関係、**設計ドキュメントへのリンク** |
| **設計ドキュメント** | 技術仕様（Single Source of Truth） | 詳細設計、データ構造、コード例、Phase詳細 |
| **ROADMAP** | 全体計画・優先順位 | 俯瞰的スケジュール、優先順位マトリクス、**設計ドキュメントへのリンク** |

**原則**: 詳細な技術情報は設計ドキュメントに一元化し、IssueとROADMAPはリンクで参照

---

## 📋 目次

1. [段階的実装戦略](#段階的実装戦略)
2. [優先順位マトリクス](#優先順位マトリクス)
3. [Tier 1: デバッグ表示完成（最優先）](#tier-1-デバッグ表示完成最優先)
4. [Tier 2: エンティティ基盤・Octree（高優先）](#tier-2-エンティティ基盤octree高優先)
5. [Tier 3: CAM演算・Phase 4完全版（中優先）](#tier-3-cam演算phase-4完全版中優先)
6. [実施スケジュール](#実施スケジュール)
7. [マイルストーン](#マイルストーン)

---

## 段階的実装戦略

**方針**: 着実に進捗させるため、大きな機能を段階的に分割して実装

### Phase 4の3段階実装

```text
現在（Phase 3完了）:
  幾何形状 + 衝突判定・交差判定
      ↓
Phase 4.0（基礎版）- Issue #208:
  エンティティ + 属性のみ
  ├─ EntityId (UUID)
  ├─ DisplayAttributes (色・線種)
  ├─ Metadata (名前・タグ)
  └─ GeometricEntity / CAMEntity
      ↓
Phase 4.1-4.3（完全版）- Issue #205:
  トポロジー + エンティティ統合
  ├─ B-Rep (Vertex/Edge/Face/Solid)
  ├─ Euler操作
  └─ パラメータ管理
```

### デバッグ機能の充実

```text
形状デバッグ (Issue #204):
  15形状の基本表示
      ↓
Octree実装 (Issue #206):
  空間分割データ構造
      ↓
Octree可視化 (Issue #207):
  デバッグ用視覚化
      ↓
切削シミュレーション対応
```

---

## 優先順位マトリクス

| Tier | 領域 | 項目 | Issue | 工数 | 依存関係 |
|------|------|------|-------|------|----------|
| 🔴 1 | デバッグ表示 | 形状可視化完成（15形状） | [#204](https://github.com/RedRing2020/RedRing/issues/204) | 2週間 | なし（最優先） |
| � 1 | デバッグ表示 | NURBS GPU描画実装 | [#210](https://github.com/RedRing2020/RedRing/issues/210) | 3-4週間 | #204 |
| �🔴 1 | CAM表示 | CAM可視化基礎版 | [#203](https://github.com/RedRing2020/RedRing/issues/203) | 1週間 | #204 |
| 🟠 2 | 空間分割 | Octree実装 | [#206](https://github.com/RedRing2020/RedRing/issues/206) | 2週間 | #204, #203 |
| 🟠 2 | デバッグ | Octree可視化 | [#207](https://github.com/RedRing2020/RedRing/issues/207) | 1週間 | #206 |
| 🟠 2 | CAM | 切削シミュレーション | [#214](https://github.com/RedRing2020/RedRing/issues/214) | 4-5週間 | #206 |
| 🟠 2 | エンティティ | エンティティ層基礎（Phase 4.0） | [#208](https://github.com/RedRing2020/RedRing/issues/208) | 3.5週間 | #204, #203 |
| 🟢 2 | アーキテクチャ | ECSプロトタイプ・評価 | [#215](https://github.com/RedRing2020/RedRing/issues/215) | 1週間 | #208 Phase 1 |
| 🟠 2 | リファクタ | レガシーAPI移行 | [#202](https://github.com/RedRing2020/RedRing/issues/202) | 1週間 | 並行可能 |
| 🟡 3 | CAM演算 | テセレーション機能 | [#42](https://github.com/RedRing2020/RedRing/issues/42) | 1-2週間 | #204, #208 |
| 🟡 3 | CAM演算 | 2D輪郭線オフセット | [#40](https://github.com/RedRing2020/RedRing/issues/40) | 1週間 | #42 |
| 🟡 3 | CAM演算 | 3Dメッシュオフセット | [#41](https://github.com/RedRing2020/RedRing/issues/41) | 1-2週間 | #42 |
| 🟡 3 | トポロジー | Phase 4完全版（B-Rep） | [#205](https://github.com/RedRing2020/RedRing/issues/205) | 6週間 | #208 |

---

## Tier 1: デバッグ表示完成（最優先）

### 1.1 形状可視化システム完成 (Issue #204)

**目的**: 15形状の基本表示機能を実装し、デバッグ・検証基盤を確立

**対象形状**:
- **優先度1（基本形状）**: Plane3D, Ellipse3D, EllipseArc3D, Ray3D, InfiniteLine3D
- **優先度2（サーフェス）**: CylindricalSurface3D, SphericalSurface3D, ConicalSurface3D, TorusSurface3D, EllipsoidalSurface3D
- **優先度3（ソリッド）**: BoxSolid3D, CylindricalSolid3D, SphericalSolid3D, ConicalSolid3D, TorusSolid3D

**実装内容**:
- `viewmodel/converter/src/shape_converter.rs` に変換関数追加
- TessellationQuality による適応的メッシュ生成
- 既存の LineResources / MeshResources を活用

**工数**: 2週間（9営業日）  
**実施時期**: Week 1-2（2月第2週〜第3週）

---

### 1.2 CAM可視化システム設計・実装 (Issue #203)

**目的**: 工具経路の基本表示機能を実装（エンティティ化は #208で実施）

**実装内容**:
1. **データモデル**: `ToolPath` 構造体（`geo_algorithms/src/toolpath.rs`）
2. **ViewModel変換**: `toolpath_to_vertices()` 変換器
3. **View層**: ToolPathResources による描画
4. **色分け**: 早送り（青）、切削（白）、アプローチ（緑）

**実装しない内容**（#208で実施）:
- ❌ CAMエンティティ（EntityId, 属性管理）
- ❌ 加工パラメータ管理
- ❌ 工具経路の選択・編集

**工数**: 1週間（5営業日）  
**実施時期**: Week 3（2月第4週）

---

### 1.3 NURBS形状の適応的テッセレーション実装 (Issue #210)

**目的**: NurbsCurve3D/NurbsSurface3DのGPU直接描画を実装

**背景**:
- 現状: NurbsCurve3DはSVG経由で表示（Issue #188で実装済み）
- 課題: 3DビューワーでのリアルタイムNURBS表示が未対応
- Issue #204で15形状の基盤が完成し、NURBS追加の準備が整った

**実装内容**:

**Phase 1: 固定分割実装（1週間）**:
- `NurbsCurve3D` の固定分割テッセレーション
- `NurbsSurface3D` の固定分割テッセレーション
- `viewmodel/converter/src/shape_converter.rs` に変換関数追加

**Phase 2: 適応的細分化（1-2週間）**:
- 曲率ベース適応的細分化アルゴリズム
- トレランスパラメータの導入（弦高誤差制御）
- U/V独立分割数の最適化

**Phase 3: パフォーマンス最適化（1週間）**:
- メッシュキャッシング機構
- LOD（Level of Detail）対応
- ベンチマーク・最適化（10000制御点で60fps目標）

**依存関係**:
- Issue #204（形状可視化基盤）完了後
- Issue #42（トレランス設計）は参考のみ

**工数**: 3-4週間  
**実施時期**: Week 6-9（3月第3週〜4月第2週）

---

## Tier 2: エンティティ基盤・Octree（高優先）

### 2.1 Octree空間分割実装 (Issue #206)

**目的**: 切削シミュレーション・衝突判定高速化のための空間データ構造

**工数**: 2週間  
**実施時期**: Week 4-5（3月第1週〜第2週）

**詳細設計**: [OCTREE_DESIGN.md](../architecture/OCTREE_DESIGN.md)

---

### 2.2 Octree可視化 (Issue #207)

**目的**: Octree構造のデバッグ用視覚化

**工数**: 1週間  
**実施時期**: Week 6（3月第3週）

**詳細設計**: [OCTREE_DESIGN.md](../architecture/OCTREE_DESIGN.md)

---

### 2.3 エンティティ層基礎 (Issue #208)

**目的**: Phase 4.0 - エンティティ+属性のみの基礎実装

**ECS移行判断（Week 9）**:
- ECSプロトタイプ作成・ベンチマーク評価（Issue #215）
- 2倍以上高速化達成 → Phase 2からECS採用

**工数**: 3.5週間  
**実施時期**: Week 7-10（4月第1週〜4月下旬）

**詳細設計**:
- [ENTITY_FOUNDATION_DESIGN.md](../architecture/ENTITY_FOUNDATION_DESIGN.md)
- [ECS_EVALUATION.md](../architecture/ECS_EVALUATION.md)（ECS移行評価）

---

### 2.4 切削シミュレーション (Issue #214)

**目的**: VoxelOctreeを用いた切削シミュレーション（距離ベーススナップショット方式）

**工数**: 4-5週間  
**実施時期**: Week 6-10（4月中旬〜5月中旬）

**詳細設計**: [CUTTING_SIMULATION_DESIGN.md](../architecture/CUTTING_SIMULATION_DESIGN.md)

---

### 2.5 レガシーAPI問題解決 (Issue #202)

**目的**: Foundation Pattern への完全移行

**工数**: 1週間（並行実施可能）  
**実施時期**: Week 11（5月第1週）

---

## Tier 3: CAM演算・Phase 4完全版（中優先）

### 3.1 テセレーション機能 (Issue #42)

**目的**: NURBS/曲面の適応的メッシュ生成

**依存関係**: Issue #204（形状可視化）, #208（エンティティ層基礎）完了後

**工数**: 1-2週間  
**実施時期**: Week 12-13（5月第2週〜第3週）

---

### 3.2 2D輪郭線オフセット (Issue #40)

**目的**: CAM基礎機能（2Dオフセット）

**工数**: 1週間  
**実施時期**: Week 14（5月第4週）

---

### 3.3 3Dメッシュオフセット (Issue #41)

**目的**: 荒加工・仕上げの基盤

**工数**: 1-2週間  
**実施時期**: Week 15-16（6月第1週〜第2週）

---

### 3.4 Phase 4 完全版 (Issue #205)

**目的**: B-Repトポロジー層の完全実装（ECS版で実装）

**工数**: 6週間  
**実施時期**: Week 19-24（7月第1週〜8月中旬）

**詳細設計**: [PHASE4_TOPOLOGY_ENTITY_DESIGN.md](../architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md)

---

## 実施スケジュール

### Week 1-2: Tier 1 - デバッグ表示基盤（完了）
- **Issue #204**（形状可視化完成）✅ マージ完了

### Week 3: Tier 1 - CAM可視化基礎（完了）
- **Issue #203**（CAM可視化基礎版）✅ マージ完了

### Week 4-5: Tier 2 - 空間データ構造
- **Issue #206**（Octree実装）

### Week 6: Tier 2 - Octree可視化
- **Issue #207**（Octree可視化）

### Week 6-10: Tier 2 - 切削シミュレーション
- **Issue #214**（切削シミュレーション Phase 1a/1b/1c/2/3）

### Week 7-10: Tier 2 - エンティティ層基礎
- **Week 7-8**: Issue #208 Phase 1（エンティティ基盤）
- **Week 9**: Issue #215（ECSプロトタイプ作成・評価）← 判断ポイント
- **Week 10**: Issue #208 Phase 2（ViewModel/App統合）← ECS採用判断

### Week 11: Tier 2 - リファクタリング
- **Issue #202**（レガシーAPI移行）

### Week 12-16: Tier 3 - CAM演算基礎
- **Week 12-13**: Issue #42（テセレーション）
- **Week 14**: Issue #40（2Dオフセット）
- **Week 15-16**: Issue #41（3Dオフセット）

### Week 17-18: 予備・テスト・ドキュメント
- 統合テスト
- パフォーマンステスト
- ドキュメント整備

### Week 19-24: Tier 3 - Phase 4完全版
- **Issue #205**（Phase 4.1/4.2/4.3、ECS版トポロジー）

### Week 25-28: 予備・統合・リリース準備
- 全体統合テスト
- パフォーマンス最適化
- リリースノート作成

---

## マイルストーン

### M1: デバッグ基盤完成（Week 3終了時 - 2月末）

**成果物**:
- ✅ 20形状すべての可視化対応（#204）
- ✅ CAM工具経路の基本表示（#203）
- ✅ デバッグ効率の大幅向上

**検証項目**:
- すべての形状が正しく描画される
- FPS 60以上を維持（1000形状）

---

### M2: エンティティ・Octree完成（Week 11終了時 - 5月初旬）

**成果物**:
- ✅ Octree実装・可視化完了（#206, #207）
- ✅ エンティティ層基礎実装完了（#208）
- ✅ レガシーAPI完全移行（#202）
- ✅ エンティティ選択・色変更機能

**検証項目**:
- Octreeで衝突判定が99%高速化
- エンティティ1000個の管理・描画が可能
- Foundation Pattern 100%適用

---

### M3: CAM演算基礎完成（Week 16終了時 - 6月中旬）

**成果物**:
- ✅ テセレーション機能（#42）
- ✅ 2Dオフセット（#40）
- ✅ 3Dオフセット（#41）
- ✅ CAM基礎機能の実用化

**検証項目**:
- NURBS曲面のテセレーション品質
- オフセット精度 0.01mm 以内
- 実際のCAM加工パスの生成

---

### M4: Phase 4完全版完成（Week 24終了時 - 8月中旬）

**成果物**:
- ✅ B-Repトポロジー層（#205 Phase 4.1）
- ✅ エンティティ統合（#205 Phase 4.2）
- ✅ パラメータ管理（#205 Phase 4.3）
- ✅ CADシステムの基盤確立

**検証項目**:
- Euler操作の正確性（Euler標数検証）
- トポロジー整合性チェック
- 属性・メタデータの永続化

---

## まとめ

### 段階的実装の利点

1. **リスク低減**:
   - 大きな変更を段階的に実施
   - 各ステップでの検証・修正が可能

2. **早期フィードバック**:
   - デバッグ表示（Week 3）で即座に可視化可能
   - エンティティ基礎（Week 10）で選択・属性管理を体験

3. **着実な進捗**:
   - 2週間単位のマイルストーン
   - 明確な完了条件

### Phase 4の段階的実装

**Phase 4.0（#208）**: エンティティ+属性（Week 7-10）  
→ **Phase 4.1-4.3（#205）**: トポロジー完全版（Week 19-24）

この分割により、**4ヶ月早く**エンティティ管理機能を利用開始できます。

---

## 関連情報

### 設計文書
- `dev/architecture/PHASE4_TOPOLOGY_ENTITY_DESIGN.md` - Phase 4完全版設計
- `dev/architecture/ENTITY_FOUNDATION_DESIGN.md` - Phase 4.0基礎設計
- `dev/architecture/OCTREE_DESIGN.md` - Octree詳細設計
- `dev/architecture/CAM_VISUALIZATION_REQUIREMENTS.md` - CAM可視化要件

### GitHub Issues
- [#204](https://github.com/RedRing2020/RedRing/issues/204) - 形状可視化完成（✅ 完了）
- [#210](https://github.com/RedRing2020/RedRing/issues/210) - NURBS GPU描画実装
- [#203](https://github.com/RedRing2020/RedRing/issues/203) - CAM可視化基礎
- [#206](https://github.com/RedRing2020/RedRing/issues/206) - Octree実装
- [#207](https://github.com/RedRing2020/RedRing/issues/207) - Octree可視化
- [#208](https://github.com/RedRing2020/RedRing/issues/208) - エンティティ層基礎
- [#202](https://github.com/RedRing2020/RedRing/issues/202) - レガシーAPI移行
- [#205](https://github.com/RedRing2020/RedRing/issues/205) - Phase 4完全版
- [#40](https://github.com/RedRing2020/RedRing/issues/40) - 2Dオフセット
- [#41](https://github.com/RedRing2020/RedRing/issues/41) - 3Dオフセット
- [#42](https://github.com/RedRing2020/RedRing/issues/42) - テセレーション

### 既存ドキュメント
- `dev/foundation/PHASE3_COMPLETION_REPORT.md` - Phase 3完了報告
- `dev/foundation/ROADMAP_2026_Q1_Q2.md` - **本ドキュメント（旧版）**
