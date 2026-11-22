//! 色の異なる複数のメッシュの描画（半透明を含む）
//!
//! メッシュは数値のキーで持ち、キーごとに差し替え・削除できる。不透明なメッシュを先に描き、半透明（色のアルファ < 1）の
//! メッシュは色を混ぜて、深度を書かずに後から描く。ワイヤーフレームの表示では、すべてのメッシュの三角形の辺を描く。
//!
//! 行列と色の uniform は色ごとに 1 つ持ち、同じ色のメッシュで共有する（カメラが動いたときの書き込みを色の数だけにする）。

use std::collections::{BTreeMap, HashMap};
use std::ops::RangeInclusive;

use crate::gpu_buffer::{GpuDataError, IndexedPart, indexed_parts};
use crate::mesh::MeshUniforms;
use crate::shader;
use crate::vertex_3d::MeshVertex;
use viewmodel_graphics::build_view_projection_matrix;
use wgpu::util::DeviceExt;

/// 描画する 1 つのメッシュ（頂点・頂点番号・色）
pub struct MeshListItem {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub color: [f32; 4],
}

/// GPU に置いた 1 つのメッシュ（バッファー 1 つの大きさの上限に収まるように分けた塊）
struct GpuMesh {
    parts: Vec<IndexedPart>,
    color: [f32; 4],
}

impl GpuMesh {
    fn is_translucent(&self) -> bool {
        self.color[3] < 1.0
    }
}

/// 1 つの色の、行列と色の uniform
struct ColorUniform {
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

/// 色ごとに 1 つ持つ値と、その色を使うメッシュの数（使うメッシュがなくなった色の値は消す）
struct SharedByColor<T> {
    entries: HashMap<[u32; 4], (T, usize)>,
}

impl<T> Default for SharedByColor<T> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

impl<T> SharedByColor<T> {
    /// 色 `color` を使うメッシュを 1 つ増やす（その色の値がなければ `make` で作る）
    fn acquire(&mut self, color: [f32; 4], make: impl FnOnce() -> T) {
        self.entries
            .entry(color_key(color))
            .or_insert_with(|| (make(), 0))
            .1 += 1;
    }

    /// 色 `color` を使うメッシュを 1 つ減らす（使うメッシュがなくなれば値を消す）
    fn release(&mut self, color: [f32; 4]) {
        let key = color_key(color);
        if let Some((_, count)) = self.entries.get_mut(&key) {
            *count -= 1;
            if *count == 0 {
                self.entries.remove(&key);
            }
        }
    }

    fn get(&self, color: [f32; 4]) -> Option<&T> {
        self.entries.get(&color_key(color)).map(|(value, _)| value)
    }

