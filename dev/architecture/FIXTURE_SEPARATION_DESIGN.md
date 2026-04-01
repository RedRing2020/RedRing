# Fixture Separation Design

**作成日**: 2026年4月1日
**関連Issue**: #516 - sample/demo データ生成を ViewModel から分離する方針
**ステータス**: 設計提案

---

## 1. 背景

現在の RedRing では、sample/demo 用データ生成が複数層に分散している。

- `model/cam_core/src/fixtures.rs` には `ToolPath` と `Tool` の sample 生成がある
- `model/geo_algorithms/src/octree/fixtures.rs` には `VoxelOctree` の sample 生成がある
- `model/geo_algorithms/src/nurbs_fixtures.rs` には NURBS sample がある
- 一方で `viewmodel/converter` には `create_sample_*` 系 API が残っており、変換責務と生成責務が混在している
- `view/app/src/app_state/debug_scene/` には UI 操作と sample シナリオ選択が同居している

Issue #516 の論点は、ViewModel から sample/demo データ生成責務を外し、既に進めている「owner クレート内 fixtures へ寄せる」方針を明文化して、残存箇所の整理方針を確定することにある。

---

## 2. 現状整理

### 2.1 すでに owner が明確な sample

以下はすでに「どのクレートが sample を持つべきか」が比較的明確である。

| 種別 | 現在の配置 | 評価 |
|---|---|---|
| `ToolPath<f64>` | `cam_core::fixtures` | 妥当 |
| `Tool<f64>` | `cam_core::fixtures` | 妥当 |
| `VoxelOctree<f64>` | `geo_algorithms::octree::fixtures` | 妥当 |
| NURBS surface | `geo_algorithms::nurbs_fixtures` | 概ね妥当 |

この時点で、選択肢 1 の「各ドメインクレート内 fixtures」は既に部分採用済みであり、#516 は新方式の導入というより既存方針の継続と未整理箇所の回収に近い。

### 2.2 ViewModel 側に残っている混在

| ファイル | 現状 | 問題 |
|---|---|---|
| `viewmodel/converter/src/toolpath_converter.rs` | `cam_core::fixtures` への薄い委譲 | 互換 API としては成立するが、変換層に sample 入口が残る |
| `viewmodel/converter/src/octree_converter.rs` | sample `VoxelOctree` を生成して wireframe に変換 | 生成と変換が同居 |
| `viewmodel/converter/src/nurbs_eval_loader.rs` | sample NURBS を読み出して評価 | sample 選択導線が変換層に残る |
| `viewmodel/converter/src/snapshot_converter.rs` | demo 実行まで含めた sample 系 API | orchestration と変換が同居 |
| `viewmodel/converter/src/cam_sim_visualization_converter.rs` | sample toolpath/tool 選択、simulation 実行、ViewModel bundle 構築が同居 | 変換層を超えている |
| `viewmodel/converter/src/stl_loader.rs` | sample STL ファイルを生成してから読み込み | I/O fixture と loader が同居 |

---

## 3. 選択肢比較

### Option 1: 各ドメインクレート内 `fixtures` に分散配置

#### メリット

- データ owner と fixture owner が一致しやすい
- 依存方向を壊しにくい
- 既存の `cam_core::fixtures` / `geo_algorithms::octree::fixtures` をそのまま拡張できる
- unit test で同じ fixture を流用しやすい

#### デメリット

- 複数クレートに跨るデモシナリオは置き場が曖昧になる
- UI デモ用の「完成シナリオ」を 1 箇所で眺めにくい

#### 評価

RedRing の現在構造と最も整合する。既存実装とも一致しており、基本方針の継続先として最有力。

### Option 2: 専用クレート `cam_fixtures` を新設

#### メリット

- sample/demo データを 1 箇所へ集約できる
- 将来的に大量のデモシナリオを持つ場合は一覧性が高い

#### デメリット

- `cam_core` だけでなく `geo_algorithms` や `geo_io` 由来 fixture も必要になり、依存集約クレートになりやすい
- 小さな fixture まで新クレート経由にすると、owner がぼやける
- RedRing の層境界に対して「sample だけの横断クレート」を増やす価値がまだ小さい

#### 評価

現段階では過剰。特に #516 の対象は「ViewModel から分離」が目的であり、「fixture を 1 クレートへ統合」は必須条件ではない。

### Option 3: `fixtures` feature flag で同クレート内に残す

#### メリット

- 本番ビルドから debug fixture を外しやすい
- バイナリサイズや API 面積の制御余地がある

#### デメリット

- 置き場問題そのものは解決しない
- feature 組み合わせのテスト負荷が増える
- 現状の RedRing では sample API 自体の存在が大きな問題であり、feature 化は第二段の最適化に近い

#### 評価

初手の設計決定には採用しない。必要なら実装完了後の整理項目として扱う。

