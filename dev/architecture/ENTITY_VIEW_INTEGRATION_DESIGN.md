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
5. **relation モデル方針**
   - group / layer の正本モデルは並列軸とし、View のツリー表示だけを必要に応じて階層化する

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
- 共通表示属性と stroke 表示属性を分離し、線種と線幅は `StrokeDisplayProperties` 経由で受ける
- 2D 線種の表示判定に必要な定義平面情報は `PlanarStroke2DProperties<T>` 経由で受ける
- 失敗は `EntityConvertError` で返す（未対応形状・空データ等）

### 5.3 contract 拡張方針

- `EntityDisplayProperties` には `visible` と `color` だけを残す
- 線種と線幅は `StrokeDisplayProperties` へ分離する
- 2D 線種表示の条件判定に必要な平面情報は `PlanarStroke2DProperties<T>` へ分離する
- これにより、3D mesh entity や solid entity が 2D 線種用の責務を持たずに済む

---

## 6. App設計（`entity_manager.rs`）

## 6.1 管理対象

- `GeometricEntity`（複数形状型を扱うため enum/trait object のどちらかを採用）
- `CAMEntity`
- group 定義と entity-group 所属関係
- layer 定義と entity-layer 所属関係
- 選択状態（`EntityId`）
- dirtyフラグ（再変換が必要か）

現行PoCの `view/app` 側 `EntityManager` は、shape 種別そのものではなく View ローカルな描画キャッシュを保持する。
このため、LineSegment3D / Circle3D / Arc3D のような異なる shape でも、`MeshStage::set_line_data` に渡す `LineList` 頂点列へ落ちた時点では同一の管理対象とみなす。
命名も shape 名ベースの `line_*` ではなく、描画トポロジを表す `line_list_*` を優先する。
さらに将来は 2D / 3D の描画カテゴリを分離し、2D 線種は定義平面に対する直交ビュー時のみ表示する前提で cache を分ける。
group はこの描画 cache 分類とは別軸で持ち、複数所属を許可しつつ、group 単位の表示 ON/OFF を強い制御として扱う。
layer も同様に描画 cache 分類とは別軸で持つが、こちらは図面管理の基準単位として扱い、単一所属を前提とする。
View は layer 中心のツリーや group 一覧を構成してよいが、それは表示用の投影であり、内部 relation を `Layer > Group` 階層へ固定することは意味しない。

## 6.2 最小API

```rust
pub struct EntityManager {
    // 内部実装は enum ベースを推奨
}

impl EntityManager {
    pub fn add_geometric(&mut self, entity: GeometricEntityItem) -> geo_entity::EntityId;
    pub fn add_cam(&mut self, entity: cam_entity::CAMEntity) -> geo_entity::EntityId;
    pub fn remove(&mut self, id: geo_entity::EntityId) -> bool;

   pub fn create_group(&mut self, name: String) -> geo_entity::GroupId;
   pub fn add_entity_to_group(&mut self, entity_id: geo_entity::EntityId, group_id: geo_entity::GroupId) -> bool;
   pub fn remove_entity_from_group(&mut self, entity_id: geo_entity::EntityId, group_id: geo_entity::GroupId) -> bool;
   pub fn set_group_visible(&mut self, group_id: geo_entity::GroupId, visible: bool) -> bool;
   pub fn groups_for_entity(&self, entity_id: geo_entity::EntityId) -> Vec<geo_entity::GroupId>;
   pub fn entities_for_group(&self, group_id: geo_entity::GroupId) -> Vec<geo_entity::EntityId>;

   pub fn create_layer(&mut self, name: String) -> geo_entity::LayerId;
   pub fn add_entity_to_layer(&mut self, entity_id: geo_entity::EntityId, layer_id: geo_entity::LayerId) -> bool;
   pub fn remove_entity_from_layer(&mut self, entity_id: geo_entity::EntityId, layer_id: geo_entity::LayerId) -> bool;
   pub fn set_layer_visible(&mut self, layer_id: geo_entity::LayerId, visible: bool) -> bool;
   pub fn layer_for_entity(&self, entity_id: geo_entity::EntityId) -> Option<geo_entity::LayerId>;
   pub fn entities_for_layer(&self, layer_id: geo_entity::LayerId) -> Vec<geo_entity::EntityId>;

    pub fn select(&mut self, id: geo_entity::EntityId) -> bool;
    pub fn clear_selection(&mut self);
    pub fn selected(&self) -> Option<geo_entity::EntityId>;

    pub fn set_visible(&mut self, id: geo_entity::EntityId, visible: bool) -> bool;
    pub fn set_color(&mut self, id: geo_entity::EntityId, color: [f32; 4]) -> bool;

      // 現行PoCでは wireframe 系 shape を LineList 頂点として保持する
      pub fn add_line_list_entity(&mut self, vertices: Vec<MeshVertex>) -> geo_entity::EntityId;
      pub fn line_list_vertices(&self) -> Vec<MeshVertex>;

   // 将来拡張では 2D / 3D の描画カテゴリ別 cache を導入する
   pub fn add_2d_line_pattern_entity(&mut self, entity: Pattern2DDisplayItem) -> geo_entity::EntityId;
   pub fn add_3d_wireframe_entity(&mut self, entity: Wireframe3DDisplayItem) -> geo_entity::EntityId;
   pub fn add_3d_mesh_entity(&mut self, entity: Mesh3DDisplayItem) -> geo_entity::EntityId;

    pub fn is_dirty(&self) -> bool;
    pub fn clear_dirty(&mut self);
}
```

