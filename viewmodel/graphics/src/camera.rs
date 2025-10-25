use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};
use std::f32::consts::PI;

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
        }
    }

    /// ビュー行列を計算
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        // 回転行列を取得
        let rotation_matrix = quaternion_to_matrix(&self.rotation);

        // カメラの位置を計算（ターゲット - 距離 * 回転後のZ軸）
        // Z軸の負方向（カメラから見て奥）にターゲットが見えるように
        let forward = Vec3f::new(
            -rotation_matrix[2][0], // Z軸を反転
            -rotation_matrix[2][1],
            -rotation_matrix[2][2],
        );

        let camera_pos = self.target + forward * self.distance;

        // ビュー行列を計算
        look_at(camera_pos, self.target, Vec3f::new(0.0, 1.0, 0.0))
    }

    /// プロジェクション行列を計算
    pub fn projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        match self.projection_mode {
            ProjectionMode::Perspective => {
                // 距離に応じて適切なnear/farを設定
                let near = (self.distance * 0.01).max(0.001); // 距離の1%、最小0.001
                let far = (self.distance * 100.0).min(1000.0); // 距離の100倍、最大1000

                perspective(45.0 * PI / 180.0, aspect, near, far)
            }
            ProjectionMode::Orthographic => {
                // 平行投影：距離とズームに基づいてサイズを決定
                let size = self.distance * self.zoom;
                let left = -size * aspect * 0.5;
                let right = size * aspect * 0.5;
                let bottom = -size * 0.5;
                let top = size * 0.5;
                let near = -1000.0; // 平行投影では大きな範囲を使用
                let far = 1000.0;

                orthographic(left, right, bottom, top, near, far)
            }
        }
    }

    /// 投影モードを切り替え
    pub fn set_projection_mode(&mut self, mode: ProjectionMode) {
        self.projection_mode = mode;
        tracing::info!("投影モード変更: {:?}", mode);
    }

    /// マウス操作による回転（analysisクレートのクォータニオンを使用）
    pub fn rotate(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.01;

        // Y軸回転（水平方向のマウス移動）
        let y_axis = Vec3f::new(0.0, 1.0, 0.0);
        let y_rotation = Quaternionf::from_axis_angle(&y_axis, -delta_x * sensitivity);

        // X軸回転（垂直方向のマウス移動）
        let x_axis = Vec3f::new(1.0, 0.0, 0.0);
        let x_rotation = Quaternionf::from_axis_angle(&x_axis, -delta_y * sensitivity);

        // 回転を合成（analysisクレートのクォータニオン乗算を使用）
        self.rotation = (y_rotation * self.rotation * x_rotation)
            .normalize()
            .unwrap_or(self.rotation);
    }

    /// パン操作（移動）- マウス座標系→カメラ座標系→ワールド座標系変換
    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.01;

        // 現在のカメラ回転からカメラ座標系の軸ベクトルを計算
        let rotation_matrix = quaternion_to_matrix(&self.rotation);
        let right = Vec3f::new(
            rotation_matrix[0][0],
            rotation_matrix[0][1],
            rotation_matrix[0][2],
        );
        let up = Vec3f::new(
            rotation_matrix[1][0],
            rotation_matrix[1][1],
            rotation_matrix[1][2],
        );

        // マウス移動量をカメラ座標系での移動量に変換
        // マウス右移動 = カメラ右軸方向、マウス上移動 = カメラ上軸方向
        let move_distance = sensitivity * self.distance;
        let offset = right * (-delta_x * move_distance) + up * (-delta_y * move_distance);

        // ワールド座標系でターゲット位置を更新
        self.target = self.target + offset;
    }

    /// ズーム操作（距離調整）
    pub fn zoom(&mut self, delta_x: f32, delta_y: f32) {
        let sensitivity = 0.01; // 感度を上げて分かりやすく

        // 斜め移動の合計でズーム量を計算
        let zoom_factor = (delta_x + delta_y) * sensitivity;

        // 距離を調整（最小・最大制限付き）- より広い範囲に拡大
        let new_distance = self.distance * (1.0 + zoom_factor);
        self.distance = new_distance.clamp(0.1, 200.0); // 最大距離を200に拡大

        tracing::debug!(
            "ズーム: delta=({:.2},{:.2}), factor={:.3}, distance={:.2}",
            delta_x,
            delta_y,
            zoom_factor,
            self.distance
        );
    }

    /// メッシュの境界ボックスに基づいてカメラを自動調整
    pub fn fit_to_mesh(&mut self, min_bounds: Vec3f, max_bounds: Vec3f) {
        // メッシュの中心を計算
        let center = (min_bounds + max_bounds) * 0.5;
        self.target = center;

        // メッシュのサイズを計算
        let size = max_bounds - min_bounds;

        // メッシュの最大サイズを計算（対角線長さ）
        let diagonal = (size.x().powi(2) + size.y().powi(2) + size.z().powi(2)).sqrt();

        // 視野角45度、適切なマージンを考慮してカメラ距離を計算
        // distance = (diagonal * margin) / (2 * tan(fov/2))
        let fov_rad = 45.0_f32.to_radians();
        let margin = 2.5; // より余裕を持ったマージン
        let distance = (diagonal * margin) / (2.0 * (fov_rad / 2.0).tan());

        // 最小距離をdiagonalの0.1倍に設定（非常に小さなオブジェクト対応）
        let min_distance = (diagonal * 0.1).max(0.01); // 最低0.01単位
        let max_distance = 100.0; // 最大距離
        self.distance = distance.clamp(min_distance, max_distance);

        // アイソメトリック風の俯瞰視点を設定（オブジェクトが確実に見える角度）
        // X軸回転: 約30度上から見下ろす
        let x_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -30.0_f32.to_radians());
        // Y軸回転: 約45度斜めから
        let y_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 45.0_f32.to_radians());
        // 正しい順序で回転を合成（Y軸回転後にX軸回転）
        self.rotation = (y_rotation * x_rotation)
            .normalize()
            .unwrap_or(Quaternionf::identity());

        tracing::info!(
            "カメラをメッシュに適応: center={:?}, distance={:.2}, diagonal={:.2}, min_distance={:.4}",
            [center.x(), center.y(), center.z()],
            self.distance,
            diagonal,
            min_distance
        );
    }

    /// 小さなオブジェクト（マイクロメートル〜ミリメートル）専用のカメラ設定
    pub fn fit_to_small_mesh(&mut self, min_bounds: Vec3f, max_bounds: Vec3f) {
        // メッシュの中心を計算
        let center = (min_bounds + max_bounds) * 0.5;
        self.target = center;

        // メッシュのサイズを計算
        let size = max_bounds - min_bounds;
        let diagonal = (size.x().powi(2) + size.y().powi(2) + size.z().powi(2)).sqrt();

        // 小さなオブジェクト専用：対角線の5〜10倍の距離に設定
        let distance_factor = if diagonal < 0.01 {
            15.0 // 非常に小さい場合
        } else if diagonal < 0.1 {
            10.0 // 小さい場合
        } else {
            5.0 // 通常の小オブジェクト
        };

        self.distance = diagonal * distance_factor;

        // 真俯瞰に近い角度でオブジェクトを確実に捉える
        let x_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -60.0_f32.to_radians());
        let y_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 30.0_f32.to_radians());
        self.rotation = (y_rotation * x_rotation)
            .normalize()
            .unwrap_or(Quaternionf::identity());

        tracing::info!(
            "小オブジェクト対応カメラ設定: center={:?}, distance={:.4}, diagonal={:.4}, factor={}",
            [center.x(), center.y(), center.z()],
            self.distance,
            diagonal,
            distance_factor
        );
    }

    /// カメラ状態をリセット
    pub fn reset(&mut self) {
        self.target = Vec3f::new(0.0, 0.0, 0.0);
        self.distance = 5.0;
        self.rotation = Quaternionf::identity();
        self.zoom = 1.0;
        tracing::info!("カメラをリセット");
    }

    /// 標準CAD視点にリセット（固定の適切な距離と角度）
    pub fn reset_to_standard_cad_view(&mut self) {
        self.target = Vec3f::new(0.0, 0.0, 0.0);
        self.distance = 10.0; // より遠い距離で確実に見える
        self.zoom = 1.0;

        // 斜め上からの標準CAD視点（35°/45°）
        let x_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -35.0_f32.to_radians());
        let y_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 45.0_f32.to_radians());
        self.rotation = (y_rotation * x_rotation)
            .normalize()
            .unwrap_or(Quaternionf::identity());

        tracing::info!(
            "カメラを標準CAD視点にリセット（距離: {:.1}）",
            self.distance
        );
    }

    /// 正面視点にリセット（デバッグ用・確実に見える）
    pub fn reset_to_front_view(&mut self) {
        self.target = Vec3f::new(0.0, 0.0, 0.0);
        self.distance = 15.0; // さらに遠い距離
        self.zoom = 1.0;
        self.rotation = Quaternionf::identity(); // 回転なし、正面から

        tracing::info!(
            "カメラを正面視点にリセット（距離: {:.1}、1単位立方体が確実に見える位置）",
            self.distance
        );
    }

    /// メッシュ表示用の安全な初期位置にリセット
    pub fn reset_to_safe_view(&mut self, min_bounds: Vec3f, max_bounds: Vec3f) {
        // メッシュの中心とサイズを計算
        let center = (min_bounds + max_bounds) * 0.5;
        let size = max_bounds - min_bounds;
        let diagonal = (size.x().powi(2) + size.y().powi(2) + size.z().powi(2)).sqrt();

        // 安全な距離（対角線の3倍）
        let safe_distance = (diagonal * 3.0).max(2.0);

        self.target = center;
        self.distance = safe_distance;
        self.zoom = 1.0;

        // 斜め上からの標準視点
        let x_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -30.0_f32.to_radians());
        let y_rotation =
            Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 45.0_f32.to_radians());
        self.rotation = (y_rotation * x_rotation)
            .normalize()
            .unwrap_or(Quaternionf::identity());

        tracing::info!(
            "安全な視点にリセット: center={:?}, distance={:.2}",
            [center.x(), center.y(), center.z()],
            safe_distance
        );
    }

    /// 強制的に最小距離を確保（緊急脱出用）
    pub fn ensure_minimum_distance(&mut self) {
        const MIN_SAFE_DISTANCE: f32 = 1.0;
        if self.distance < MIN_SAFE_DISTANCE {
            self.distance = MIN_SAFE_DISTANCE;
            tracing::warn!("最小距離を強制適用: {:.2}", MIN_SAFE_DISTANCE);
        }
    }

    /// 緊急時のカメラ脱出（めり込み状態から強制回復）
    pub fn emergency_camera_escape(&mut self) {
        self.target = Vec3f::new(0.0, 0.0, 0.0);
        self.distance = 20.0; // 最も遠い距離
        self.zoom = 1.0;
        self.rotation = Quaternionf::identity(); // 正面視点で確実

        tracing::warn!("緊急カメラ脱出実行（距離: {:.1}、正面視点）", self.distance);
    }

    /// カメラの現在状態をログ出力（デバッグ用）
    pub fn log_state(&self) {
        tracing::info!(
            "カメラ状態 - target: {:?}, distance: {:.2}, rotation: [{:.3}, {:.3}, {:.3}, {:.3}]",
            [self.target.x(), self.target.y(), self.target.z()],
            self.distance,
            self.rotation.x(),
            self.rotation.y(),
            self.rotation.z(),
            self.rotation.w()
        );
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
        })
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

