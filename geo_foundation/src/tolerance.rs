//! 幾何計算のための許容誤差管理
//!
//! アプリケーションレベルでの許容誤差制御を提供

use crate::Scalar;

/// アプリケーション固有の許容誤差設定
///
/// 各種幾何計算の許容誤差をドメイン要求に応じて設定
#[derive(Debug, Clone, Copy)]
pub struct ToleranceSettings<T: Scalar> {
    /// 距離計算用の許容誤差（点の包含判定、距離比較など）
    pub distance_tolerance: T,

    /// 角度計算用の許容誤差（平行・垂直判定など）
    pub angle_tolerance: T,

    /// 面積計算用の許容誤差
    pub area_tolerance: T,

    /// 長さ計算用の許容誤差
    pub length_tolerance: T,
}

impl<T: Scalar> ToleranceSettings<T> {
    /// 高精度設定（CAD/精密加工用）
    pub fn precision() -> Self {
        Self {
            distance_tolerance: T::from_f64(1e-12),
            angle_tolerance: T::from_f64(1e-10),
            area_tolerance: T::from_f64(1e-10),
            length_tolerance: T::from_f64(1e-12),
        }
    }

    /// 標準設定（一般的な工学計算用）
    pub fn standard() -> Self {
        Self {
            distance_tolerance: T::from_f64(1e-6),
            angle_tolerance: T::from_f64(1e-4),
            area_tolerance: T::from_f64(1e-6),
            length_tolerance: T::from_f64(1e-6),
        }
    }

    /// 緩い設定（ゲーム・リアルタイム用）
    pub fn relaxed() -> Self {
        Self {
            distance_tolerance: T::from_f64(1e-3),
            angle_tolerance: T::from_f64(1e-2),
            area_tolerance: T::from_f64(1e-3),
            length_tolerance: T::from_f64(1e-3),
        }
    }

    /// カスタム設定
    pub fn custom(distance: T, angle: T, area: T, length: T) -> Self {
        Self {
            distance_tolerance: distance,
            angle_tolerance: angle,
            area_tolerance: area,
            length_tolerance: length,
        }
    }
}

/// デフォルトは標準設定
impl<T: Scalar> Default for ToleranceSettings<T> {
    fn default() -> Self {
        Self::standard()
    }
}

/// 幾何計算コンテキスト
///
/// 計算に必要な許容誤差設定を保持し、一貫した判定を提供
#[derive(Debug, Clone, Copy)]
pub struct GeometryContext<T: Scalar> {
    pub tolerances: ToleranceSettings<T>,
}

impl<T: Scalar> GeometryContext<T> {
    /// 新しいコンテキストを作成
    pub fn new(tolerances: ToleranceSettings<T>) -> Self {
        Self { tolerances }
    }

    /// 標準コンテキスト
    pub fn standard() -> Self {
        Self::new(ToleranceSettings::standard())
    }

    /// 高精度コンテキスト
    pub fn precision() -> Self {
        Self::new(ToleranceSettings::precision())
    }

    /// 緩いコンテキスト
    pub fn relaxed() -> Self {
        Self::new(ToleranceSettings::relaxed())
    }
}

impl<T: Scalar> Default for GeometryContext<T> {
    fn default() -> Self {
        Self::standard()
    }
}
