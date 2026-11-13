//! AppState のデバッグ用の形状（線分・円・円弧・三角形のエンティティ）の表示を扱うモジュール。

use super::super::AppState;
use render::vertex_3d::MeshVertex;
use render::vertex_3d::convert_vertex_data_to_mesh_vertices;
use stage::MeshStage;
use viewmodel::feature_command_converter::{
    create_debug_arc_vertices, create_debug_circle_vertices, create_debug_line_vertices,
    create_debug_triangle_vertices,
};

impl AppState {
    fn apply_mesh_stage(
        &mut self,
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        reset_camera: bool,
    ) {
        self.enter_debug_view();
        if reset_camera {
            self.camera.reset_to_standard_cad_view();
        }

        let mut mesh_stage = Box::new(MeshStage::new(
            &self.graphic.device,
            self.graphic.config.format,
        ));
        let meshes = mesh_stage.set_mesh_data(&self.graphic.device, &vertices, &indices);
        self.report_not_drawn(meshes, crate::help_ui::Notice::ShapesNotDrawn);
        self.show_debug_stage(mesh_stage, false);
    }

    /// デバッグ用：LineSegment3Dを表示
    /// EntityManager経由の管理対象として扱う。
    pub fn load_debug_line(&mut self) {
        tracing::info!("Debug shape: LineSegment3D (through the application)");

        let Ok(vertex_data) = create_debug_line_vertices() else {
            tracing::error!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "debug line feature command failed"
            );
            return;
        };

        let vertices = convert_vertex_data_to_mesh_vertices(&vertex_data);

        let id = self.entity_manager.add_line_list_entity(vertices);
        let _ = self.entity_manager.select(id);

        self.enter_debug_view();
        self.camera.reset_to_standard_cad_view();
        self.rebuild_stage_from_entities();
    }

    /// デバッグ用：Circle3Dを表示
    /// EntityManager経由の管理対象として扱う。
    pub fn load_debug_circle(&mut self) {
        tracing::info!("Debug shape: Circle3D (through feature/entity)");

        let Ok(vertex_data) = create_debug_circle_vertices() else {
            tracing::error!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "debug circle feature command failed"
            );
            return;
        };

        let vertices = convert_vertex_data_to_mesh_vertices(&vertex_data);
        let id = self.entity_manager.add_line_list_entity(vertices);
        let _ = self.entity_manager.select(id);

        self.enter_debug_view();
        self.camera.reset_to_standard_cad_view();
        self.rebuild_stage_from_entities();
    }

    /// デバッグ用：Triangle3Dを表示
    /// 直接MeshStage経路で扱う。
    pub fn load_debug_triangle(&mut self) {
        tracing::info!("Debug shape: Triangle3D (through the application)");

        let Ok(vertex_data) = create_debug_triangle_vertices() else {
            tracing::error!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "debug triangle geometry command failed"
            );
            return;
        };

        let vertices = convert_vertex_data_to_mesh_vertices(&vertex_data);
        let indices: Vec<u32> = vec![0, 1, 2];
        self.apply_mesh_stage(vertices, indices, true);
    }

    /// デバッグ用：Arc3Dを表示
    /// EntityManager経由の管理対象として扱う。
    pub fn load_debug_arc(&mut self) {
        tracing::info!("Debug shape: Arc3D (through feature/entity)");

        let Ok(vertex_data) = create_debug_arc_vertices() else {
            tracing::error!(
                error_kind = logging_foundation::ERROR_KIND_APP,
                "debug arc feature command failed"
            );
            return;
        };

        let vertices = convert_vertex_data_to_mesh_vertices(&vertex_data);
        let id = self.entity_manager.add_line_list_entity(vertices);
        let _ = self.entity_manager.select(id);

        self.enter_debug_view();
        self.camera.reset_to_standard_cad_view();
        self.rebuild_stage_from_entities();
    }
}
