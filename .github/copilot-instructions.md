# Copilot Instructions for RedRing

**最終更新日: 2025 年 10 月 28 日**

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。現在は描画基盤と幾何要素の設計段階にあり、NURBS や CAM 機能は未実装です。

## クイックリファレンス

**⚠️ 重要な制約**:

- プロジェクトは正常にビルド・実行可能（警告のみ、主にコードスタイル関連）
- `render` と `stage` は独立してビルド可能（model に依存しない）
- 新規コード追加時は既存のトレイト設計と責務分離を尊重すること

**よく使うコマンド**:

```bash
cargo build                 # 全体ビルド（成功）
cargo run                   # メイン実行（GUI環境が必要）
cargo test -p render        # render テスト（ビルド成功、テストなし）
cargo test -p stage         # stage テスト（ビルド成功、テストなし）
cargo test --workspace      # 全体テスト実行
cargo tree --depth 1        # クレート間依存確認
mdbook build                # ドキュメント生成（manual/ -> docs/）
```

## アーキテクチャ

### Workspace 構成

- `foundation`: 独立した基礎機能
  -- `analysis`: 数値解析・線形代数
- `model`: 高次曲線・曲面（NURBS 等）
  -- `geo_foundation`: 抽象化レイヤー（トレイト定義・型システム）
  -- `geo_primitives`: 具体実装（基本幾何要素）
  -- `geo_core`: 幾何計算基盤・共通幾何計算
  -- `geo_algorithms`: 高レベル幾何アルゴリズム
  -- `geo_io`: データ変換
  -- `geo_nurbs`: NURBS 曲線・曲面（予定）
- `view`: アプリケーション・ビュー
  -- `app`: メインアプリケーション
  -- `render`: GPU 描画基盤（wgpu + WGSL）
  -- `stage`: レンダリングステージ管理（`RenderStage` トレイト）
- `viewmodel`: ビュー操作実装・変換ロジック
  -- `converter`: view に model をデータ変換する
  -- `graphics`: view に model の描画基盤（wgpu + WGSL）の情報を橋渡しする

### 依存関係の方向性

```
model → geo_algorithms → geo_primitives → geo_foundation ← geo_core
                                                      ↘     ↙
                                                        analysis
redring → viewmodel → model
       ↘  stage → render
```

**重要**: Foundation パターンにより統一されたトレイト実装、`render` は幾何データ層に依存しない

## 幾何データの設計パターン（現在の構造）

### モジュール構成

```rust
// Foundation レイヤー: geo_foundation/src/
pub mod extension_foundation;     // ExtensionFoundation トレイト
pub mod extensions;               // BasicTransform, 型安全操作
pub mod primitive_kind;           // PrimitiveKind 分類
pub mod scalar;                   // Scalar トレイト
pub mod bounding_box;             // BoundingBox3D

// Core レイヤー: geo_core/src/
pub mod scalar_operations;        // 数値演算実装
pub mod tolerance;                // ToleranceContext 管理
pub mod robust_calculations;      // ロバスト幾何判定

// Primitives レイヤー: geo_primitives/src/
pub mod point_3d;                 // Point3D<T>
pub mod vector_3d;                // Vector3D<T>
pub mod plane_3d;                 // Plane3D<T>
// + 各要素の foundation, extensions, transform ファイル

// Legacy: model/src/
pub mod geometry;                 // 高次曲線・曲面（NURBS等）
pub mod geometry_trait;           // Curve2D, Curve3D, Surface トレイト
```

### Foundation パターンの実装

Foundation パターンは全ての幾何プリミティブに統一インターフェースを提供します：

```rust
// geo_foundation/src/extension_foundation.rs
pub trait ExtensionFoundation<T: Scalar> {
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> Option<BoundingBox3D<T>>;
    fn measure(&self) -> Option<T>;
}

// geo_primitives/src/plane_3d_foundation.rs の例
impl<T: Scalar> ExtensionFoundation<T> for Plane3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }

    fn bounding_box(&self) -> Option<BoundingBox3D<T>> {
        None // 無限平面はバウンディングボックスなし
    }

    fn measure(&self) -> Option<T> {
        None // 無限平面の測度は定義されない
    }
}
```

### 分離されたファイル構成

各幾何プリミティブは以下の構成で実装されています：

