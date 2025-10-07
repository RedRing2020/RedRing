/// 幾何計算で使用される定数
///
/// 用途別にf32/f64の定数を分離して提供します:
/// - `game`: 高速ゲーム処理用（f32）
/// - `precision`: 高精度CAD処理用（f64）
///
/// **注意**: このモジュールは将来的に `analysis` クレートに移動予定です。
/// 線形代数計算や数値解析で使用する定数の統一管理を目的とします。
///
/// **用途の詳細**:
/// - **game モジュール**: リアルタイム描画、物理シミュレーション、形状ピック処理、
///   簡易表示用図形など、軽量・高速処理が求められる場面
/// - **precision モジュール**: CAD/CAM の精密計算、科学技術計算、
///   製図・設計で高精度が要求される場面
///
/// 高速ゲーム処理用の定数（f32）
///
/// GPU処理、リアルタイム描画、大量の図形処理に最適です。
/// CADでも表示用図形や簡易計算（形状ピック処理等）で使用されます。
pub mod game {
    /// 数学定数（f32）
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;

    /// 円・円弧関連の定数（f32）
    pub const PI_2: f32 = PI / 2.0; // π/2 (90度)
    pub const PI_4: f32 = PI / 4.0; // π/4 (45度)
    pub const PI_3: f32 = PI / 3.0; // π/3 (60度)
    pub const PI_6: f32 = PI / 6.0; // π/6 (30度)

    /// 角度変換定数（f32）
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;

    /// 幾何計算用の許容誤差（f32用）
    pub const GEOMETRIC_TOLERANCE: f32 = 1e-6;

    /// よく使われる角度（f32）
    pub const ANGLE_0: f32 = 0.0;
    pub const ANGLE_30: f32 = PI_6;
    pub const ANGLE_45: f32 = PI_4;
    pub const ANGLE_60: f32 = PI_3;
    pub const ANGLE_90: f32 = PI_2;
    pub const ANGLE_180: f32 = PI;
    pub const ANGLE_270: f32 = 3.0 * PI_2;
    pub const ANGLE_360: f32 = TAU;
}

/// 高精度CAD処理用の定数（f64）
///
/// 精密計算、科学技術計算、CAD/CAM処理に最適です。
/// 製図・設計など高精度が要求される計算で使用されます。
pub mod precision {
    /// 数学定数（f64）
    pub const PI: f64 = std::f64::consts::PI;
    pub const TAU: f64 = std::f64::consts::TAU;
    pub const E: f64 = std::f64::consts::E;

    /// 円・円弧関連の定数（f64）
    pub const PI_2: f64 = PI / 2.0; // π/2 (90度)
    pub const PI_4: f64 = PI / 4.0; // π/4 (45度)
    pub const PI_3: f64 = PI / 3.0; // π/3 (60度)
    pub const PI_6: f64 = PI / 6.0; // π/6 (30度)

    /// 角度変換定数（f64）
    pub const DEG_TO_RAD: f64 = PI / 180.0;
    pub const RAD_TO_DEG: f64 = 180.0 / PI;

    /// 幾何計算用の許容誤差（f64用）
    pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;

    /// よく使われる角度（f64）
    pub const ANGLE_0: f64 = 0.0;
    pub const ANGLE_30: f64 = PI_6;
    pub const ANGLE_45: f64 = PI_4;
    pub const ANGLE_60: f64 = PI_3;
    pub const ANGLE_90: f64 = PI_2;
    pub const ANGLE_180: f64 = PI;
    pub const ANGLE_270: f64 = 3.0 * PI_2;
    pub const ANGLE_360: f64 = TAU;
}

/// 後方互換性のために既存の定数を維持（precision モジュールのエイリアス）
pub const PI: f64 = precision::PI;
pub const TAU: f64 = precision::TAU;
pub const E: f64 = precision::E;
pub const PI_2: f64 = precision::PI_2;
pub const PI_4: f64 = precision::PI_4;
pub const PI_3: f64 = precision::PI_3;
pub const PI_6: f64 = precision::PI_6;
pub const DEG_TO_RAD: f64 = precision::DEG_TO_RAD;
pub const RAD_TO_DEG: f64 = precision::RAD_TO_DEG;
pub const GEOMETRIC_TOLERANCE: f64 = precision::GEOMETRIC_TOLERANCE;
