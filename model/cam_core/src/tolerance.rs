//! CAM用トレランス管理
//!
//! このモジュールは、CAM演算における数値誤差許容値（トレランス）を管理します。
//!
//! # 概要
//!
//! CAMでは実際の機械加工を想定するため、以下のトレランスが必要です：
//!
//! - **閉曲線判定トレランス**: 始点と終点の距離がこの値以内なら閉じていると判定
//! - **工具クリアランス比率**: 工具径に対する最小クリアランス（干渉回避）
//! - **機械精度**: NC機械の位置決め精度
//!
//! # 例
//!
//! ```
//! use cam_core::CamTolerance;
//!
//! // デフォルト値（推奨）
//! let tolerance = CamTolerance::<f64>::default();
//! assert_eq!(tolerance.closure_tolerance, 0.001);
//!
//! // カスタム設定
//! let custom = CamTolerance {
//!     closure_tolerance: 0.0001,  // 高精度機械
//!     tool_clearance_ratio: 0.05,  // タイトなクリアランス
//!     machine_accuracy: 0.001,
//! };
//! ```

use analysis::Scalar;

pub const CAM_DEFAULT_CLOSURE_TOLERANCE_F64: f64 = 0.001;
pub const CAM_DEFAULT_TOOL_CLEARANCE_RATIO_F64: f64 = 0.1;
pub const CAM_DEFAULT_MACHINE_ACCURACY_F64: f64 = 0.01;

pub const CAM_HIGH_PRECISION_CLOSURE_TOLERANCE_F64: f64 = 0.0001;
pub const CAM_HIGH_PRECISION_TOOL_CLEARANCE_RATIO_F64: f64 = 0.05;
pub const CAM_HIGH_PRECISION_MACHINE_ACCURACY_F64: f64 = 0.001;

pub const CAM_LOW_PRECISION_CLOSURE_TOLERANCE_F64: f64 = 0.01;
pub const CAM_LOW_PRECISION_TOOL_CLEARANCE_RATIO_F64: f64 = 0.2;
pub const CAM_LOW_PRECISION_MACHINE_ACCURACY_F64: f64 = 0.1;

/// CAM用トレランス設定
///
/// CAM演算における数値誤差許容値を管理します。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CamTolerance<T: Scalar = f64> {
    /// 閉曲線判定トレランス（mm）
    ///
    /// 2D輪郭の始点と終点の距離がこの値以内であれば、
    /// 閉じた曲線として扱います。
    ///
    /// **推奨値**: `0.001` mm（一般的なNC機械）
    pub closure_tolerance: T,

    /// 工具径に対する最小クリアランス比率
    ///
    /// 工具径 × この比率 = 最小クリアランス
    ///
    /// **例**: 工具径10mm、比率0.1 → 最小クリアランス1mm
    ///
    /// **推奨値**: `0.1`（工具径の10%）
    pub tool_clearance_ratio: T,

    /// 機械精度（mm）
    ///
    /// NC機械の位置決め精度。この値以下の差異は無視されます。
    ///
    /// **推奨値**:
    /// - 一般的なNCフライス: `0.01` mm
    /// - 高精度機: `0.001` mm
    pub machine_accuracy: T,
}

impl<T: Scalar> Default for CamTolerance<T> {
    /// デフォルトのトレランス値
    ///
    /// - `closure_tolerance`: 0.001 mm
    /// - `tool_clearance_ratio`: 0.1（10%）
    /// - `machine_accuracy`: 0.01 mm（一般的なNCフライス）
    fn default() -> Self {
        Self {
            closure_tolerance: T::from_f64(CAM_DEFAULT_CLOSURE_TOLERANCE_F64),
            tool_clearance_ratio: T::from_f64(CAM_DEFAULT_TOOL_CLEARANCE_RATIO_F64),
            machine_accuracy: T::from_f64(CAM_DEFAULT_MACHINE_ACCURACY_F64),
        }
    }
}

impl<T: Scalar> CamTolerance<T> {
    /// 新しいトレランス設定を作成
    ///
    /// # 引数
    ///
    /// - `closure_tolerance`: 閉曲線判定トレランス（mm）
    /// - `tool_clearance_ratio`: 工具クリアランス比率
    /// - `machine_accuracy`: 機械精度（mm）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::CamTolerance;
    ///
    /// let tolerance = CamTolerance::new(0.001, 0.1, 0.01);
    /// ```
    pub fn new(closure_tolerance: T, tool_clearance_ratio: T, machine_accuracy: T) -> Self {
        Self {
            closure_tolerance,
            tool_clearance_ratio,
            machine_accuracy,
        }
    }

