/// RedRing 数値解析用のシステム定数群

/// 交差判定・近似比較に使用する誤差許容値
pub const EPSILON: f64 = 1e-10;

/// ニュートン法で微分がゼロとみなされる閾値
pub const DERIVATIVE_ZERO_THRESHOLD: f64 = 1e-12;