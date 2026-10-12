# GPU 描画システム

## 最終更新日: 2026年10月12日

RedRing の wgpu ベースの描画（`view/render` / `view/stage`）の構成と実装パターンを定義します。wgpu の API はバージョンで変わるため、コードは実装のファイル（`view/render/src/`）を正として参照してください（現在の wgpu は `view/render/Cargo.toml` を参照）。

---

## クレートの役割と依存

- `render`（`view/render`）: wgpu のデバイス・パイプライン・シェーダ・頂点型・GPU リソース
- `stage`（`view/stage`）: 描画の単位（ステージ）。`RenderStage` を実装する
- `render` / `stage` はモデルのクレートに直接依存しない。描画データは `viewmodel` で変換したものを受け取る（依存の規則は `.github/skills/architecture/skill.md`）

---

## render クレート

`view/render/src/` に、wgpu のデバイス・サーフェス・パイプライン・シェーダの生成・ユニフォーム・頂点型と、描画対象ごとの GPU リソースを置く。モジュールの一覧は `view/render/src/lib.rs` を参照する。

### シェーダ

シェーダは `view/render/shaders/*.wgsl` に置き、`shader.rs` で `include_str!` によりコンパイル時に埋め込む。

```rust
// view/render/src/shader.rs
pub fn mesh_shader(device: &Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Mesh Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/mesh.wgsl").into()),
    })
}
```

- シェーダを変更したら Rust 側の再ビルドが必要
- 実行時のファイル読み込みに失敗することがない

### 頂点型

GPU に送る型は `#[repr(C)]` と `bytemuck::Pod` / `Zeroable` を付け、`desc()` で頂点バッファのレイアウトを返す。

```rust
// view/render/src/vertex_3d.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
}

impl Vertex3D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x3, // @location(0) に対応
            ],
        }
    }
}
```

- `desc()` の `shader_location` と WGSL の `@location(n)` を一致させる

---

## stage クレート

### RenderStage

```rust
// view/stage/src/render_stage.rs（抜粋）
pub trait RenderStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);

    /// 深度バッファ付きの描画（既定では depth を使わず render を呼ぶ）
    fn render_with_depth(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        _depth_view: &wgpu::TextureView,
    ) {
        self.render(encoder, view);
    }

    /// カメラ行列の更新（GPU ユニフォームでカメラを使うステージが実装する）
    fn update_camera(
        &mut self,
        _queue: &wgpu::Queue,
        _view_matrix: [[f32; 4]; 4],
        _proj_matrix: [[f32; 4]; 4],
    ) {
    }

    fn update(&mut self) {}
    fn on_surface_resized(&mut self, _device: &wgpu::Device, _size: (u32, u32)) {}

    fn as_any_mut(&mut self) -> &mut dyn Any;
    // ほかに、ワイヤーフレームの切り替え・スナップショットの適用等の任意のフック（既定は未対応）
}
```

- `render` と `as_any_mut` だけが必須。他のメソッドは既定の実装があり、必要なステージだけが上書きする
- 既存のステージは `view/stage/src/lib.rs` を参照する

---

## 実装チェックリスト

新しい描画を追加するとき:

### 1. シェーダ
- [ ] `view/render/shaders/{name}.wgsl`（`vs_main` / `fs_main`）
- [ ] `view/render/src/shader.rs` に生成関数を追加

### 2. 頂点型・GPU リソース
- [ ] 既存の頂点型（`view/render/src/vertex_2d.rs` / `vertex_3d.rs`）で足りるか確認し、足りない場合だけ追加する
- [ ] 描画データは viewmodel で変換し、render からモデル層を参照しない

### 3. ステージ
- [ ] `view/stage/src/{name}_stage.rs` に `RenderStage` を実装
- [ ] `view/stage/src/lib.rs` で再エクスポート

### 4. 確認
- [ ] `cargo build -p render -p stage -p redring`
- [ ] `.\scripts\check_architecture_dependencies.ps1`（render / stage がモデル層に依存していないこと）

---

## 参照文書

- **アーキテクチャ・依存関係**: `.github/skills/architecture/skill.md`
- **MVVM 設計**: `dev/architecture/VIEWMODEL_ARCHITECTURE_DESIGN.md`
- **CAM の可視化**: `dev/architecture/CAM_VISUALIZATION_REQUIREMENTS.md`
- **形状の可視化**: `dev/architecture/SHAPE_VISUALIZATION_DESIGN.md`
- **wgpu**: https://wgpu.rs/