    /// 高精度機械用のトレランス設定
    ///
    /// - `closure_tolerance`: 0.0001 mm
    /// - `tool_clearance_ratio`: 0.05（5%）
    /// - `machine_accuracy`: 0.001 mm
    pub fn high_precision() -> Self {
        Self {
            closure_tolerance: T::from_f64(CAM_HIGH_PRECISION_CLOSURE_TOLERANCE_F64),
            tool_clearance_ratio: T::from_f64(CAM_HIGH_PRECISION_TOOL_CLEARANCE_RATIO_F64),
            machine_accuracy: T::from_f64(CAM_HIGH_PRECISION_MACHINE_ACCURACY_F64),
        }
    }

    /// 低精度機械用のトレランス設定
    ///
    /// - `closure_tolerance`: 0.01 mm
    /// - `tool_clearance_ratio`: 0.2（20%）
    /// - `machine_accuracy`: 0.1 mm
    pub fn low_precision() -> Self {
        Self {
            closure_tolerance: T::from_f64(CAM_LOW_PRECISION_CLOSURE_TOLERANCE_F64),
            tool_clearance_ratio: T::from_f64(CAM_LOW_PRECISION_TOOL_CLEARANCE_RATIO_F64),
            machine_accuracy: T::from_f64(CAM_LOW_PRECISION_MACHINE_ACCURACY_F64),
        }
    }

    /// 工具径から最小クリアランスを計算
    ///
    /// # 引数
    ///
    /// - `tool_diameter`: 工具径（mm）
    ///
    /// # 戻り値
    ///
    /// 最小クリアランス（mm） = `tool_diameter * tool_clearance_ratio`
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::CamTolerance;
    ///
    /// let tolerance = CamTolerance::default();
    /// let clearance = tolerance.min_clearance(10.0);
    /// assert_eq!(clearance, 1.0);  // 10mm × 0.1 = 1mm
    /// ```
    pub fn min_clearance(&self, tool_diameter: T) -> T {
        tool_diameter * self.tool_clearance_ratio
    }

    /// 2つの値が機械精度内で等しいか判定
    ///
    /// # 引数
    ///
    /// - `a`, `b`: 比較する値
    ///
    /// # 戻り値
    ///
    /// `|a - b| <= machine_accuracy` なら `true`
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::CamTolerance;
    ///
    /// let tolerance = CamTolerance::default();
    /// assert!(tolerance.is_equal_within_accuracy(10.0, 10.005));
    /// assert!(!tolerance.is_equal_within_accuracy(10.0, 10.05));
    /// ```
    pub fn is_equal_within_accuracy(&self, a: T, b: T) -> bool {
        (a - b).abs() <= self.machine_accuracy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tolerance() {
        let tolerance = CamTolerance::<f64>::default();
        assert_eq!(
            tolerance.closure_tolerance,
            CAM_DEFAULT_CLOSURE_TOLERANCE_F64
        );
        assert_eq!(
            tolerance.tool_clearance_ratio,
            CAM_DEFAULT_TOOL_CLEARANCE_RATIO_F64
        );
        assert_eq!(tolerance.machine_accuracy, CAM_DEFAULT_MACHINE_ACCURACY_F64);
    }

    #[test]
    fn test_high_precision() {
        let tolerance = CamTolerance::<f64>::high_precision();
        assert_eq!(
            tolerance.closure_tolerance,
            CAM_HIGH_PRECISION_CLOSURE_TOLERANCE_F64
        );
        assert_eq!(
            tolerance.tool_clearance_ratio,
            CAM_HIGH_PRECISION_TOOL_CLEARANCE_RATIO_F64
        );
        assert_eq!(
            tolerance.machine_accuracy,
            CAM_HIGH_PRECISION_MACHINE_ACCURACY_F64
        );
    }

    #[test]
    fn test_min_clearance() {
        let tolerance = CamTolerance::<f64>::default();
        assert_eq!(tolerance.min_clearance(10.0), 1.0);
        assert_eq!(tolerance.min_clearance(5.0), 0.5);
    }

    #[test]
    fn test_is_equal_within_accuracy() {
        let tolerance = CamTolerance::<f64>::default();
        assert!(tolerance.is_equal_within_accuracy(10.0, 10.0));
        assert!(tolerance.is_equal_within_accuracy(10.0, 10.005));
        assert!(!tolerance.is_equal_within_accuracy(10.0, 10.05));
    }
}
