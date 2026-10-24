//! 三角形の形状品質

use geo_algorithms::Point3D;
use geo_contracts::default_kernel_numerical_zero_tolerance;

/// 三角形のアスペクト比 = 最長辺 / (2√3 × 内接円半径)
///
/// 正三角形で 1、細長いほど大きい。内接円半径が数値的にゼロの三角形は `f64::INFINITY`。
///
/// 面積は外積から求める（Heron の公式は細長三角形で桁落ちするため用いない）。
pub fn triangle_aspect_ratio(a: Point3D<f64>, b: Point3D<f64>, c: Point3D<f64>) -> f64 {
    let (la, lb, lc) = (distance(b, c), distance(c, a), distance(a, b));
    let semi_perimeter = 0.5 * (la + lb + lc);

    let ab = [b.x() - a.x(), b.y() - a.y(), b.z() - a.z()];
    let ac = [c.x() - a.x(), c.y() - a.y(), c.z() - a.z()];
    let cross = [
        ab[1] * ac[2] - ab[2] * ac[1],
        ab[2] * ac[0] - ab[0] * ac[2],
        ab[0] * ac[1] - ab[1] * ac[0],
    ];
    let area = 0.5 * (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt();

    // 退化判定は面積（長さの二乗の次元）ではなく内接円半径（長さの次元）を、固定の長さ閾値
    // （カーネルのゼロ判定トレランス）と比較する。寸法の二乗で効く面積比較より寸法依存を抑えるが、
    // 閾値自体は固定長のため完全なスケール非依存ではない
    // （3 点が一致すると 0 / 0 で NaN になるため、それも退化として扱う）
    let inradius = area / semi_perimeter;
    if inradius.is_nan() || inradius <= default_kernel_numerical_zero_tolerance::<f64>() {
        return f64::INFINITY;
    }
    la.max(lb).max(lc) / (2.0 * 3.0_f64.sqrt() * inradius)
}

fn distance(a: Point3D<f64>, b: Point3D<f64>) -> f64 {
    let dx = a.x() - b.x();
    let dy = a.y() - b.y();
    let dz = a.z() - b.z();
    (dx * dx + dy * dy + dz * dz).sqrt()
}
