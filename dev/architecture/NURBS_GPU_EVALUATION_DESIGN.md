# NURBS GPU評価パイプライン設計 (Issue #210 Phase 2)

**最終更新**: 2026-02-13  
**関連Issue**: #210  
**依存**: NURBS_ADAPTIVE_TESSELLATION_DESIGN.md  
**ステータス**: 設計フェーズ

---

## 1. 概要

### 目的
CPU側で生成した適応的パラメータリストを用いて、GPU上でNURBS曲線・サーフェスを評価し直接レンダリングする。

### スコープ
- **Phase 2**: Vertex Shader評価（Plan D: Hybrid CPU/GPU）
  - CPU: 適応パラメータ分割 (`AdaptiveParamList<T>`, `AdaptiveParamGrid<T>`)
  - GPU: NURBS評価（basis functions + 制御点補間）
  - 出力: LineStrip (曲線), Wireframe (サーフェス)

- **将来拡張**: Phase 3でCompute Shader評価（Plan C）への移行可能性

### 技術制約
- **Scalar型変換**: Rust側 `T: Scalar` → GPU側 `f32`
- **WGSL制限**: 動的配列はStorage Bufferで対応
- **既存パターン踏襲**: `LineResources`, `line.wgsl` の設計を参考

---

## 2. アーキテクチャ

### 2.1 全体フロー

```
[geo_nurbs: CPU側]
  NurbsCurve3D::adaptive_params_curve()
    -> AdaptiveParamList<T> { params: Vec<T> }

[viewmodel-converter: 変換層]
  NurbsParamListViewModel {
    params: Vec<f32>,          // T -> f32変換済み
    control_points: Vec<f32>,  // flatten: [(x,y,z), ...] -> [x,y,z,x,y,z,...]
    weights: Option<Vec<f32>>,
    knots: Vec<f32>,
    degree: u32,
  }

[view/render: GPU渡し層]
  NurbsEvalResources {
    render_pipeline: RenderPipeline,
    uniform_buffer: Buffer (view_proj, model),
    nurbs_buffer: Buffer (control points, weights, knots, degree),
    param_buffer: Buffer (params),
    bind_group: BindGroup,
  }

[GPU: WGSL Vertex Shader]
  vs_nurbs_curve_main()
    -> @builtin(vertex_index)でparam_buffer[index]取得
    -> NURBS basis functions計算
    -> 制御点補間 -> world position
    -> view_proj変換 -> clip_position
```

### 2.2 データフロー図

```
CPU (Rust)                  GPU (WGSL)
-------------               -------------
AdaptiveParamList<f64>
  |
  v (convert)
Vec<f32> params
  |
  v (create_buffer)
param_buffer ───────────> @group(1) @binding(0) var<storage> params: array<f32>;
  
NurbsCurve3D<f64>
  control_points: Vec<(f64,f64,f64)>
  knots: Vec<f64>
  weights: Option<Vec<f64>>
  |
  v (flatten + convert)
Vec<f32> flat_cp, flat_weights, flat_knots
  |
  v (create_buffer)
nurbs_buffer ────────────> @group(1) @binding(1) var<storage> nurbs: NurbsData;
```

---

## 3. 詳細設計

### 3.1 Model Layer (geo_nurbs): 既存実装（Phase 1完了）

**ファイル**: `model/geo_nurbs/src/adaptive_tessellation.rs`

```rust
pub struct AdaptiveTessellationSettings<T: Scalar> {
    pub display_tolerance: T,
    pub max_depth: usize,
    pub min_segments: usize,
}

pub struct AdaptiveParamList<T: Scalar> {
    pub params: Vec<T>,
}

pub trait NurbsCurveAdaptiveTessellation<T: Scalar> {
    fn adaptive_params_curve(
        &self,
        settings: &AdaptiveTessellationSettings<T>,
    ) -> AdaptiveParamList<T>;
}
```

