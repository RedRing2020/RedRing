use crate::camera::ProjectionMode;
use crate::camera_math::quaternion_to_matrix;
use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};

/// 画面座標をArcball用の単位球面上の点へ変換する。
pub(crate) fn project_on_sphere(
    screen_x: f32,
    screen_y: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec3f {
    let radius = (viewport_width.min(viewport_height)) * 0.5;
    let cx = screen_x - viewport_width * 0.5;
    let cy = screen_y - viewport_height * 0.5;

    let x = cx / radius;
    let y = cy / radius;

    let d = (x * x + y * y).sqrt();
    let z = if d < 0.70710677 {
        (1.0 - d * d).sqrt()
    } else {
        0.5 / d
    };

    let sphere_point = Vec3f::new(x, -y, z);
    sphere_point
        .normalize()
        .unwrap_or(Vec3f::new(0.0, 0.0, 1.0))
}

/// 球面上の2点から回転クォータニオンを計算する。
pub(crate) fn compute_rotation_from_sphere_points(
    sphere_from: Vec3f,
    sphere_to: Vec3f,
) -> Quaternionf {
    let dot = sphere_from.dot(&sphere_to).clamp(-1.0, 1.0);
    let angle = dot.acos();
    let axis = sphere_from.cross(&sphere_to);
    let axis_magnitude = axis.dot(&axis).sqrt();
    if axis_magnitude < 1e-6 {
        tracing::debug!("球面回転: ベクトルがほぼ平行 (dot={:.4})", dot);
        return Quaternionf::identity();
    }

    let axis_normalized = axis.normalize().unwrap_or(Vec3f::new(0.0, 0.0, 1.0));
    Quaternionf::from_axis_angle(&axis_normalized, angle)
}

/// ドラッグ量に応じてカメラ回転を更新する。
pub(crate) fn rotate(rotation: &mut Quaternionf, delta_x: f32, delta_y: f32, sensitivity: f32) {
    let y_axis = Vec3f::new(0.0, 1.0, 0.0);
    let y_rotation = Quaternionf::from_axis_angle(&y_axis, -delta_x * sensitivity);
    let x_axis = Vec3f::new(1.0, 0.0, 0.0);
    let x_rotation = Quaternionf::from_axis_angle(&x_axis, -delta_y * sensitivity);
    *rotation = (y_rotation * *rotation * x_rotation)
        .normalize()
        .unwrap_or(*rotation);
}

/// Arcballの前後2点から回転を適用する。
pub(crate) fn rotate_arcball(
    rotation: &mut Quaternionf,
    prev_x: f32,
    prev_y: f32,
    curr_x: f32,
    curr_y: f32,
    viewport_width: f32,
    viewport_height: f32,
) {
    let sphere_from = project_on_sphere(curr_x, curr_y, viewport_width, viewport_height);
    let sphere_to = project_on_sphere(prev_x, prev_y, viewport_width, viewport_height);
    let q_rotation = compute_rotation_from_sphere_points(sphere_from, sphere_to);
    *rotation = (q_rotation * *rotation).normalize().unwrap_or(*rotation);

    tracing::debug!(
        "✓ Arcball回転: prev=({:.0},{:.0}) → curr=({:.0},{:.0})",
        prev_x,
        prev_y,
        curr_x,
        curr_y
    );
}

/// Arcballのデルタ入力から回転を適用する。
pub(crate) fn rotate_arcball_from_delta(
    rotation: &mut Quaternionf,
    delta_x: f32,
    delta_y: f32,
    viewport_width: f32,
    viewport_height: f32,
    arcball_scale: f32,
) {
    let center_x = viewport_width * 0.5;
    let center_y = viewport_height * 0.5;

    let curr_x = (center_x + delta_x * arcball_scale).clamp(0.0, viewport_width);
    let curr_y = (center_y + delta_y * arcball_scale).clamp(0.0, viewport_height);

    let sphere_from = project_on_sphere(curr_x, curr_y, viewport_width, viewport_height);
    let sphere_to = project_on_sphere(center_x, center_y, viewport_width, viewport_height);

    let axis = sphere_from.cross(&sphere_to);
    let dot = sphere_from.dot(&sphere_to).clamp(-1.0, 1.0);
    let angle_rad = dot.acos();
    let angle_deg = angle_rad.to_degrees();

    let q_rotation = compute_rotation_from_sphere_points(sphere_from, sphere_to);
    let prev_rotation = *rotation;
    *rotation = (q_rotation * *rotation).normalize().unwrap_or(*rotation);

    tracing::debug!(
        "✓ Arcball回転(Delta): delta=({:.1},{:.1}), center=({:.1},{:.1}), curr=({:.1},{:.1}), sphere_from=({:.3},{:.3},{:.3}), sphere_to=({:.3},{:.3},{:.3}), axis=({:.3},{:.3},{:.3}), angle_deg={:.2}, q=({:.4},{:.4},{:.4},{:.4}), rot_prev=({:.4},{:.4},{:.4},{:.4}), rot_new=({:.4},{:.4},{:.4},{:.4}), viewport=({:.0}x{:.0})",
        delta_x,
        delta_y,
        center_x,
        center_y,
        curr_x,
        curr_y,
        sphere_from.x(),
        sphere_from.y(),
        sphere_from.z(),
        sphere_to.x(),
        sphere_to.y(),
        sphere_to.z(),
        axis.x(),
        axis.y(),
        axis.z(),
        angle_deg,
        q_rotation.w(),
        q_rotation.x(),
        q_rotation.y(),
        q_rotation.z(),
        prev_rotation.w(),
        prev_rotation.x(),
        prev_rotation.y(),
        prev_rotation.z(),
        rotation.w(),
        rotation.x(),
        rotation.y(),
        rotation.z(),
        viewport_width,
        viewport_height
    );
}

/// 画面平面に沿って注視点を移動する。
pub(crate) fn pan(
    target: &mut Vec3f,
    rotation: &Quaternionf,
    distance: f32,
    delta_x: f32,
    delta_y: f32,
    sensitivity: f32,
) {
    let rotation_matrix = quaternion_to_matrix(rotation);
    let right = Vec3f::new(
        rotation_matrix[0][0],
        rotation_matrix[1][0],
        rotation_matrix[2][0],
    );
    let up = Vec3f::new(
        rotation_matrix[0][1],
        rotation_matrix[1][1],
        rotation_matrix[2][1],
    );

    let move_distance = sensitivity * distance;
    let offset = right * (-delta_x * move_distance) + up * (delta_y * move_distance);

    *target = *target + offset;

    tracing::debug!(
        "🎮 パン移動: delta=({:.1}, {:.1}), offset=({:.3}, {:.3}, {:.3}), target_new=({:.1}, {:.1}, {:.1})",
        delta_x,
        delta_y,
        offset.x(),
        offset.y(),
        offset.z(),
        target.x(),
        target.y(),
        target.z()
    );
}

/// 入力デルタに応じて距離または表示範囲を更新する。
pub(crate) fn zoom(
    distance: &mut f32,
    orthographic_bounds: &mut Option<(f32, f32, f32, f32)>,
    projection_mode: ProjectionMode,
    delta_x: f32,
    delta_y: f32,
    sensitivity: f32,
) {
    let dominant_delta = if delta_y.abs() >= delta_x.abs() {
        -delta_y
    } else {
        delta_x
    };

    if dominant_delta.abs() < 1e-6 {
        return;
    }

    let zoom_factor = (-dominant_delta * sensitivity).clamp(-0.8, 0.8);

    if projection_mode == ProjectionMode::Orthographic {
        if let Some((left, right, bottom, top)) = *orthographic_bounds {
            let scale = (1.0 + zoom_factor).clamp(0.1, 10.0);
            let center_x = (left + right) * 0.5;
            let center_y = (bottom + top) * 0.5;
            let half_w = (right - left) * 0.5 * scale;
            let half_h = (top - bottom) * 0.5 * scale;

            *orthographic_bounds = Some((
                center_x - half_w,
                center_x + half_w,
                center_y - half_h,
                center_y + half_h,
            ));
        } else {
            let new_distance = *distance * (1.0 + zoom_factor);
            *distance = new_distance.clamp(0.1, 200.0);
        }
    } else {
        let new_distance = *distance * (1.0 + zoom_factor);
        *distance = new_distance.clamp(0.1, 200.0);
    }

    tracing::debug!(
        "🔍 ズーム: mode={:?}, delta=({:.2},{:.2}), dominant={:.2}, factor={:.4}, distance={:.2}, bounds={:?}",
        projection_mode,
        delta_x,
        delta_y,
        dominant_delta,
        zoom_factor,
        *distance,
        *orthographic_bounds
    );
}

/// ホイール入力をズーム操作へ変換して適用する。
pub(crate) fn zoom_wheel(
    distance: &mut f32,
    orthographic_bounds: &mut Option<(f32, f32, f32, f32)>,
    projection_mode: ProjectionMode,
    wheel_line_delta_y: f32,
    zoom_wheel_sensitivity: f32,
    zoom_drag_sensitivity: f32,
) {
    let scaled = wheel_line_delta_y * zoom_wheel_sensitivity;
    zoom(
        distance,
        orthographic_bounds,
        projection_mode,
        0.0,
        -scaled,
        zoom_drag_sensitivity,
    );
}
