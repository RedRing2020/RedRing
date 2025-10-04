//! Rodrigues 回転および軸回り回転の共通ユーティリティ
//! f64 ドメインで計算し、必要に応じて Scalar/Vector へ写像する。

use geo_core::Vector3D;
use geo_core::tolerance::ToleranceContext;
use geo_core::vector::Vector; // dot 用

/// Rodrigues の回転公式 (v を軸 k (正規化想定) で angle ラジアン回転)
/// 戻りは f64 成分 (x,y,z)
pub fn rodrigues_f64(k: (f64,f64,f64), v: (f64,f64,f64), angle: f64) -> (f64,f64,f64) {
    let (kx,ky,kz) = k;
    let (vx,vy,vz) = v;
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    // k × v
    let cross_x = ky * vz - kz * vy;
    let cross_y = kz * vx - kx * vz;
    let cross_z = kx * vy - ky * vx;
    // k · v
    let dot_kv = kx * vx + ky * vy + kz * vz;
    // v_rot = v cosθ + (k×v) sinθ + k (k·v)(1-cosθ)
    let one_minus_cos = 1.0 - cos_a;
    let rx = vx * cos_a + cross_x * sin_a + kx * dot_kv * one_minus_cos;
    let ry = vy * cos_a + cross_y * sin_a + ky * dot_kv * one_minus_cos;
    let rz = vz * cos_a + cross_z * sin_a + kz * dot_kv * one_minus_cos;
    (rx, ry, rz)
}

/// Vector3D を Rodrigues 回転
pub fn rotate_vector3d(v: &Vector3D, axis: &Vector3D, angle: f64, ctx: &ToleranceContext) -> Option<Vector3D> {
    // 軸を正規化
    let k = axis.normalize(ctx)?;
    let (kx,ky,kz) = (k.x().value(), k.y().value(), k.z().value());
    let (vx,vy,vz) = (v.x().value(), v.y().value(), v.z().value());
    let (rx,ry,rz) = rodrigues_f64((kx,ky,kz),(vx,vy,vz), angle);
    Some(Vector3D::from_f64(rx, ry, rz))
}

/// 方向ベクトル (既存 f64 ベース Direction3D 想定) に対する回転補助
/// x,y,z を取得するためのクロージャを受け取り、結果を (f64,f64,f64) で返す。
pub fn rotate_direction_f64<F>(get: F, axis: (f64,f64,f64), angle: f64) -> (f64,f64,f64)
where
    F: Fn() -> (f64,f64,f64)
{
    let (vx,vy,vz) = get();
    rodrigues_f64(axis, (vx,vy,vz), angle)
}
