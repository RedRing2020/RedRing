# Entity View統合設計（Issue #208 残タスク）

**作成日**: 2026年2月21日  
**対象Issue**: #208（Phase 4.0 エンティティ層基礎）  
**目的**: 未対応の ViewModel/App 統合の設計確定（実装前）

---

## 1. 現状整理

### 1.1 実装済み（Model）

- `model/geo_entity` に `EntityId` / `DisplayAttributes` / `Metadata` / `GeometricEntity` / 属性・関係管理を実装済み
- `model/cam_entity` に `CAMEntity` を分離実装済み（CAD/CAM分離）
- 依存境界ルール（`geo_* -> cam_*` 禁止）をスクリプト/CIで検証可能

### 1.2 未対応（Issue #208 の残り）

- `viewmodel/converter/src/entity_converter.rs`（未作成）
- `view/app/src/entity_manager.rs`（未作成）
- App層での「エンティティ単位」選択/表示更新フロー（未接続）

---

## 2. 設計制約

1. **既存描画パイプラインを壊さない**
   - `shape_converter.rs` / `toolpath_converter.rs` の既存直接変換は維持
2. **責務分離**
   - ViewModel は「変換のみ」
   - App は「保持・選択・表示状態更新」
3. **依存方向の維持**
   - `view/app` → `viewmodel/converter` → `model/*`
4. **段階導入**
   - まずは debug系導線に統合し、既存の `load_debug_*` と共存

---

## 3. 統合方式の選択肢

### 案A（推奨）: 薄い統合レイヤー追加

- 既存 converter を活かし、`entity_converter` でエンティティ入力に薄いアダプタを追加
- App に `EntityManager` を追加し、最小操作（追加/削除/選択/可視切替）から導入

**利点**
- 既存コード変更が最小
- リスクが低く、段階導入しやすい

### 案B: 変換系を全面的に Entity 中心へ再編

- `shape_converter` / `toolpath_converter` を Entity前提APIに置換

**課題**
- 変更範囲が大きく、Issue #208 の範囲を超えやすい

> 採用: **案A**（Issue #208 の「基礎統合」に適合）

---

## 4. 目標アーキテクチャ

```text
model/geo_entity, model/cam_entity
        ↓
viewmodel/converter/entity_converter.rs
  - geometric_entity_to_vertices(...)
  - cam_entity_to_vertices(...)
        ↓
view/app/entity_manager.rs
  - Entity保存
  - 選択状態
  - 可視/色更新
        ↓
view/app/app_state.rs
  - EntityManager経由で描画データ再生成
```

---

## 5. ViewModel設計（`entity_converter.rs`）

## 5.1 追加API

```rust
pub fn geometric_entity_to_vertices<T, G>(
    entity: &geo_entity::GeometricEntity<T, G>,
    quality: &shape_converter::TessellationQuality,
) -> Result<Vec<mesh_converter::VertexData>, EntityConvertError>
where
    T: geo_contracts::Scalar,
    G: 'static;

pub fn cam_entity_to_vertices(
    entity: &cam_entity::CAMEntity,
    settings: &toolpath_converter::ToolPathVisualizationSettings,
) -> toolpath_converter::ToolPathVertices;
```

## 5.2 設計方針

- `geometric_entity_to_vertices` は内部で既存 `shape_converter` を呼ぶ
- `DisplayAttributes` の色を最終頂点へ適用（ViewModel責務）
- 失敗は `EntityConvertError` で返す（未対応形状・空データ等）

---

## 6. App設計（`entity_manager.rs`）

## 6.1 管理対象

- `GeometricEntity`（複数形状型を扱うため enum/trait object のどちらかを採用）
- `CAMEntity`
- 選択状態（`EntityId`）
- dirtyフラグ（再変換が必要か）

## 6.2 最小API

```rust
pub struct EntityManager {
    // 内部実装は enum ベースを推奨
}

impl EntityManager {
    pub fn add_geometric(&mut self, entity: GeometricEntityItem) -> geo_entity::EntityId;
    pub fn add_cam(&mut self, entity: cam_entity::CAMEntity) -> geo_entity::EntityId;
    pub fn remove(&mut self, id: geo_entity::EntityId) -> bool;

    pub fn select(&mut self, id: geo_entity::EntityId) -> bool;
    pub fn clear_selection(&mut self);
    pub fn selected(&self) -> Option<geo_entity::EntityId>;

    pub fn set_visible(&mut self, id: geo_entity::EntityId, visible: bool) -> bool;
    pub fn set_color(&mut self, id: geo_entity::EntityId, color: [f32; 4]) -> bool;

    pub fn is_dirty(&self) -> bool;
    pub fn clear_dirty(&mut self);
}
```

---

## 7. `app_state.rs` 統合ポイント

1. `AppState` に `entity_manager: EntityManager` を追加
2. 既存 `load_debug_line` / `load_debug_toolpath` に「Entity経由パス」を追加
3. `rebuild_stage_from_entities()` を新設
   - EntityManagerの可視エンティティを収集
   - ViewModel変換
   - `MeshStage` / `ToolPathStage` へ反映
4. キー操作（最小）
   - 追加: デバッグエンティティ投入
   - 選択切替: 先頭/次要素など簡易ロジック
   - 色変更: 選択エンティティへ適用

---

## 8. テスト設計

## 8.1 `viewmodel/converter` 側

- `geometric_entity_to_vertices` が表示色を反映する
- 非対応形状で明示エラーを返す
- `cam_entity_to_vertices` が `CAMEntity` 入力でも既存変換結果と一致する

## 8.2 `view/app` 側

- Entity追加/削除/選択の基本操作
- `set_color` / `set_visible` で dirty が立つ
- dirty時のみ `rebuild_stage_from_entities()` 実行

---

## 9. 実装順序（提案）

1. `entity_converter.rs` 追加（既存 converter 呼び出しの薄いアダプタ）
2. `entity_manager.rs` 追加（最小 API）
3. `app_state.rs` へ manager 組み込み
4. debug導線を Entity経由へ1本追加
5. 最小テスト追加

---

## 10. 完了条件（Issue #208 残タスク）

- [ ] `viewmodel/converter/src/entity_converter.rs` が追加される
- [ ] `view/app/src/entity_manager.rs` が追加される
- [ ] `app_state.rs` から EntityManager 経由の描画再構築が呼ばれる
- [ ] 最小テスト（converter/app）を追加し、`cargo test --all` が通る

---

## 補遺: デバッグキー割り当て（2026-02-26時点）

`view/app/src/app_state.rs` の実装に合わせ、デバッグ用途のキー割り当てを以下の通り整理する。

- `p`: カッターパスのみ表示（色分けライン）
- `Shift+P`: CAMシミュレーション可視化（ToolPath + ワーク + 除去）
- `w`: CAMシミュレーション表示中に Wire/Solid を切替
- `k` / `j`: スナップショットの次フレーム / 前フレーム

注記:

- `p` と `Shift+P` を分離した目的は、デバッグテスト時に「経路確認」と「切削結果確認」を使い分けるため。

---

## 11. 非スコープ（この設計では扱わない）

- B-Rep トポロジー（Issue #205）
- 高度な選択（BVH/レイピッキング）
- 永続化フォーマット（ファイル保存）
- パラメトリック制約

