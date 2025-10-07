//! 数値型対応の設計選択肢比較
//!
//! f32/f64両対応のための4つのアプローチを比較

// ========================================
// 選択肢1: 分離した定数モジュール
// ========================================

pub mod f32_constants {
    /// f32用の数学定数
    pub const PI: f32 = std::f32::consts::PI;
    pub const TAU: f32 = std::f32::consts::TAU;
    pub const E: f32 = std::f32::consts::E;

    /// 角度関連定数
    pub const PI_2: f32 = PI / 2.0;
    pub const PI_4: f32 = PI / 4.0;
    pub const PI_3: f32 = PI / 3.0;
    pub const PI_6: f32 = PI / 6.0;

    /// 変換定数
    pub const DEG_TO_RAD: f32 = PI / 180.0;
    pub const RAD_TO_DEG: f32 = 180.0 / PI;

    /// 許容誤差（f32用）
    pub const GEOMETRIC_TOLERANCE: f32 = 1e-6;
}

pub mod f64_constants {
    /// f64用の数学定数
    pub const PI: f64 = std::f64::consts::PI;
    pub const TAU: f64 = std::f64::consts::TAU;
    pub const E: f64 = std::f64::consts::E;

    /// 角度関連定数
    pub const PI_2: f64 = PI / 2.0;
    pub const PI_4: f64 = PI / 4.0;
    pub const PI_3: f64 = PI / 3.0;
    pub const PI_6: f64 = PI / 6.0;

    /// 変換定数
    pub const DEG_TO_RAD: f64 = PI / 180.0;
    pub const RAD_TO_DEG: f64 = 180.0 / PI;

    /// 許容誤差（f64用）
    pub const GEOMETRIC_TOLERANCE: f64 = 1e-10;
}

// 使用例
fn example_separated_modules() {
    use f32_constants as f32c;
    use f64_constants as f64c;

    let game_circle_area = f32c::PI * radius_f32 * radius_f32;
    let precise_circle_area = f64c::PI * radius_f64 * radius_f64;
}

// ========================================
// 選択肢2: ジェネリック関数アプローチ
// ========================================

pub trait Constants {
    fn pi() -> Self;
    fn tau() -> Self;
    fn e() -> Self;
    fn pi_2() -> Self;
    fn pi_4() -> Self;
    fn deg_to_rad() -> Self;
    fn rad_to_deg() -> Self;
    fn geometric_tolerance() -> Self;
}

impl Constants for f32 {
    fn pi() -> Self {
        std::f32::consts::PI
    }
    fn tau() -> Self {
        std::f32::consts::TAU
    }
    fn e() -> Self {
        std::f32::consts::E
    }
    fn pi_2() -> Self {
        Self::pi() / 2.0
    }
    fn pi_4() -> Self {
        Self::pi() / 4.0
    }
    fn deg_to_rad() -> Self {
        Self::pi() / 180.0
    }
    fn rad_to_deg() -> Self {
        180.0 / Self::pi()
    }
    fn geometric_tolerance() -> Self {
        1e-6
    }
}

impl Constants for f64 {
    fn pi() -> Self {
        std::f64::consts::PI
    }
    fn tau() -> Self {
        std::f64::consts::TAU
    }
    fn e() -> Self {
        std::f64::consts::E
    }
    fn pi_2() -> Self {
        Self::pi() / 2.0
    }
    fn pi_4() -> Self {
        Self::pi() / 4.0
    }
    fn deg_to_rad() -> Self {
        Self::pi() / 180.0
    }
    fn rad_to_deg() -> Self {
        180.0 / Self::pi()
    }
    fn geometric_tolerance() -> Self {
        1e-10
    }
}

// 使用例
fn circle_area<T: Constants + std::ops::Mul<Output = T> + Copy>(radius: T) -> T {
    T::pi() * radius * radius
}

// ========================================
// 選択肢3: マクロによる定数生成
// ========================================

macro_rules! define_constants {
    ($float_type:ty, $tolerance:expr) => {
        pub mod consts {
            pub const PI: $float_type = <$float_type>::consts::PI;
            pub const TAU: $float_type = <$float_type>::consts::TAU;
            pub const E: $float_type = <$float_type>::consts::E;
            pub const PI_2: $float_type = PI / 2.0;
            pub const PI_4: $float_type = PI / 4.0;
            pub const DEG_TO_RAD: $float_type = PI / 180.0;
            pub const RAD_TO_DEG: $float_type = 180.0 / PI;
            pub const GEOMETRIC_TOLERANCE: $float_type = $tolerance;
        }
    };
}

