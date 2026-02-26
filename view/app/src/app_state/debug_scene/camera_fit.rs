//! AppState のデバッグ表示向けカメラfit処理を扱うモジュール。

use super::AppState;
use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};

#[derive(Clone, Copy)]
pub(super) struct CameraFit {
    pub(super) center_x: f32,
    pub(super) center_y: f32,
    pub(super) center_z: f32,
    pub(super) size_z: f32,
    pub(super) half_extent_xy: f32,
}

impl AppState {
    pub(super) fn build_camera_fit(positions: &[[f32; 3]]) -> Option<CameraFit> {
        if positions.is_empty() {
            return None;
        }

        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut min_z = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for pos in positions {
            min_x = min_x.min(pos[0]);
            min_y = min_y.min(pos[1]);
            min_z = min_z.min(pos[2]);
            max_x = max_x.max(pos[0]);
            max_y = max_y.max(pos[1]);
            max_z = max_z.max(pos[2]);
        }

        let center_x = (min_x + max_x) * 0.5;
        let center_y = (min_y + max_y) * 0.5;
        let center_z = (min_z + max_z) * 0.5;
        let size_x = (max_x - min_x).max(1.0);
        let size_y = (max_y - min_y).max(1.0);
        let size_z = (max_z - min_z).max(1.0);
        let half_extent_xy = (size_x.max(size_y) * 0.5 * 1.4).max(10.0);

        Some(CameraFit {
            center_x,
            center_y,
            center_z,
            size_z,
            half_extent_xy,
        })
    }

    pub(super) fn apply_camera_fit(&mut self, fit: CameraFit) {
        self.camera.target = Vec3f::new(fit.center_x, fit.center_y, fit.center_z);
        self.camera.distance = (fit.size_z * 6.0 + fit.half_extent_xy).max(80.0);
        self.camera.zoom = 1.0;
        self.camera.rotation = Quaternionf::identity();
        self.camera
            .set_projection_mode(viewmodel_graphics::camera::ProjectionMode::Orthographic);
        self.camera.set_orthographic_bounds(
            -fit.half_extent_xy,
            fit.half_extent_xy,
            -fit.half_extent_xy,
            fit.half_extent_xy,
        );
    }
}