    /// 色と値の組
    fn iter(&self) -> impl Iterator<Item = ([f32; 4], &T)> {
        self.entries
            .iter()
            .map(|(key, (value, _))| (key.map(f32::from_bits), value))
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

/// 色を表のキーにする（成分のビット列）
fn color_key(color: [f32; 4]) -> [u32; 4] {
    color.map(f32::to_bits)
}

/// 色の異なる複数のメッシュの描画リソース
pub struct MeshListResources {
    opaque_pipeline: wgpu::RenderPipeline,
    translucent_pipeline: wgpu::RenderPipeline,
    wireframe_pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    meshes: BTreeMap<u64, GpuMesh>,
    colors: SharedByColor<ColorUniform>,
    /// 最後に設定したビュー・プロジェクション行列（後から加えたメッシュにも使う）
    view_proj: [[f32; 4]; 4],
    wireframe: bool,
}

impl MeshListResources {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = shader::mesh_shader(device);
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("mesh_list_bind_group_layout"),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Mesh List Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = |label: &str,
                        blend: wgpu::BlendState,
                        depth_write: bool,
                        polygon_mode: wgpu::PolygonMode| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Some(MeshVertex::desc())],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    polygon_mode,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
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
            })
        };
        Self {
            opaque_pipeline: pipeline(
                "Mesh List Opaque Pipeline",
                wgpu::BlendState::REPLACE,
                true,
                wgpu::PolygonMode::Fill,
            ),
            translucent_pipeline: pipeline(
                "Mesh List Translucent Pipeline",
                // 色だけを混ぜ、描画先のアルファは変えない（ウィンドウが透けないようにする）
                wgpu::BlendState {
                    color: wgpu::BlendState::ALPHA_BLENDING.color,
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::Zero,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                },
                false,
                wgpu::PolygonMode::Fill,
            ),
            wireframe_pipeline: pipeline(
                "Mesh List Wireframe Pipeline",
                wgpu::BlendState::REPLACE,
                true,
                wgpu::PolygonMode::Line,
            ),
            bind_group_layout,
            meshes: BTreeMap::new(),
            colors: SharedByColor::default(),
            view_proj: MeshUniforms::default().view_proj,
            wireframe: false,
        }
    }

    /// 描画するメッシュをすべて置き換える（三角形のないメッシュは描かない）
    ///
    /// # Errors
    ///
    /// GPU のメモリが足りないなどで置けないメッシュがある場合（そのメッシュは描かず、ほかのメッシュは置く。最初の
    /// 誤りを返す）。
    pub fn set_meshes(
        &mut self,
        device: &wgpu::Device,
        items: Vec<(u64, MeshListItem)>,
    ) -> Result<(), GpuDataError> {
        self.meshes.clear();
        self.colors.clear();
        let mut result = Ok(());
        for (key, item) in items {
            // 最初の誤りを残す
            result = result.and(self.set_mesh(device, key, item));
        }
        result
    }

    /// キー `key` のメッシュを差し替える（三角形がなければ削除する）
    ///
    /// # Errors
    ///
    /// GPU のメモリが足りないなどで置けない場合（前のメッシュも削除し、描かない）。
    pub fn set_mesh(
        &mut self,
        device: &wgpu::Device,
        key: u64,
        item: MeshListItem,
    ) -> Result<(), GpuDataError> {
        // 前のメッシュを先に放す（新しいメッシュを置くメモリに使えるように）。色は、新しい色を数えてから減らす
        // （同じ色への差し替えでは uniform を作り直さない）
        let previous = self.meshes.remove(&key).map(|mesh| mesh.color);
        let placed = if item.vertices.is_empty() || item.indices.is_empty() {
            Ok(())
        } else {
            match indexed_parts(device, "Mesh List Buffer", &item.vertices, &item.indices) {
                Ok(parts) => {
                    let (view_proj, layout) = (self.view_proj, &self.bind_group_layout);
                    self.colors.acquire(item.color, || {
                        color_uniform(device, layout, view_proj, item.color)
                    });
                    self.meshes.insert(
                        key,
                        GpuMesh {
                            parts,
                            color: item.color,
                        },
                    );
                    Ok(())
                }
                Err(error) => Err(error),
            }
        };
        if let Some(previous) = previous {
            self.colors.release(previous);
        }
        placed
    }

    /// キーが `keys` の範囲のメッシュの色を `color` にする（頂点と頂点番号のバッファは作り直さない）
    pub fn set_color(&mut self, device: &wgpu::Device, keys: RangeInclusive<u64>, color: [f32; 4]) {
        let (view_proj, layout) = (self.view_proj, &self.bind_group_layout);
        for mesh in self.meshes.range_mut(keys).map(|(_, mesh)| mesh) {
            self.colors
                .acquire(color, || color_uniform(device, layout, view_proj, color));
            self.colors.release(mesh.color);
            mesh.color = color;
        }
    }

    /// ワイヤーフレームの表示にするか
    pub fn set_wireframe(&mut self, wireframe: bool) {
        self.wireframe = wireframe;
    }

    /// カメラ行列を更新する
    pub fn update_camera(
        &mut self,
        queue: &wgpu::Queue,
        view_matrix: [[f32; 4]; 4],
        proj_matrix: [[f32; 4]; 4],
    ) {
        let view_proj = build_view_projection_matrix(view_matrix, proj_matrix);
        self.view_proj = view_proj;
        for (color, uniform) in self.colors.iter() {
            let uniforms = MeshUniforms {
                view_proj,
                base_color: color,
                ..MeshUniforms::default()
            };
            queue.write_buffer(
                &uniform.uniform_buffer,
                0,
                bytemuck::cast_slice(&[uniforms]),
            );
        }
    }

    /// 不透明なメッシュ、半透明のメッシュの順に描く（ワイヤーフレームの表示では、すべてのメッシュの辺を描く）
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        if self.wireframe {
            render_pass.set_pipeline(&self.wireframe_pipeline);
            for mesh in self.meshes.values() {
                draw(render_pass, &self.colors, mesh);
            }
            return;
        }
        for translucent in [false, true] {
            let pipeline = if translucent {
                &self.translucent_pipeline
            } else {
                &self.opaque_pipeline
            };
            render_pass.set_pipeline(pipeline);
            for mesh in self
                .meshes
                .values()
                .filter(|mesh| mesh.is_translucent() == translucent)
            {
                draw(render_pass, &self.colors, mesh);
            }
        }
    }
}