- `{shape}_3d.rs`: 基本実装
- `{shape}_3d_foundation.rs`: Foundation トレイト実装
- `{shape}_3d_extensions.rs`: 基本操作・拡張機能
- `{shape}_3d_transform.rs`: 変換操作（BasicTransform）
- `{shape}_3d_tests.rs`: テストスイート

### 型安全パターン

Direction と Vector の明確な分離により型安全性を保証：

```rust
// geo_primitives/src/direction_3d.rs
pub struct Direction3D<T: Scalar>(Vector3D<T>);

impl<T: Scalar> Direction3D<T> {
    pub fn from_vector(v: Vector3D<T>) -> Option<Self> {
        let len = v.norm();
        if len.is_zero() {
            None
        } else {
            Some(Direction3D(v.normalize()))
        }
    }

    // アクセサメソッド
    pub fn x(&self) -> T { self.0.x() }
    pub fn y(&self) -> T { self.0.y() }
    pub fn z(&self) -> T { self.0.z() }
}
```

### トレイト設計

```rust
// model/src/geometry_trait.rs: 各曲線型が実装する共通インターフェース
pub trait Curve3D: Any {
    fn kind(&self) -> CurveKind3D;
    fn evaluate(&self, t: f64) -> Point3D<f64>;
    fn derivative(&self, t: f64) -> Vector3D<f64>;
    fn length(&self) -> f64;
}
```

## GPU 描画システム (`render/`)

### モジュール構成

```rust
// render/src/lib.rs
pub mod device;      // wgpu Device 管理
pub mod pipeline;    // レンダリングパイプライン構築
pub mod shader;      // シェーダモジュール生成関数
pub mod wireframe;   // ワイヤーフレーム描画リソース
pub mod render_2d;   // 2D 描画リソース
pub mod render_3d;   // 3D 描画リソース
pub mod surface;     // サーフェス管理
pub mod vertex_2d;   // 2D 頂点型
pub mod vertex_3d;   // 3D 頂点型
```

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

### RenderStage トレイト

各描画ステージの統一インターフェース（`render()` + `update()`）

### 実装例と具体的なパターン

```rust
// OutlineStage: ワイヤーフレーム描画（編集時のエッジ表示用）
pub struct OutlineStage {
    resources: WireframeResources,  // render クレートの wireframe モジュールから
}
impl RenderStage for OutlineStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        // RenderPass を作成し、wireframe パイプラインで描画
    }
    // update() はデフォルト実装を使用（アニメーションなし）
}

// DraftStage: 2D プレビュー描画（アニメーション更新機能付き）
pub struct DraftStage {
    resources: Render2dResources,
    frame_count: u64,  // アニメーション用のフレームカウンタ
}
impl RenderStage for DraftStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) { /* ... */ }
    fn update(&mut self) {
        self.frame_count += 1;
        // 頂点バッファを動的に更新してアニメーション
    }
}
```

### 共通パターン

- 各ステージは `render` クレートのリソース型（`WireframeResources`, `Render2dResources` など）を保持
- `new()` で device と format を受け取り、リソースを初期化
- `update()` の実装はオプション（静的な描画のみなら不要）

## 開発ワークフロー

