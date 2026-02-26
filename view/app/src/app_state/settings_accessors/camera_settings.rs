use crate::app_state::{AppState, ViewingOperationSettings};
use viewmodel_graphics::CameraControlSensitivity;

impl AppState {
    /// カメラ操作感度を設定
    pub fn set_camera_control_sensitivity(&mut self, sensitivity: CameraControlSensitivity) {
        self.viewing_operation_settings.camera_control_sensitivity = sensitivity;
        self.apply_viewing_operation_settings();
    }

    /// 回転感度を更新
    pub fn set_camera_rotate_sensitivity(&mut self, rotate: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .rotate = rotate;
        self.apply_viewing_operation_settings();
    }

    /// Arcballデルタ変換スケールを更新
    pub fn set_camera_arcball_sensitivity(&mut self, arcball: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .arcball = arcball;
        self.apply_viewing_operation_settings();
    }

    /// パン感度を更新
    pub fn set_camera_pan_sensitivity(&mut self, pan: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .pan = pan;
        self.apply_viewing_operation_settings();
    }

    /// ドラッグズーム感度を更新
    pub fn set_camera_zoom_drag_sensitivity(&mut self, zoom_drag: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_drag = zoom_drag;
        self.apply_viewing_operation_settings();
    }

    /// ホイールズーム感度を更新
    pub fn set_camera_zoom_wheel_sensitivity(&mut self, zoom_wheel: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_wheel = zoom_wheel;
        self.apply_viewing_operation_settings();
    }

    /// ピクセルホイール→ライン変換係数を更新
    pub fn set_camera_wheel_pixel_to_line(&mut self, wheel_pixel_to_line: f32) {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .wheel_pixel_to_line = wheel_pixel_to_line;
        self.apply_viewing_operation_settings();
    }

    /// ビュー操作設定を一括設定
    pub fn set_viewing_operation_settings(&mut self, settings: ViewingOperationSettings) {
        self.viewing_operation_settings = settings;
        self.apply_viewing_operation_settings();
    }

    /// ビュー操作設定を取得
    pub fn viewing_operation_settings(&self) -> ViewingOperationSettings {
        self.viewing_operation_settings
    }

    /// カメラ操作感度を取得
    pub fn camera_control_sensitivity(&self) -> CameraControlSensitivity {
        self.viewing_operation_settings.camera_control_sensitivity
    }

    /// 回転感度を取得
    pub fn camera_rotate_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .rotate
    }

    /// Arcballデルタ変換スケールを取得
    pub fn camera_arcball_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .arcball
    }

    /// パン感度を取得
    pub fn camera_pan_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .pan
    }

    /// ドラッグズーム感度を取得
    pub fn camera_zoom_drag_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_drag
    }

    /// ホイールズーム感度を取得
    pub fn camera_zoom_wheel_sensitivity(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .zoom_wheel
    }

    /// ピクセルホイール→ライン変換係数を取得
    pub fn camera_wheel_pixel_to_line(&self) -> f32 {
        self.viewing_operation_settings
            .camera_control_sensitivity
            .wheel_pixel_to_line
    }
}
