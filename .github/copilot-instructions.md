# Copilot Instructions for RedRing

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあり、NURBS やCAM機能は未実装です。

## アーキテクチャ

### Workspace 構成
- `model`: 幾何データ（Point, Vector, Curve, Surface など）、`model` に依存なし
- `render`: GPU 描画基盤（wgpu + WGSL）、`model` に依存しない設計
- `viewmodel`: ビュー操作・変換ロジック、`model` に依存
- `stage`: レンダリングステージ管理（`RenderStage` トレイト）、`render` に依存
- `redring`: メインアプリケーション、すべてのクレートを統合

### 依存関係の方向性
```
redring → viewmodel → model
       ↘  stage → render
```
**重要**: `render` は `model` に依存しない（GPU層と幾何データ層の分離）

## 幾何データの設計パターン (`model/`)

### 階層構造
- `geometry/geometry3d/`: `Point`, `Vector`, `Direction`, `Line`, `Circle`, `Ellipse`, `NurbsCurve` など
- `geometry/geometry2d/`: 2次元対応の基本要素
- `geometry_trait/`: `Curve2D`, `Curve3D`, `Surface`, `Normalize`, `Normed` など
- `geometry_kind/`: `CurveKind3D`, `SurfaceKind` による型分類

### 型安全パターン
```rust
// Direction は正規化されたベクトルをラップ
pub struct Direction(Vector);

impl Direction {
    pub fn from_vector(v: Vector) -> Option<Self> {
        v.normalize().map(Direction)  // normalize() は Option<Vector> を返す
    }
}
```

### トレイト設計
```rust
// Curve3D: 各曲線型が実装する共通インターフェース
pub trait Curve3D: Any {
    fn kind(&self) -> CurveKind3D;
    fn evaluate(&self, t: f64) -> Point;
    fn derivative(&self, t: f64) -> Vector;
    fn length(&self) -> f64;
}
```

## GPU 描画システム (`render/`)

### シェーダ管理
```rust
// shader.rs: コンパイル時埋め込みパターン
pub fn render_3d_shader(device: &Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render 3D Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_3d.wgsl").into()),
    })
}
```
シェーダは `render/shaders/` に分離: `render_2d.wgsl`, `render_3d.wgsl`, `wireframe.wgsl`

### 頂点データパターン
```rust
// vertex_3d.rs: bytemuck による GPU 転送可能な型
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
}
```
**必須**: GPU に送るデータは `#[repr(C)]` + `Pod` + `Zeroable` を実装

## レンダリングステージ (`stage/`)

### RenderStage トレイト
```rust
pub trait RenderStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);
    fn update(&mut self) {}  // デフォルト実装あり
}
```
実装例: `OutlineStage`, `DraftStage`, `ShadingStage`

## 開発ワークフロー

```bash
# 全体ビルド（現在ビルドエラーあり - 設計段階のため）
cargo build

# メインアプリ実行
cargo run

# 個別クレートのテスト
cargo test -p model
cargo test -p render

# ドキュメント生成
mdbook build  # manual/ -> docs/ に生成

# クレート間依存確認
cargo tree --depth 1
```

## 重要な設計原則

### 1. Option/Result による失敗の明示化
```rust
Direction::from_vector(v)  // Option<Direction> を返す
v.normalize()               // Option<Vector> を返す（ゼロベクトルは None）
```

### 2. トレイト境界による抽象化
- `Normalize` トレイト: 正規化可能な型を抽象化
- `Curve2D`/`Curve3D` トレイト: 曲線の共通操作を定義
- 動的ディスパッチには `dyn Trait` または `Any` によるダウンキャストを使用

### 3. モジュール公開 API
各クレートの `lib.rs` で `pub use` により主要型を再エクスポート
```rust
pub mod geometry;
pub mod geometry_trait;
pub mod geometry_kind;
```

### 4. WGSL シェーダの統合
- シェーダは独立した `.wgsl` ファイルで管理
- `include_str!` でコンパイル時に埋め込み
- シェーダ変更時は Rust 側の再ビルドが必要

## ドキュメント管理

- **技術ドキュメント**: `manual/` (mdbook) → `docs/` に生成
- **構造**: `intro.md`, `modules.md`, `kinds.md`, `philosophy.md`
- **設計方針**: 責務分離、型安全性、国際化対応（英語中心）
- **README.md**: 安定機能のみ記載、詳細は Issues/Projects 参照

## デバッグ・トレース

- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- ログレベルは `RUST_LOG` 環境変数で制御
- 進捗管理: GitHub Issues/Projects で追跡

## 現在の状態と制約

- **ビルド状況**: 一部のトレイト実装が未完成で現在ビルドエラーあり
- **未実装機能**: NURBS の完全実装、CAM パス生成、切削シミュレーション
- **WebAssembly**: 将来対応予定（現在は wgpu のネイティブバックエンドのみ）
- **viewmodel**: 現在は最小実装（今後の拡張予定）

コードを変更する際は、既存のトレイト設計と責務分離を尊重し、型安全性を保つことを優先してください。
