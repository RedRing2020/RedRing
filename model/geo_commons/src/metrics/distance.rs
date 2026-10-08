//! 距離計算の共通実装
//!
//! 幾何形状間の距離計算アルゴリズムを提供します。

use analysis::linalg::vector::Vector3;
use analysis::Scalar;

/// 楕円の最近点を求める二分法の反復回数の上限
///
/// 二分法は区間の中点が端点と一致した時点（浮動小数点で区間をこれ以上分割できない時点）で終了する。
/// 上限は、f64 の仮数部と指数部の範囲を区間の分割で使い切る回数を上回る値とする。
const ELLIPSE_CLOSEST_POINT_MAX_BISECTION_ITERATIONS: usize = 1100;

/// 2D楕円（曲線）上で点に最も近い点を計算
///
/// # 引数
/// * `point_x`, `point_y` - 計算対象の点の座標（楕円のローカル座標系。中心が原点）
/// * `semi_major` - x 軸方向の半軸の長さ
/// * `semi_minor` - y 軸方向の半軸の長さ
///
/// # 戻り値
/// 楕円上の最近点の座標（楕円のローカル座標系）
///
/// # 計算アルゴリズム
/// 1. 点を第 1 象限へ折り返し、長い半軸を x 軸に揃える
/// 2. 最近点の条件（点と最近点を結ぶ線分が楕円の法線方向）を 1 変数の単調な方程式にし、その根を二分法で求める
/// 3. 符号と軸の入れ替えを戻す
///
/// 楕円の内部・外部・軸上のいずれの点でも最近点を返す。
pub fn ellipse_2d_closest_point<T: Scalar>(
    point_x: T,
    point_y: T,
    semi_major: T,
    semi_minor: T,
) -> (T, T) {
    let restore_sign = |value: T, reference: T| {
        if reference < T::ZERO {
            -value
        } else {
            value
        }
    };

    if semi_major >= semi_minor {
        let (x, y) = ellipse_closest_point_in_first_quadrant(
            semi_major,
            semi_minor,
            point_x.abs(),
            point_y.abs(),
        );
        (restore_sign(x, point_x), restore_sign(y, point_y))
    } else {
        let (y, x) = ellipse_closest_point_in_first_quadrant(
            semi_minor,
            semi_major,
            point_y.abs(),
            point_x.abs(),
        );
        (restore_sign(x, point_x), restore_sign(y, point_y))
    }
}

/// 第 1 象限の点に対する楕円上の最近点（`e0 >= e1 > 0`、`y0 >= 0`、`y1 >= 0`）
fn ellipse_closest_point_in_first_quadrant<T: Scalar>(e0: T, e1: T, y0: T, y1: T) -> (T, T) {
    if y1 > T::ZERO {
        if y0 > T::ZERO {
            let z0 = y0 / e0;
            let z1 = y1 / e1;
            let g = z0 * z0 + z1 * z1 - T::ONE;
            if g == T::ZERO {
                return (y0, y1);
            }
            let r0 = (e0 / e1) * (e0 / e1);
            let s = ellipse_closest_point_root(r0, z0, z1, g);
            (r0 * y0 / (s + r0), y1 / (s + T::ONE))
        } else {
            // 短軸上の点は短軸の端点が最近点となる
            (T::ZERO, e1)
        }
    } else {
        // 長軸上の点は、中心に近い範囲では長軸から外れた点が、それ以外では長軸の端点が最近点となる
        let numer0 = e0 * y0;
        let denom0 = e0 * e0 - e1 * e1;
        if numer0 < denom0 {
            let xde0 = numer0 / denom0;
            (e0 * xde0, e1 * (T::ONE - xde0 * xde0).sqrt())
        } else {
            (e0, T::ZERO)
        }
    }
}

/// 最近点の条件式 `(r0 z0 / (s + r0))² + (z1 / (s + 1))² - 1 = 0` の根を二分法で求める
///
/// 左辺は `s > -1` で単調減少し、根は `[z1 - 1, |(r0 z0, z1)| - 1]`（点が楕円の内部なら上端は 0）にある。
fn ellipse_closest_point_root<T: Scalar>(r0: T, z0: T, z1: T, g: T) -> T {
    let n0 = r0 * z0;
    let mut s0 = z1 - T::ONE;
    let mut s1 = if g < T::ZERO {
        T::ZERO
    } else {
        (n0 * n0 + z1 * z1).sqrt() - T::ONE
    };
    let two = T::ONE + T::ONE;
    let mut s = T::ZERO;
    for _ in 0..ELLIPSE_CLOSEST_POINT_MAX_BISECTION_ITERATIONS {
        s = (s0 + s1) / two;
        if s == s0 || s == s1 {
            break;
        }
        let ratio0 = n0 / (s + r0);
        let ratio1 = z1 / (s + T::ONE);
        let value = ratio0 * ratio0 + ratio1 * ratio1 - T::ONE;
        if value > T::ZERO {
            s0 = s;
        } else if value < T::ZERO {
            s1 = s;
        } else {
            break;
        }
    }
    s
}

/// 2D楕円（曲線）から点への最短距離を計算
///
/// # 引数
/// * `point_x`, `point_y` - 計算対象の点の座標（楕円のローカル座標系。中心が原点）
/// * `semi_major` - x 軸方向の半軸の長さ
/// * `semi_minor` - y 軸方向の半軸の長さ
///
/// # 戻り値
/// 楕円（曲線）上の最近点までの距離。楕円の内部の点でも曲線までの距離を返す
pub fn ellipse_2d_distance_to_point<T: Scalar>(
    point_x: T,
    point_y: T,
    semi_major: T,
    semi_minor: T,
) -> T {
    let (closest_x, closest_y) = ellipse_2d_closest_point(point_x, point_y, semi_major, semi_minor);
    let dx = point_x - closest_x;
    let dy = point_y - closest_y;
    (dx * dx + dy * dy).sqrt()
}

