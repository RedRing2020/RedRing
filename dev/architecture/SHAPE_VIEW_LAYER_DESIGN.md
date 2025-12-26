# View層形状描画システム設計

**作成日**: 2025年12月26日  
**最終更新**: 2025年12月26日  
**関連Issue**: [#188 形状可視化システムの実装](https://github.com/RedRing2020/RedRing/issues/188)  
**関連ドキュメント**: [形状可視化システム設計仕様](./SHAPE_VISUALIZATION_DESIGN.md)

## 📋 概要

ViewModel層で変換された形状データをView層で受け取り、GPU描画を行うための設計仕様。

## 🎯 設計方針

### Option 1: MeshStage拡張アプローチ（推奨）

**メリット**:
- 既存のMeshStageインフラを再利用
- 実装コストが低い
- 既存のワイヤーフレームモード切り替え機能を活用

**デメリット**:
- MeshStageの責務が広がる
- 線分専用の描画パイプラインが必要

**実装方針**:
```rust
// MeshStageに描画モードを追加
pub enum RenderMode {
    Solid,          // ソリッド描画（既存）
    Wireframe,      // ワイヤーフレーム描画（既存）
    Lines,          // 線分描画（新規 - LineSegmentなど）
    Points,         // 点群描画（新規 - 将来的に追加）
}

impl MeshStage {
    // 線分データを設定する新しいメソッド
    pub fn set_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>);
    
    // 描画モードを設定
    pub fn set_render_mode(&mut self, mode: RenderMode);
}
```

### Option 2: 新規ShapeStage作成アプローチ

**メリット**:
- 責務が明確に分離
- 将来的な拡張性が高い
- 形状専用の最適化が可能

**デメリット**:
- 実装コストが高い
- コードの重複が発生する可能性

**実装方針**:
```rust
pub struct ShapeStage {
    solid_resources: Option<MeshResources>,    // ソリッド形状用
    line_resources: Option<LineResources>,      // 線分形状用
    point_resources: Option<PointResources>,    // 点群用
}

impl RenderStage for ShapeStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView);
}
```

## 🏗️ 推奨実装: MeshStage拡張アプローチ

### ステップ1: 線分描画パイプラインの追加

```rust
// view/render/src/line.rs（新規作成）
pub struct LineResources {
    device: Arc<wgpu::Device>,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
}

impl LineResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // ラインパイプライン作成
        // PrimitiveTopology::LineList を使用
    }
    
    pub fn update_line_data(&mut self, device: &wgpu::Device, vertices: &[MeshVertex]) {
        // 頂点バッファ更新（2頂点で1線分）
    }
    
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        // 線分描画
    }
}
```

### ステップ2: MeshStageの拡張

```rust
// view/stage/src/mesh_stage.rs（既存ファイル拡張）
pub struct MeshStage {
    mesh_resources: MeshResources,      // 既存（ソリッド/ワイヤーフレーム）
    line_resources: Option<LineResources>, // 新規（線分描画）
    render_mode: RenderMode,             // 描画モード
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Solid,
    Wireframe,
    Lines,
}

impl MeshStage {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let mesh_resources = MeshResources::new(device, format);
        
        Self {
            mesh_resources,
            line_resources: None,
            render_mode: RenderMode::Solid,
        }
    }
    
    // 既存のメッシュデータ設定（ソリッド/ワイヤーフレーム）
    pub fn set_mesh_data(&mut self, device: &Device, vertices: Vec<MeshVertex>, indices: Vec<u32>) {
        // 既存実装
    }
    
    // 新規：線分データ設定
    pub fn set_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>) {
        if self.line_resources.is_none() {
            self.line_resources = Some(LineResources::new(device, /* format */));
        }
        
        if let Some(line_res) = &mut self.line_resources {
            line_res.update_line_data(device, &vertices);
        }
        
        self.render_mode = RenderMode::Lines;
    }
    
    pub fn set_render_mode(&mut self, mode: RenderMode) {
        self.render_mode = mode;
    }
}

impl RenderStage for MeshStage {
    fn render(&mut self, encoder: &mut CommandEncoder, view: &TextureView) {
        let mut render_pass = encoder.begin_render_pass(/* ... */);
        
        match self.render_mode {
            RenderMode::Solid | RenderMode::Wireframe => {
                self.mesh_resources.render(&mut render_pass);
            },
            RenderMode::Lines => {
                if let Some(line_res) = &self.line_resources {
                    line_res.render(&mut render_pass);
                }
            },
        }
    }
}
```

### ステップ3: アプリケーション層での統合

```rust
// view/app/src/app_state.rs（既存ファイル拡張）
impl AppState {
    /// LineSegment3Dを表示
    pub fn show_line_segment(&mut self, segment: &LineSegment3D<f64>) -> Result<(), Box<dyn std::error::Error>> {
        use viewmodel::shape_converter::line_segment_to_vertices;
        
        // ViewModel層で変換
        let vertex_data = line_segment_to_vertices(segment);
        
        // View層（render）の形式に変換
        let vertices: Vec<MeshVertex> = vertex_data
            .iter()
            .map(|vd| MeshVertex::from_vertex_data(vd))
            .collect();
        
        // MeshStageに線分データを設定
        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);
        
        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
        
        Ok(())
    }
    
    /// Circle3Dを表示
    pub fn show_circle(&mut self, circle: &Circle3D<f64>) -> Result<(), Box<dyn std::error::Error>> {
        use viewmodel::shape_converter::{circle_to_vertices, TessellationQuality};
        
        let quality = TessellationQuality::default();
        let vertex_data = circle_to_vertices(circle, &quality);
        
        let vertices: Vec<MeshVertex> = vertex_data
            .iter()
            .map(|vd| MeshVertex::from_vertex_data(vd))
            .collect();
        
        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_line_data(&self.graphic.device, vertices);
        
        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
        
        Ok(())
    }
}
```

## 🎨 シェーダ設計

### 線分描画用シェーダ

```wgsl
// view/render/shaders/line.wgsl（新規作成）

struct CameraUniform {
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,  // 線分では使用しないが、互換性のため残す
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.projection * camera.view * vec4<f32>(in.position, 1.0);
    out.color = vec3<f32>(1.0, 1.0, 1.0);  // デフォルトは白
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
```

## 📊 パフォーマンス考慮事項

### 描画プリミティブの選択

- **LineSegment3D**: `PrimitiveTopology::LineList`
- **Circle3D（ワイヤーフレーム）**: `PrimitiveTopology::LineStrip`
- **Circle3D（ソリッド）**: `PrimitiveTopology::TriangleList`
- **Triangle3D**: `PrimitiveTopology::TriangleList`

### バッファ管理

- 動的な形状切り替えに対応するため、頂点バッファは動的に再作成
- 将来的にはバッファプールを実装してメモリ再利用を最適化

## 🔧 実装順序

### Phase 1（Week 1）
1. **LineResources の実装** - 線分描画の基礎
2. **line.wgsl シェーダ作成** - 線分描画シェーダ
3. **MeshStage の拡張** - RenderModeとset_line_dataの追加

### Phase 2（Week 2）
4. **AppState の拡張** - show_line_segment, show_circle メソッド追加
5. **テスト実装** - 実際にLineSegment3DとCircle3Dを表示
6. **キーバインド追加** - デバッグ用の形状表示切り替え

### Phase 3（Week 3）
7. **複数形状の同時表示** - シーングラフの基礎
8. **カラー制御** - 形状ごとに色を指定可能に
9. **ドキュメント更新** - 使用方法の文書化

## 🔗 関連ファイル

### 新規作成予定
- `view/render/src/line.rs` - 線分描画リソース
- `view/render/shaders/line.wgsl` - 線分描画シェーダ

### 修正予定
- `view/stage/src/mesh_stage.rs` - RenderMode追加、line_resources統合
- `view/app/src/app_state.rs` - 形状表示メソッド追加

### 参考実装
- `view/render/src/mesh.rs` - メッシュ描画リソース
- `view/render/shaders/mesh.wgsl` - メッシュ描画シェーダ
- `view/stage/src/mesh_stage.rs` - 既存のMeshStage実装

## 📝 実装時の注意事項

1. **カメラ制御**: 既存のカメラシステムと統合（update_camera_uniformsの呼び出しを忘れない）
2. **深度バッファ**: 線分描画でも深度テストを有効化（奥行き関係の正確な表現）
3. **線の太さ**: wgpuでは線の太さは制御できないため、将来的にはジオメトリシェーダまたは四角形化が必要
4. **アンチエイリアシング**: MSAA対応を検討

## 🚀 次のステップ

- Task 1.3完了後、Phase 2の実装に移行
- LineResources実装（Issue #188 Task 2.1.1）
- 実際の表示テスト

---

**設計承認**: @TBD  
**実装担当**: @TBD
