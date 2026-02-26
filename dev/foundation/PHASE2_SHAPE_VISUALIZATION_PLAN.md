# Issue #188 Phase 2 実装計画

**作成日**: 2026年1月2日  
**関連Issue**: [#188 形状可視化システムの実装](https://github.com/TakSung/RedRing/issues/188)  
**前提**: Phase 1（設計 + ViewModel層）完了

---

## 📋 Phase 2の目的

**ViewModel層で変換されたデータを実際に画面に表示できるようにする**

- View層の線分描画インフラ構築
- MeshStageの拡張（RenderMode::Lines対応）
- アプリケーション層統合（デバッグ用キーバインド）
- エンドツーエンド動作確認

---

## 🎯 完了条件

1. ✅ `cargo run` でアプリケーション起動
2. ✅ キーバインドで各形状を表示
   - `l` キー: LineSegment3D
   - `c` キー: Circle3D
   - `t` キー: Triangle3D
   - `a` キー: Arc3D
3. ✅ 各形状が正しく描画される（スクリーンショット記録）
4. ✅ 60FPS以上で動作（パフォーマンス確認）

---

## 📝 タスク一覧

### Task 2.1: LineResources実装

**ファイル**: `view/render/src/line.rs`（新規作成）

**実装内容**:
```rust
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
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self;
    pub fn update_line_data(&mut self, device: &wgpu::Device, vertices: &[MeshVertex]);
    pub fn render(&self, render_pass: &mut wgpu::RenderPass);
}
```

**ポイント**:
- `PrimitiveTopology::LineList` を使用（2頂点で1線分）
- 既存のMeshResourcesを参考に実装
- ユニフォームバッファでカメラ行列を共有

**見積もり**: 2-3時間

---

### Task 2.2: line.wgslシェーダ実装

**ファイル**: `view/render/shaders/line.wgsl`（新規作成）

**実装内容**:
```wgsl
struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);  // 白色の線
}
```

**ポイント**:
- render_3d.wgsl を参考にカメラ変換を実装
- フラグメントシェーダは単色でOK（将来的に色を可変化）

**見積もり**: 1時間

---

### Task 2.3: MeshStage拡張

**ファイル**: `view/stage/src/mesh_stage.rs`（既存ファイル拡張）

**変更内容**:

1. RenderMode enum に Lines を追加:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    Solid,
    Wireframe,
    Lines,  // 新規追加
}
```

2. MeshStageに LineResources フィールド追加:
```rust
pub struct MeshStage {
    mesh_resources: MeshResources,
    line_resources: Option<LineResources>,  // 新規追加
    render_mode: RenderMode,
}
```

3. set_line_data() メソッド追加:
```rust
pub fn set_line_data(&mut self, device: &Device, vertices: Vec<MeshVertex>) {
    if self.line_resources.is_none() {
        self.line_resources = Some(LineResources::new(device, /* format */));
    }
    
    if let Some(line_res) = &mut self.line_resources {
        line_res.update_line_data(device, &vertices);
    }
    
    self.render_mode = RenderMode::Lines;
}
```

4. render() メソッドで RenderMode::Lines に対応:
```rust
fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
    match self.render_mode {
        RenderMode::Solid | RenderMode::Wireframe => {
            // 既存のメッシュ描画
            self.mesh_resources.render(encoder, view);
        }
        RenderMode::Lines => {
            // 線分描画
            if let Some(line_res) = &self.line_resources {
                line_res.render(encoder, view);
            }
        }
    }
}
```

**見積もり**: 2時間

---

### Task 2.4: アプリケーション層統合

**ファイル**: `view/app/src/app_state.rs`（既存ファイル拡張）

**実装内容**:

1. デバッグ形状表示関数を追加:
```rust
/// デバッグ用：LineSegment3Dを表示
pub fn load_debug_line(&mut self) {
    use geo_primitives::{LineSegment3D, Point3D};
    use viewmodel::shape_converter::line_segment_to_vertices;
    
    let line = LineSegment3D::new(
        Point3D::new(-1.0, 0.0, 0.0),
        Point3D::new(1.0, 0.0, 0.0)
    );
    
    let vertex_data = line_segment_to_vertices(&line);
    
    // VertexData -> MeshVertex 変換
    let vertices: Vec<MeshVertex> = vertex_data.iter()
        .map(|v| MeshVertex {
            position: v.position,
            normal: v.normal,
        })
        .collect();
    
    // MeshStageに設定
    if let Some(mesh_stage) = self.renderer.stage_as_mesh_mut() {
        mesh_stage.set_line_data(&self.graphic.device, vertices);
    }
    
    // カメラを適切な位置に設定
    self.camera.reset_to_standard_cad_view();
    self.update_camera_uniforms();
}

