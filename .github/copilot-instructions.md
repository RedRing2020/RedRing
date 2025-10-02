# Copilot Instructions for RedRing

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあります。

## アーキテクチャ

- **Workspace 構成**: `model`, `render`, `redring`(main), `viewmodel`, `stage` の 5 つのクレート
- **responsibility separation**: 幾何データ(model)、GPU 描画(render)、UI(redring)、ビュー操作(viewmodel)、シーン管理(stage)
- **依存関係**: `redring` がすべてを統合、`render` は `model` に依存しない

## 幾何データの設計パターン (`model/`)

- **階層構造**: `geometry/geometry3d/` に `Point`, `Vector`, `Direction` など基本要素
- **トレイト設計**: `geometry_trait/` に `Normalize`, `Normed`, `Curve2D`, `Curve3D` など
- **型安全な方向ベクトル**: `Direction` は正規化されたベクトルをラップ
- **例**: `model/src/geometry/geometry3d/direction.rs` の `Direction::from_vector()` で安全な変換

## GPU 描画システム (`render/`)

- **wgpu + WGSL**: シェーダは `render/shaders/*.wgsl` に分離
- **頂点データ**: `vertex_3d.rs` で `bytemuck` を使った `Pod` + `Zeroable` パターン
- **シェーダローダー**: `shader.rs` で `include_str!` を使ったコンパイル時埋め込み
- **レンダリングパイプライン**: `render_2d.rs`, `render_3d.rs`, `wireframe.rs` で用途別に分離

## シーン管理 (`stage/`)

- **RenderStage トレイト**: すべてのステージが実装する共通インターフェース
- **ステージ種別**: `DraftStage` (2D描画), `OutlineStage` (ワイヤフレーム), `ShadingStage` (3Dシェーディング)
- **切り替え機構**: `AppState::set_stage_*()` でステージを動的に変更
- **例**: `stage/src/draft.rs` で `RenderStage` トレイトを実装し、`render()` と `update()` を定義

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

- **Option/Result 活用**: `Direction::from_vector()` など、失敗可能な操作には `Option<T>` を使用
- **トレイト境界**: 幾何操作は専用トレイトで抽象化（`Normalize`, `Normed` など）
- **モジュール公開**: 各クレートの `lib.rs` で `pub use` によるフラットな公開 API
- **WGSL 統合**: シェーダファイルは独立管理、Rust から `include_str!` で参照

## ViewModel パターン (`viewmodel/`)

- **現在のステータス**: プレースホルダーとして存在、本格実装は未着手
- **今後の役割**: カメラ制御、ビュー変換、ユーザー入力の変換などを担当予定

## プロジェクトの現状と制約

- **実装済み**: 描画基盤 (wgpu/winit)、基本的な幾何要素、ステージ切り替え機構
- **未実装**: NURBS、プリミティブ形状、CAM パス生成、切削シミュレーション
- **WebAssembly 対応**: 依存関係に記載はあるが、本格対応は今後の開発予定
- **ビルド状態**: `model` クレートに未実装トレイトメソッドによるコンパイルエラーあり（開発途上）

## テスト戦略

- **個別クレートテスト**: `cargo test -p <crate_name>` で各クレート単位でテスト
- **現状**: 基本的なプレースホルダーテスト（`viewmodel` など）のみ存在
- **推奨**: 幾何演算の正確性検証、GPU リソース管理の妥当性検証を今後追加

## デバッグ・開発支援

- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- 進捗・設計詳細は GitHub Issues/Projects で管理
- ドキュメントは `manual/` (mdbook) で技術詳細を記録
- ビルドエラーは開発途上の状態を反映（未実装機能による警告・エラーは既知の課題）
