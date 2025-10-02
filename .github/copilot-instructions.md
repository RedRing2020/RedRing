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

## シーン管理システム (`stage/`)

- **RenderStage トレイト**: 各描画ステージの統一インターフェース（`render()` + `update()`）
- **実装例**:
  - `OutlineStage`: ワイヤーフレーム描画（編集時のエッジ表示用）
  - `DraftStage`: 2D プレビュー描画（アニメーション更新機能付き）
  - `ShadingStage`: 3D シェーディング描画（将来的にカメラ制御を追加予定）
- **パターン**: 各ステージは独自の `resources` を保持し、`update()` でフレーム毎の状態更新を実装

## シーン管理 (`stage/`)

- **RenderStage トレイト**: すべてのステージが実装する共通インターフェース
- **ステージ種別**: `DraftStage` (2D描画), `OutlineStage` (ワイヤフレーム), `ShadingStage` (3Dシェーディング)
- **切り替え機構**: `AppState::set_stage_*()` でステージを動的に変更
- **例**: `stage/src/draft.rs` で `RenderStage` トレイトを実装し、`render()` と `update()` を定義

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

### 現在の状態と制約

- **ビルドエラー**: `model` クレートに未実装トレイトメソッドあり（開発中）
- **テスト**: 現在は `viewmodel/src/lib.rs` のみに基本テストあり
- **WebAssembly**: 将来対応予定（README記載）だが、現状は native のみ

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
