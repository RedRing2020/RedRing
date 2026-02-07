# CAM可視化システム 要件定義・技術調査

**作成日**: 2026年2月8日  
**最終更新**: 2026年2月8日  
**ステータス**: ドラフト - 技術調査中  
**関連Issue**: （作成予定）

---

## 📋 目次

1. [要件定義](#要件定義)
2. [技術調査](#技術調査)
3. [アーキテクチャ設計](#アーキテクチャ設計)
4. [実装計画](#実装計画)
5. [テスト戦略](#テスト戦略)

---

## 要件定義

### 1. 表示対象

#### 1.1 工具経路（ToolPath）表示

**必須機能**:
- [ ] 線分ベースのパス表示
  - 切削送り（G01, G02, G03）
  - 早送り（G00）
- [ ] 色分け表示
  - 早送り: 青色（RGB: 0.2, 0.5, 1.0）
  - 切削送り: 白色（RGB: 1.0, 1.0, 1.0）
  - アプローチ: 緑色（RGB: 0.2, 1.0, 0.2）
  - 退避: 黄色（RGB: 1.0, 1.0, 0.2）
- [ ] 線の太さ調整（1-5ピクセル）

**将来機能**:
- [ ] 工具補正表示（左/右補正の可視化）
- [ ] 送り速度による色グラデーション
- [ ] 加工順序の番号表示

#### 1.2 オフセット結果表示

**2D輪郭オフセット**:
- [ ] オリジナル輪郭（グレー）
- [ ] オフセット輪郭（赤色）
- [ ] オフセット方向矢印表示

**3Dメッシュオフセット**:
- [ ] オリジナルメッシュ（ワイヤーフレーム、グレー）
- [ ] オフセットメッシュ（ソリッド、緑色）
- [ ] 法線ベクトル表示（オプション）

#### 1.3 工具形状表示

**Phase 3機能（後回し）**:
- [ ] 円筒工具（エンドミル）
  - 工具径の円筒表示
  - 工具長さ表示
- [ ] ボールエンドミル
  - 半球形状表示
- [ ] 工具姿勢（傾斜角）の可視化

#### 1.4 加工シミュレーション結果

**Phase 4機能（将来）**:
- [ ] 削り残しメッシュ表示
- [ ] 過切削領域の警告表示（赤色）
- [ ] ワーク形状の透過表示

---

### 2. 表示モード

#### 2.1 ワイヤーフレームモード
- 線分のみ表示
- 背景透過
- 軽量・高速描画

#### 2.2 ソリッドカラーモード
- メッシュの塗りつぶし表示
- 単色または色分け

#### 2.3 工具径表示モード
- 工具中心パスに加えて工具外形を表示
- 円筒プリミティブによる可視化

---

### 3. インタラクション

#### 3.1 パス上の任意点選択
- [ ] マウスクリックでパス上の点を選択
- [ ] 選択点の座標表示
- [ ] 選択点のパラメータ表示（t値、送り速度など）

#### 3.2 パラメータ値表示（HUD）
- [ ] 工具経路長さ
- [ ] 推定加工時間
- [ ] 最大/最小送り速度
- [ ] 最大/最小Z高さ

#### 3.3 アニメーション再生
- [ ] 再生/一時停止/停止
- [ ] 再生速度調整（0.1x - 10x）
- [ ] スライダーによる位置指定
- [ ] ステップ実行（1セグメントずつ進む）

---

## 技術調査

### 1. 大量パス表示の最適化

#### 1.1 インスタンシング

**調査結果**:
- wgpu の `draw_indexed()` でインスタンシング対応
- 同一形状を複数描画する際に有効
- 工具形状の大量描画に適用可能

**実装方針**:
```rust
// インスタンスデータをGPUバッファに格納
struct InstanceData {
    position: [f32; 3],
    rotation: [f32; 4], // クォータニオン
    scale: f32,
}

// シェーダ側でインスタンスごとに変換
@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) instance_pos: vec3<f32>,
    @location(2) instance_rot: vec4<f32>,
    @location(3) instance_scale: f32,
) -> VertexOutput {
    // インスタンスごとの変換を適用
}
```

**メリット**:
- GPU側で並列処理
- CPUからGPUへのデータ転送量削減
- 1000個以上の工具形状を60FPS以上で描画可能

**デメリット**:
- 全インスタンスが同一形状である必要がある
- 動的な形状変更には不向き

#### 1.2 LOD（Level of Detail）

**調査結果**:
- カメラ距離に応じて詳細度を変更
- 遠方のパスは低解像度、近傍は高解像度

**実装方針**:
```rust
enum LODLevel {
    High,    // カメラ距離 < 100
    Medium,  // 100 <= 距離 < 500
    Low,     // 距離 >= 500
}

impl ToolPath {
    fn tessellate_with_lod(&self, camera_pos: Point3D, level: LODLevel) -> Vec<Vertex> {
        match level {
            LODLevel::High => self.tessellate(64),   // 円弧を64分割
            LODLevel::Medium => self.tessellate(32), // 32分割
            LODLevel::Low => self.tessellate(16),    // 16分割
        }
    }
}
```

**メリット**:
- 描画負荷の削減
- 遠方の細かいディテールは視覚的に不要

**デメリット**:
- LOD切り替え時のポッピング（突然の見た目変化）
- 実装の複雑化

**推奨**:
- Phase 1では LOD なし
- パフォーマンス問題が発生した場合に Phase 2 で導入

---

### 2. 工具形状の効率的な描画

#### 2.1 円筒プリミティブの描画

**調査項目**:
- [ ] プリミティブメッシュの事前生成
  - 円筒メッシュ（側面）
  - 円盤メッシュ（上下面）
- [ ] インスタンシングによる大量描画
- [ ] シェーダによる手続き的生成（Procedural）

**実装方針（推奨）**:
```rust
// モデル層で円筒メッシュを事前生成
pub struct CylinderMesh {
    radius: f32,
    height: f32,
    segments: u32, // 円周分割数
}

impl CylinderMesh {
    pub fn generate_vertices(&self) -> Vec<Vertex3D> {
        // 円筒メッシュ生成
        // segments 個の四角形で側面を構成
    }
}

// View層でインスタンシング描画
pub struct ToolShapeResources {
    cylinder_mesh: CylinderMesh,
    instance_buffer: wgpu::Buffer,
}
```

**パフォーマンス目標**:
- 1000個の円筒工具を60FPS以上で描画

#### 2.2 GPUインスタンシングの活用

**wgpu実装例**:
```rust
// インスタンスバッファの作成
let instance_data: Vec<InstanceData> = tool_positions
    .iter()
    .map(|pos| InstanceData {
        position: [pos.x as f32, pos.y as f32, pos.z as f32],
        rotation: [0.0, 0.0, 0.0, 1.0],
        scale: tool_radius as f32,
    })
    .collect();

let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    label: Some("Instance Buffer"),
    contents: bytemuck::cast_slice(&instance_data),
    usage: wgpu::BufferUsages::VERTEX,
});

// 描画時
render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
render_pass.draw(0..vertex_count, 0..instance_count);
```

---

### 3. アニメーション制御のアーキテクチャ

#### 3.1 タイムライン管理

**設計方針**:
```rust
pub struct AnimationController {
    current_time: f64,        // 現在時刻（秒）
    total_duration: f64,      // 総時間（秒）
    playback_speed: f64,      // 再生速度（1.0 = 等倍）
    is_playing: bool,         // 再生中フラグ
    loop_enabled: bool,       // ループ再生フラグ
}

impl AnimationController {
    pub fn update(&mut self, delta_time: f64) {
        if self.is_playing {
            self.current_time += delta_time * self.playback_speed;
            
            if self.current_time >= self.total_duration {
                if self.loop_enabled {
                    self.current_time = 0.0;
                } else {
                    self.is_playing = false;
                    self.current_time = self.total_duration;
                }
            }
        }
    }
    
    pub fn get_normalized_time(&self) -> f64 {
        self.current_time / self.total_duration
    }
}
```

#### 3.2 再生速度制御

**速度オプション**:
- 0.1x（スロー再生、詳細確認用）
- 0.5x
- 1.0x（等倍、デフォルト）
- 2.0x
- 5.0x
- 10.0x（高速プレビュー）

**UI制御**:
- スライダーによる連続調整
- プリセットボタン（0.1x, 1x, 5x, 10x）

---

## アーキテクチャ設計

### 1. データモデル（Model層）

#### 1.1 ToolPath構造体

```rust
// model/geo_algorithms/src/toolpath.rs

use geo_primitives::{LineSegment3D, Arc3D, Point3D};
use analysis::Scalar;

/// 工具経路セグメント
#[derive(Debug, Clone)]
pub enum PathSegment<T: Scalar> {
    /// 直線補間（G01）
    Linear(LineSegment3D<T>),
    
    /// 円弧補間（G02/G03）
    Arc(Arc3D<T>),
    
    /// 早送り（G00）
    Rapid(LineSegment3D<T>),
}

impl<T: Scalar> PathSegment<T> {
    /// セグメントの長さを取得
    pub fn length(&self) -> T {
        match self {
            Self::Linear(seg) => seg.length(),
            Self::Arc(arc) => arc.arc_length(),
            Self::Rapid(seg) => seg.length(),
        }
    }
    
    /// パラメータ t (0.0-1.0) の位置を取得
    pub fn point_at(&self, t: T) -> Point3D<T> {
        match self {
            Self::Linear(seg) => seg.point_at_parameter(t),
            Self::Arc(arc) => arc.point_at_parameter(t),
            Self::Rapid(seg) => seg.point_at_parameter(t),
        }
    }
}

/// 工具経路メタデータ
#[derive(Debug, Clone)]
pub struct PathMetadata {
    /// 加工番号（オペレーション番号）
    pub operation_id: u32,
    
    /// 説明
    pub description: String,
    
    /// 推定加工時間（秒）
    pub estimated_time: f64,
}

/// 工具経路全体
#[derive(Debug, Clone)]
pub struct ToolPath<T: Scalar> {
    /// パスセグメント列
    segments: Vec<PathSegment<T>>,
    
    /// 工具径
    tool_diameter: T,
    
    /// 送り速度（mm/min）
    feed_rate: T,
    
    /// メタデータ
    metadata: PathMetadata,
}

impl<T: Scalar> ToolPath<T> {
    /// 新規作成
    pub fn new(
        segments: Vec<PathSegment<T>>,
        tool_diameter: T,
        feed_rate: T,
        metadata: PathMetadata,
    ) -> Self {
        Self {
            segments,
            tool_diameter,
            feed_rate,
            metadata,
        }
    }
    
    /// 総パス長さを計算
    pub fn total_length(&self) -> T {
        self.segments.iter().map(|seg| seg.length()).sum()
    }
    
    /// セグメント数
    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }
    
    /// セグメント参照
    pub fn segments(&self) -> &[PathSegment<T>] {
        &self.segments
    }
}
```

#### 1.2 OffsetResult構造体

```rust
// model/geo_algorithms/src/offset.rs

/// 2D輪郭オフセット結果
#[derive(Debug, Clone)]
pub struct OffsetResult2D<T: Scalar> {
    /// オリジナル輪郭
    original: Vec<LineSegment2D<T>>,
    
    /// オフセット輪郭
    offset: Vec<LineSegment2D<T>>,
    
    /// オフセット距離
    distance: T,
    
    /// 内部島（ポケット加工時）
    islands: Vec<Vec<LineSegment2D<T>>>,
}

/// 3Dメッシュオフセット結果
#[derive(Debug, Clone)]
pub struct OffsetResult3D<T: Scalar> {
    /// オリジナルメッシュ
    original: TriangleMesh3D<T>,
    
    /// オフセットメッシュ
    offset: TriangleMesh3D<T>,
    
    /// オフセット距離
    distance: T,
}
```

---

### 2. ViewModel層変換（ViewModel層）

#### 2.1 ToolPath変換

```rust
// viewmodel/converter/src/toolpath_converter.rs

use model_geo_algorithms::toolpath::{ToolPath, PathSegment};
use view_render::vertex_3d::Vertex3D;

/// 可視化オプション
#[derive(Debug, Clone)]
pub struct VisualizationOptions {
    /// 早送りを表示するか
    pub show_rapid_moves: bool,
    
    /// 工具表示モード
    pub tool_display: ToolDisplayMode,
    
    /// パスの線の太さ（ピクセル）
    pub path_thickness: f32,
    
    /// テセレーション品質（円弧の分割数）
    pub arc_segments: u32,
}

impl Default for VisualizationOptions {
    fn default() -> Self {
        Self {
            show_rapid_moves: true,
            tool_display: ToolDisplayMode::None,
            path_thickness: 2.0,
            arc_segments: 32,
        }
    }
}

/// 工具表示モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolDisplayMode {
    /// 表示しない
    None,
    
    /// 簡易表示（点のみ）
    Simple,
    
    /// 円筒表示
    Cylinder,
}

/// 工具経路頂点データ
#[derive(Debug)]
pub struct ToolPathVertices {
    /// パス線分の頂点
    pub path_vertices: Vec<Vertex3D>,
    
    /// 工具形状の頂点（オプション）
    pub tool_vertices: Option<Vec<Vertex3D>>,
    
    /// 色情報（セグメント種別ごと）
    pub colors: Vec<[f32; 4]>,
}

/// 工具経路をGPU頂点データに変換
pub fn toolpath_to_vertices(
    path: &ToolPath<f64>,
    options: &VisualizationOptions,
) -> ToolPathVertices {
    let mut path_vertices = Vec::new();
    let mut colors = Vec::new();
    
    for segment in path.segments() {
        let (vertices, color) = match segment {
            PathSegment::Linear(seg) => {
                let v = vec![
                    vertex_from_point(seg.start_position()),
                    vertex_from_point(seg.end_position()),
                ];
                (v, [1.0, 1.0, 1.0, 1.0]) // 白色
            },
            PathSegment::Arc(arc) => {
                let v = tessellate_arc(arc, options.arc_segments);
                (v, [1.0, 1.0, 1.0, 1.0]) // 白色
            },
            PathSegment::Rapid(seg) => {
                if !options.show_rapid_moves {
                    continue;
                }
                let v = vec![
                    vertex_from_point(seg.start_position()),
                    vertex_from_point(seg.end_position()),
                ];
                (v, [0.2, 0.5, 1.0, 1.0]) // 青色
            },
        };
        
        path_vertices.extend(vertices);
        colors.push(color);
    }
    
    // 工具形状の生成（オプション）
    let tool_vertices = match options.tool_display {
        ToolDisplayMode::None => None,
        ToolDisplayMode::Simple => Some(generate_tool_points(path)),
        ToolDisplayMode::Cylinder => Some(generate_tool_cylinders(path)),
    };
    
    ToolPathVertices {
        path_vertices,
        tool_vertices,
        colors,
    }
}

fn vertex_from_point(p: &Point3D<f64>) -> Vertex3D {
    Vertex3D {
        position: [p.x() as f32, p.y() as f32, p.z() as f32],
    }
}

fn tessellate_arc(arc: &Arc3D<f64>, segments: u32) -> Vec<Vertex3D> {
    // 円弧を線分に分割
    (0..=segments)
        .map(|i| {
            let t = i as f64 / segments as f64;
            vertex_from_point(&arc.point_at_parameter(t))
        })
        .collect()
}
```

---

### 3. View層レンダリング（View層）

#### 3.1 ToolPathResources

```rust
// view/render/src/toolpath.rs

use wgpu;
use std::sync::Arc;

pub struct ToolPathResources {
    device: Arc<wgpu::Device>,
    
    /// パス描画パイプライン
    path_pipeline: wgpu::RenderPipeline,
    
    /// 工具描画パイプライン（オプション）
    tool_pipeline: Option<wgpu::RenderPipeline>,
    
    /// 頂点バッファ
    vertex_buffer: Option<wgpu::Buffer>,
    
    /// 頂点数
    vertex_count: u32,
    
    /// ユニフォームバッファ（カメラ行列）
    uniform_buffer: wgpu::Buffer,
    
    /// バインドグループ
    bind_group: wgpu::BindGroup,
}

impl ToolPathResources {
    pub fn new(device: &Arc<wgpu::Device>, format: wgpu::TextureFormat) -> Self {
        // パイプライン構築
        let path_pipeline = create_toolpath_pipeline(device, format);
        
        // ユニフォームバッファ作成
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ToolPath Uniform Buffer"),
            size: 64, // mat4x4<f32>
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        // バインドグループ作成
        let bind_group_layout = path_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ToolPath Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        
        Self {
            device: Arc::clone(device),
            path_pipeline,
            tool_pipeline: None,
            vertex_buffer: None,
            vertex_count: 0,
            uniform_buffer,
            bind_group,
        }
    }
    
    /// パスデータを更新
    pub fn update_path_data(&mut self, vertices: &[Vertex3D]) {
        if vertices.is_empty() {
            self.vertex_buffer = None;
            self.vertex_count = 0;
            return;
        }
        
        let buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ToolPath Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        self.vertex_buffer = Some(buffer);
        self.vertex_count = vertices.len() as u32;
    }
    
    /// カメラ行列を更新
    pub fn update_camera(&mut self, queue: &wgpu::Queue, view_proj_matrix: &[[f32; 4]; 4]) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(view_proj_matrix));
    }
    
    /// 描画
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        if self.vertex_buffer.is_none() || self.vertex_count == 0 {
            return;
        }
        
        render_pass.set_pipeline(&self.path_pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
        render_pass.draw(0..self.vertex_count, 0..1);
    }
}

fn create_toolpath_pipeline(device: &wgpu::Device, format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
    // シェーダモジュールのロード
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("ToolPath Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/toolpath.wgsl").into()),
    });
    
    // パイプライン構築
    // （詳細は省略）
    todo!()
}
```

#### 3.2 toolpath.wgsl シェーダ

```wgsl
// view/render/shaders/toolpath.wgsl

struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    output.color = vec4<f32>(1.0, 1.0, 1.0, 1.0); // デフォルト白色
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
```

---

### 4. Stage管理（View層）

```rust
// view/stage/src/toolpath_stage.rs

use view_render::toolpath::ToolPathResources;
use super::render_stage::RenderStage;

pub struct ToolPathStage {
    resources: ToolPathResources,
    animation_controller: AnimationController,
}

impl ToolPathStage {
    pub fn new(device: &Arc<wgpu::Device>, format: wgpu::TextureFormat) -> Self {
        Self {
            resources: ToolPathResources::new(device, format),
            animation_controller: AnimationController::default(),
        }
    }
    
    pub fn set_toolpath(&mut self, vertices: &[Vertex3D]) {
        self.resources.update_path_data(vertices);
    }
    
    pub fn update_camera(&mut self, queue: &wgpu::Queue, view_proj: &[[f32; 4]; 4]) {
        self.resources.update_camera(queue, view_proj);
    }
}

impl RenderStage for ToolPathStage {
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ToolPath Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        
        self.resources.render(&mut render_pass);
    }
    
    fn update(&mut self) {
        // アニメーション更新などをここで実行
        self.animation_controller.update(0.016); // 60FPS想定
    }
}

struct AnimationController {
    current_time: f64,
    total_duration: f64,
    playback_speed: f64,
    is_playing: bool,
}

impl Default for AnimationController {
    fn default() -> Self {
        Self {
            current_time: 0.0,
            total_duration: 1.0,
            playback_speed: 1.0,
            is_playing: false,
        }
    }
}

impl AnimationController {
    fn update(&mut self, delta_time: f64) {
        if self.is_playing {
            self.current_time += delta_time * self.playback_speed;
            if self.current_time >= self.total_duration {
                self.current_time = self.total_duration;
                self.is_playing = false;
            }
        }
    }
}
```

---

## 実装計画

### Phase 1: 基本的な工具経路表示（2日）

**Day 1**:
- [ ] `ToolPath` データ構造実装
- [ ] `toolpath_to_vertices()` 変換関数実装
- [ ] 単体テスト作成

**Day 2**:
- [ ] `ToolPathResources` 実装
- [ ] `toolpath.wgsl` シェーダ作成
- [ ] `ToolPathStage` 実装
- [ ] 統合テスト

### Phase 2: オフセット結果表示（1日）

**Day 3**:
- [ ] `OffsetResult2D` / `OffsetResult3D` 実装
- [ ] オフセット結果の頂点変換
- [ ] 表示確認

### Phase 3: 工具形状表示（後回し）

**将来実装**:
- [ ] 円筒工具メッシュ生成
- [ ] インスタンシング実装
- [ ] アニメーション制御

---

## テスト戦略

### 単体テスト

**Model層**:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_toolpath_length() {
        let seg1 = PathSegment::Linear(LineSegment3D::new(
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(10.0, 0.0, 0.0),
        ));
        let seg2 = PathSegment::Rapid(LineSegment3D::new(
            Point3D::new(10.0, 0.0, 0.0),
            Point3D::new(10.0, 10.0, 0.0),
        ));
        
        let path = ToolPath::new(
            vec![seg1, seg2],
            5.0,
            1000.0,
            PathMetadata::default(),
        );
        
        assert_eq!(path.total_length(), 20.0);
    }
}
```

**ViewModel層**:
```rust
#[test]
fn test_toolpath_conversion() {
    let path = create_sample_toolpath();
    let options = VisualizationOptions::default();
    
    let vertices = toolpath_to_vertices(&path, &options);
    
    assert!(!vertices.path_vertices.is_empty());
    assert_eq!(vertices.colors.len(), path.segment_count());
}
```

### 統合テスト

**サンプルデータ**:
- [ ] 単純パス（直線のみ、10セグメント）
- [ ] 複雑パス（円弧含む、100セグメント）
- [ ] 大規模パス（1000セグメント以上）

**検証項目**:
- [ ] 描画の正確性（目視確認）
- [ ] FPS測定（60FPS以上）
- [ ] メモリ使用量確認

---

## 次のアクション

1. **即座に着手**:
   - [ ] GitHub Issue作成
   - [ ] ToolPath データ構造実装開始

2. **今週中に完了**:
   - [ ] Phase 1 実装完了
   - [ ] 基本的な工具経路表示の動作確認

3. **来週着手**:
   - [ ] オフセット結果表示の実装
   - [ ] パフォーマンステスト

**次回レビュー**: Phase 1完了時（2026年2月12日予定）
