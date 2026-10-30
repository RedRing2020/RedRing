//! デクセルの区間の端の法線（材料の外向きの単位ベクトル）を 2 × 16 bit に量子化した値
//!
//! 単位球を八面体に写し、八面体を正方形に開いた座標（八面体の符号化）を、各成分 16 bit に量子化する。
//! 量子化による向きの誤差は 0.01 度程度である。

/// 量子化した単位法線
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DexelNormal(pub(crate) [u16; 2]);

const QUANTIZATION_MAX: f64 = u16::MAX as f64;

impl DexelNormal {
    /// 単位ベクトル `unit` を量子化する
    pub fn from_unit(unit: [f64; 3]) -> Self {
        let [x, y, z] = unit;
        let length = x.abs() + y.abs() + z.abs();
        let (u, v) = (x / length, y / length);
        // 下半分の八面体は、正方形の角へ折り返す
        let (u, v) = if z < 0.0 { fold(u, v) } else { (u, v) };
        let quantize = |value: f64| ((value * 0.5 + 0.5) * QUANTIZATION_MAX).round() as u16;
        Self([quantize(u), quantize(v)])
    }

    /// 量子化した値の単位ベクトル
    pub fn to_unit(self) -> [f64; 3] {
        let [u, v] = self
            .0
            .map(|value| f64::from(value) / QUANTIZATION_MAX * 2.0 - 1.0);
        let z = 1.0 - u.abs() - v.abs();
        let (x, y) = if z < 0.0 { fold(u, v) } else { (u, v) };
        let length = (x * x + y * y + z * z).sqrt();
        [x / length, y / length, z / length]
    }
}

/// 八面体の下半分と、開いた正方形の角の間の写像（自身の逆写像）
fn fold(u: f64, v: f64) -> (f64, f64) {
    let sign = |value: f64| if value >= 0.0 { 1.0 } else { -1.0 };
    ((1.0 - v.abs()) * sign(u), (1.0 - u.abs()) * sign(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn angle_deg(a: [f64; 3], b: [f64; 3]) -> f64 {
        let dot: f64 = a.iter().zip(b).map(|(a, b)| a * b).sum();
        dot.clamp(-1.0, 1.0).acos().to_degrees()
    }

    #[test]
    fn quantized_normals_keep_the_direction_within_a_hundredth_of_a_degree() {
        let mut directions = vec![
            [1.0, 0.0, 0.0],
            [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
        for i in 0..40 {
            for j in 0..20 {
                let (theta, phi) = (
                    f64::from(i) * std::f64::consts::TAU / 40.0,
                    f64::from(j) * std::f64::consts::PI / 19.0,
                );
                directions.push([phi.sin() * theta.cos(), phi.sin() * theta.sin(), phi.cos()]);
            }
        }
        for direction in directions {
            let restored = DexelNormal::from_unit(direction).to_unit();
            assert!(
                angle_deg(direction, restored) < 0.01,
                "{direction:?} -> {restored:?}"
            );
        }
    }
}
