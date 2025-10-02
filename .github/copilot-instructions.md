# Copilot Instructions for RedRing

RedRingは、Rust + wgpu によるCAD/CAM研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあります。

## アーキテクチャ
- **Workspace構成**: `model`, `render`, `redring`(main), `viewmodel`, `stage` の5つのクレート
- **responsibility separation**: 幾何データ(model)、GPU描画(render)、UI(redring)、ビュー操作(viewmodel)、シーン管理(stage)
- **依存関係**: `redring` がすべてを統合、`render` は `model` に依存しない

## 幾何データの設計パターン (`model/`)
- **階層構造**: `geometry/geometry3d/` に `Point`, `Vector`, `Direction` など基本要素
- **トレイト設計**: `geometry_trait/` に `Normalize`, `Normed`, `Curve2D`, `Curve3D` など
- **型安全な方向ベクトル**: `Direction` は正規化されたベクトルをラップ
- **例**: `model/src/geometry/geometry3d/direction.rs` の `Direction::from_vector()` で安全な変換

## GPU描画システム (`render/`)
- **wgpu + WGSL**: シェーダは `render/shaders/*.wgsl` に分離
- **頂点データ**: `vertex_3d.rs` で `bytemuck` を使った `Pod` + `Zeroable` パターン
- **シェーダローダー**: `shader.rs` で `include_str!` を使ったコンパイル時埋め込み
- **レンダリングパイプライン**: `render_2d.rs`, `render_3d.rs`, `wireframe.rs` で用途別に分離

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

## 重要な設計原則
- **Option/Result活用**: `Direction::from_vector()` など、失敗可能な操作には `Option<T>` を使用
- **トレイト境界**: 幾何操作は専用トレイトで抽象化（`Normalize`, `Normed` など）
- **モジュール公開**: 各クレートの `lib.rs` で `pub use` によるフラットな公開API
- **WGSL統合**: シェーダファイルは独立管理、Rustから `include_str!` で参照

## デバッグ・開発支援
- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- 進捗・設計詳細は GitHub Issues/Projects で管理
- ドキュメントは `manual/` (mdbook) で技術詳細を記録
