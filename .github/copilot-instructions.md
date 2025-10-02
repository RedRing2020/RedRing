# Copilot Instructions for RedRing

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあり、**コンパイルエラーが存在する開発中のプロジェクト**です。

## ⚠️ プロジェクトの現状

- **開発初期段階**: 幾何要素やCAM処理は未実装、段階的に導入予定
- **コンパイルエラーあり**: `model` クレートに既知のエラーが存在（メソッド未実装など）
- **タスクルール**: 無関係なビルドエラーは修正しない。タスク関連のエラーのみ対応

## アーキテクチャ

### Workspace 構成と依存関係

```
redring (main) ──┬─→ model (幾何データ、依存なし)
                 ├─→ viewmodel (ビュー変換、model に依存)
                 ├─→ render (GPU描画、model に依存しない)
                 └─→ stage (シーン管理、render に依存)
```

- **model**: 幾何データ（Point, Vector, Direction, Curve, Surface等）、依存なし
- **render**: GPU描画基盤（wgpu, WGSL シェーダ）、model と独立
- **stage**: レンダリングシーン管理（`RenderStage` トレイト）
- **viewmodel**: ビュー操作（現在はプレースホルダーコード）
- **redring**: メインアプリ、全クレートを統合

## 幾何データの設計パターン (`model/`)

### 階層構造

- `geometry/geometry2d/`: 2D幾何要素（Point, Vector, Direction, Line, Circle, Arc等）
- `geometry/geometry3d/`: 3D幾何要素（Point, Vector, Direction, Plane, Curve等）
- `geometry_kind/`: 型分類 Enum（`CurveKind2D`, `CurveKind3D`, `SurfaceKind`）
- `geometry_trait/`: 抽象化トレイト（`Normalize`, `Normed`, `Curve2D`, `Curve3D`, `Surface`）

### 重要なパターン

- **型安全な Direction**: `Direction::from_vector()` は `Option<Direction>` を返し、ゼロベクトルを安全に扱う
  ```rust
  // model/src/geometry/geometry3d/direction.rs
  pub fn from_vector(v: Vector) -> Option<Self> {
      v.normalize().map(Direction)
  }
  ```

- **Enum による型分類**: `CurveKind2D` で曲線種別を明示
  ```rust
  // model/src/geometry_kind/curve2d.rs
  pub enum CurveKind2D {
      InfiniteLine, Ray, Line, Circle, Arc, Ellipse, EllipseArc, NurbsCurve, Unknown
  }
  ```

- **フラットな公開API**: `lib.rs` で `pub use` により階層を隠蔽
  ```rust
  // model/src/geometry/geometry3d/mod.rs
  pub use point::Point;
  pub use vector::Vector;
  pub use direction::Direction;
  ```

## GPU 描画システム (`render/`)

### WGSL シェーダ統合

- シェーダは `render/shaders/*.wgsl` に独立管理
- `include_str!` でコンパイル時埋め込み
  ```rust
  // render/src/shader.rs
  source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/render_3d.wgsl").into())
  ```

### 頂点データパターン

- `bytemuck` の `Pod` + `Zeroable` でメモリ安全性を確保
  ```rust
  // render/src/vertex_3d.rs
  #[repr(C)]
  #[derive(Copy, Clone, Debug, Pod, Zeroable)]
  pub struct Vertex3D {
      pub position: [f32; 3],
  }
  ```

### レンダリングパイプライン分離

- `render_2d.rs`: 2D描画（Draft用）
- `render_3d.rs`: 3D描画（Shading用）
- `wireframe.rs`: ワイヤーフレーム描画

## シーン管理 (`stage/`)

### RenderStage トレイトパターン

すべてのステージは `RenderStage` トレイトを実装：
```rust
// stage/src/render_stage.rs
pub trait RenderStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);
    fn update(&mut self) {}
}
```

実装例:
- `DraftStage`: 2D描画（アニメーション三角形）
- `ShadingStage`: 3D描画（将来的にカメラ制御追加予定）
- `OutlineStage`: アウトライン描画

## 開発ワークフロー

### ビルドとテスト

```bash
# 全体ビルド（エラーあり）
cargo build

# 個別クレートのビルド（render/stage は成功する）
cargo build -p render
cargo build -p stage

# テスト実行
cargo test -p render
cargo test -p model  # エラーにより失敗する可能性

# メインアプリ実行（model のエラー修正後）
cargo run
```

### ドキュメント生成

```bash
# mdbook でドキュメント生成
mdbook build  # manual/ -> docs/ に出力

# book.toml 設定:
# src = "manual"
# build-dir = "docs"
```

## 重要な設計原則

### 型安全性とエラーハンドリング

- **Option/Result 活用**: 失敗可能な操作には `Option<T>` を必ず使用
- **型分類 Enum**: `CurveKind2D/3D` で動的な型判定を補完

### トレイト設計

- **抽象化**: `Curve2D`, `Curve3D`, `Surface` で共通インターフェース定義
- **特殊化**: `Normalize`, `Normed` で特定操作を分離

### モジュール設計

- **フラットAPI**: `pub use` で階層を隠し、ユーザー側のインポートを簡素化
- **責務分離**: render は model に依存せず、GPU描画のみに集中

## 技術スタック

- **Rust Edition 2024**: 全クレートで統一
- **wgpu 26.0.1**: GPU描画（WebGPU準拠）
- **winit 0.30**: ウィンドウ管理
- **bytemuck 1.23.2**: 型安全なバイト変換
- **tracing**: ロギング（workspace 共通依存）
- **mdBook**: ドキュメント生成（manual/ -> docs/）

## 将来の拡張方向

- **WebAssembly対応**: 計画中（現在は未実装）
- **CAM機能**: 切削シミュレーション、パス生成（未実装）
- **NURBS/STEP対応**: 設計で考慮中
- **多言語ドキュメント**: mdBook 多言語化予定

## デバッグ・開発支援

- **tracing**: `tracing` + `tracing-subscriber` で構造化ログ
- **進捗管理**: GitHub Issues/Projects で詳細な開発状況を追跡
- **技術ドキュメント**: `manual/` (mdbook) に設計思想・モジュール構成を記録