/// 3D楕円（曲線）から点への最短距離を計算
///
/// # 引数
/// * `local_x`, `local_y` - 楕円平面内のローカル座標
/// * `normal_distance` - 楕円平面からの法線方向距離
/// * `semi_major` - x 軸方向の半軸の長さ
/// * `semi_minor` - y 軸方向の半軸の長さ
///
/// # 戻り値
/// 楕円（曲線）上の最近点までの 3D 空間での距離
///
/// # 計算アルゴリズム
/// 平面上の曲線への最近点は、点を平面へ投影した点の最近点と一致するため、
/// 平面内の距離と法線方向の距離を合成する。
pub fn ellipse_3d_distance_to_point<T: Scalar>(
    local_x: T,
    local_y: T,
    normal_distance: T,
    semi_major: T,
    semi_minor: T,
) -> T {
    let planar_distance = ellipse_2d_distance_to_point(local_x, local_y, semi_major, semi_minor);
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

    /// 楕円上の最近点の条件（楕円上にあり、点との差が法線方向）を満たすか
    fn assert_closest_point_on_ellipse(point: (f64, f64), semi_major: f64, semi_minor: f64) {
        let (cx, cy) = ellipse_2d_closest_point(point.0, point.1, semi_major, semi_minor);
        let on_ellipse = (cx / semi_major).powi(2) + (cy / semi_minor).powi(2) - 1.0;
        assert!(on_ellipse.abs() < DISTANCE_TOLERANCE_F64);

        let normal = (
            cx / (semi_major * semi_major),
            cy / (semi_minor * semi_minor),
        );
        let offset = (point.0 - cx, point.1 - cy);
        let cross = normal.0 * offset.1 - normal.1 * offset.0;
        assert!(cross.abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_2d_distance_reference_values() {
        // (点, 長半軸, 短半軸, 楕円までの距離)。距離は曲線のパラメータ方程式の数値解で求めた値
        let cases = [
            ((1.0, 0.5), 2.0, 1.0, 0.349_605_694_569_673),
            ((4.0, 4.0), 4.0, 1.0, 3.484_883_349_011_337),
            ((5.0, 2.0), 4.0, 1.0, 2.070_522_627_212_593),
            ((-3.0, -0.2), 4.0, 1.0, 0.442_182_830_138_887),
        ];
        for (point, a, b, expected) in cases {
            let dist = ellipse_2d_distance_to_point(point.0, point.1, a, b);
            assert!((dist - expected).abs() < DISTANCE_TOLERANCE_F64);
            assert_closest_point_on_ellipse(point, a, b);
        }
    }

    #[test]
    fn test_ellipse_2d_distance_inside_returns_distance_to_curve() {
        // 中心からは短半軸の長さ
        let dist = ellipse_2d_distance_to_point(0.0_f64, 0.0, 2.0, 1.0);
        assert!((dist - 1.0).abs() < DISTANCE_TOLERANCE_F64);

        // 長軸上の内部の点は長軸から外れた点が最近点となる（距離 √6 / 3）
        let dist = ellipse_2d_distance_to_point(1.0_f64, 0.0, 2.0, 1.0);
        assert!((dist - 6.0_f64.sqrt() / 3.0).abs() < DISTANCE_TOLERANCE_F64);
        let (cx, cy) = ellipse_2d_closest_point(1.0_f64, 0.0, 2.0, 1.0);
        assert!((cx - 4.0 / 3.0).abs() < DISTANCE_TOLERANCE_F64);
        assert!((cy - 5.0_f64.sqrt() / 3.0).abs() < DISTANCE_TOLERANCE_F64);

        // 短軸上の内部の点は短軸の端点が最近点となる
        let dist = ellipse_2d_distance_to_point(0.0_f64, 0.5, 2.0, 1.0);
        assert!((dist - 0.5).abs() < DISTANCE_TOLERANCE_F64);
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

        let t = 0.7_f64;
        let dist = ellipse_2d_distance_to_point(2.0 * t.cos(), t.sin(), 2.0, 1.0);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_2d_distance_with_longer_y_axis() {
        // y 軸方向の半軸が長い場合は軸を入れ替えて計算する
        let dist = ellipse_2d_distance_to_point(0.5_f64, 1.0, 1.0, 2.0);
        assert!((dist - 0.349_605_694_569_673).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_2d_distance_circle() {
        // 半軸が等しい場合は円周までの距離
        let dist = ellipse_2d_distance_to_point(0.3_f64, 0.4, 1.0, 1.0);
        assert!((dist - 0.5).abs() < DISTANCE_TOLERANCE_F64);
        let dist = ellipse_2d_distance_to_point(0.0_f64, 0.0, 1.0, 1.0);
        assert!((dist - 1.0).abs() < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_3d_distance_on_plane() {
        let dist = ellipse_3d_distance_to_point(2.0_f64, 0.0, 0.0, 2.0, 1.0);
        assert!(dist < DISTANCE_TOLERANCE_F64);
    }

    #[test]
    fn test_ellipse_3d_distance_off_plane() {
        // 中心の真上の点は短軸の端点までの距離
        let dist = ellipse_3d_distance_to_point(0.0_f64, 0.0, 3.0, 2.0, 1.0);
        assert!((dist - 10.0_f64.sqrt()).abs() < DISTANCE_TOLERANCE_F64);
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
        assert!((dist - 0.349_605_7).abs() < DISTANCE_TOLERANCE_F32);
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
