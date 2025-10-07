/// 幾何計算で使用される定数

/// 幾何計算用のデフォルト許容誤差
pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;

/// 数学定数
pub const PI: f64 = std::f64::consts::PI;
pub const TAU: f64 = 2.0 * PI;
pub const E: f64 = std::f64::consts::E;

/// 角度変換定数
pub const DEG_TO_RAD: f64 = PI / 180.0;
pub const RAD_TO_DEG: f64 = 180.0 / PI;