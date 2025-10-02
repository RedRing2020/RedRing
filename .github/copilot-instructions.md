# Copilot Instructions for RedRing

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあり、幾何要素とCAM処理は段階的に実装中です。

## ワークスペース構成

5つのクレートで責務を明確に分離：

- **`model/`**: 幾何データ（Point, Vector, Direction, Curve, Surface）- 純粋な幾何計算、GPU非依存
- **`render/`**: GPU描画基盤（wgpu/WGSL）- `model` に依存せず、独立した描画プリミティブ
- **`stage/`**: レンダリングステージ管理（`RenderStage` トレイト、`OutlineStage`, `DraftStage`, `ShadingStage`）
- **`viewmodel/`**: ビュー操作（現在は placeholder、将来的にカメラ/ビューポート管理）
- **`redring/`**: メインアプリ（winit統合、`App`/`AppState`/`AppRenderer`）- 全クレートを統合

**依存方向**: `redring` → `stage` → `render`（描画）/ `model`（幾何）は独立

## 幾何データ設計 (`model/`)

### 型安全パターン
- **`Direction`**: 正規化保証ベクトル - `Direction::from_vector(v)` は `Option<Direction>` を返す
- **`Vector`**: 汎用3Dベクトル - `normalize()` で `Option<Vector>` を返す
- **失敗可能操作**: ゼロ除算やゼロベクトル正規化は `Option`/`Result` で明示

### モジュール構造
- `geometry/geometry2d/`: 2D要素（Line, Circle, Arc, Point, Vector）
- `geometry/geometry3d/`: 3D要素（Point, Vector, Direction, Plane, Curve, Surface）
- `geometry_trait/`: 抽象化（`Normalize`, `Normed`, `Curve2D`, `Curve3D`, `Surface`）
- `geometry_kind/`: 分類列挙（`CurveKind2D`, `CurveKind3D`, `SurfaceKind`）

### 実装例
```rust
// model/src/geometry/geometry3d/direction.rs
pub fn from_vector(v: Vector) -> Option<Self> {
    v.normalize().map(Direction)
}
```

## GPU描画システム (`render/`)

### WGSL シェーダパターン
シェーダは `render/shaders/*.wgsl` に独立配置し、Rustから `include_str!` で埋め込み：
```rust
// render/src/shader.rs
pub fn render_3d_shader(device: &Device) -> ShaderModule {
    device.create_shader_module(ShaderModuleDescriptor {
        source: ShaderSource::Wgsl(include_str!("../shaders/render_3d.wgsl").into()),
    })
}
```

### 頂点データ
`bytemuck` の `Pod` + `Zeroable` で GPU転送可能な型を定義：
```rust
// render/src/vertex_3d.rs
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
}
```

### レンダリング分離
- `render_2d.rs`: 2Dプリミティブ描画
- `render_3d.rs`: 3D描画（Vertex3D利用）
- `wireframe.rs`: ワイヤーフレーム描画
- `pipeline.rs`: パイプライン生成ヘルパー

## ステージ管理 (`stage/`)

### `RenderStage` トレイト
各レンダリングモードを統一インターフェースで管理：
```rust
pub trait RenderStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView);
    fn update(&mut self) {}  // オプション
}
```

### ステージ実装
- **`OutlineStage`**: ワイヤーフレーム描画（白線）
- **`DraftStage`**: 2D図面モード（青三角形）
- **`ShadingStage`**: 3Dシェーディング（白メッシュ）

### API公開パターン
`stage/src/lib.rs` で `pub use` によるフラット化：
```rust
pub use render_stage::RenderStage;
pub use outline::OutlineStage;
```

## 開発ワークフロー

```bash
# 全体ビルド（注意: model/ に既知のコンパイルエラーあり）
cargo build

# メインアプリ実行
cargo run

# 個別クレートテスト
cargo test -p model
cargo test -p render
cargo test -p stage

# ドキュメント生成（mdbook）
mdbook build  # manual/ → docs/ に出力
mdbook serve  # ローカルサーバー起動
```

## 重要な設計原則

- **責務分離**: 各クレートの役割を厳密に守る（render は model に依存しない）
- **型安全**: 失敗可能操作は `Option`/`Result`、制約は型で表現（`Direction`）
- **トレイト抽象化**: `Curve2D`, `Surface`, `RenderStage` など、共通操作をトレイトで定義
- **シェーダ分離**: WGSL ファイルは独立管理、`include_str!` で埋め込み
- **API フラット化**: `pub use` で利用者が深い階層を意識せず使える

## プロジェクト状態

- **現在**: 描画基盤と幾何要素の基礎実装中（NURBSやCAM処理は未実装）
- **既知の問題**: `model/` に一部未実装機能によるコンパイルエラーあり（将来実装予定）
- **WebAssembly**: 将来対応予定（wgpu は WebGPU 対応済み）
- **ドキュメント**: `manual/` (mdbook) で設計思想と構造を記録
- **進捗管理**: GitHub Issues/Projects で開発タスクを追跡

## デバッグ・ツール

- **ロギング**: `tracing` + `tracing-subscriber` をワークスペース共通で使用
- **エディタ設定**: `.vscode/`, `.editorconfig` で一貫したフォーマット
- **ドキュメント**: 技術詳細は `manual/intro.md` と `manual/philosophy.md` を参照
