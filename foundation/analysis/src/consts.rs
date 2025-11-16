//! RedRing 数値解析・幾何計算用の統一定数群
//!
//! 用途別にf32/f64の定数を分離して提供し、数値解析と幾何計算の両方をサポートします。
//!
//! # 設計方針と使い分け
//!
//! ## 定数使用の責務分離
//!
//! RedRingでは定数の使用について明確な責務分離を行っています：
//!
//! ### 🔹 **基礎レイヤー（geo_primitives等）**: 標準ライブラリ定数を直接使用
//! ```rust
//! // テストコードやプリミティブ実装では標準定数を直接使用
//! use std::f64::consts::PI;
//! let half_pi = PI / 2.0;  // ✅ 推奨
//! ```
//! **理由**: シンプル性、明確性、依存関係の最小化、パフォーマンス
//!
//! ### 🔹 **アプリケーションレイヤー（analysis等）**: 用途別定数を使用
//! ```rust
//! // CAD/CAM計算では精度・用途別の定数を使用
//! use analysis::precision::PI as PRECISION_PI;  // 高精度計算用
//! use analysis::game::PI as GAME_PI;            // ゲーム用f32
//! ```
//! **理由**: 用途別最適化、許容誤差の統一管理、f32/f64の使い分け
//!
//! ### 🔹 **Scalarトレイト**: 数値定数は含まない
//! ```rust
//! // Scalarは型の基本操作のみを定義
//! trait Scalar: Copy + Clone + PartialEq {
//!     const ZERO: Self;     // ✅ 型の基本要素
//!     const ONE: Self;      // ✅ 型の基本要素
//!     // const PI: Self;    // ❌ 数学定数は含めない
//! }
//! ```
//! **理由**: 責務の明確化、型システムの単純性維持
//!
//! ## この設計の利点
//!
//! - **シンプルな基礎**: テストやプリミティブ実装が軽量
//! - **用途別最適化**: CAD（高精度）とゲーム（高速）の使い分け
//! - **保守性**: 許容誤差や定数の変更が局所化される
//! - **性能**: 不要な抽象化による間接参照を回避
//!
//! ## 実装例の比較
//!
//! ```rust
//! // ❌ すべてを統一しようとすると複雑になる（例示のみ）
//! // impl<T: Scalar> Point2D<T> {
//! //     fn rotate(&self, angle: T) { /* 処理 */ }  // シンプル
//! // }
//!
//! // ✅ レイヤー別に適切な定数を使用
//!
//! // 基礎レイヤー（geo_primitives）
//! use std::f64::consts::PI;
//! // let rotated = point.rotate(center, angle_from_radians(PI / 4.0));
//!
//! // アプリケーションレイヤー（analysis）
//! use analysis::precision::{PI as PRECISION_PI, GEOMETRIC_TOLERANCE};
//! // if angle_diff.abs() < GEOMETRIC_TOLERANCE { /* 処理 */ }
//! ```
//!
//! この設計により、各レイヤーが適切な責務を持ちながら、
//! 全体として一貫性のあるシステムを構築しています。

/// 数値解析専用定数
pub mod numerical {
    /// ニュートン法で微分がゼロとみなされる閾値
    pub const DERIVATIVE_ZERO_THRESHOLD: f64 = 1e-12;
}

/// 特殊数学定数（黄金比、自然対数など）
///
/// 基本的な数学定数（π, e, τ）は game/precision モジュールを使用してください。
/// ここでは数学・科学計算でよく使われる特殊な定数を提供します。
pub mod special {
    /// 黄金比 φ = (1 + √5) / 2 ≈ 1.618033988749...
    ///
    /// 美術、建築、自然界の比例に現れる特別な比率。
    /// フィボナッチ数列の隣接項の比の極限値としても知られています。
    pub const GOLDEN_RATIO_F64: f64 = 1.618033988749894;
    pub const GOLDEN_RATIO_F32: f32 = 1.618033988749894_f32;

    /// 自然対数の底 ln(2) ≈ 0.693147180559...
    ///
    /// 対数計算、指数関数、情報理論で頻繁に使用される定数。
    pub const LN_2_F64: f64 = std::f64::consts::LN_2;
    pub const LN_2_F32: f32 = std::f32::consts::LN_2;

    /// 自然対数の底 ln(10) ≈ 2.302585092994...
    ///
    /// 常用対数と自然対数の変換で使用される定数。
    pub const LN_10_F64: f64 = std::f64::consts::LN_10;
    pub const LN_10_F32: f32 = std::f32::consts::LN_10;

    /// 平方根 √3 ≈ 1.732050807568...
    ///
    /// 正三角形の高さ計算、60度の三角関数値などで使用。
    pub const SQRT_3_F64: f64 = 1.7320508075688772;
    pub const SQRT_3_F32: f32 = 1.7320508075688772_f32;

    // 注意: ジェネリック版は必要に応じて analysis::abstract_types::Scalar トレイトで実装可能
}

/// 幾何計算で使用される定数
///
/// 用途別にf32/f64の定数を分離して提供します:
/// - `game`: 高速ゲーム処理用（f32）
/// - `precision`: 高精度CAD処理用（f64）
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

    /// 距離計算用の許容誤差（f32用）
    pub const GEOMETRIC_DISTANCE_TOLERANCE: f32 = 1e-6;

    /// 角度計算用の許容誤差（f32用、ラジアン）
    pub const GEOMETRIC_ANGLE_TOLERANCE: f32 = 1e-6;

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

    /// 距離計算用の許容誤差（f64用）
    pub const GEOMETRIC_DISTANCE_TOLERANCE: f64 = 1e-10;

    /// 角度計算用の許容誤差（f64用、ラジアン）
    pub const GEOMETRIC_ANGLE_TOLERANCE: f64 = 1e-12;

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
pub const GEOMETRIC_DISTANCE_TOLERANCE: f64 = precision::GEOMETRIC_DISTANCE_TOLERANCE;
pub const GEOMETRIC_ANGLE_TOLERANCE: f64 = precision::GEOMETRIC_ANGLE_TOLERANCE;