/// デバッグ用：Circle3Dを表示
pub fn load_debug_circle(&mut self) {
    // 同様の実装...
}

/// デバッグ用：Triangle3Dを表示
pub fn load_debug_triangle(&mut self) {
    // 同様の実装...
}

/// デバッグ用：Arc3Dを表示
pub fn load_debug_arc(&mut self) {
    // 同様の実装...
}
```

2. AppRendererに stage_as_mesh_mut() を追加:
```rust
// view/app/src/app_renderer.rs
impl AppRenderer {
    pub fn stage_as_mesh_mut(&mut self) -> Option<&mut MeshStage> {
        self.stage.downcast_mut::<MeshStage>()
    }
}
```

**ファイル**: `view/app/src/app.rs`（既存ファイル拡張）

**キーバインド追加**:
```rust
WindowEvent::KeyboardInput { event, .. } => {
    if pressed {
        match &event.logical_key {
            // ... 既存のキーバインド ...
            
            Key::Character(c) if c.as_str() == "l" => {
                state.load_debug_line();
                tracing::info!("デバッグ: LineSegment3D表示");
            }
            Key::Character(c) if c.as_str() == "c" => {
                state.load_debug_circle();
                tracing::info!("デバッグ: Circle3D表示");
            }
            Key::Character(c) if c.as_str() == "t" => {
                state.load_debug_triangle();
                tracing::info!("デバッグ: Triangle3D表示");
            }
            Key::Character(c) if c.as_str() == "a" => {
                state.load_debug_arc();
                tracing::info!("デバッグ: Arc3D表示");
            }
            
            // ... 以下既存 ...
        }
    }
}
```

**ヘルプメッセージ更新**:
```rust
Key::Character(c) if c.as_str() == "h" => {
    tracing::info!("=== キーバインド ===");
    tracing::info!("--- デバッグ形状表示 ---");
    tracing::info!("l: LineSegment3D表示");
    tracing::info!("c: Circle3D表示");
    tracing::info!("t: Triangle3D表示");
    tracing::info!("a: Arc3D表示");
    tracing::info!("--- レンダリング ---");
    // ... 既存のヘルプ ...
}
```

**見積もり**: 3時間

---

### Task 2.5: エンドツーエンドテスト

**テスト手順**:

1. アプリケーションビルド・起動:
```powershell
cargo build
$env:RUST_LOG="info"
cargo run
```

2. 各キーで形状表示確認:
   - `l` キー押下 → 白い線分が表示される
   - `c` キー押下 → 白い円が表示される
   - `t` キー押下 → 白い三角形が表示される
   - `a` キー押下 → 白い円弧が表示される

3. スクリーンショット記録:
   - 各形状のスクリーンショットを `dev/screenshots/` に保存
   - README.md に表示例を追加

4. パフォーマンス確認:
   - FPSカウンタ実装（オプション）
   - 少なくとも60FPS以上で動作することを確認

**見積もり**: 2時間

---

## 📊 総見積もり時間

- Task 2.1: LineResources実装 - 2-3時間
- Task 2.2: line.wgslシェーダ - 1時間
- Task 2.3: MeshStage拡張 - 2時間
- Task 2.4: アプリケーション層統合 - 3時間
- Task 2.5: エンドツーエンドテスト - 2時間

**合計**: 10-11時間

---

## 🚨 リスクと対策

### リスク1: VertexData ↔ MeshVertex 変換の不一致

**対策**: 
- 既存の mesh_converter.rs の変換ロジックを参照
- 型定義を統一するか、明示的な変換関数を用意

### リスク2: カメラ行列のユニフォームバッファ共有

**対策**:
- 既存の MeshResources がどのようにユニフォームを管理しているか確認
- 必要に応じて LineResources 独自のユニフォームバッファを持つ

### リスク3: 線が細すぎて見えない

**対策**:
- wgpuの line width 設定を確認（WebGPUでは width=1 固定の可能性）
- 必要に応じて太い線を三角形で描画する方式に変更

---

## ✅ 完了後の次のステップ

Phase 2完了後は以下を検討：

1. **Phase 3: 適応的テッセレーション**
   - カメラ距離・画面解像度に基づくLOD切り替え
   - AdaptiveTessellation 構造体の実装

2. **その他のプリミティブ対応**
   - Plane3D, Sphere3D, Cylinder3D等の可視化

3. **色・スタイル設定**
   - 形状ごとの色指定
   - 線の太さ調整（可能であれば）

---

**作成者**: GitHub Copilot  
**レビュー**: 実装開始前にユーザー承認必須
