//! NURBS曲線 GPU評価リソース
//!
//! CPU側で生成した適応パラメータリストを用いて、
//! GPU上でNURBS曲線を直接評価・描画するためのリソース管理。

use crate::uniform_factory;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// NURBS評価用Uniform（カメラ行列）
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct NurbsEvalUniforms {
    /// ビュー・プロジェクション行列
    pub view_proj: [[f32; 4]; 4],
    /// モデル行列
    pub model: [[f32; 4]; 4],
}

impl Default for NurbsEvalUniforms {
    fn default() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

/// NURBS曲線評価リソース
///
/// Vertex ShaderでNURBS basis functionsを計算し、
/// LineStrip トポロジーで描画するためのGPUリソース。
pub struct NurbsCurveEvalResources {
    /// レンダーパイプライン
    pub render_pipeline: wgpu::RenderPipeline,

    // @group(0): カメラ行列
    /// Uniform バッファ（view_proj, model）
    pub uniform_buffer: wgpu::Buffer,
    /// Uniform Bind Group
    pub uniform_bind_group: wgpu::BindGroup,

    // @group(1): NURBSデータ
    /// パラメータバッファ（評価点のu値リスト）
    pub param_buffer: wgpu::Buffer,
    /// 制御点バッファ（flatten: [x,y,z,x,y,z,...]）
    pub control_point_buffer: wgpu::Buffer,
    /// 重みバッファ（有理NURBS用、非有理の場合はダミー）
    pub weight_buffer: wgpu::Buffer,
    /// ノットベクトルバッファ
    pub knot_buffer: wgpu::Buffer,
    /// 次数バッファ（1要素配列）
    pub degree_buffer: wgpu::Buffer,
    /// NURBS Data Bind Group
    pub nurbs_bind_group: wgpu::BindGroup,

    /// 評価点数（LineStrip頂点数）
    pub num_eval_points: u32,
}

impl NurbsCurveEvalResources {
    /// NURBS曲線評価リソースを作成
    ///
    /// # Arguments
    /// * `device` - wgpuデバイス
    /// * `format` - テクスチャフォーマット
    /// * `eval_data` - NURBS評価用データ（viewmodel層で変換済み）
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        eval_data: &viewmodel::nurbs_view::NurbsCurveEvalData,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NURBS Curve Eval Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/nurbs_curve_eval.wgsl").into(),
            ),
        });

        let uniforms = NurbsEvalUniforms::default();
        let (uniform_bind_group_layout, uniform_buffer, uniform_bind_group) =
            uniform_factory::create_uniform_binding(
                device,
                &uniforms,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                "nurbs_uniform_bind_group_layout",
                "NURBS Eval Uniform Buffer",
                "nurbs_uniform_bind_group",
            );

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

        // 重み: 非有理の場合は全て1.0のダミーバッファ作成
        let weight_buffer = if let Some(ref weights) = eval_data.weights {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NURBS Weight Buffer"),
                contents: bytemuck::cast_slice(weights),
                usage: wgpu::BufferUsages::STORAGE,
            })
        } else {
            // 非有理: 制御点数と同じサイズの1.0配列
            let dummy_weights = vec![1.0f32; eval_data.num_control_points()];
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NURBS Dummy Weight Buffer"),
                contents: bytemuck::cast_slice(&dummy_weights),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

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
        let nurbs_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
                    // @binding(2): weights
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
                    resource: weight_buffer.as_entire_binding(),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("NURBS Eval Pipeline Layout"),
            bind_group_layouts: &[
                Some(&uniform_bind_group_layout),
                Some(&nurbs_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NURBS Curve Eval Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[], // vertex_indexのみでバッファなし
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
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
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
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
    ///
    /// # Arguments
    /// * `queue` - wgpuキュー
    /// * `uniforms` - 更新するUniformデータ
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &NurbsEvalUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// レンダリング実行
    ///
    /// # Arguments
    /// * `render_pass` - wgpuレンダーパス
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        tracing::debug!(
            "📊 NurbsCurveEvalResources.render(): pipeline設定、頂点数={}",
            self.num_eval_points
        );

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.nurbs_bind_group, &[]);
        render_pass.draw(0..self.num_eval_points, 0..1);

        tracing::debug!(
            "📊 NurbsCurveEvalResources.render(): draw call実行 (0..{})",
            self.num_eval_points
        );
    }
}

/// NURBS曲面評価リソース
///
/// Vertex ShaderでNURBS basis functionsを計算し、
/// 三角形メッシュで描画するためのGPUリソース。
///
/// # メモリ最適化
/// - 頂点バッファ: (u,v)パラメータをインター リーブド配列で格納
/// - Storage Buffer数削減: 5個（u_params/v_paramsを統合）
pub struct NurbsSurfaceEvalResources {
    /// ソリッドレンダリングパイプライン
    pub solid_pipeline: wgpu::RenderPipeline,
    /// ワイヤーフレームレンダリングパイプライン
    pub wireframe_pipeline: wgpu::RenderPipeline,

    // @group(0): カメラ行列
    pub uniform_buffer: wgpu::Buffer,
    pub uniform_bind_group: wgpu::BindGroup,

    // 頂点バッファ: @location(0) uv_param: vec2<f32>
    pub vertex_buffer: wgpu::Buffer,

    // @group(1): NURBSデータ（Storage Buffers）
    pub control_point_buffer: wgpu::Buffer,
    pub weight_buffer: wgpu::Buffer,
    pub u_knot_buffer: wgpu::Buffer,
    pub v_knot_buffer: wgpu::Buffer,
    pub metadata_buffer: wgpu::Buffer, // [u_degree, v_degree, u_count, v_count]
    pub nurbs_bind_group: wgpu::BindGroup,

