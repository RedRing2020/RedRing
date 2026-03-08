use crate::camera::Camera;
use crate::camera_math::{lerp_f32, lerp_vector3};

/// 2つのカメラ状態を補間し、中間状態を生成する。
pub(crate) fn slerp_to(current: &Camera, target_camera: &Camera, t: f32) -> Result<Camera, String> {
    let interpolated_rotation = current.rotation.slerp(&target_camera.rotation, t)?;
    let interpolated_target = lerp_vector3(current.target, target_camera.target, t);
    let interpolated_distance = lerp_f32(current.distance, target_camera.distance, t);

    Ok(Camera {
        position: current.position,
        rotation: interpolated_rotation,
        zoom: lerp_f32(current.zoom, target_camera.zoom, t),
        target: interpolated_target,
        distance: interpolated_distance,
        projection_mode: current.projection_mode,
        orthographic_bounds: current.orthographic_bounds,
        control_sensitivity: current.control_sensitivity,
    })
}
