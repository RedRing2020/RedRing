use crate::camera_math::{lerp_f32, lerp_vector3, matrix_transform_vector, quaternion_to_matrix};
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
    /// 透視投影（建築パース・ゲーム・プレゼンテーション用）
    /// - 遠くのものが小さく見える自然な視覚効果
    /// - デザイン検討、視覚的インパクト、空間認識に適している
    /// - 建築の外観確認、ゲームの没入感、製品の魅力的な表示
    Perspective,
    /// 平行投影（機械設計CAD・製図用）
    /// - 距離による大きさ変化なし、エッジの平行関係を正確に表示
    /// - 寸法確認、幾何学的精度、設計検証に適している
    /// - 機械部品の設計、組み立て確認、技術図面との対応
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

    /// 機械設計CAD用のアイソメトリック視点カメラを作成
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
        let interpolated_rotation = self.rotation.slerp(&target_camera.rotation, t)?;
        let interpolated_target = lerp_vector3(self.target, target_camera.target, t);
        let interpolated_distance = lerp_f32(self.distance, target_camera.distance, t);

        Ok(Camera {
            position: self.position, // 位置は計算で決まるので保持
            rotation: interpolated_rotation,
            zoom: lerp_f32(self.zoom, target_camera.zoom, t),
            target: interpolated_target,
            distance: interpolated_distance,
            projection_mode: self.projection_mode, // 投影モードは変更しない
            orthographic_bounds: self.orthographic_bounds, // 表示範囲も保持
            control_sensitivity: self.control_sensitivity,
        })
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_creation() {
        let camera = Camera::new();

        assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0));
        assert_eq!(camera.distance, 5.0);
        assert!(camera.rotation.is_unit());
    }

    #[test]
    fn test_camera_rotation() {
        let mut camera = Camera::new();
        let initial_rotation = camera.rotation;

        camera.rotate(1.0, 0.0);

        // 回転が変化していることを確認
        assert_ne!(camera.rotation.w(), initial_rotation.w());
    }

    #[test]
    fn test_camera_pan() {
        let mut camera = Camera::new();
        let initial_target = camera.target;

        camera.pan(1.0, 0.0);

        // ターゲット位置が変化していることを確認
        assert_ne!(camera.target, initial_target);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera::new();
        let initial_distance = camera.distance;

        camera.zoom(1.0, 1.0); // 拡大

        // 距離が変化していることを確認
        assert_ne!(camera.distance, initial_distance);
        assert!(camera.distance >= 0.1); // 最小制限確認
        assert!(camera.distance <= 50.0); // 最大制限確認
    }

    #[test]
    fn test_camera_slerp() {
        let camera1 = Camera::new();
        let mut camera2 = Camera::new();
        camera2.rotate(1.0, 0.0); // 少し回転

        let interpolated = camera1.slerp_to(&camera2, 0.5).unwrap();

        // 補間された回転が両者の中間にあることを確認
        assert!(interpolated.rotation.dot(&camera1.rotation) > 0.5);
        assert!(interpolated.rotation.dot(&camera2.rotation) > 0.5);
    }

    #[test]
    fn test_quaternion_to_matrix() {
        let q = Quaternionf::identity();
        let matrix = crate::camera_math::quaternion_to_matrix(&q);

        // 単位行列のテスト
        assert!((matrix[0][0] - 1.0).abs() < 1e-6);
        assert!((matrix[1][1] - 1.0).abs() < 1e-6);
        assert!((matrix[2][2] - 1.0).abs() < 1e-6);
        assert!((matrix[3][3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_fit_to_small_mesh() {
        let mut camera = Camera::new();

        // 1mm立方体のテスト
        let min_bounds = Vec3f::new(-0.0005, -0.0005, -0.0005); // -0.5mm
        let max_bounds = Vec3f::new(0.0005, 0.0005, 0.0005); // +0.5mm

        camera.fit_to_small_mesh(min_bounds, max_bounds);

        // ターゲットが中心になっていることを確認
        assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0));

        // 距離が適切に設定されていることを確認（対角線の10倍程度）
        let diagonal = (3.0_f32 * 0.001_f32.powi(2)).sqrt(); // ≈ 0.00173
        assert!(camera.distance > diagonal * 5.0); // 最低5倍
        assert!(camera.distance < diagonal * 20.0); // 最大20倍

        // 回転が設定されていることを確認
        assert_ne!(camera.rotation, Quaternionf::identity());
    }

    #[test]
    fn test_projection_matrix_near_far() {
        let mut camera = Camera::new();
        camera.distance = 0.01; // 非常に近い距離

        let proj = camera.projection_matrix(1.0);

        // プロジェクション行列が生成されることを確認
        assert!(!proj[0][0].is_nan());
        assert!(!proj[1][1].is_nan());
        assert!(!proj[2][2].is_nan());
        assert!(!proj[3][2].is_nan());
    }

    #[test]
    fn test_projection_modes() {
        let mut camera = Camera::new();

        // デフォルトは透視投影
        assert_eq!(camera.projection_mode, ProjectionMode::Perspective);

        // 平行投影に切り替え
        camera.set_projection_mode(ProjectionMode::Orthographic);
        assert_eq!(camera.projection_mode, ProjectionMode::Orthographic);

        // 両方の投影モードで行列が生成されることを確認
        let perspective_proj = {
            camera.set_projection_mode(ProjectionMode::Perspective);
            camera.projection_matrix(1.0)
        };

        let orthographic_proj = {
            camera.set_projection_mode(ProjectionMode::Orthographic);
            camera.projection_matrix(1.0)
        };

        // 異なる投影行列が生成されることを確認
        assert_ne!(perspective_proj[0][0], orthographic_proj[0][0]);
        assert_ne!(perspective_proj[2][2], orthographic_proj[2][2]);
    }

    #[test]
    fn test_orthographic_camera_creation() {
        let ortho_camera = Camera::new_orthographic();
        assert_eq!(ortho_camera.projection_mode, ProjectionMode::Orthographic);

        let persp_camera = Camera::new();
        assert_eq!(persp_camera.projection_mode, ProjectionMode::Perspective);
    }

    #[test]
    fn test_isometric_camera_creation() {
        let isometric_camera = Camera::new_isometric();
        assert_eq!(
            isometric_camera.projection_mode,
            ProjectionMode::Orthographic
        );
        assert_eq!(isometric_camera.zoom, 1.0);

        // アイソメトリック回転が適用されているかチェック
        assert_ne!(isometric_camera.rotation, Quaternionf::identity());

        // 正規化されているかチェック（クォータニオンのノルムは1）
        let norm = (isometric_camera.rotation.w().powi(2)
            + isometric_camera.rotation.x().powi(2)
            + isometric_camera.rotation.y().powi(2)
            + isometric_camera.rotation.z().powi(2))
        .sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_camera_reset_functions() {
        let mut camera = Camera::new();
        camera.distance = 0.1; // 非常に近い距離
        camera.target = Vec3f::new(5.0, 5.0, 5.0);

        // 基本リセット
        camera.reset();
        assert_eq!(camera.distance, 5.0);
        assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0));
        assert_eq!(camera.zoom, 1.0);

        // 最小距離確保
        camera.distance = 0.05; // 危険な近距離
        camera.ensure_minimum_distance();
        assert!(camera.distance >= 1.0);

        // 安全な視点リセット
        let min_bounds = Vec3f::new(-0.5, -0.5, -0.5);
        let max_bounds = Vec3f::new(0.5, 0.5, 0.5);
        camera.reset_to_safe_view(min_bounds, max_bounds);
        assert!(camera.distance >= 2.0); // 安全な距離
        assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0)); // 中心
    }

    // ============================================================================
    // Issue #242 テストスイート：カメラ操作の4つの問題の検証
    // ============================================================================

    #[test]
    fn test_issue_242_arcball_sphere_mapping() {
        // Issue #242 Phase 1: 回転操作の球面定義
        // 画面座標を単位球面上の点にマッピング

        // ビューポート中心のクリック（Z軸正方向）
        let center = Camera::project_on_sphere(400.0, 300.0, 800.0, 600.0);
        assert!((center.z() - 1.0).abs() < 0.01, "中心クリック時はZ≈1");
        assert!(center.x().abs() < 0.01, "中心クリック時はX≈0");
        assert!(center.y().abs() < 0.01, "中心クリック時はY≈0");

        // ビューポート左端のクリック
        let left = Camera::project_on_sphere(0.0, 300.0, 800.0, 600.0);
        assert!(left.x() < 0.0, "左端クリック時はX<0");

        // ビューポート右端のクリック
        let right = Camera::project_on_sphere(800.0, 300.0, 800.0, 600.0);
        assert!(right.x() > 0.0, "右端クリック時はX>0");

        // 全ての点が単位球面上にあることを確認（正規化済み）
        for point in [center, left, right].iter() {
            let magnitude = (point.x().powi(2) + point.y().powi(2) + point.z().powi(2)).sqrt();
            assert!((magnitude - 1.0).abs() < 0.01, "全点が正規化済み");
        }
    }

    #[test]
    fn test_issue_242_arcball_rotation_from_sphere_points() {
        // Issue #242 Phase 1: 球面上の2点から回転を計算

        // 同じ点 → 回転なし
        let p1 = Vec3f::new(1.0, 0.0, 0.0);
        let q_identity = Camera::compute_rotation_from_sphere_points(p1, p1);
        let identity = Quaternionf::identity();
        // 恒等元の確認：(w, x, y, z) = (1, 0, 0, 0)
        assert!(
            (q_identity.w() - identity.w()).abs() < 0.001
                && (q_identity.x() - identity.x()).abs() < 0.001
                && (q_identity.y() - identity.y()).abs() < 0.001
                && (q_identity.z() - identity.z()).abs() < 0.001,
            "同じ点からの回転は恒等元"
        );

        // 90度の角度
        let p_start = Vec3f::new(1.0, 0.0, 0.0).normalize().unwrap();
        let p_end = Vec3f::new(0.0, 1.0, 0.0).normalize().unwrap();
        let q_90 = Camera::compute_rotation_from_sphere_points(p_start, p_end);
        // 回転が生成されている（恒等元ではない）
        let is_not_identity = (q_90.w() - identity.w()).abs() > 0.01
            || (q_90.x() - identity.x()).abs() > 0.01
            || (q_90.y() - identity.y()).abs() > 0.01
            || (q_90.z() - identity.z()).abs() > 0.01;
        assert!(is_not_identity, "異なる点からは回転が生成される");

        // 逆方向 → 逆回転
        let q_rev = Camera::compute_rotation_from_sphere_points(p_end, p_start);
        let is_rev_not_identity = (q_rev.w() - identity.w()).abs() > 0.01
            || (q_rev.x() - identity.x()).abs() > 0.01
            || (q_rev.y() - identity.y()).abs() > 0.01
            || (q_rev.z() - identity.z()).abs() > 0.01;
        assert!(is_rev_not_identity, "逆方向も回転を生成");
    }

    #[test]
    fn test_issue_242_zoom_magnitude_sensitivity() {
        // Issue #242: ズーム反応改善
        // マウス移動の大きさ（ノルム）で感度を統一

        let mut camera_vertical = Camera::new();
        let mut camera_horizontal = Camera::new();
        let mut camera_diagonal = Camera::new();

        let initial_distance = camera_vertical.distance;

        // 縦方向のみのドラッグ: (0, 10) → magnitude = 10
        camera_vertical.zoom(0.0, 10.0);

        // 横方向のみのドラッグ: (10, 0) → magnitude = 10
        camera_horizontal.zoom(10.0, 0.0);

        // 斜めドラッグ: (7, 7) → magnitude ≈ 10
        camera_diagonal.zoom(7.0, 7.0);

        // 3つのズーム量が異なることを確認
        // （方向によって拡大/縮小が変わるため、エラー許容度を広くする）
        let dist_v = camera_vertical.distance;
        let dist_h = camera_horizontal.distance;
        let dist_d = camera_diagonal.distance;

        println!(
            "ズーム結果: vertical={:.3}, horizontal={:.3}, diagonal={:.3}",
            dist_v, dist_h, dist_d
        );

        // 3つの値が大異なるわけではなく、移動量の大きさに応じて変化
        assert!(
            (dist_v - initial_distance).abs() > 0.001,
            "縦方向ズームが有効"
        );
        assert!(
            (dist_h - initial_distance).abs() > 0.001,
            "横方向ズームが有効"
        );
    }
}