/// 型に応じた適切な許容誤差を取得するトレイト
///
/// f32とf64それぞれの精度に適した許容誤差を提供します。
/// 型パラメータ化された幾何計算で使用されます。
///
/// # 設計方針
///
/// - **責務分離**: 数値型（Scalar trait）は機械誤差（EPSILON）のみ保持
/// - **許容誤差統一**: すべての幾何計算用許容誤差はこのtraitで管理
/// - **用途別最適化**: f32 は game モジュール、f64 は precision モジュールの定数を使用
///
/// # 使用パターン
///
/// ## ❌ 避けるべきパターン
/// ```rust
/// // Scalarトレイトに数学定数を含めるのは責務違反
/// trait Scalar {
///     const PI: Self;  // ❌ 数学定数は含めない
/// }
///
/// // テストで独自定数を定義するのは無駄
/// const PI: f64 = std::f64::consts::PI;  // ❌ 標準定数を直接使う
/// ```
///
/// ## ✅ 推奨パターン
/// ```rust
/// // 基礎レイヤー: 標準定数を直接使用
/// use std::f64::consts::PI;
/// let half_pi = PI / 2.0;
///
/// // アプリケーションレイヤー: このtraitで許容誤差を統一
/// use analysis::{GeometricTolerance, Scalar};
/// use std::ops::Sub;
/// fn nearly_equal<T: GeometricTolerance + Scalar + Sub<Output=T> + PartialOrd + Copy>(a: T, b: T) -> bool {
///     (a - b).abs() < T::TOLERANCE
/// }
/// ```
pub trait GeometricTolerance {
    /// その型に適した幾何計算用の許容誤差（汎用）
    const TOLERANCE: Self;

    /// その型に適した距離計算用の許容誤差
    const DISTANCE_TOLERANCE: Self;

    /// その型に適した角度計算用の許容誤差（ラジアン単位）
    const ANGLE_TOLERANCE: Self;
}

impl GeometricTolerance for f32 {
    const TOLERANCE: f32 = game::GEOMETRIC_TOLERANCE;
    const DISTANCE_TOLERANCE: f32 = game::GEOMETRIC_DISTANCE_TOLERANCE;
    const ANGLE_TOLERANCE: f32 = game::GEOMETRIC_ANGLE_TOLERANCE;
}

impl GeometricTolerance for f64 {
    const TOLERANCE: f64 = precision::GEOMETRIC_TOLERANCE;
    const DISTANCE_TOLERANCE: f64 = precision::GEOMETRIC_DISTANCE_TOLERANCE;
    const ANGLE_TOLERANCE: f64 = precision::GEOMETRIC_ANGLE_TOLERANCE;
}

// 数値解析専用定数の再エクスポート（後方互換性）
pub const DERIVATIVE_ZERO_THRESHOLD: f64 = numerical::DERIVATIVE_ZERO_THRESHOLD;

/// テストで使用する統一された許容誤差定数
///
/// テストコードでの使用を推奨します。
/// 型に応じた適切な許容誤差が自動的に選択されます。
///
/// # 使用指針
///
/// ## 基礎レイヤーのテスト（geo_primitives等）
/// ```rust
/// // 標準定数を直接使用（推奨）
/// use std::f64::consts::PI;
/// let result = PI / 2.0;
/// assert!((result - PI/2.0).abs() < 1e-10);
/// ```
///
/// ## アプリケーションレイヤーのテスト（analysis等）
/// ```rust
/// // このモジュールの定数を使用
/// use analysis::test_constants::TOLERANCE_F64;
/// let result: f64 = 1.5;
/// let expected: f64 = 1.5;
/// assert!((result - expected).abs() < TOLERANCE_F64);
/// ```
///
/// ## 型パラメータ化されたテスト
/// ```rust
/// use analysis::{GeometricTolerance, Scalar};
/// fn test_generic<T: GeometricTolerance + Scalar>(value: T) {
///     let diff = value;  // 例として
///     assert!(diff.abs() < T::TOLERANCE);  // trait経由で使用
/// }
/// ```
pub mod test_constants {
    use super::GeometricTolerance;

    /// f64 テスト用の標準許容誤差
    pub const TOLERANCE_F64: f64 = <f64 as GeometricTolerance>::TOLERANCE;

    /// f32 テスト用の標準許容誤差
    pub const TOLERANCE_F32: f32 = <f32 as GeometricTolerance>::TOLERANCE;

    /// 角度テスト用の許容誤差（f64）
    pub const ANGLE_TOLERANCE_F64: f64 = <f64 as GeometricTolerance>::ANGLE_TOLERANCE;

    /// 角度テスト用の許容誤差（f32）
    pub const ANGLE_TOLERANCE_F32: f32 = <f32 as GeometricTolerance>::ANGLE_TOLERANCE;

    /// 距離テスト用の許容誤差（f64）
    pub const DISTANCE_TOLERANCE_F64: f64 = <f64 as GeometricTolerance>::DISTANCE_TOLERANCE;

    /// 距離テスト用の許容誤差（f32）
    pub const DISTANCE_TOLERANCE_F32: f32 = <f32 as GeometricTolerance>::DISTANCE_TOLERANCE;
}