**実装済み**: `curve_3d_extensions.rs`, `surface_3d_extensions.rs`

---

### 3.2 ViewModel Layer (viewmodel-converter)

**新規ファイル**: `viewmodel/converter/src/nurbs_view.rs`

#### 3.2.1 データ構造

```rust
use geo_foundation::Scalar;

/// NURBS曲線の評価に必要なデータ（GPU渡し用）
#[derive(Debug, Clone)]
pub struct NurbsCurveEvalData {
    /// 評価パラメータ列（ソート済み）
    pub params: Vec<f32>,
    
    /// 制御点（flattenされた配列: [x0, y0, z0, x1, y1, z1, ...]）
    pub control_points: Vec<f32>,
    
    /// 重み（有理NURBS用、Noneなら非有理）
    pub weights: Option<Vec<f32>>,
    
    /// ノットベクトル
    pub knots: Vec<f32>,
    
    /// 次数
    pub degree: u32,
}

impl NurbsCurveEvalData {
    /// NurbsCurve3DとAdaptiveParamListから生成
    pub fn from_curve_params<T: Scalar>(
        curve: &geo_nurbs::NurbsCurve3D<T>,
        param_list: &geo_nurbs::adaptive_tessellation::AdaptiveParamList<T>,
    ) -> Self {
        use geo_foundation::core::NurbsCurve3DProperties;
        
        let degree = curve.degree() as u32;
        let knots = curve.knots().iter().map(|k| k.to_f32().unwrap()).collect();
        
        // 制御点のflatten
        let control_points: Vec<f32> = curve.control_points()
            .iter()
            .flat_map(|(x, y, z)| vec![
                x.to_f32().unwrap(),
                y.to_f32().unwrap(),
                z.to_f32().unwrap(),
            ])
            .collect();
        
        // 重みの変換
        let weights = curve.weights().map(|w| {
            w.iter().map(|wi| wi.to_f32().unwrap()).collect()
        });
        
        // パラメータの変換
        let params = param_list.params.iter()
            .map(|p| p.to_f32().unwrap())
            .collect();
        
        Self {
            params,
            control_points,
            weights,
            knots,
            degree,
        }
    }
    
    /// 制御点数
    pub fn num_control_points(&self) -> usize {
        self.control_points.len() / 3
    }
    
    /// 評価点数
    pub fn num_eval_points(&self) -> usize {
        self.params.len()
    }
}
```

#### 3.2.2 サーフェス用データ構造（将来）

```rust
/// NURBSサーフェスの評価に必要なデータ（GPU渡し用）
#[derive(Debug, Clone)]
pub struct NurbsSurfaceEvalData {
    pub u_params: Vec<f32>,
    pub v_params: Vec<f32>,
    pub control_points: Vec<f32>, // flatten: row-major [cp[0][0], cp[0][1], ...]
    pub weights: Option<Vec<f32>>, // flatten同様
    pub u_knots: Vec<f32>,
    pub v_knots: Vec<f32>,
    pub u_degree: u32,
    pub v_degree: u32,
    pub u_count: u32, // control points u方向数
    pub v_count: u32, // control points v方向数
}
```

---

### 3.3 View Layer (view/render)

**新規ファイル**: `view/render/src/nurbs_eval.rs`

#### 3.3.1 リソース構造

