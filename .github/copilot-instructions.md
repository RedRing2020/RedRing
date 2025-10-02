# Copilot Instructions for RedRing

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあります。

## アーキテクチャ

- **Workspace 構成**: `model`, `render`, `redring`(main), `viewmodel`, `stage` の 5 つのクレート
- **responsibility separation**: 幾何データ(model)、GPU 描画(render)、UI(redring)、ビュー操作(viewmodel)、シーン管理(stage)
- **依存関係**: `redring` がすべてを統合、`render` は `model` に依存しない（GPUレンダリングと幾何演算の分離）

## 幾何データの設計パターン (`model/`)

- **階層構造**: `geometry/geometry3d/` に `Point`, `Vector`, `Direction` など基本要素
- **トレイト設計**: `geometry_trait/` に `Normalize`, `Normed`, `Curve2D`, `Curve3D` など
- **型安全な方向ベクトル**: `Direction` は正規化されたベクトルをラップ
- **例**: `model/src/geometry/geometry3d/direction.rs` の `Direction::from_vector()` で安全な変換

## GPU 描画システム (`render/`)

- **wgpu + WGSL**: シェーダは `render/shaders/*.wgsl` に分離
- **頂点データ**: `vertex_3d.rs` で `bytemuck` を使った `Pod` + `Zeroable` パターン
  - `#[repr(C)]` + `#[derive(Pod, Zeroable)]` で GPU メモリレイアウトを保証
  - 例: `Vertex3D { position: [f32; 3] }` は直接 GPU バッファに転送可能
- **シェーダローダー**: `shader.rs` で `include_str!` を使ったコンパイル時埋め込み
- **レンダリングパイプライン**: `render_2d.rs`, `render_3d.rs`, `wireframe.rs` で用途別に分離

## シーン管理システム (`stage/`)

- **RenderStage トレイト**: 各描画ステージの統一インターフェース（`render()` + `update()`）
- **実装例**:
  - `OutlineStage`: ワイヤーフレーム描画（編集時のエッジ表示用）
  - `DraftStage`: 2D プレビュー描画（アニメーション更新機能付き）
  - `ShadingStage`: 3D シェーディング描画（将来的にカメラ制御を追加予定）
- **パターン**: 各ステージは独自の `resources` を保持し、`update()` でフレーム毎の状態更新を実装

## 開発ワークフロー

```powershell
# 全体ビルド
cargo build

# メインアプリ実行
cargo run

# 個別クレートのテスト
cargo test -p model
cargo test -p render

# ドキュメント生成 (mdbook使用)
mdbook build  # manual/ -> docs/ に生成
```

### 現在の状態と制約

- **ビルドエラー**: `model` クレートに未実装トレイトメソッドあり（開発中）
- **テスト**: 現在は `viewmodel/src/lib.rs` のみに基本テストあり
- **WebAssembly**: 将来対応予定（README記載）だが、現状は native のみ

## 重要な設計原則

- **Option/Result 活用**: `Direction::from_vector()` など、失敗可能な操作には `Option<T>` を使用
- **トレイト境界**: 幾何操作は専用トレイトで抽象化（`Normalize`, `Normed` など）
- **モジュール公開**: 各クレートの `lib.rs` で `pub use` によるフラットな公開 API
- **WGSL 統合**: シェーダファイルは独立管理、Rust から `include_str!` で参照
- **型安全性**: `Direction` は正規化保証、`#[repr(C)]` で GPU メモリレイアウト制御

## ViewModelパターン（今後実装予定）

- 現在 `viewmodel/src/lib.rs` はスタブ実装（`add()` 関数のみ）
- 将来的にカメラ制御、ビュー変換、インタラクション管理を担当予定

## テスト戦略

- 現状: 最小限のテストインフラ（`viewmodel` に基本テストのみ）
- 推奨: クレート毎に `#[cfg(test)] mod tests` でユニットテスト追加
- 幾何演算は数値精度テストが重要（浮動小数点比較に注意）

## CAM機能（将来実装予定）

- 現在未実装（README/manual に記載）
- 設計方針: 幾何カーネル(`model`)上にツールパス生成・切削シミュレーション層を構築予定
- STEP/NURBS 対応も視野に入れた拡張可能な設計

## デバッグ・開発支援

- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- 進捗・設計詳細は GitHub Issues/Projects で管理
- ドキュメントは `manual/` (mdbook) で技術詳細を記録
