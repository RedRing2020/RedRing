# Issue #258 Phase7: render_2d/render_3d 重複削減

- 対象Issue: #258 (Phase7準備)
- 作成日: 2026-02-26
- スコープ: `view/render` の2D/3D初期描画実装の重複解消

## 1. 背景

`render_2d.rs` と `render_3d.rs` は、頂点型差分を除きほぼ同一の初期化・パイプライン生成・draw処理を持つ。
重複が高く、変更時に差分漏れが発生しやすい。

## 2. 目的

- 重複コードを削減し、保守性を向上
- API互換を維持し、`view/stage` 側影響を最小化
- 挙動不変（既存描画結果を維持）

## 3. 実施方針

- 共通化対象:
  - 三角形サンプル頂点バッファ初期化
  - `RenderPipelineDescriptor` 構築の共通ロジック
  - `draw_*` の実行手順
- 互換維持:
  - 既存の公開関数名は維持し、内部で共通ヘルパを利用
- 禁止事項:
  - 見た目変更、トポロジ変更、描画アルゴリズム変更

## 4. 対象ファイル（予定）

- `view/render/src/render_2d.rs`
- `view/render/src/render_3d.rs`
- `view/render/src/pipeline.rs`

## 5. DoD

- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- `render_2d` / `render_3d` の公開API互換維持
- 目視差分で重複初期化コードが減少

## 6. リスクと対策

- リスク: pipeline共通化で片側の設定差分を取りこぼす
- 対策: 2D/3D差分（頂点レイアウト・深度設定）を明示パラメータ化