```rust
use wgpu::util::DeviceExt;
use bytemuck::{Pod, Zeroable};

/// NURBS評価用Uniform（カメラ行列）
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct NurbsEvalUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub model: [[f32; 4]; 4],
}

impl Default for NurbsEvalUniforms {
    fn default() -> Self {
        Self {
            view_proj: Matrix4x4::identity().to_array(),
            model: Matrix4x4::identity().to_array(),
        }
    }
}

/// NURBS評価リソース（曲線用）
pub struct NurbsCurveEvalResources {
    pub render_pipeline: wgpu::RenderPipeline,
    
    // @group(0): カメラ行列
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,
    
    // @group(1): NURBSデータ
    pub param_buffer: wgpu::Buffer,         // @binding(0): array<f32> params
    pub control_point_buffer: wgpu::Buffer, // @binding(1): array<f32> cp (flatten)
    pub weight_buffer: Option<wgpu::Buffer>, // @binding(2): array<f32> weights
    pub knot_buffer: wgpu::Buffer,          // @binding(3): array<f32> knots
    pub degree_buffer: wgpu::Buffer,        // @binding(4): u32 degree
    
    pub nurbs_bind_group: wgpu::BindGroup,
    
    pub num_eval_points: u32,
}

impl NurbsCurveEvalResources {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        eval_data: &viewmodel_converter::NurbsCurveEvalData,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NURBS Curve Eval Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/nurbs_curve_eval.wgsl").into()
            ),
        });
        
        // === @group(0): Uniform (view_proj, model) ===
        let uniform_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("nurbs_uniform_bind_group_layout"),
            }
        );
        
        let uniforms = NurbsEvalUniforms::default();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Eval Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("nurbs_uniform_bind_group"),
        });
        
        // === @group(1): NURBS Data (Storage Buffers) ===
        let param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Param Buffer"),
            contents: bytemuck::cast_slice(&eval_data.params),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        let control_point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Control Point Buffer"),
            contents: bytemuck::cast_slice(&eval_data.control_points),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        let weight_buffer = eval_data.weights.as_ref().map(|weights| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NURBS Weight Buffer"),
                contents: bytemuck::cast_slice(weights),
                usage: wgpu::BufferUsages::STORAGE,
            })
        });
        
        let knot_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Knot Buffer"),
            contents: bytemuck::cast_slice(&eval_data.knots),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        let degree_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Degree Buffer"),
            contents: bytemuck::cast_slice(&[eval_data.degree]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        // Bind Group Layout for NURBS data
        let nurbs_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // @binding(0): params
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(1): control_points
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(2): weights (optional, 常に存在前提でダミー作成も可)
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(3): knots
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // @binding(4): degree
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("nurbs_data_bind_group_layout"),
            }
        );
        
        // Weights: 非有理の場合はダミーバッファ作成
        let weight_buffer_binding = weight_buffer.as_ref().unwrap_or_else(|| {
            // ダミーバッファ（サイズ1のf32配列）
            &device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NURBS Dummy Weight Buffer"),
                contents: bytemuck::cast_slice(&[1.0f32]),
                usage: wgpu::BufferUsages::STORAGE,
            })
        });
        
        let nurbs_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &nurbs_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: control_point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: weight_buffer_binding.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: knot_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: degree_buffer.as_entire_binding(),
                },
            ],
            label: Some("nurbs_data_bind_group"),
        });
        
        // === Render Pipeline ===
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("NURBS Eval Pipeline Layout"),
            bind_group_layouts: &[
                &uniform_bind_group_layout,
                &nurbs_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NURBS Curve Eval Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[], // vertex_indexのみでバッファなし
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        
        Self {
            render_pipeline,
            uniform_buffer,
            uniform_bind_group,
            param_buffer,
            control_point_buffer,
            weight_buffer,
            knot_buffer,
            degree_buffer,
            nurbs_bind_group,
            num_eval_points: eval_data.num_eval_points() as u32,
        }
    }
    
    /// カメラ行列更新
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &NurbsEvalUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }
    
    /// レンダリング実行
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.nurbs_bind_group, &[]);
        render_pass.draw(0..self.num_eval_points, 0..1);
    }
}
```

---

### 3.4 WGSL Shader

**新規ファイル**: `view/render/shaders/nurbs_curve_eval.wgsl`