/// 色 `color` の、行列と色の uniform を作る
fn color_uniform(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view_proj: [[f32; 4]; 4],
    color: [f32; 4],
) -> ColorUniform {
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Mesh List Uniform Buffer"),
        contents: bytemuck::cast_slice(&[MeshUniforms {
            view_proj,
            base_color: color,
            ..MeshUniforms::default()
        }]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
        label: Some("mesh_list_bind_group"),
    });
    ColorUniform {
        uniform_buffer,
        bind_group,
    }
}

fn draw<'a>(
    render_pass: &mut wgpu::RenderPass<'a>,
    colors: &'a SharedByColor<ColorUniform>,
    mesh: &'a GpuMesh,
) {
    let uniform = colors
        .get(mesh.color)
        .expect("every mesh's color has a uniform");
    render_pass.set_bind_group(0, &uniform.bind_group, &[]);
    for part in &mesh.parts {
        part.draw(render_pass);
    }
}

#[cfg(test)]
mod tests {
    use super::{SharedByColor, color_key};

    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const GLASS: [f32; 4] = [0.5, 0.5, 0.5, 0.4];

    /// 色と、その色の値（作った順の番号）と、その色を使うメッシュの数（値の順）
    fn entries(shared: &SharedByColor<usize>) -> Vec<([f32; 4], usize, usize)> {
        let mut entries: Vec<_> = shared
            .iter()
            .map(|(color, &value)| (color, value, shared.entries[&color_key(color)].1))
            .collect();
        entries.sort_by_key(|&(_, value, _)| value);
        entries
    }

    /// 呼ぶたびに次の番号を返す、値の作り方
    fn numbering() -> impl FnMut() -> usize {
        let mut made = 0;
        move || {
            made += 1;
            made
        }
    }

    #[test]
    fn meshes_of_the_same_color_share_one_value_until_the_last_is_released() {
        let mut shared = SharedByColor::default();
        let mut make = numbering();
        shared.acquire(RED, &mut make);
        shared.acquire(RED, &mut make);
        shared.acquire(GLASS, &mut make);
        assert_eq!(entries(&shared), vec![(RED, 1, 2), (GLASS, 2, 1)]);
        assert_eq!(shared.get(RED), Some(&1));
        shared.release(RED);
        assert_eq!(entries(&shared), vec![(RED, 1, 1), (GLASS, 2, 1)]);
        shared.release(RED);
        assert_eq!(shared.get(RED), None);
        assert_eq!(entries(&shared), vec![(GLASS, 2, 1)]);
        // 使わなくなった色をもう一度使うと、値を作り直す
        shared.acquire(RED, &mut make);
        assert_eq!(shared.get(RED), Some(&3));
    }

    #[test]
    fn replacing_a_mesh_keeps_the_value_of_the_same_color_and_moves_to_another_color() {
        let mut shared = SharedByColor::default();
        let mut make = numbering();
        // 差し替えは、新しい色を数えてから前の色を減らす
        shared.acquire(RED, &mut make);
        shared.acquire(RED, &mut make);
        shared.release(RED);
        assert_eq!(entries(&shared), vec![(RED, 1, 1)]);
        shared.acquire(GLASS, &mut make);
        shared.release(RED);
        assert_eq!(entries(&shared), vec![(GLASS, 2, 1)]);
    }

    #[test]
    fn recoloring_meshes_makes_the_new_color_once_and_drops_the_old_after_the_last() {
        let mut shared = SharedByColor::default();
        let mut make = numbering();
        for _ in 0..3 {
            shared.acquire(RED, &mut make);
        }
        // メッシュごとに、新しい色を数えてから前の色を減らす（`set_color`）
        for remaining in (0..3).rev() {
            shared.acquire(GLASS, &mut make);
            shared.release(RED);
            let red = (remaining > 0).then_some((RED, 1, remaining));
            let glass = (GLASS, 2, 3 - remaining);
            assert_eq!(
                entries(&shared),
                red.into_iter().chain([glass]).collect::<Vec<_>>()
            );
        }
    }
}