---

## 4. 推奨方針

### 方針A: fixture は owner クレートへ置く

sample データの生成責務は、原則として生成対象の型を所有するクレートへ置く。

- `ToolPath` / `Tool` / machine constraint 系 sample: `cam_core::fixtures`
- `VoxelOctree` / NURBS / 幾何 sample: `geo_algorithms` 側 fixture
- STL 等のファイル sample: `geo_io` もしくは asset loader 側 fixture helper

ViewModel は「与えられた domain object を表示用データへ変換する」責務に限定する。

### 方針B: 複数クレートを跨ぐ demo シナリオは Application へ置く

複数の domain fixture を組み合わせて demo を実行する処理は、fixture ではなく Application に置く。

対象例:

- CAM simulation 用に `ToolPath + Tool + work bounds + snapshot export` を組み立てる
- 複数 fixture を束ねて debug scene 用 DTO を返す

理由:

- これは単一型の sample 生成ではなく「処理の組み立て」であり、orchestration に近い
- `viewmodel/converter/src/cam_sim_visualization_converter.rs` の sample 実行導線は、この責務が漏れている例である

### 方針C: ViewModel の sample API は段階的に削る

ViewModel に残る `create_sample_*` は次のいずれかに整理する。

- 一時互換の薄い委譲 API として残し、実装 Issue で削除対象にする
- 即時に削除し、呼び出し側を owner fixture / application service へ付け替える

ただし、sample を前提にした「表示専用 convenience API」は残さない。

---

## 5. 配置ルール

### 5.1 分類ルール

1. **単一ドメイン型を返すだけ**
   - owner クレートの `fixtures` へ置く

2. **複数ドメイン型を組み合わせて処理を実行する**
   - Application の demo/orchestration モジュールへ置く

3. **表示用頂点や wireframe を返す**
   - sample 生成ではなく変換結果なので、入力は外から受ける
   - ViewModel に残すのは `*_to_vertices` / `*_to_wireframe` のみ

4. **ファイル生成を伴うサンプル**
   - `geo_io` 側の fixture helper か、App の asset demo helper に分離する
   - loader 本体と同じファイルへ置かない

### 5.2 命名ルール

- owner クレートの fixture: `create_sample_*`
- Application の demo シナリオ: `build_demo_*` または `run_demo_*`
- ViewModel の変換: `*_to_*`

ViewModel に `create_sample_*` を新設しない。

---

## 6. 具体的な移行先

| 現在 | 移行方針 |
|---|---|
| `toolpath_converter::load_demo_toolpath()` | 生成本体は `cam_core::fixtures` へ集約し、ViewModel は層境界用の薄い facade のみ保持 |
| `octree_converter::create_sample_voxel_octree_wireframe*()` | sample tree 生成は `geo_algorithms::octree::fixtures` に限定し、ViewModel 側は `voxel_octree_to_wireframe*` のみ残す |
| `nurbs_eval_loader::create_sample_nurbs_surface_eval()` | sample surface 取得は `geo_algorithms::nurbs_fixtures`、評価と変換だけ ViewModel に残す |
| `snapshot_converter::load_demo_cam_snapshot_domain_series()` | Application 側の demo 実行モジュール経由に統一し、ViewModel は DTO 変換のみ保持 |
| `cam_sim_visualization_converter::build_demo_cam_simulation_visualization_bundle*()` | demo scenario 構築を Application 側へ分離。ViewModel converter は request/result 変換へ寄せる |
| `stl_loader::create_sample_stl_mesh()` | `geo_io` または App asset 側の fixture helper へ移し、loader と分離 |

---

## 7. 実装順序

### Phase 1: wrapper 除去の影響が小さい箇所から移す

- `toolpath_converter` の sample wrapper 整理
- `octree_converter` の sample 生成 API 整理
- `nurbs_eval_loader` の sample 選択導線整理

### Phase 2: orchestration 混在箇所を切る

- `snapshot_converter` の sample series 生成を Application へ移す
- `cam_sim_visualization_converter` を
  - demo scenario 構築
  - simulation 実行
  - ViewModel bundle 変換
 へ分離する

### Phase 3: file fixture を分離する

- `stl_loader` の sample STL 生成を別モジュール化

---

## 8. 結論

Issue #516 では、以下を設計方針として採用する。

1. sample fixture の基本配置は **各 owner クレート内 `fixtures` モジュール** とする
2. **複数 fixture を束ねる demo シナリオは Application 層** に置く
3. ViewModel は **変換専用** とし、新しい `create_sample_*` を持たない
4. `fixtures` feature flag と専用 fixture クレート新設は、現時点では採用しない

この方針は新規方針というより、既に進めている owner クレート fixtures への分離を設計上明文化するものである。これにより、ViewModel から生成責務を外しつつ、RedRing の既存アーキテクチャと依存方向を維持できる。