```wgsl
// NURBS曲線 GPU評価シェーダー
// Vertex Shaderで直接NURBS basis functionsを計算し、LineStripで描画

// === @group(0): カメラ行列 ===
struct Uniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// === @group(1): NURBSデータ ===
@group(1) @binding(0)
var<storage, read> params: array<f32>; // 評価パラメータ列

@group(1) @binding(1)
var<storage, read> control_points: array<f32>; // flatten: [x0,y0,z0, x1,y1,z1, ...]

@group(1) @binding(2)
var<storage, read> weights: array<f32>; // 重み（非有理の場合は全て1.0のダミー）

@group(1) @binding(3)
var<storage, read> knots: array<f32>; // ノットベクトル

@group(1) @binding(4)
var<storage, read> degree_storage: array<u32>; // [degree] (1要素配列)

// === ヘルパー関数 ===

/// ノットスパン検索（Cox-de Boor再帰用）
fn find_knot_span(u: f32, degree: u32, num_cp: u32) -> u32 {
    let n = num_cp - 1u;
    let p = degree;
    
    // 最後のノット値の場合
    if u >= knots[n + p + 1u] {
        return n;
    }
    
    // バイナリサーチ
    var low = p;
    var high = n + 1u;
    var mid = (low + high) / 2u;
    
    while u < knots[mid] || u >= knots[mid + 1u] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2u;
    }
    
    return mid;
}

/// NURBS basis function (Cox-de Boor再帰)
fn basis_function(i: u32, p: u32, u: f32) -> f32 {
    // p=0の場合（ベース）
    if p == 0u {
        if u >= knots[i] && u < knots[i + 1u] {
            return 1.0;
        } else {
            return 0.0;
        }
    }
    
    // 再帰的計算
    let left_num = u - knots[i];
    let left_den = knots[i + p] - knots[i];
    var left = 0.0;
    if abs(left_den) > 1e-10 {
        left = left_num / left_den * basis_function(i, p - 1u, u);
    }
    
    let right_num = knots[i + p + 1u] - u;
    let right_den = knots[i + p + 1u] - knots[i + 1u];
    var right = 0.0;
    if abs(right_den) > 1e-10 {
        right = right_num / right_den * basis_function(i + 1u, p - 1u, u);
    }
    
    return left + right;
}

/// NURBS曲線評価（有理・非有理両対応）
fn evaluate_nurbs(u: f32, degree: u32, num_cp: u32) -> vec3<f32> {
    let span = find_knot_span(u, degree, num_cp);
    
    var numerator = vec3<f32>(0.0, 0.0, 0.0);
    var denominator = 0.0;
    
    // スパンに影響する制御点のみ計算（degree+1個）
    for (var j = 0u; j <= degree; j = j + 1u) {
        let idx = span - degree + j;
        let N = basis_function(idx, degree, u);
        let w = weights[idx];
        
        let cp_base = idx * 3u;
        let cp = vec3<f32>(
            control_points[cp_base],
            control_points[cp_base + 1u],
            control_points[cp_base + 2u]
        );
        
        numerator = numerator + N * w * cp;
        denominator = denominator + N * w;
    }
    
    if abs(denominator) > 1e-10 {
        return numerator / denominator;
    } else {
        return numerator; // 非有理の場合（全てw=1.0）
    }
}

// === Vertex Shader ===
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let degree = degree_storage[0];
    let num_cp = arrayLength(&control_points) / 3u;
    
    // パラメータ取得
    let u = params[vertex_index];
    
    // NURBS評価
    let position = evaluate_nurbs(u, degree, num_cp);
    
    // ワールド座標 -> クリップ座標
    let world_position = uniforms.model * vec4<f32>(position, 1.0);
    out.clip_position = uniforms.view_proj * world_position;
    
    return out;
}

// === Fragment Shader ===
@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // NURBS曲線は青色で表示
    return vec4<f32>(0.0, 0.5, 1.0, 1.0);
}
```

---

## 4. 実装手順

### Phase 2-1: ViewModel Layer実装
1. **ファイル作成**: `viewmodel/converter/src/nurbs_view.rs`
2. **データ構造**: `NurbsCurveEvalData`実装
3. **変換関数**: `from_curve_params<T>()`実装
4. **エクスポート**: `viewmodel/converter/src/lib.rs`に追加

