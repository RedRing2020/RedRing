//! AppState の製品形状の取り込みと表示を扱うモジュール。

use super::AppState;
use render::mesh_list::MeshListItem;
use render::vertex_3d::convert_vertex_data_to_mesh_vertices;
use stage::MeshStage;
use std::path::Path;

impl AppState {
    /// STL ファイルを製品形状として取り込み、表示中の製品形状を表示する
    pub fn import_product_shape(&mut self, path: &Path) {
        if let Err(error) = self.product_shapes.import_stl(path) {
            tracing::warn!(path = %path.display(), "product shape import failed: {error}");
            return;
        }
        tracing::info!(path = %path.display(), "product shape imported");
        self.show_product_shapes();
    }

    /// 表示中の製品形状をメッシュステージに描き、全体が見えるようにカメラを合わせる
    fn show_product_shapes(&mut self) {
        self.cutting_animation = None;
        self.cutting_animation_job_id = None;
        self.debug_snapshot.clear();
        let meshes = self.product_shapes.visible_meshes();
        let positions: Vec<[f32; 3]> = meshes
            .iter()
            .flat_map(|mesh| mesh.vertices.iter().map(|vertex| vertex.position))
            .collect();
        let items = meshes
            .into_iter()
            .enumerate()
            .map(|(index, mesh)| {
                let item = MeshListItem {
                    vertices: convert_vertex_data_to_mesh_vertices(&mesh.vertices),
                    indices: mesh.indices,
                    color: mesh.color,
                };
                (index as u64, item)
            })
            .collect();

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        mesh_stage.set_mesh_list(&self.graphic.device, items);
        if let Some(fit) = Self::build_camera_fit(&positions) {
            self.apply_camera_fit(fit);
        }
        self.renderer.set_stage(mesh_stage);
        self.update_camera_uniforms();
    }
}
