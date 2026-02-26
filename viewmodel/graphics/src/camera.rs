use crate::camera_math::{matrix_transform_vector, quaternion_to_matrix};
use crate::camera_navigation::{
    pan as navigate_pan, project_on_sphere as project_arcball_on_sphere, rotate as navigate_rotate,
    rotate_arcball as navigate_rotate_arcball,
    rotate_arcball_from_delta as navigate_rotate_arcball_from_delta, zoom as navigate_zoom,
    zoom_wheel as navigate_zoom_wheel,
};
use crate::camera_presets::{
    emergency_camera_escape as apply_emergency_camera_escape,
    ensure_minimum_distance as apply_minimum_distance, fit_to_mesh as apply_fit_to_mesh,
    fit_to_small_mesh as apply_fit_to_small_mesh, log_state as emit_camera_log_state,
    reset as apply_reset, reset_to_front_view as apply_reset_to_front_view,
    reset_to_safe_view as apply_reset_to_safe_view,
    reset_to_standard_cad_view as apply_reset_to_standard_cad_view,
};
use crate::camera_projection::projection_matrix as build_projection_matrix;
use crate::camera_transition::slerp_to as apply_camera_slerp_to;
use analysis::linalg::{matrix::Matrix4x4, quaternion::Quaternionf, vector::Vec3f};

/// ビュー行列とプロジェクション行列から、GPU描画用のview-projection行列を生成
///
/// 入力は `to_column_major()` 形式の列優先配列を想定。
/// シェーダーで `clip = view_proj * world` を使う前提で、`proj * view` の順に合成する。
pub fn build_view_projection_matrix(
    view_matrix: [[f32; 4]; 4],
    proj_matrix: [[f32; 4]; 4],
) -> [[f32; 4]; 4] {
    let view = Matrix4x4::from_column_major(view_matrix);
    let proj = Matrix4x4::from_column_major(proj_matrix);
    (proj * view).to_column_major()
}

/// 投影方式の種類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectionMode {
    /// 透視投影（遠近感あり）
    Perspective,
    /// 平行投影（遠近感なし）
    Orthographic,
}

/// カメラの各ビュー操作感度設定
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraControlSensitivity {
    /// 回転（ドラッグ）感度
    pub rotate: f32,
    /// Arcballデルタ変換スケール
    pub arcball: f32,
    /// パン（移動）感度
    pub pan: f32,
    /// ズーム（ドラッグ）感度
    pub zoom_drag: f32,
    /// ズーム（ホイール）感度（ライン単位への倍率）
    pub zoom_wheel: f32,
    /// ピクセルホイールをライン相当へ変換する係数
    pub wheel_pixel_to_line: f32,
}

impl Default for CameraControlSensitivity {
    fn default() -> Self {
        Self {
            rotate: 0.01,
            arcball: 2.0,
            pan: 0.003,
            zoom_drag: 0.04,
            zoom_wheel: 6.0,
            wheel_pixel_to_line: 0.2,
        }
    }
}

/// 3Dカメラの制御システム
/// analysisクレートの高品質なクォータニオンとベクトル実装を使用
#[derive(Debug, Clone)]
pub struct Camera {
    /// カメラの位置
    pub position: Vec3f,
    /// カメラの回転（クォータニオン）
    pub rotation: Quaternionf,
    /// ズーム係数
    pub zoom: f32,
    /// 注視点
    pub target: Vec3f,
    /// カメラからターゲットまでの距離
    pub distance: f32,
    /// 投影方式
    pub projection_mode: ProjectionMode,
    /// 平行投影の明示的な表示範囲 (left, right, bottom, top)
    /// Some が指定されている場合、distance と zoom の代わりにこれを使用
    pub orthographic_bounds: Option<(f32, f32, f32, f32)>,
    /// ビュー操作感度設定
    pub control_sensitivity: CameraControlSensitivity,
}

impl Camera {
    /// 新しいカメラを作成（デフォルトは透視投影）
    pub fn new() -> Self {
        Self {
            position: Vec3f::new(0.0, 0.0, 5.0),
            rotation: Quaternionf::identity(),
            zoom: 1.0,
            target: Vec3f::new(0.0, 0.0, 0.0),
            distance: 5.0,
            projection_mode: ProjectionMode::Perspective,
            orthographic_bounds: None,
            control_sensitivity: CameraControlSensitivity::default(),
        }
    }

