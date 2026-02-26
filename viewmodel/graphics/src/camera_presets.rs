use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};

/// 境界ボックス全体が見えるようにカメラ状態を調整する。
pub(crate) fn fit_to_mesh(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    min_bounds: Vec3f,
    max_bounds: Vec3f,
) {
    let center = (min_bounds + max_bounds) * 0.5;
    *target = center;

    let size = max_bounds - min_bounds;
    let diagonal = (size.x().powi(2) + size.y().powi(2) + size.z().powi(2)).sqrt();

    let fov_rad = 45.0_f32.to_radians();
    let margin = 2.5;
    let computed_distance = (diagonal * margin) / (2.0 * (fov_rad / 2.0).tan());

    let min_distance = (diagonal * 0.1).max(0.01);
    let max_distance = 100.0;
    *distance = computed_distance.clamp(min_distance, max_distance);

    let x_rotation =
        Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -30.0_f32.to_radians());
    let y_rotation =
        Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 45.0_f32.to_radians());
    *rotation = (y_rotation * x_rotation)
        .normalize()
        .unwrap_or(Quaternionf::identity());

    tracing::info!(
        "カメラをメッシュに適応: center={:?}, distance={:.2}, diagonal={:.2}, min_distance={:.4}",
        [center.x(), center.y(), center.z()],
        *distance,
        diagonal,
        min_distance
    );
}

/// 小さな境界ボックス向けにカメラ状態を調整する。
pub(crate) fn fit_to_small_mesh(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    min_bounds: Vec3f,
    max_bounds: Vec3f,
) {
    let center = (min_bounds + max_bounds) * 0.5;
    *target = center;

    let size = max_bounds - min_bounds;
    let diagonal = (size.x().powi(2) + size.y().powi(2) + size.z().powi(2)).sqrt();

    let distance_factor = if diagonal < 0.01 {
        15.0
    } else if diagonal < 0.1 {
        10.0
    } else {
        5.0
    };

    *distance = diagonal * distance_factor;

    let x_rotation =
        Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -60.0_f32.to_radians());
    let y_rotation =
        Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 30.0_f32.to_radians());
    *rotation = (y_rotation * x_rotation)
        .normalize()
        .unwrap_or(Quaternionf::identity());

    tracing::info!(
        "小オブジェクト対応カメラ設定: center={:?}, distance={:.4}, diagonal={:.4}, factor={}",
        [center.x(), center.y(), center.z()],
        *distance,
        diagonal,
        distance_factor
    );
}

/// 基本状態へカメラをリセットする。
pub(crate) fn reset(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    zoom: &mut f32,
) {
    *target = Vec3f::new(0.0, 0.0, 0.0);
    *distance = 5.0;
    *rotation = Quaternionf::identity();
    *zoom = 1.0;
    tracing::info!("カメラをリセット");
}

/// 標準の直交表示向け状態へリセットする。
pub(crate) fn reset_to_standard_cad_view(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    zoom: &mut f32,
    orthographic_bounds: &mut Option<(f32, f32, f32, f32)>,
) {
    *target = Vec3f::new(0.0, 0.0, 0.0);
    *distance = 10.0;
    *zoom = 1.0;
    *orthographic_bounds = None;
    *rotation = Quaternionf::identity();

    tracing::info!(
        "カメラを標準CAD視点にリセット（距離: {:.1}、回転なし・Z正方向から正面）",
        *distance
    );
}

/// 正面向きの状態へリセットする。
pub(crate) fn reset_to_front_view(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    zoom: &mut f32,
) {
    *target = Vec3f::new(0.0, 0.0, 0.0);
    *distance = 15.0;
    *zoom = 1.0;
    *rotation = Quaternionf::identity();

    tracing::info!(
        "カメラを正面視点にリセット（距離: {:.1}、1単位立方体が確実に見える位置）",
        *distance
    );
}

/// 境界ボックスに基づく安全な表示状態へリセットする。
pub(crate) fn reset_to_safe_view(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    zoom: &mut f32,
    min_bounds: Vec3f,
    max_bounds: Vec3f,
) {
    let center = (min_bounds + max_bounds) * 0.5;
    let size = max_bounds - min_bounds;
    let diagonal = (size.x().powi(2) + size.y().powi(2) + size.z().powi(2)).sqrt();

    let safe_distance = (diagonal * 3.0).max(2.0);

    *target = center;
    *distance = safe_distance;
    *zoom = 1.0;

    let x_rotation =
        Quaternionf::from_axis_angle(&Vec3f::new(1.0, 0.0, 0.0), -30.0_f32.to_radians());
    let y_rotation =
        Quaternionf::from_axis_angle(&Vec3f::new(0.0, 1.0, 0.0), 45.0_f32.to_radians());
    *rotation = (y_rotation * x_rotation)
        .normalize()
        .unwrap_or(Quaternionf::identity());

    tracing::info!(
        "安全な視点にリセット: center={:?}, distance={:.2}",
        [center.x(), center.y(), center.z()],
        safe_distance
    );
}

/// カメラ距離が下限未満の場合に補正する。
pub(crate) fn ensure_minimum_distance(distance: &mut f32) {
    const MIN_SAFE_DISTANCE: f32 = 1.0;
    if *distance < MIN_SAFE_DISTANCE {
        *distance = MIN_SAFE_DISTANCE;
        tracing::warn!("最小距離を強制適用: {:.2}", MIN_SAFE_DISTANCE);
    }
}

/// 緊急復帰用の安全なカメラ状態を適用する。
pub(crate) fn emergency_camera_escape(
    target: &mut Vec3f,
    distance: &mut f32,
    rotation: &mut Quaternionf,
    zoom: &mut f32,
) {
    *target = Vec3f::new(0.0, 0.0, 0.0);
    *distance = 20.0;
    *zoom = 1.0;
    *rotation = Quaternionf::identity();

    tracing::warn!("緊急カメラ脱出実行（距離: {:.1}、正面視点）", *distance);
}

/// 現在のカメラ状態をログ出力する。
pub(crate) fn log_state(target: &Vec3f, distance: f32, rotation: &Quaternionf) {
    tracing::info!(
        "カメラ状態 - target: {:?}, distance: {:.2}, rotation: [{:.3}, {:.3}, {:.3}, {:.3}]",
        [target.x(), target.y(), target.z()],
        distance,
        rotation.x(),
        rotation.y(),
        rotation.z(),
        rotation.w()
    );
}