### Phase 2-2: View Layer実装
1. **ファイル作成**: `view/render/src/nurbs_eval.rs`
2. **リソース**: `NurbsCurveEvalResources`実装
3. **Shader作成**: `view/render/shaders/nurbs_curve_eval.wgsl`
4. **エクスポート**: `view/render/src/lib.rs`に追加

### Phase 2-3: 統合テスト
1. **単体テスト**: WGSL basis function精度検証（既存geo_nurbsテストと比較）
2. **レンダリングテスト**: 簡単な曲線（直線、円弧）の描画確認
3. **パフォーマンス**: CPU vs GPU評価の速度比較

### Phase 2-4: ドキュメント更新
1. **manual/nurbs.md**: GPU評価の使用法追記
2. **SHAPE_VISUALIZATION_DESIGN.md**: NURBS直接レンダリング追記

---

## 5. テスト戦略

### 5.1 精度検証
- **直線**: 全評価点が一直線上に並ぶ
- **円弧**: 半径一定確認
- **高次曲線**: CPU側評価結果（既存`point_at()`）との誤差計測

### 5.2 レンダリング検証
- **視覚確認**: 既存メッシュテッセレーションと同じ曲線が描画される
- **パラメータ増減**: `display_tolerance`を変えて適応分割が反映されるか

### 5.3 パフォーマンス計測
- **評価点数**: 100, 1000, 10000でFPS計測
- **曲線数**: 複数曲線同時描画時のGPU負荷

---

## 6. 将来の拡張: Phase 3 (Compute Shader)

**Plan C**: CPU側のパラメータ分割もGPUで実施

```
CPU: NurbsCurve3D送信のみ
GPU Compute Shader: 
  1. 適応分割（弦誤差計算 + subdivision）
  2. NURBS評価
  3. LineStripバッファ生成
GPU Vertex Shader:
  上記バッファを直接描画
```

**移行条件**:
- Phase 2でパフォーマンスボトルネックがCPU側パラメータ生成にある場合
- WGSL Compute Shaderでの再帰的分割アルゴリズム実装可能性検証後

---

## 7. 関連ドキュメント

- [NURBS_ADAPTIVE_TESSELLATION_DESIGN.md](NURBS_ADAPTIVE_TESSELLATION_DESIGN.md) - Phase 1 CPU実装
- [SHAPE_TESSELLATION_DESIGN.md](SHAPE_TESSELLATION_DESIGN.md) - テッセレーション全体設計
- [SHAPE_VISUALIZATION_DESIGN.md](SHAPE_VISUALIZATION_DESIGN.md) - 可視化アーキテクチャ

---

## 8. 既知の制約・留意点

### 8.1 WGSL再帰の制限
- WGSL 1.0では再帰関数が使えない可能性がある環境も存在
- `basis_function()`の再帰深度は`degree`に依存（通常3〜5程度なので実用上問題なし）
- 必要に応じてループ展開版に書き換え可能

### 8.2 Storage Bufferサイズ
- WebGPU環境によっては最大サイズ制限あり
- 大規模サーフェス（制御点数万個）の場合は分割描画が必要

### 8.3 Scalar型の精度損失
- Rust側 `f64` → GPU側 `f32` 変換で精度劣化
- 表示用途なら問題ないが、数値計算用には注意

---

## 9. まとめ

Phase 2では**Vertex Shader評価（Plan D）**を実装し、CPU側の適応パラメータリストをGPUで直接評価・描画します。これにより：

✅ **geo_nurbsの責務明確化**: CPU側適応分割に特化  
✅ **GPU直接レンダリング実現**: 中間メッシュ不要  
✅ **Foundation Pattern準拠**: viewmodel層での型変換、view層でのGPU処理  
✅ **将来拡張性**: Phase 3でCompute Shader化も可能