/// analysisクレートのクォータニオンを4x4行列に変換
fn quaternion_to_matrix(q: &Quaternionf) -> [[f32; 4]; 4] {
    // 正規化されたクォータニオンを使用
    let normalized = q.normalize().unwrap_or(*q);

    let x = normalized.x();
    let y = normalized.y();
    let z = normalized.z();
    let w = normalized.w();

    let xx = x * x;
    let yy = y * y;
    let zz = z * z;
    let xy = x * y;
    let xz = x * z;
    let yz = y * z;
    let wx = w * x;
    let wy = w * y;
    let wz = w * z;

    [
        [1.0 - 2.0 * (yy + zz), 2.0 * (xy - wz), 2.0 * (xz + wy), 0.0],
        [2.0 * (xy + wz), 1.0 - 2.0 * (xx + zz), 2.0 * (yz - wx), 0.0],
        [2.0 * (xz - wy), 2.0 * (yz + wx), 1.0 - 2.0 * (xx + yy), 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/// Look-at ビュー行列を作成（analysisクレートのベクトルを使用）
fn look_at(eye: Vec3f, center: Vec3f, up: Vec3f) -> [[f32; 4]; 4] {
    let forward = (center - eye)
        .normalize()
        .unwrap_or(Vec3f::new(0.0, 0.0, -1.0));
    let up_normalized = up.normalize().unwrap_or(Vec3f::new(0.0, 1.0, 0.0));
    let right = forward
        .cross(&up_normalized)
        .normalize()
        .unwrap_or(Vec3f::new(1.0, 0.0, 0.0));
    let up_final = right.cross(&forward);

    [
        [right.x(), up_final.x(), -forward.x(), 0.0],
        [right.y(), up_final.y(), -forward.y(), 0.0],
        [right.z(), up_final.z(), -forward.z(), 0.0],
        [
            -right.dot(&eye),
            -up_final.dot(&eye),
            forward.dot(&eye),
            1.0,
        ],
    ]
}

/// 透視投影行列を作成
fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
    let f = 1.0 / (fovy / 2.0).tan();

    [
        [f / aspect, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (far + near) / (near - far), -1.0],
        [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0],
    ]
}

/// 平行投影行列を作成
fn orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> [[f32; 4]; 4] {
    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, -2.0 / (far - near), 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            -(far + near) / (far - near),
            1.0,
        ],
    ]
}

/// Vector3の線形補間
fn lerp_vector3(a: Vec3f, b: Vec3f, t: f32) -> Vec3f {
    a * (1.0 - t) + b * t
}

/// f32の線形補間
fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
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
        let matrix = quaternion_to_matrix(&q);

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
}