pub mod f32_math {
    define_constants!(f32, 1e-6);
}

pub mod f64_math {
    define_constants!(f64, 1e-10);
}

// ========================================
// 選択肢4: 型エイリアス + 条件付きコンパイル
// ========================================

#[cfg(feature = "f32-precision")]
pub type Float = f32;

#[cfg(not(feature = "f32-precision"))]
pub type Float = f64;

pub mod adaptive_constants {
    use super::Float;

    pub const PI: Float = if cfg!(feature = "f32-precision") {
        std::f32::consts::PI as Float
    } else {
        std::f64::consts::PI as Float
    };

    pub const GEOMETRIC_TOLERANCE: Float = if cfg!(feature = "f32-precision") {
        1e-6 as Float
    } else {
        1e-10 as Float
    };
}

// ========================================
// 実用性とパフォーマンス比較
// ========================================

#[cfg(test)]
mod performance_comparison {
    use super::*;

    // ベンチマーク用のテストケース
    const ITERATIONS: usize = 1_000_000;

    #[test]
    fn benchmark_separated_modules() {
        let radius = 5.0f32;
        let start = std::time::Instant::now();

        for _ in 0..ITERATIONS {
            let _area = f32_constants::PI * radius * radius;
        }

        println!("分離モジュール: {:?}", start.elapsed());
    }

    #[test]
    fn benchmark_trait_approach() {
        let radius = 5.0f32;
        let start = std::time::Instant::now();

        for _ in 0..ITERATIONS {
            let _area = f32::pi() * radius * radius;
        }

        println!("トレイトアプローチ: {:?}", start.elapsed());
    }
}

// ========================================
// 推奨アプローチ: ハイブリッド設計
// ========================================

/// 推奨: 分離モジュール + トレイト補完
pub mod recommended {
    /// 高性能ゲーム用（f32）
    pub mod game {
        pub const PI: f32 = std::f32::consts::PI;
        pub const TAU: f32 = std::f32::consts::TAU;
        pub const PI_2: f32 = PI / 2.0;
        pub const PI_4: f32 = PI / 4.0;
        pub const DEG_TO_RAD: f32 = PI / 180.0;
        pub const RAD_TO_DEG: f32 = 180.0 / PI;
        pub const TOLERANCE: f32 = 1e-6;
    }

    /// 高精度CAD用（f64）
    pub mod precision {
        pub const PI: f64 = std::f64::consts::PI;
        pub const TAU: f64 = std::f64::consts::TAU;
        pub const PI_2: f64 = PI / 2.0;
        pub const PI_4: f64 = PI / 4.0;
        pub const DEG_TO_RAD: f64 = PI / 180.0;
        pub const RAD_TO_DEG: f64 = 180.0 / PI;
        pub const TOLERANCE: f64 = 1e-10;
    }

    /// ジェネリック関数（必要に応じて）
    pub trait MathConstants {
        fn pi() -> Self;
        fn tau() -> Self;
        fn tolerance() -> Self;
    }

    impl MathConstants for f32 {
        fn pi() -> Self {
            game::PI
        }
        fn tau() -> Self {
            game::TAU
        }
        fn tolerance() -> Self {
            game::TOLERANCE
        }
    }

    impl MathConstants for f64 {
        fn pi() -> Self {
            precision::PI
        }
        fn tau() -> Self {
            precision::TAU
        }
        fn tolerance() -> Self {
            precision::TOLERANCE
        }
    }
}

// 使用例
fn usage_examples() {
    use recommended::{game, precision};

    // ゲーム用（高速）
    let game_circle_area = game::PI * 5.0f32 * 5.0f32;
    let game_angle_deg = 90.0f32 * game::DEG_TO_RAD;

    // CAD用（高精度）
    let cad_circle_area = precision::PI * 5.0f64 * 5.0f64;
    let cad_angle_deg = 90.0f64 * precision::DEG_TO_RAD;

    // ジェネリック（必要時）
    fn generic_area<T: MathConstants + std::ops::Mul<Output = T> + Copy>(radius: T) -> T {
        T::pi() * radius * radius
    }
}
