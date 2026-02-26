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

## 7. 事前確認結果（2026-02-26）

- `view/render/src/nurbs_eval.rs`
  - `tracing::info!` を 4 箇所確認（フレーム更新系の出力）。
  - Phase9 では `debug!/trace!` への変更、または間引きログ化を検討対象とする。
- `view/render/src/render_2d.rs`
  - `pub device: Arc<wgpu::Device>` を保持していることを確認。
- `view/render/src/render_3d.rs`
  - `pub device: Arc<wgpu::Device>` を保持していることを確認。
- `view/render/src/mesh_convert.rs`
  - テストは `test_mvvm_mesh_conversion` のみで、実データ検証が未実装（プレースホルダ状態）。

## 8. Phase9準備チェックリスト

- [ ] ログ方針を確定（`info!` を削減し、必要な診断性を維持）
- [ ] `Render2D` / `Render3D` の `device` 保持の実利用箇所を確認
- [ ] `mesh_convert` に最小実データの変換テスト追加方針を確定
- [ ] `wireframe.rs` の未使用要素を洗い出し（削除候補のみ整理）
- [ ] 検証コマンドの実行順序を固定（build → clippy）

## 9. 推奨実装順序（Phase9本体）

1. `nurbs_eval.rs` のログレベル最適化（影響範囲が局所的）
2. `mesh_convert.rs` の最小変換テスト追加（品質担保の先行）
3. `render_2d.rs` / `render_3d.rs` の `Arc<Device>` 保持見直し
4. `wireframe.rs` の未使用要素整理
5. `cargo build -p redring` / `cargo clippy -p redring -- -D warnings` で最終確認
