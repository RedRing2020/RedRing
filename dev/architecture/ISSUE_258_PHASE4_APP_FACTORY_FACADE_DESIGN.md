# Issue #258 Phase4: app loader/renderer 接続の Facade + Factory 化

- 日付: 2026-02-26
- 対象Issue: #258 (Phase4)
- スコープ: `view/app/src/app_asset_loader.rs`, `view/app/src/app_renderer.rs`, `view/app/src/app_state.rs`, `view/app/src/app_state/debug_scene.rs`

## 背景

Phase3 で loader/renderer 側の重複は一部共通化済みだが、
`app_state` から見た接続責務が「関数群ベース」のため、責務境界の意図が型として読み取りづらい。

## 目的

- loader 接続を `AppAssetLoaderFacade` へ集約し、呼び出し側依存を一本化
- renderer 生成責務を `AppRendererFactory` として明示し、初期化経路を読み取りやすくする
- 既存挙動・既存 API 互換を維持する

## 方針

1. `app_asset_loader.rs`
   - `AppAssetLoaderFacade`（ユニット struct）を追加
   - 既存関数 (`load_stl` / `load_sample_stl_with_bounds` / `load_svg`) は互換維持の薄い委譲として残す
2. `app_renderer.rs`
   - `AppRendererFactory` を追加し、stage + overlay 初期化を factory に集約
   - `AppRenderer::new_draft/new_outline/new_shading` は factory 呼び出しへ委譲
3. `app_state.rs` / `debug_scene.rs`
   - 呼び出し側を Facade/Factory 型経由へ切り替え

## 受け入れ条件

- `cargo fmt` 成功
- `cargo build -p redring` 成功
- `cargo clippy -p redring -- -D warnings` 成功
- 挙動変更なし（デバッグロード/ステージ切替/overlay 描画）

## 非対象

- 新機能追加
- renderer/stage の責務再定義
- model 層への変更
