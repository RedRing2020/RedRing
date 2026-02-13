# GPU 描画システム

## 最終更新日: 2026年2月13日

RedRingにおけるwgpuベースのGPU描画システムの設計と実装パターンを定義します。

---

## モジュール構成

### render クレート (`view/render/`)

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

### 依存関係

```text
render → analysis （幾何データ層に依存しない）
```

**重要**: `render` クレートは `model/` レイヤーに依存してはいけません。

---

## シェーダ管理

### シェーダファイルの配置

```
render/
├── shaders/
│   ├── render_2d.wgsl     # 2D描画シェーダ
│   ├── render_3d.wgsl     # 3D描画シェーダ
│   └── wireframe.wgsl     # ワイヤーフレーム描画
└── src/
    └── shader.rs          # シェーダモジュール生成
```

### シェーダのロードパターン

```rust
// shader.rs: コンパイル時埋め込み
pub fn render_3d_shader(device: &Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Render 3D Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../shaders/render_3d.wgsl").into()
        ),
    })
}

pub fn wireframe_shader(device: &Device) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Wireframe Shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../shaders/wireframe.wgsl").into()
        ),
    })
}
```

**特徴**:
- `include_str!` でコンパイル時に埋め込み
- シェーダ変更時は Rust 側の再ビルドが必要
- 実行時のファイル読み込みエラーを回避

---

## 頂点データパターン

### 必須トレイト

GPU に送るデータは以下を実装：
- `#[repr(C)]`: C言語互換のメモリレイアウト
- `Pod`: Plain Old Data（任意のバイト列として解釈可能）
- `Zeroable`: ゼロ初期化可能

### 2D 頂点型

```rust
// vertex_2d.rs
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub color: [f32; 4],
}

impl Vertex2D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex2D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
```

### 3D 頂点型

```rust
// vertex_3d.rs
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex3D {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
```

---

## レンダリングパイプライン

### パイプライン構築

```rust
// pipeline.rs
pub fn create_render_pipeline(
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    vertex_layout: &wgpu::VertexBufferLayout,
    surface_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[vertex_layout.clone()],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
```

---

## RenderStage トレイト

### トレイト定義

```rust
// stage/src/lib.rs
pub trait RenderStage {
    /// レンダリング実行
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    );

    /// フレーム更新（デフォルト実装あり）
    fn update(&mut self) {}
}
```

### 実装例

```rust
pub struct WireframeStage {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl RenderStage for WireframeStage {
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Wireframe Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    fn update(&mut self) {
        // アニメーション等の更新処理
    }
}
```

---

## デバイス管理

### Device と Queue の初期化

```rust
// device.rs
pub async fn create_device_and_queue() -> (wgpu::Device, wgpu::Queue) {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find an appropriate adapter");

    adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device"),
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device")
}
```

---

## バッファ作成パターン

### 頂点バッファ

```rust
use wgpu::util::DeviceExt;

pub fn create_vertex_buffer<T: bytemuck::Pod>(
    device: &wgpu::Device,
    vertices: &[T],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    })
}
```

### インデックスバッファ

```rust
pub fn create_index_buffer(
    device: &wgpu::Device,
    indices: &[u16],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    })
}
```

---

## WGSL シェーダの基本構造

### 3D レンダリングシェーダ例

```wgsl
// render_3d.wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
```

---

## 実装チェックリスト

新規レンダリング機能追加時：

### 1. シェーダ作成
- [ ] `render/shaders/{name}.wgsl` 作成
- [ ] 頂点シェーダ (`vs_main`) 定義
- [ ] フラグメントシェーダ (`fs_main`) 定義

### 2. 頂点型定義
- [ ] `#[repr(C)]` 付与
- [ ] `Pod`, `Zeroable` 実装
- [ ] `desc()` メソッド実装

### 3. パイプライン構築
- [ ] シェーダモジュール生成関数追加
- [ ] パイプライン作成ヘルパー実装

### 4. RenderStage 実装
- [ ] `render()` メソッド実装
- [ ] 必要に応じて `update()` 実装

### 5. テスト
- [ ] ビルド確認: `cargo build -p render`
- [ ] 依存関係確認: `cargo tree -p render`

---

## 参照文書

- **CAM Visualization**: `dev/architecture/CAM_VISUALIZATION_REQUIREMENTS.md`
- **Shape Visualization**: `dev/architecture/SHAPE_VISUALIZATION_DESIGN.md`
- **wgpu公式ドキュメント**: https://wgpu.rs/