```bash
# 全体ビルド
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

- **ビルド状況**: 一部エラーあり（geo_primitives 内の型変換作業中）
- **コード整理状況**: テストファイル分離完了（2024 年 10 月 9 日）
  - InfiniteLine2D: 436 行 → 250 行（43%削減）
  - InfiniteLine3D: 617 行 → 393 行（36%削減）
  - traits_tests.rs を 4 ファイルに分割
- **型変換戦略**: f64 固定 → ジェネリック<T: Scalar>への段階的移行
  - ✅ Direction3D<T>: 完了（74 テスト通過）
  - ✅ Ray3D<T>: 完了（Direction3D<T>統合済み）
  - ✅ Plane3D<T>: 完了（Foundation パターン実装済み）
  - ❌ InfiniteLine3D: 一時無効化（40+エラー、複雑すぎるため後回し）
  - 📋 Circle3D → Ellipse3D → Arc3D → InfiniteLine2D/3D の順で再有効化予定
- **テスト**: 現在 74 テスト通過、分離したテストファイルでコードサイズ大幅削減
- **wgpu バージョン**: 27.0.1 に統一済み
- **WebAssembly**: 将来対応予定（README 記載）だが、現状は native のみ

## 重要な設計原則

### 1. Option/Result による失敗の明示化

```rust
Direction::from_vector(v)  // Option<Direction> を返す（ゼロベクトルは None）
v.normalize()               // Vector を返す（ゼロベクトルは Vector::ZERO）
```

**重要**: `Normalize` トレイトの `normalize()` は `Self` を返し、ゼロベクトルの場合は `Vector::ZERO` を返す。
`Direction::from_vector()` は内部で長さチェックを行い、ゼロベクトルの場合は `None` を返すことで型安全性を保証する。

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

- **技術ドキュメント**: `manual/` (mdbook ソース) → `book.toml` で `docs/` に生成
  - `book/` ディレクトリも存在するが、これは古い出力先の可能性あり
- **構造**: `intro.md`, `modules.md`, `kinds.md`, `philosophy.md`, `SUMMARY.md`
- **設計方針**: 責務分離、型安全性、国際化対応（英語中心）
- **README.md**: 安定機能のみ記載、詳細は Issues/Projects 参照
- **生成コマンド**: `mdbook build` で `manual/` → `docs/` へビルド

## デバッグ・トレース

- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- ログレベルは `RUST_LOG` 環境変数で制御
- 進捗管理: GitHub Issues/Projects で追跡

## 現在の状態と制約

- **ビルド状況**: 一部エラーあり（geo_primitives 内の型変換作業中）
- **進行中の型変換**: f64 固定 → ジェネリック<T: Scalar>
  - ✅ Direction3D<T>, Ray3D<T>: 完了
  - 🔄 Circle3D → Ellipse3D → Arc3D → InfiniteLine2D/3D: 段階的変換予定
  - ❌ InfiniteLine3D: 一時無効化（複雑すぎるため最後に対応）
- **未実装機能**: NURBS の完全実装、CAM パス生成、切削シミュレーション
- **wgpu バージョン**: 27.0.1 に統一済み（2025 年）
- **WebAssembly**: 将来対応予定（現在は wgpu のネイティブバックエンドのみ）
- **viewmodel**: 現在は最小実装（今後の拡張予定）
  - 現状はサンプルの `add()` 関数のみ
  - ビュー変換やカメラ制御は未実装

## 型変換戦略（進行中）

### 段階的変換アプローチ

f64 固定型からジェネリック<T: Scalar>への変換を、依存性と複雑さに基づいて段階的に実施：

1. **Phase 1 - 基本型（完了）** ✅

   - Direction3D<T>: 正規化ベクトル型、74 テスト通過
   - Ray3D<T>: Direction3D<T>統合済み
   - Plane3D<T>: Foundation パターン実装済み

2. **Phase 2 - 幾何プリミティブ（次の作業）** 🔄

   - Circle3D → Ellipse3D → Arc3D の順で変換
   - 比較的単純で依存関係が少ない

3. **Phase 3 - 複雑な型（最後）** 📋
   - InfiniteLine2D/3D: Vector3D→Vector<T>、40+エラー
   - トレイト実装の型パラメータ修正が必要

### 変換時の重要原則

- 一つずつ確実に変換（並行作業禁止）
- テスト分離でファイルサイズ削減を優先
- エラーが複雑な場合は一時無効化して後回し
- Direction3D<T>のヘルパーメソッド活用

## テスト戦略

- **現状**: `render` と `stage` はビルド可能だがテストコードなし、`viewmodel` に基本テストあり
- **実行方法**: `cargo test -p <crate_name>` で個別クレートのテスト実行、`cargo test --workspace` で全体実行
- **推奨**: 新機能追加時は独立したユニットテストを追加（特に render/stage は model に依存しないため追加しやすい）
- **テストファイル整理**: 大きなテストファイルを分離して管理性向上
  - traits_tests.rs → 4 ファイルに分割
  - InfiniteLine2D/3D テスト → 専用ファイルに分離
  - コードサイズ大幅削減（430-617 行 → 250-393 行）

## wgpu 27.0.1 への対応

- **主な変更点**: `DeviceDescriptor` に `experimental_features` フィールドが追加
  ```rust
  wgpu::DeviceDescriptor {
      label: Some("Device"),
      required_features: wgpu::Features::empty(),
      required_limits: wgpu::Limits::default(),
      memory_hints: Default::default(),
      trace: wgpu::Trace::default(),
      experimental_features: wgpu::ExperimentalFeatures::default(), // 追加
  }
  ```
- **影響範囲**: `render/src/device.rs` と `redring/src/graphic.rs`

コードを変更する際は、既存のトレイト設計と責務分離を尊重し、型安全性を保つことを優先してください。
