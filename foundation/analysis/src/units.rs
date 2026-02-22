//! 単位系定義
//!
//! 物理量の単位変換とトレランス管理を提供します。
//! このモジュールはアプリケーション非依存の汎用的な単位系実装です。

/// 単位変換係数
mod conversion {
    /// ミリメートル基準値
    pub const MM_TO_MM_FACTOR: f64 = 1.0;

    /// メートル → ミリメートル変換係数
    pub const METER_TO_MM_FACTOR: f64 = 1000.0;

    /// センチメートル → ミリメートル変換係数
    pub const CM_TO_MM_FACTOR: f64 = 10.0;

    /// インチ → ミリメートル変換係数 (1 inch = 25.4 mm)
    pub const INCH_TO_MM_FACTOR: f64 = 25.4;
}

/// デフォルトトレランス (ミリメートル単位)
const DEFAULT_TOLERANCE_MM: f64 = 0.01;

/// 長さの単位系
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnit {
    /// ミリメートル
    Millimeter,
    /// メートル
    Meter,
    /// センチメートル
    Centimeter,
    /// インチ
    Inch,
}

impl LengthUnit {
    /// 基準単位（ミリメートル）への変換係数
    ///
    /// 全ての単位をミリメートルに統一することで、
    /// 浮動小数点誤差を最小化し、数値計算の安定性を向上させます。
    ///
    /// # Examples
    ///
    /// ```
    /// # use analysis::units::LengthUnit;
    /// assert_eq!(LengthUnit::Millimeter.to_millimeter_factor(), 1.0);
    /// assert_eq!(LengthUnit::Meter.to_millimeter_factor(), 1000.0);
    /// assert_eq!(LengthUnit::Centimeter.to_millimeter_factor(), 10.0);
    /// assert_eq!(LengthUnit::Inch.to_millimeter_factor(), 25.4);
    /// ```
    #[must_use]
    pub const fn to_millimeter_factor(&self) -> f64 {
        match self {
            Self::Millimeter => conversion::MM_TO_MM_FACTOR,
            Self::Meter => conversion::METER_TO_MM_FACTOR,
            Self::Centimeter => conversion::CM_TO_MM_FACTOR,
            Self::Inch => conversion::INCH_TO_MM_FACTOR,
        }
    }

    /// 単位の文字列表現
    ///
    /// # Examples
    ///
    /// ```
    /// # use analysis::units::LengthUnit;
    /// assert_eq!(LengthUnit::Millimeter.as_str(), "mm");
    /// assert_eq!(LengthUnit::Meter.as_str(), "m");
    /// assert_eq!(LengthUnit::Centimeter.as_str(), "cm");
    /// assert_eq!(LengthUnit::Inch.as_str(), "in");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Millimeter => "mm",
            Self::Meter => "m",
            Self::Centimeter => "cm",
            Self::Inch => "in",
        }
    }

    /// ある単位から別の単位への変換係数を取得
    ///
    /// # Arguments
    ///
    /// * `to` - 変換先の単位
    ///
    /// # Examples
    ///
    /// ```
    /// # use analysis::units::LengthUnit;
    /// // メートルからミリメートル: 1m = 1000mm
    /// assert_eq!(LengthUnit::Meter.conversion_factor_to(LengthUnit::Millimeter), 1000.0);
    ///
    /// // インチからミリメートル: 1in = 25.4mm
    /// assert_eq!(LengthUnit::Inch.conversion_factor_to(LengthUnit::Millimeter), 25.4);
    ///
    /// // 同じ単位: 係数は1.0
    /// assert_eq!(LengthUnit::Meter.conversion_factor_to(LengthUnit::Meter), 1.0);
    /// ```
    #[must_use]
    pub const fn conversion_factor_to(&self, to: Self) -> f64 {
        self.to_millimeter_factor() / to.to_millimeter_factor()
    }
}

/// 表示トレランス設定
///
/// 数値計算における許容誤差を単位付きで管理します。
/// 主に曲線のテッセレーション（分割）や近似計算で使用されます。
#[derive(Debug, Clone, Copy)]
pub struct Tolerance {
    /// トレランス値（指定された単位での値）
    pub value: f64,
    /// 単位系
    pub unit: LengthUnit,
}

impl Tolerance {
    /// 新しいトレランスを作成
    ///
    /// # Arguments
    ///
    /// * `value` - トレランス値
    /// * `unit` - 単位系
    ///
    /// # Examples
    ///
    /// ```
    /// # use analysis::units::{Tolerance, LengthUnit};
    /// let tol = Tolerance::new(0.01, LengthUnit::Millimeter);
    /// assert_eq!(tol.value, 0.01);
    /// assert_eq!(tol.unit, LengthUnit::Millimeter);
    /// ```
    #[must_use]
    pub const fn new(value: f64, unit: LengthUnit) -> Self {
        Self { value, unit }
    }

    /// ミリメートル単位での値を取得
    ///
    /// 基準単位への統一により、単位変換の誤差を最小化します。
    ///
    /// # Examples
    ///
    /// ```
    /// # use analysis::units::{Tolerance, LengthUnit};
    /// let tol_mm = Tolerance::new(1.0, LengthUnit::Millimeter);
    /// assert_eq!(tol_mm.in_millimeters(), 1.0);
    ///
    /// let tol_m = Tolerance::new(0.001, LengthUnit::Meter);
    /// assert_eq!(tol_m.in_millimeters(), 1.0);
    ///
    /// let tol_cm = Tolerance::new(0.1, LengthUnit::Centimeter);
    /// assert_eq!(tol_cm.in_millimeters(), 1.0);
    /// ```
    #[must_use]
    pub fn in_millimeters(&self) -> f64 {
        self.value * self.unit.to_millimeter_factor()
    }

    /// 指定された単位での値を取得
    ///
    /// # Arguments
    ///
    /// * `target_unit` - 変換先の単位
    ///
    /// # Examples
    ///
    /// ```
    /// # use analysis::units::{Tolerance, LengthUnit};
    /// let tol = Tolerance::new(1.0, LengthUnit::Millimeter);
    ///
    /// assert_eq!(tol.in_unit(LengthUnit::Millimeter), 1.0);
    /// assert_eq!(tol.in_unit(LengthUnit::Meter), 0.001);
    /// assert_eq!(tol.in_unit(LengthUnit::Centimeter), 0.1);
    /// ```
    #[must_use]
    pub fn in_unit(&self, target_unit: LengthUnit) -> f64 {
        let mm_value = self.in_millimeters();
        mm_value / target_unit.to_millimeter_factor()
    }
}

impl Default for Tolerance {
    /// デフォルトトレランス: 0.01 mm
    ///
    /// 精密な幾何計算に適した値です。
    fn default() -> Self {
        Self::new(DEFAULT_TOLERANCE_MM, LengthUnit::Millimeter)
    }
}

#[cfg(test)]
#[path = "units_tests.rs"]
mod units_tests;
