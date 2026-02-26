# Issue #258 Phase9: render品質改善（ログ抑制・軽量化・テスト補強）

- 対象Issue: #258 (Phase9準備)
- 作成日: 2026-02-26
- スコープ: 挙動不変の品質改善

## 1. 背景

- `nurbs_eval.rs` の描画パスにフレーム毎 `tracing::info!` が含まれ、ログノイズと負荷増加の懸念がある。
- `render_2d.rs` / `render_3d.rs` の `Arc<Device>` 保持は現状用途が限定的で、構造簡素化余地がある。
- `mesh_convert.rs` のテストがプレースホルダで、変換保証が弱い。

## 2. 目的

- ログ方針を統一し、デバッグ容易性と実行時ノイズのバランスを改善
- 不要保持を見直し、リソース構造を軽量化
- 最小単体テストで変換品質を担保

## 3. 実施方針

- `nurbs_eval.rs`:
  - フレーム間引きまたは `debug/trace` レベルへ変更
- `render_2d.rs` / `render_3d.rs`:
  - `Arc<Device>` 保持の必要性を再評価し、不要なら除去
- `mesh_convert.rs`:
  - 最小の `VertexData -> MeshVertex` 検証テストを追加
- `wireframe.rs`:
  - 未使用記述（例: 未使用の頂点レイアウト定義）を整理

## 4. 対象ファイル（予定）

- `view/render/src/nurbs_eval.rs`
- `view/render/src/render_2d.rs`
- `view/render/src/render_3d.rs`
- `view/render/src/mesh_convert.rs`
- `view/render/src/wireframe.rs`

## 5. DoD

- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- 描画結果が既存と同等
- 変換テストが実行される

## 6. リスクと対策

- リスク: ログ削減で調査性が落ちる
- 対策: 環境変数で頻度制御可能な形で残す