    // インデックスバッファ
    pub solid_index_buffer: wgpu::Buffer,
    pub wireframe_index_buffer: wgpu::Buffer,

    /// 頂点総数
    pub num_vertices: u32,
    /// ソリッド用インデックス数
    pub num_solid_indices: u32,
    /// ワイヤーフレーム用インデックス数
    pub num_wireframe_indices: u32,
}

impl NurbsSurfaceEvalResources {
    /// NURBS曲面評価リソースを作成
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        eval_data: &viewmodel::nurbs_view::NurbsSurfaceEvalData,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("NURBS Surface Eval Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/nurbs_surface_eval.wgsl").into(),
            ),
        });

        let uniforms = NurbsEvalUniforms::default();
        let (uniform_bind_group_layout, uniform_buffer, uniform_bind_group) =
            uniform_factory::create_uniform_binding(
                device,
                &uniforms,
                wgpu::ShaderStages::VERTEX_FRAGMENT,
                "nurbs_surface_uniform_bind_group_layout",
                "NURBS Surface Uniform Buffer",
                "nurbs_surface_uniform_bind_group",
            );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Surface Vertex Param Buffer"),
            contents: bytemuck::cast_slice(&eval_data.vertex_params),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let control_point_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Surface Control Point Buffer"),
            contents: bytemuck::cast_slice(&eval_data.control_points),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // 重み: 非有理の場合は全て1.0のダミーバッファ
        let num_control_points = (eval_data.u_count * eval_data.v_count) as usize;
        let weight_buffer = if let Some(ref weights) = eval_data.weights {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NURBS Surface Weight Buffer"),
                contents: bytemuck::cast_slice(weights),
                usage: wgpu::BufferUsages::STORAGE,
            })
        } else {
            let dummy_weights = vec![1.0f32; num_control_points];
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("NURBS Surface Dummy Weight Buffer"),
                contents: bytemuck::cast_slice(&dummy_weights),
                usage: wgpu::BufferUsages::STORAGE,
            })
        };

        let u_knot_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS U Knot Buffer"),
            contents: bytemuck::cast_slice(&eval_data.u_knots),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let v_knot_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS V Knot Buffer"),
            contents: bytemuck::cast_slice(&eval_data.v_knots),
            usage: wgpu::BufferUsages::STORAGE,
        });

        // Metadata: [u_degree, v_degree, u_count, v_count]
        let metadata = [
            eval_data.u_degree,
            eval_data.v_degree,
            eval_data.u_count,
            eval_data.v_count,
        ];
        let metadata_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Surface Metadata Buffer"),
            contents: bytemuck::cast_slice(&metadata),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let nurbs_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // @binding(0): control_points
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
                    // @binding(1): weights
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
                    // @binding(2): u_knots
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
                    // @binding(3): v_knots
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
                    // @binding(4): metadata
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
                label: Some("nurbs_surface_data_bind_group_layout"),
            });

        let nurbs_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &nurbs_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: control_point_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: weight_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: u_knot_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: v_knot_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: metadata_buffer.as_entire_binding(),
                },
            ],
            label: Some("nurbs_surface_data_bind_group"),
        });

        let solid_indices = eval_data.generate_indices();
        let wireframe_indices = eval_data.generate_wireframe_indices();

        let solid_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Surface Solid Index Buffer"),
            contents: bytemuck::cast_slice(&solid_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let wireframe_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("NURBS Surface Wireframe Index Buffer"),
            contents: bytemuck::cast_slice(&wireframe_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("NURBS Surface Pipeline Layout"),
            bind_group_layouts: &[
                Some(&uniform_bind_group_layout),
                Some(&nurbs_bind_group_layout),
            ],
            immediate_size: 0,
        });

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (std::mem::size_of::<f32>() * 2) as wgpu::BufferAddress, // vec2<f32>
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0, // @location(0) uv_param: vec2<f32>
                format: wgpu::VertexFormat::Float32x2,
            }],
        };

        let solid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NURBS Surface Solid Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: std::slice::from_ref(&vertex_buffer_layout),
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let wireframe_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("NURBS Surface Wireframe Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_buffer_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            solid_pipeline,
            wireframe_pipeline,
            uniform_buffer,
            uniform_bind_group,
            vertex_buffer,
            control_point_buffer,
            weight_buffer,
            u_knot_buffer,
            v_knot_buffer,
            metadata_buffer,
            nurbs_bind_group,
            solid_index_buffer,
            wireframe_index_buffer,
            num_vertices: eval_data.num_vertices() as u32,
            num_solid_indices: solid_indices.len() as u32,
            num_wireframe_indices: wireframe_indices.len() as u32,
        }
    }

    /// カメラ行列更新
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &NurbsEvalUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// レンダリング実行
    ///
    /// # Arguments
    /// * `render_pass` - wgpuレンダーパス
    /// * `wireframe` - trueならワイヤーフレーム、falseならソリッド
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, wireframe: bool) {
        render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.nurbs_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        if wireframe {
            render_pass.set_pipeline(&self.wireframe_pipeline);
            render_pass.set_index_buffer(
                self.wireframe_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            render_pass.draw_indexed(0..self.num_wireframe_indices, 0, 0..1);
            tracing::debug!(
                "📊 NurbsSurfaceEvalResources.render(): wireframe, {} indices",
                self.num_wireframe_indices
            );
        } else {
            render_pass.set_pipeline(&self.solid_pipeline);
            render_pass
                .set_index_buffer(self.solid_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.num_solid_indices, 0, 0..1);
            tracing::debug!(
                "📊 NurbsSurfaceEvalResources.render(): solid, {} indices ({} triangles)",
                self.num_solid_indices,
                self.num_solid_indices / 3
            );
        }
    }
}
