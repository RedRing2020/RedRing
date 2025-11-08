# Copilot Instructions for RedRing

**最終更新日: 2025 年 11 月 1 日**

RedRing は、Rust + wgpu による CAD/CAM 研究用プラットフォームです。

## 現在の状態

**✅ ビルド状況**: 正常（cargo build/test 成功）
**✅ 型システム**: ジェネリック<T: Scalar>対応完了
**✅ Foundation パターン**: 実装完了
**✅ 情報管理**: GitHub Issues/Projects 移行済み

## クイックリファレンス

**よく使うコマンド**:

```bash
cargo build                 # 全体ビルド
cargo run                   # メイン実行（GUI環境が必要）
cargo test --workspace      # 全体テスト実行
mdbook build                # ドキュメント生成（manual/ -> docs/）
```

**重要な制約**:

- 新規コード追加時は既存のトレイト設計と責務分離を尊重
- `render` と `stage` は独立してビルド可能（model に依存しない）

## アーキテクチャ概要

### Workspace 構成

```
foundation/         # 基礎機能（analysis: 数値解析・線形代数）
model/             # 幾何データ層
├── geo_foundation # トレイト定義・型システム
├── geo_primitives # 基本幾何要素
├── geo_core      # 幾何計算基盤
├── geo_algorithms # 高レベル幾何アルゴリズム
└── geo_nurbs     # NURBS（予定）
view/              # アプリケーション・描画層
├── app           # メインアプリケーション
├── render        # GPU描画基盤（wgpu + WGSL）
└── stage         # レンダリングステージ管理
viewmodel/         # ビュー変換ロジック
```

### Foundation パターン

```rust
// 全ての幾何プリミティブが実装する統一インターフェース
pub trait ExtensionFoundation<T: Scalar> {
    type BBox: AbstractBBox<T>;
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> Self::BBox;
    fn measure(&self) -> Option<T>;
}
```

model → geo_algorithms → geo_primitives → geo_foundation ← geo_core
↘ ↙
analysis
redring → viewmodel → model
↘ stage → render

````

**重要**: Foundation パターンにより統一されたトレイト実装、`render` は幾何データ層に依存しない

## geo_primitives 実装の作法

### ファイル命名規則と責務分離

各幾何プリミティブは以下の構成で実装されています：

- `{shape}_3d.rs`: 基本実装
- `{shape}_3d_foundation.rs`: Foundation トレイト実装
- `{shape}_3d_extensions.rs`: 基本操作・拡張機能
- `{shape}_3d_transform.rs`: 変換操作（BasicTransform）
- `{shape}_3d_transform_safe.rs`: 安全な変換操作（SafeTransform）
- `{shape}_3d_tests.rs`: 基本機能のテストスイート
- `{shape}_3d_transform_safe_tests.rs`: 安全な変換操作のテストスイート

### テストコード配置ルール

**重要**: テストコードの配置は厳格にルール化されています：

1. **基本テスト**: `{shape}_3d_tests.rs` に集約
   - 基本機能、拡張機能、Foundation実装のテスト
   - 通常の変換操作（BasicTransform）のテスト

2. **エラーハンドリング版テスト**: `{shape}_3d_transform_safe_tests.rs` に分離
   - SafeTransform トレイトのテスト
   - エラーケースの検証
   - 境界値テスト

3. **実装ファイル内のテスト禁止**:
   - 実装ファイル（`{shape}_3d.rs`, `{shape}_3d_transform_safe.rs` など）内でのテストコード記載は禁止
   - 必ず独立したテストファイルに分離する

### ネスティング設定

新規ファイル追加時は必ず `.vscode/settings.json` のネスティング設定を更新：

```jsonc
"{shape}_3d.rs": "{shape}_3d_extensions.rs,{shape}_3d_foundation.rs,{shape}_3d_transform.rs,{shape}_3d_transform_safe.rs,{shape}_3d_transform_safe_tests.rs,{shape}_3d_tests.rs"
```

### エラーハンドリングパターン

SafeTransform実装では以下のエラー型を適切に使用：

- `TransformError::ZeroVector` - ゼロ・無効ベクトル
- `TransformError::InvalidScaleFactor` - 無効なスケール倍率
- `TransformError::InvalidRotation` - 無効な回転パラメータ
- `TransformError::InvalidGeometry` - 変換後の幾何的無効性

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

## 幾何データの設計パターン（現在の構造）

### Foundation パターンの実装

Foundation パターンは全ての幾何プリミティブに統一インターフェースを提供します：

```rust
// geo_foundation/src/extension_foundation.rs
pub trait ExtensionFoundation<T: Scalar> {
    type BBox: AbstractBBox<T>;
    fn primitive_kind(&self) -> PrimitiveKind;
    fn bounding_box(&self) -> Self::BBox;
    fn measure(&self) -> Option<T>;
}

// geo_primitives/src/plane_3d_foundation.rs の例
impl<T: Scalar> ExtensionFoundation<T> for Plane3D<T> {
    type BBox = BBox3D<T>;

    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::Plane
    }

    fn bounding_box(&self) -> Self::BBox {
        // 無限平面用の微小境界ボックス
        let origin = crate::Point3D::origin();
        let epsilon = T::EPSILON;
        BBox3D::new(
            crate::Point3D::new(origin.x() - epsilon, origin.y() - epsilon, origin.z() - epsilon),
            crate::Point3D::new(origin.x() + epsilon, origin.y() + epsilon, origin.z() + epsilon)
        )
    }

    fn measure(&self) -> Option<T> {
        None // 無限平面の測度は定義されない
    }
}
````

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

## 重要な設計原則

### 1. Option/Result による失敗の明示化

```rust
Direction::from_vector(v)  // Option<Direction> を返す（ゼロベクトルは None）
v.normalize()               // Vector を返す（ゼロベクトルは Vector::ZERO）
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

- **技術ドキュメント**: `manual/` (mdbook ソース) → `book.toml` で `docs/` に生成
- **設計方針**: 責務分離、型安全性、国際化対応（英語中心）
- **README.md**: 安定機能のみ記載、詳細は Issues/Projects 参照
- **生成コマンド**: `mdbook build` で `manual/` → `docs/` へビルド
- **ドキュメント作成時**: 必ずタイムスタンプを記載（作成日・最終更新日）

## デバッグ・トレース

- `tracing` + `tracing-subscriber` をワークスペース共通依存として使用
- ログレベルは `RUST_LOG` 環境変数で制御
- 進捗管理: GitHub Issues/Projects で追跡

コードを変更する際は、既存のトレイト設計と責務分離を尊重し、型安全性を保つことを優先してください。
