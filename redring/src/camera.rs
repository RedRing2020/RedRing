use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};
use std::f32::consts::PI;

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
}

impl Camera {
    /// 新しいカメラを作成
    pub fn new() -> Self {
        Self {
            position: Vec3f::new(0.0, 0.0, 5.0),
            rotation: Quaternionf::identity(),
            zoom: 1.0,
            target: Vec3f::new(0.0, 0.0, 0.0),
            distance: 5.0,
        }
    }

    /// ビュー行列を計算
    pub fn view_matrix(&self) -> [[f32; 4]; 4] {
        // 回転行列を取得
        let rotation_matrix = quaternion_to_matrix(&self.rotation);

        // カメラの位置を計算（ターゲット + 距離 * 回転後のZ軸）
        let forward = Vec3f::new(
            rotation_matrix[2][0],
            rotation_matrix[2][1],
            rotation_matrix[2][2],
        );

        let camera_pos = self.target + forward * self.distance;

        // ビュー行列を計算
        look_at(camera_pos, self.target, Vec3f::new(0.0, 1.0, 0.0))
    }

    /// プロジェクション行列を計算
    pub fn projection_matrix(&self, aspect: f32) -> [[f32; 4]; 4] {
        perspective(45.0 * PI / 180.0, aspect, 0.1, 100.0)
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

        // 距離を調整（最小・最大制限付き）
        let new_distance = self.distance * (1.0 + zoom_factor);
        self.distance = new_distance.max(0.1).min(50.0);

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

        // 視野角45度、1.2倍のマージンを考慮してカメラ距離を計算
        // distance = (diagonal * margin) / (2 * tan(fov/2))
        let fov_rad = 45.0_f32.to_radians();
        let margin = 1.2; // 1.2倍のマージン
        let distance = (diagonal * margin) / (2.0 * (fov_rad / 2.0).tan());

        self.distance = distance.max(1.0).min(50.0); // 制限

        // より良い初期視点のために少し斜めから見るように回転を設定
        let x_rotation = Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -0.3); // 少し上から
        let y_rotation = Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 0.5); // 少し右から
        self.rotation = (y_rotation * x_rotation)
            .normalize()
            .unwrap_or(Quaternionf::identity());

        tracing::info!(
            "カメラをメッシュに適応: center={:?}, distance={:.2}, diagonal={:.2}",
            [center.x(), center.y(), center.z()],
            self.distance,
            diagonal
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
}