## 6.3 group 可視制御の扱い

- group 可視状態は entity 自身の `visible` に上乗せされる上位フィルタとする
- entity が複数 group に属する場合、1つでも `visible = false` の group があれば非表示とする
- group 未所属 entity は group 側要因では非表示化されない
- 有効表示判定は以下とする

```text
effective_visible = entity.visible AND all_visible(groups_for_entity(entity))
```

- `all_visible(empty)` は `true` とみなす
- この規則により、group 所属が複数に跨っても group ON/OFF が強い制御になる
- View 側は group ごとに別バッファを維持するのではなく、可視 entity の再収集時にこの条件を評価する

## 6.4 layer 可視制御と所属制約の扱い

- layer 可視状態も entity 自身の `visible` に上乗せされる上位フィルタとする
- entity は最大 1 layer にのみ属する
- 所属 layer が `visible = false` の場合、その entity は非表示とする
- layer 未所属 entity は layer 側要因では非表示化されない
- 有効表示判定は以下とする

```text
effective_visible = entity.visible
                 AND all_visible(groups_for_entity(entity))
                 AND layer_visible_or_true(layer_for_entity(entity))
```

- `layer_visible_or_true(None)` は `true` とみなす
- layer 所属追加時は既存 layer 所属の有無を検証し、重複所属を View ローカルでも防げるようにする
- layer は group より図面管理寄りの単位であるため、将来の既定色・既定線種・ロック制御の付与先として拡張しやすい形を保つ

## 6.5 EntityManager の内部責務整理

- `EntityManager` は Model 正本の group relation store そのものにはならない
- ただし View ローカルで必要な最小の group / layer 可視状態と membership 参照結果は保持してよい
- 初期段階では以下の分離を推奨する
   1. entity display cache
   2. group visibility state
   3. layer visibility state
   4. entity-group membership index
   5. entity-layer index
- group / layer 名やツリー構造など UI 表示用データは query DTO から再投入してよく、描画 cache と同一構造へ混在させない

## 6.6 UI ハイブリッド運用の扱い

- View は layer 主体のツリー表示を提供してよい
- ただし group は layer の子ノードとして固定せず、必要に応じて以下の複数ビューを併存させてよい
   1. layer 中心ビュー
   2. group 中心ビュー
   3. entity 詳細ビュー
- layer 中心ビューで group を補助表示する場合も、それは relation の集計結果であり parent-child 正本構造ではない
- この方針により、操作感としては階層的に見せつつ、複数所属 group の柔軟性を維持する

---

## 7. `app_state.rs` 統合ポイント

1. `AppState` に `entity_manager: EntityManager` を追加
2. 既存 `load_debug_line` / `load_debug_toolpath` に「Entity経由パス」を追加
3. `rebuild_stage_from_entities()` を新設
   - EntityManagerの可視エンティティを収集
   - entity 自身の visible と group / layer 可視状態を合成して有効表示を判定
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
- group 作成、group 所属追加/解除、group 可視切替の基本操作
- 複数 group 所属時に 1つでも非表示 group があれば有効表示が false になる
- group 未所属 entity は entity.visible のみで判定される
- layer 作成、layer 所属追加/解除、layer 可視切替の基本操作
- 既に layer 所属を持つ entity へ別 layer を追加しようとすると失敗する
- 所属 layer が非表示の場合に有効表示が false になる
- dirty時のみ `rebuild_stage_from_entities()` 実行
- 2D 線種はビュー始点が定義平面に直交しない場合、非表示または実線フォールバックとなる

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

