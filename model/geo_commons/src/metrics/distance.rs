//! 距離計算の共通実装
//!
//! 幾何形状間の距離計算アルゴリズムを提供します。

use analysis::linalg::vector::Vector3;
use analysis::Scalar;

/// 2D楕円上の点から任意の点への距離を計算
///
/// # 引数
/// * `point_x`, `point_y` - 計算対象の点の座標（楕円のローカル座標系）
/// * `semi_major` - 長半軸の長さ
/// * `semi_minor` - 短半軸の長さ
///
/// # 戻り値
/// 楕円境界からの最短距離（点が楕円内部の場合は0）
///
/// # 計算アルゴリズム
/// 1. 正規化座標系で点の位置を判定
/// 2. 楕円内部ならば距離0
/// 3. 楕円外部ならば境界までの近似距離を計算
pub fn ellipse_2d_distance_to_point<T: Scalar>(
    point_x: T,
    point_y: T,
    semi_major: T,
    semi_minor: T,
) -> T {
    // 正規化された楕円座標での距離計算
    let x_norm = point_x / semi_major;
    let y_norm = point_y / semi_minor;
    let normalized_distance = (x_norm * x_norm + y_norm * y_norm).sqrt();

    if normalized_distance <= T::ONE {
        // 点が楕円内部にある場合
        T::ZERO
    } else {
        // 点が楕円外部にある場合の近似距離
        // より正確な計算には数値的手法が必要
        let scale = T::ONE / normalized_distance;
        let boundary_x = point_x * scale;
        let boundary_y = point_y * scale;

        ((point_x - boundary_x) * (point_x - boundary_x)
            + (point_y - boundary_y) * (point_y - boundary_y))
            .sqrt()
    }
}

/// 3D楕円平面上の点から任意の点への距離を計算
///
/// # 引数
/// * `local_x`, `local_y` - 楕円平面内のローカル座標
/// * `normal_distance` - 楕円平面からの法線方向距離
/// * `semi_major` - 長半軸の長さ
/// * `semi_minor` - 短半軸の長さ
///
/// # 戻り値
/// 楕円境界からの3D空間での最短距離
///
/// # 計算アルゴリズム
/// 1. 平面内距離を2D楕円距離計算で求める
/// 2. 法線方向距離を含めた3D総距離を計算
pub fn ellipse_3d_distance_to_point<T: Scalar>(
    local_x: T,
    local_y: T,
    normal_distance: T,
    semi_major: T,
    semi_minor: T,
) -> T {
    // 平面内距離（2D楕円距離計算を再利用）
    let planar_distance = ellipse_2d_distance_to_point(local_x, local_y, semi_major, semi_minor);

    // 平面外距離を含めた総距離
    (planar_distance * planar_distance + normal_distance * normal_distance).sqrt()
}