    /// 平行投影カメラを作成
    pub fn new_orthographic() -> Self {
        Self {
            position: Vec3f::new(0.0, 0.0, 5.0),
            rotation: Quaternionf::identity(),
            zoom: 1.0,
            target: Vec3f::new(0.0, 0.0, 0.0),
            distance: 5.0,
            projection_mode: ProjectionMode::Orthographic,
            orthographic_bounds: None,
            control_sensitivity: CameraControlSensitivity::default(),
        }
    }

    /// アイソメトリック視点カメラを作成
    /// X, Y, Z軸が等しく短縮される標準的な等角投影視点
    pub fn new_isometric() -> Self {
        // アイソメトリック標準角度: X軸 -35.264°, Y軸 45°
        let x_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -35.264_f32.to_radians());
        let y_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 45.0_f32.to_radians());
        let isometric_rotation = (y_rotation * x_rotation)
            .normalize()
            .unwrap_or(Quaternionf::identity());

        Self {
            position: Vec3f::new(0.0, 0.0, 5.0),
            rotation: isometric_rotation,
            zoom: 1.0,
            target: Vec3f::new(0.0, 0.0, 0.0),
            distance: 5.0,
            projection_mode: ProjectionMode::Orthographic,
            orthographic_bounds: None,
            control_sensitivity: CameraControlSensitivity::default(),
        }
    }

    /// カメラ操作感度を設定
    pub fn set_control_sensitivity(&mut self, sensitivity: CameraControlSensitivity) {
        self.control_sensitivity = sensitivity;
    }

    /// カメラ操作感度を取得
    pub fn control_sensitivity(&self) -> CameraControlSensitivity {
        self.control_sensitivity
    }

    /// ビュー行列を計算
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        // 数値ドリフト対策：毎フレームクォータニオンをチェック・正規化
        // 複数のベクトル回転の代わりに、クォータニオンを行列に変換してから操作
        let normalized_rotation = self.rotation.normalize().unwrap_or(self.rotation);

        // クォータニオンを回転行列に変換（複数ベクトル回転より数値安定性が高い）
        let rotation_matrix = quaternion_to_matrix(&normalized_rotation);

        // カメラの基本方向ベクトル（Z軸の負方向を向く）
        let forward_base = Vec3f::new(0.0, 0.0, -1.0);
        let up_base = Vec3f::new(0.0, 1.0, 0.0);

        // 行列によるベクトル変換（一度の行列乗算で安定性向上）
        let forward = matrix_transform_vector(&rotation_matrix, &forward_base);
        let up = matrix_transform_vector(&rotation_matrix, &up_base);

        // カメラ位置 = target - (カメラが向く方向 × 距離)
        // forwardはカメラが向いている方向なので、カメラ位置はその逆方向に配置
        let camera_pos = self.target - forward * self.distance;

        tracing::debug!(
            "🎥 view_matrix計算: forward=({:.3}, {:.3}, {:.3}), camera_pos=({:.1}, {:.1}, {:.1}), target=({:.1}, {:.1}, {:.1})",
            forward.x(), forward.y(), forward.z(),
            camera_pos.x(), camera_pos.y(), camera_pos.z(),
            self.target.x(), self.target.y(), self.target.z()
        );

        Matrix4x4::look_at(&camera_pos, &self.target, &up)
            .unwrap_or_else(|_| Matrix4x4::identity())
            .to_column_major()
    }

    /// プロジェクション行列を計算
    pub fn projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        build_projection_matrix(
            self.projection_mode,
            self.distance,
            self.zoom,
            self.orthographic_bounds,
            aspect,
        )
    }

    /// 投影モードを切り替え
    pub fn set_projection_mode(&mut self, mode: ProjectionMode) {
        let old_mode = self.projection_mode;
        self.projection_mode = mode;
        tracing::warn!("🔄 投影モード変更: {:?} → {:?}", old_mode, mode);
    }

    /// 直交投影モード時の表示範囲を設定
    pub fn set_orthographic_bounds(&mut self, left: f32, right: f32, bottom: f32, top: f32) {
        self.orthographic_bounds = Some((left, right, bottom, top));
        tracing::info!(
            "📐 直交投影表示範囲設定: ({:.1}, {:.1}, {:.1}, {:.1}) - サイズ: {:.1}×{:.1}",
            left,
            right,
            bottom,
            top,
            right - left,
            top - bottom
        );
    }

    /// 画面座標を Arcball 用の単位球面点へ変換
    pub fn project_on_sphere(
        screen_x: f32,
        screen_y: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Vec3f {
        project_arcball_on_sphere(screen_x, screen_y, viewport_width, viewport_height)
    }

    /// 球面上の2点から回転クォータニオンを計算
    #[cfg(test)]
    fn compute_rotation_from_sphere_points(sphere_from: Vec3f, sphere_to: Vec3f) -> Quaternionf {
        crate::camera_navigation::compute_rotation_from_sphere_points(sphere_from, sphere_to)
    }

    /// マウス操作による回転（analysisクレートのクォータニオンを使用）
    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        navigate_rotate(
            &mut self.rotation,
            delta_x,
            delta_y,
            self.control_sensitivity.rotate,
        );
    }

    /// Arcball回転（前後2点）
    pub fn rotate_arcball(
        &mut self,
        prev_x: f32,
        prev_y: f32,
        curr_x: f32,
        curr_y: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        navigate_rotate_arcball(
            &mut self.rotation,
            prev_x,
            prev_y,
            curr_x,
            curr_y,
            viewport_width,
            viewport_height,
        );
    }

    /// Arcball回転（デルタ入力）
    pub fn rotate_arcball_from_delta(
        &mut self,
        delta_x: f32,
        delta_y: f32,
        viewport_width: f32,
        viewport_height: f32,
    ) {
        navigate_rotate_arcball_from_delta(
            &mut self.rotation,
            delta_x,
            delta_y,
            viewport_width,
            viewport_height,
            self.control_sensitivity.arcball,
        );
    }

    /// パン操作
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        navigate_pan(
            &mut self.target,
            &self.rotation,
            self.distance,
            delta_x,
            delta_y,
            self.control_sensitivity.pan,
        );
    }

    /// ズーム操作（距離調整）
    pub fn zoom(&mut self, delta_x: f32, delta_y: f32) {
        navigate_zoom(
            &mut self.distance,
            &mut self.orthographic_bounds,
            self.projection_mode,
            delta_x,
            delta_y,
            self.control_sensitivity.zoom_drag,
        );
    }

    /// マウスホイールによるズーム
    pub fn zoom_wheel(&mut self, wheel_line_delta_y: f32) {
        navigate_zoom_wheel(
            &mut self.distance,
            &mut self.orthographic_bounds,
            self.projection_mode,
            wheel_line_delta_y,
            self.control_sensitivity.zoom_wheel,
            self.control_sensitivity.zoom_drag,
        );
    }

    /// メッシュの境界ボックスに基づいてカメラを自動調整
    pub fn fit_to_mesh(&mut self, min_bounds: Vec3f, max_bounds: Vec3f) {
        apply_fit_to_mesh(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            min_bounds,
            max_bounds,
        );
    }

    /// 小さなオブジェクト（マイクロメートル〜ミリメートル）専用のカメラ設定
    pub fn fit_to_small_mesh(&mut self, min_bounds: Vec3f, max_bounds: Vec3f) {
        apply_fit_to_small_mesh(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            min_bounds,
            max_bounds,
        );
    }

    /// カメラ状態をリセット
    pub fn reset(&mut self) {
        apply_reset(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            &mut self.zoom,
        );
    }

    /// 標準CAD視点にリセット（固定の適切な距離と角度）
    pub fn reset_to_standard_cad_view(&mut self) {
        apply_reset_to_standard_cad_view(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            &mut self.zoom,
            &mut self.orthographic_bounds,
        );
    }

    /// 正面視点にリセット（デバッグ用・確実に見える）
    pub fn reset_to_front_view(&mut self) {
        apply_reset_to_front_view(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            &mut self.zoom,
        );
    }

    /// メッシュ表示用の安全な初期位置にリセット
    pub fn reset_to_safe_view(&mut self, min_bounds: Vec3f, max_bounds: Vec3f) {
        apply_reset_to_safe_view(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            &mut self.zoom,
            min_bounds,
            max_bounds,
        );
    }

    /// 強制的に最小距離を確保（緊急脱出用）
    pub fn ensure_minimum_distance(&mut self) {
        apply_minimum_distance(&mut self.distance);
    }

    /// 緊急時のカメラ脱出（めり込み状態から強制回復）
    pub fn emergency_camera_escape(&mut self) {
        apply_emergency_camera_escape(
            &mut self.target,
            &mut self.distance,
            &mut self.rotation,
            &mut self.zoom,
        );
    }

    /// カメラの現在状態をログ出力（デバッグ用）
    pub fn log_state(&self) {
        emit_camera_log_state(&self.target, self.distance, &self.rotation);
    }

    /// 球面線形補間による滑らかなカメラ遷移
    pub fn slerp_to(&self, target_camera: &Camera, t: f32) -> Result<Camera, String> {
        apply_camera_slerp_to(self, target_camera, t)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "camera_tests.rs"]
mod tests;
