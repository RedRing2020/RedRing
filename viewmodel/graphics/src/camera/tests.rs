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

    assert_ne!(camera.rotation.w(), initial_rotation.w());
}

#[test]
fn test_camera_pan() {
    let mut camera = Camera::new();
    let initial_target = camera.target;

    camera.pan(1.0, 0.0);

    assert_ne!(camera.target, initial_target);
}

#[test]
fn test_camera_zoom() {
    let mut camera = Camera::new();
    let initial_distance = camera.distance;

    camera.zoom(1.0, 1.0);

    assert_ne!(camera.distance, initial_distance);
    assert!(camera.distance >= 0.1);
    assert!(camera.distance <= 50.0);
}

#[test]
fn test_camera_slerp() {
    let camera1 = Camera::new();
    let mut camera2 = Camera::new();
    camera2.rotate(1.0, 0.0);

    let interpolated = camera1.slerp_to(&camera2, 0.5).unwrap();

    assert!(interpolated.rotation.dot(&camera1.rotation) > 0.5);
    assert!(interpolated.rotation.dot(&camera2.rotation) > 0.5);
}

#[test]
fn test_quaternion_to_matrix() {
    let q = Quaternionf::identity();
    let matrix = crate::camera_math::quaternion_to_matrix(&q);

    assert!((matrix[0][0] - 1.0).abs() < 1e-6);
    assert!((matrix[1][1] - 1.0).abs() < 1e-6);
    assert!((matrix[2][2] - 1.0).abs() < 1e-6);
    assert!((matrix[3][3] - 1.0).abs() < 1e-6);
}

#[test]
fn test_fit_to_small_mesh() {
    let mut camera = Camera::new();

    let min_bounds = Vec3f::new(-0.0005, -0.0005, -0.0005);
    let max_bounds = Vec3f::new(0.0005, 0.0005, 0.0005);

    camera.fit_to_small_mesh(min_bounds, max_bounds);

    assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0));

    let diagonal = (3.0_f32 * 0.001_f32.powi(2)).sqrt();
    assert!(camera.distance > diagonal * 5.0);
    assert!(camera.distance < diagonal * 20.0);

    assert_ne!(camera.rotation, Quaternionf::identity());
}

#[test]
fn test_projection_matrix_near_far() {
    let mut camera = Camera::new();
    camera.distance = 0.01;

    let proj = camera.projection_matrix(1.0);

    assert!(!proj[0][0].is_nan());
    assert!(!proj[1][1].is_nan());
    assert!(!proj[2][2].is_nan());
    assert!(!proj[3][2].is_nan());
}

#[test]
fn test_projection_modes() {
    let mut camera = Camera::new();

    assert_eq!(camera.projection_mode, ProjectionMode::Perspective);

    camera.set_projection_mode(ProjectionMode::Orthographic);
    assert_eq!(camera.projection_mode, ProjectionMode::Orthographic);

    let perspective_proj = {
        camera.set_projection_mode(ProjectionMode::Perspective);
        camera.projection_matrix(1.0)
    };

    let orthographic_proj = {
        camera.set_projection_mode(ProjectionMode::Orthographic);
        camera.projection_matrix(1.0)
    };

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

    assert_ne!(isometric_camera.rotation, Quaternionf::identity());

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
    camera.distance = 0.1;
    camera.target = Vec3f::new(5.0, 5.0, 5.0);

    camera.reset();
    assert_eq!(camera.distance, 5.0);
    assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0));
    assert_eq!(camera.zoom, 1.0);

    camera.distance = 0.05;
    camera.ensure_minimum_distance();
    assert!(camera.distance >= 1.0);

    let min_bounds = Vec3f::new(-0.5, -0.5, -0.5);
    let max_bounds = Vec3f::new(0.5, 0.5, 0.5);
    camera.reset_to_safe_view(min_bounds, max_bounds);
    assert!(camera.distance >= 2.0);
    assert_eq!(camera.target, Vec3f::new(0.0, 0.0, 0.0));
}

#[test]
fn test_issue_242_arcball_sphere_mapping() {
    let center = Camera::project_on_sphere(400.0, 300.0, 800.0, 600.0);
    assert!((center.z() - 1.0).abs() < 0.01, "中心クリック時はZ≈1");
    assert!(center.x().abs() < 0.01, "中心クリック時はX≈0");
    assert!(center.y().abs() < 0.01, "中心クリック時はY≈0");

    let left = Camera::project_on_sphere(0.0, 300.0, 800.0, 600.0);
    assert!(left.x() < 0.0, "左端クリック時はX<0");

    let right = Camera::project_on_sphere(800.0, 300.0, 800.0, 600.0);
    assert!(right.x() > 0.0, "右端クリック時はX>0");

    for point in &[center, left, right] {
        let magnitude = (point.x().powi(2) + point.y().powi(2) + point.z().powi(2)).sqrt();
        assert!((magnitude - 1.0).abs() < 0.01, "全点が正規化済み");
    }
}

#[test]
fn test_issue_242_arcball_rotation_from_sphere_points() {
    let p1 = Vec3f::new(1.0, 0.0, 0.0);
    let q_identity = Camera::compute_rotation_from_sphere_points(p1, p1);
    let identity = Quaternionf::identity();
    assert!(
        (q_identity.w() - identity.w()).abs() < 0.001
            && (q_identity.x() - identity.x()).abs() < 0.001
            && (q_identity.y() - identity.y()).abs() < 0.001
            && (q_identity.z() - identity.z()).abs() < 0.001,
        "同じ点からの回転は恒等元"
    );

    let p_start = Vec3f::new(1.0, 0.0, 0.0).normalize().unwrap();
    let p_end = Vec3f::new(0.0, 1.0, 0.0).normalize().unwrap();
    let q_90 = Camera::compute_rotation_from_sphere_points(p_start, p_end);
    let is_not_identity = (q_90.w() - identity.w()).abs() > 0.01
        || (q_90.x() - identity.x()).abs() > 0.01
        || (q_90.y() - identity.y()).abs() > 0.01
        || (q_90.z() - identity.z()).abs() > 0.01;
    assert!(is_not_identity, "異なる点からは回転が生成される");

    let q_rev = Camera::compute_rotation_from_sphere_points(p_end, p_start);
    let is_rev_not_identity = (q_rev.w() - identity.w()).abs() > 0.01
        || (q_rev.x() - identity.x()).abs() > 0.01
        || (q_rev.y() - identity.y()).abs() > 0.01
        || (q_rev.z() - identity.z()).abs() > 0.01;
    assert!(is_rev_not_identity, "逆方向も回転を生成");
}

#[test]
fn test_issue_242_zoom_magnitude_sensitivity() {
    let mut camera_vertical = Camera::new();
    let mut camera_horizontal = Camera::new();
    let mut camera_diagonal = Camera::new();

    let initial_distance = camera_vertical.distance;

    camera_vertical.zoom(0.0, 10.0);
    camera_horizontal.zoom(10.0, 0.0);
    camera_diagonal.zoom(7.0, 7.0);

    let dist_v = camera_vertical.distance;
    let dist_h = camera_horizontal.distance;
    let dist_d = camera_diagonal.distance;

    println!(
        "ズーム結果: vertical={:.3}, horizontal={:.3}, diagonal={:.3}",
        dist_v, dist_h, dist_d
    );

    assert!(
        (dist_v - initial_distance).abs() > 0.001,
        "縦方向ズームが有効"
    );
    assert!(
        (dist_h - initial_distance).abs() > 0.001,
        "横方向ズームが有効"
    );
}