/// 球の中心から無限直線までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標 (x, y, z)
/// * `radius` - 球の半径
/// * `line_point` - 直線上の任意の点 (x, y, z)
/// * `line_direction` - 直線の方向ベクトル (dx, dy, dz) ※正規化不要
/// * `is_solid` - true なら球体（内部含む）、false なら球面（表面のみ）
///
/// # Returns
///
/// 球（面または体）から直線までの最短距離
///
/// # Algorithm
///
/// 1. 直線上の最近点のパラメータ t を計算: `t = (C - P) · d / |d|²`
/// 2. 最近点 F = P + t·d を求める
/// 3. 中心から最近点までの距離 d_CF を計算
/// 4. 球体の場合: d_CF ≤ r なら 0、さもなくば d_CF - r
/// 5. 球面の場合: |d_CF - r|
pub fn sphere_to_infinite_line_distance<T: Scalar>(
    center: (T, T, T),
    radius: T,
    line_point: (T, T, T),
    line_direction: (T, T, T),
    is_solid: bool,
) -> T {
    let (cx, cy, cz) = center;
    let (px, py, pz) = line_point;
    let (dx, dy, dz) = line_direction;

    // ベクトル to_center = center - line_point
    let to_cx = cx - px;
    let to_cy = cy - py;
    let to_cz = cz - pz;

    // direction の内積
    let dir_dot = Vector3::new(dx, dy, dz).norm_squared();

    // パラメータ t = to_center · direction / |direction|²
    let t = (to_cx * dx + to_cy * dy + to_cz * dz) / dir_dot;

    // 最近点 = line_point + t * direction
    let closest_x = px + t * dx;
    let closest_y = py + t * dy;
    let closest_z = pz + t * dz;

    // 中心から最近点までの距離
    let diff_x = cx - closest_x;
    let diff_y = cy - closest_y;
    let diff_z = cz - closest_z;
    let distance_from_center = (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt();

    if is_solid {
        // 球体: 内部なら0、外側なら表面までの距離
        if distance_from_center <= radius {
            T::ZERO
        } else {
            distance_from_center - radius
        }
    } else {
        // 球面: 表面までの最短距離
        (distance_from_center - radius).abs()
    }
}

/// 球の中心から光線（Ray）までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標 (x, y, z)
/// * `radius` - 球の半径
/// * `ray_origin` - 光線の始点 (x, y, z)
/// * `ray_direction` - 光線の方向ベクトル (dx, dy, dz) ※正規化不要
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から光線までの最短距離
///
/// # Algorithm
///
/// 1. 無限直線として最近点のパラメータ t を計算
/// 2. t < 0 の場合: 光線の始点との距離を使用
/// 3. t ≥ 0 の場合: 無限直線と同じ処理
pub fn sphere_to_ray_distance<T: Scalar>(
    center: (T, T, T),
    radius: T,
    ray_origin: (T, T, T),
    ray_direction: (T, T, T),
    is_solid: bool,
) -> T {
    let (cx, cy, cz) = center;
    let (ox, oy, oz) = ray_origin;
    let (dx, dy, dz) = ray_direction;

    // ベクトル to_center = center - origin
    let to_cx = cx - ox;
    let to_cy = cy - oy;
    let to_cz = cz - oz;

    // direction の内積
    let dir_dot = Vector3::new(dx, dy, dz).norm_squared();

    // パラメータ t
    let t = (to_cx * dx + to_cy * dy + to_cz * dz) / dir_dot;

    if t < T::ZERO {
        // 光線の始点が最近点
        let dist_to_origin = (to_cx * to_cx + to_cy * to_cy + to_cz * to_cz).sqrt();

        if is_solid {
            if dist_to_origin <= radius {
                T::ZERO
            } else {
                dist_to_origin - radius
            }
        } else {
            (dist_to_origin - radius).abs()
        }
    } else {
        // t ≥ 0: 無限直線と同じ処理
        let closest_x = ox + t * dx;
        let closest_y = oy + t * dy;
        let closest_z = oz + t * dz;

        let diff_x = cx - closest_x;
        let diff_y = cy - closest_y;
        let diff_z = cz - closest_z;
        let distance_from_center = (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt();

        if is_solid {
            if distance_from_center <= radius {
                T::ZERO
            } else {
                distance_from_center - radius
            }
        } else {
            (distance_from_center - radius).abs()
        }
    }
}

/// 球の中心から線分までの最短距離を計算
///
/// # Arguments
///
/// * `center` - 球の中心座標 (x, y, z)
/// * `radius` - 球の半径
/// * `segment_start` - 線分の始点 (x, y, z)
/// * `segment_end` - 線分の終点 (x, y, z)
/// * `is_solid` - true なら球体、false なら球面
///
/// # Returns
///
/// 球から線分までの最短距離
///
/// # Algorithm
///
/// 1. 線分の方向ベクトル direction = end - start を計算
/// 2. 無限直線として最近点のパラメータ t を計算
/// 3. t < 0: 始点との距離
/// 4. t > 1: 終点との距離
/// 5. 0 ≤ t ≤ 1: 線分上の点との距離
pub fn sphere_to_line_segment_distance<T: Scalar>(
    center: (T, T, T),
    radius: T,
    segment_start: (T, T, T),
    segment_end: (T, T, T),
    is_solid: bool,
) -> T {
    let (cx, cy, cz) = center;
    let (sx, sy, sz) = segment_start;
    let (ex, ey, ez) = segment_end;

    // 線分の方向ベクトル
    let dx = ex - sx;
    let dy = ey - sy;
    let dz = ez - sz;

    // ベクトル to_center = center - start
    let to_cx = cx - sx;
    let to_cy = cy - sy;
    let to_cz = cz - sz;

    // direction の内積
    let dir_dot = Vector3::new(dx, dy, dz).norm_squared();

    // パラメータ t
    let t = (to_cx * dx + to_cy * dy + to_cz * dz) / dir_dot;

    // 最近点の座標を求める
    let (nearest_x, nearest_y, nearest_z) = if t < T::ZERO {
        // 始点が最近点
        segment_start
    } else if t > T::ONE {
        // 終点が最近点
        segment_end
    } else {
        // 線分上の点が最近点
        (sx + t * dx, sy + t * dy, sz + t * dz)
    };

    // 中心から最近点までの距離
    let diff_x = cx - nearest_x;
    let diff_y = cy - nearest_y;
    let diff_z = cz - nearest_z;
    let distance_from_center = (diff_x * diff_x + diff_y * diff_y + diff_z * diff_z).sqrt();

    if is_solid {
        if distance_from_center <= radius {
            T::ZERO
        } else {
            distance_from_center - radius
        }
    } else {
        (distance_from_center - radius).abs()
    }
}

/// 線分と軸平行境界ボックス（AABB）間の最短距離を計算
///
/// # Arguments
///
/// * `segment_start` - 線分の始点 (x, y, z)
/// * `segment_end` - 線分の終点 (x, y, z)
/// * `aabb_min` - AABBの最小座標 (x, y, z)
/// * `aabb_max` - AABBの最大座標 (x, y, z)
///
/// # Returns
///
/// 線分とAABBの最短距離（線分がAABB内部を通過する場合は0）
///
/// # Algorithm
///
/// 1. 線分の両端点がAABB内部にあるかチェック
/// 2. 線分上の各サンプル点をAABBにクランプして最短距離を計算
/// 3. AABBの各頂点から線分への距離も考慮
/// 4. これらの中で最小値を返す
pub fn line_segment_to_aabb_distance<T: Scalar>(
    segment_start: (T, T, T),
    segment_end: (T, T, T),
    aabb_min: (T, T, T),
    aabb_max: (T, T, T),
) -> T {
    let (sx, sy, sz) = segment_start;
    let (ex, ey, ez) = segment_end;
    let (min_x, min_y, min_z) = aabb_min;
    let (max_x, max_y, max_z) = aabb_max;

    // ヘルパー関数: 点がAABB内部にあるかチェック
    let point_in_aabb = |px: T, py: T, pz: T| -> bool {
        px >= min_x && px <= max_x && py >= min_y && py <= max_y && pz >= min_z && pz <= max_z
    };

    // ヘルパー関数: 点をAABBの最近点にクランプ
    let clamp_to_aabb = |px: T, py: T, pz: T| -> (T, T, T) {
        let cx = px.clamp(min_x, max_x);
        let cy = py.clamp(min_y, max_y);
        let cz = pz.clamp(min_z, max_z);
        (cx, cy, cz)
    };

    // ヘルパー関数: 2点間の距離
    let distance = |p1x: T, p1y: T, p1z: T, p2x: T, p2y: T, p2z: T| -> T {
        Vector3::new(p1x - p2x, p1y - p2y, p1z - p2z).norm()
    };

    // 1. 線分の端点がAABB内部にあれば距離0
    if point_in_aabb(sx, sy, sz) || point_in_aabb(ex, ey, ez) {
        return T::ZERO;
    }

    // 2. 線分の方向ベクトル
    let dx = ex - sx;
    let dy = ey - sy;
    let dz = ez - sz;

    // 3. 線分をサンプリングして最短距離を計算
    let num_samples = 10;
    let mut min_distance = T::INFINITY;

    for i in 0..=num_samples {
        let t = T::from_usize(i) / T::from_usize(num_samples);
        let px = sx + dx * t;
        let py = sy + dy * t;
        let pz = sz + dz * t;

        // 線分上の点をAABBにクランプ
        let (cx, cy, cz) = clamp_to_aabb(px, py, pz);
        let dist = distance(px, py, pz, cx, cy, cz);
        min_distance = min_distance.min(dist);

        // 距離0なら即座に返す（交差している）
        if dist <= T::EPSILON {
            return T::ZERO;
        }
    }

    // 4. AABBの8頂点から線分への距離も確認
    let vertices = [
        (min_x, min_y, min_z),
        (max_x, min_y, min_z),
        (min_x, max_y, min_z),
        (max_x, max_y, min_z),
        (min_x, min_y, max_z),
        (max_x, min_y, max_z),
        (min_x, max_y, max_z),
        (max_x, max_y, max_z),
    ];

    for &(vx, vy, vz) in &vertices {
        // 頂点から線分への距離
        // 線分上の最近点のパラメータ t を計算
        let to_vx = vx - sx;
        let to_vy = vy - sy;
        let to_vz = vz - sz;

        let dot = to_vx * dx + to_vy * dy + to_vz * dz;
        let len_sq = Vector3::new(dx, dy, dz).norm_squared();

        let t = if len_sq <= T::EPSILON {
            T::ZERO
        } else {
            (dot / len_sq).clamp(T::ZERO, T::ONE)
        };

        let closest_x = sx + dx * t;
        let closest_y = sy + dy * t;
        let closest_z = sz + dz * t;

        let dist = distance(vx, vy, vz, closest_x, closest_y, closest_z);
        min_distance = min_distance.min(dist);
    }

    min_distance
}

#[cfg(test)]
mod tests {
    use analysis::test_constants::{DISTANCE_TOLERANCE_F32, DISTANCE_TOLERANCE_F64};

    use super::*;

    // 楕円テスト（既存）
    #[test]
    fn test_ellipse_2d_distance_inside() {
        let dist = ellipse_2d_distance_to_point(1.0_f64, 0.5, 2.0, 1.0);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_2d_distance_outside() {
        let dist = ellipse_2d_distance_to_point(4.0_f64, 0.0, 2.0, 1.0);
        assert!((dist - 2.0).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_2d_distance_on_boundary() {
        let dist = ellipse_2d_distance_to_point(2.0_f64, 0.0, 2.0, 1.0);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_3d_distance_on_plane() {
        let dist = ellipse_3d_distance_to_point(1.0_f64, 0.5, 0.0, 2.0, 1.0);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_3d_distance_off_plane() {
        let dist = ellipse_3d_distance_to_point(0.0_f64, 0.0, 3.0, 2.0, 1.0);
        assert!((dist - 3.0).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_3d_distance_combined() {
        let dist = ellipse_3d_distance_to_point(4.0_f64, 0.0, 3.0, 2.0, 1.0);
        let expected = (4.0 + 9.0_f64).sqrt();
        assert!((dist - expected).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_f32_compatibility() {
        let dist = ellipse_2d_distance_to_point(1.0_f32, 0.5, 2.0, 1.0);
        assert!(dist < DISTANCE_TOLERANCE_F32);
    }

    // 球と直線のテスト（新規）
    #[test]
    fn test_sphere_to_infinite_line_intersecting_solid() {
        let center = (0.0, 0.0, 0.0);
        let radius = 1.0;
        let line_point = (-2.0, 0.0, 0.0);
        let line_direction = (1.0, 0.0, 0.0);

        let dist =
            sphere_to_infinite_line_distance(center, radius, line_point, line_direction, true);
        assert!(dist.abs() < DISTANCE_TOLERANCE_F64); // 直線が球体を貫通
    }

    #[test]
    fn test_sphere_to_infinite_line_intersecting_surface() {
        let center = (0.0, 0.0, 0.0);
        let radius = 1.0;
        let line_point = (-2.0, 0.0, 0.0);
        let line_direction = (1.0, 0.0, 0.0);

        let dist =
            sphere_to_infinite_line_distance(center, radius, line_point, line_direction, false);
        assert!((dist - 1.0).abs() < DISTANCE_TOLERANCE_F64); // 球面まで距離1
    }

    #[test]
    fn test_sphere_to_infinite_line_tangent() {
        let center = (0.0, 0.0, 0.0);
        let radius = 1.0;
        let line_point = (0.0, 1.0, 0.0); // Y軸上の点
        let line_direction = (1.0, 0.0, 0.0); // X軸方向

        let dist =
            sphere_to_infinite_line_distance(center, radius, line_point, line_direction, true);
        assert!(dist.abs() < DISTANCE_TOLERANCE_F64); // 接線は衝突
    }

    #[test]
    fn test_sphere_to_ray_behind_origin() {
        let center = (-2.0, 0.0, 0.0);
        let radius = 1.0;
        let ray_origin = (0.0, 0.0, 0.0);
        let ray_direction = (1.0, 0.0, 0.0); // X軸正方向

        let dist = sphere_to_ray_distance(center, radius, ray_origin, ray_direction, true);
        assert!((dist - 1.0).abs() < DISTANCE_TOLERANCE_F64); // 始点との距離 - 半径
    }

    #[test]
    fn test_sphere_to_ray_through_center() {
        let center = (0.0, 0.0, 0.0);
        let radius = 1.0;
        let ray_origin = (-2.0, 0.0, 0.0);
        let ray_direction = (1.0, 0.0, 0.0);

        let dist = sphere_to_ray_distance(center, radius, ray_origin, ray_direction, true);
        assert!(dist.abs() < DISTANCE_TOLERANCE_F64); // 光線が球体を貫通
    }

    #[test]
    fn test_sphere_to_line_segment_endpoint() {
        let center = (0.0, 2.0, 0.0);
        let radius = 1.0;
        let segment_start = (-1.0, 0.0, 0.0);
        let segment_end = (1.0, 0.0, 0.0);

        let dist =
            sphere_to_line_segment_distance(center, radius, segment_start, segment_end, true);
        // 線分上の最近点は (0, 0, 0) → 中心からの距離は 2.0
        // 球体なので距離は 2.0 - 1.0 = 1.0
        let expected = 1.0;
        assert!((dist - expected).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_sphere_to_line_segment_through_center() {
        let center = (0.0, 0.0, 0.0);
        let radius = 1.0;
        let segment_start = (-2.0, 0.0, 0.0);
        let segment_end = (2.0, 0.0, 0.0);

        let dist =
            sphere_to_line_segment_distance(center, radius, segment_start, segment_end, true);
        assert!(dist.abs() < DISTANCE_TOLERANCE_F64); // 線分が球体を貫通
    }

    // 線分-AABB距離テスト
    #[test]
    fn test_line_segment_to_aabb_intersection() {
        // 線分がAABBを貫通する場合
        let segment_start = (-1.0, 0.0, 0.0);
        let segment_end = (1.0, 0.0, 0.0);
        let aabb_min = (-0.5, -0.5, -0.5);
        let aabb_max = (0.5, 0.5, 0.5);

        let dist = line_segment_to_aabb_distance(segment_start, segment_end, aabb_min, aabb_max);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment_to_aabb_endpoint_inside() {
        // 線分の端点がAABB内部にある場合
        let segment_start = (0.0, 0.0, 0.0);
        let segment_end = (2.0, 0.0, 0.0);
        let aabb_min = (-1.0, -1.0, -1.0);
        let aabb_max = (1.0, 1.0, 1.0);

        let dist = line_segment_to_aabb_distance(segment_start, segment_end, aabb_min, aabb_max);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment_to_aabb_parallel_offset() {
        // 線分がAABBに平行でオフセットがある場合
        let segment_start = (0.0, 2.0, 0.0);
        let segment_end = (1.0, 2.0, 0.0);
        let aabb_min = (0.0, 0.0, 0.0);
        let aabb_max = (1.0, 1.0, 1.0);

        let dist = line_segment_to_aabb_distance(segment_start, segment_end, aabb_min, aabb_max);
        // Y方向に1.0離れている
        assert!((dist - 1.0).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment_to_aabb_diagonal() {
        // 線分がAABBの角近くを通過
        let segment_start = (2.0, 2.0, 2.0);
        let segment_end = (3.0, 3.0, 3.0);
        let aabb_min = (0.0, 0.0, 0.0);
        let aabb_max = (1.0, 1.0, 1.0);

        let dist = line_segment_to_aabb_distance(segment_start, segment_end, aabb_min, aabb_max);
        // AABBの頂点 (1,1,1) から線分への距離
        let expected = (3.0_f64).sqrt(); // sqrt((2-1)^2 + (2-1)^2 + (2-1)^2)
        assert!((dist - expected).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_line_segment_to_aabb_distant() {
        // 線分がAABBから遠く離れている場合
        let segment_start = (10.0, 0.0, 0.0);
        let segment_end = (11.0, 0.0, 0.0);
        let aabb_min = (0.0, 0.0, 0.0);
        let aabb_max = (1.0, 1.0, 1.0);

        let dist = line_segment_to_aabb_distance(segment_start, segment_end, aabb_min, aabb_max);
        // AABBの最も近い点 (1,0,0) から線分始点 (10,0,0) への距離
        assert!((dist - 9.0).abs() < DISTANCE_TOLERANCE_F64);
    }
}
