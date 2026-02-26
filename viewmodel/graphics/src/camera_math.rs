use analysis::linalg::{quaternion::Quaternionf, vector::Vec3f};

/// analysisクレートのクォータニオンを4x4行列に変換
pub(crate) fn quaternion_to_matrix(q: &Quaternionf) -> [[f32; 4]; 4] {
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

/// 行列によるベクトル変換（3x3部分のみを使用）
pub(crate) fn matrix_transform_vector(matrix: &[[f32; 4]; 4], v: &Vec3f) -> Vec3f {
    Vec3f::new(
        matrix[0][0] * v.x() + matrix[0][1] * v.y() + matrix[0][2] * v.z(),
        matrix[1][0] * v.x() + matrix[1][1] * v.y() + matrix[1][2] * v.z(),
        matrix[2][0] * v.x() + matrix[2][1] * v.y() + matrix[2][2] * v.z(),
    )
}

/// Vector3の線形補間
pub(crate) fn lerp_vector3(a: Vec3f, b: Vec3f, t: f32) -> Vec3f {
    a * (1.0 - t) + b * t
}

/// f32の線形補間
pub(crate) fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a * (1.0 - t) + b * t
}
