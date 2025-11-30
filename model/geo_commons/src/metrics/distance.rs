//! 距離計算の共通実装
//!
//! 幾何形状間の距離計算アルゴリズムを提供します。
//!
//! 作成日: 2025年11月29日

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipse_2d_distance_inside() {
        // 楕円内部の点
        let dist = ellipse_2d_distance_to_point(1.0_f64, 0.5, 2.0, 1.0);
        assert!(dist < 1e-10);
    }

    #[test]
    fn test_ellipse_2d_distance_outside() {
        // 楕円外部の点
        let dist = ellipse_2d_distance_to_point(4.0_f64, 0.0, 2.0, 1.0);
        assert!((dist - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_ellipse_2d_distance_on_boundary() {
        // 楕円境界上の点（長軸端）
        let dist = ellipse_2d_distance_to_point(2.0_f64, 0.0, 2.0, 1.0);
        assert!(dist < 1e-10);
    }

    #[test]
    fn test_ellipse_3d_distance_on_plane() {
        // 平面上の点（法線距離=0）
        let dist = ellipse_3d_distance_to_point(1.0_f64, 0.5, 0.0, 2.0, 1.0);
        assert!(dist < 1e-10);
    }

    #[test]
    fn test_ellipse_3d_distance_off_plane() {
        // 平面外の点
        let dist = ellipse_3d_distance_to_point(0.0_f64, 0.0, 3.0, 2.0, 1.0);
        assert!((dist - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_ellipse_3d_distance_combined() {
        // 平面内距離と法線距離の組み合わせ
        // 平面内: (4,0) → 距離2.0, 法線: 3.0 → 総距離 sqrt(4+9) = sqrt(13)
        let dist = ellipse_3d_distance_to_point(4.0_f64, 0.0, 3.0, 2.0, 1.0);
        let expected = (4.0 + 9.0_f64).sqrt();
        assert!((dist - expected).abs() < 1e-10);
    }

    #[test]
    fn test_f32_compatibility() {
        let dist = ellipse_2d_distance_to_point(1.0_f32, 0.5, 2.0, 1.0);
        assert!(dist < 1e-6);
    }
}
