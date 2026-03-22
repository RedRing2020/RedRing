//! 幾何計算のための許容誤差管理
//!
//! geo_contracts で共有するトレランス設定。

use crate::Scalar;

/// アプリケーション固有の許容誤差設定
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

impl<T: Scalar> Default for ToleranceSettings<T> {
    fn default() -> Self {
        Self::standard()
    }
}

/// `standard` プロファイルの距離トレランスを返す。
pub fn default_distance_tolerance<T: Scalar>() -> T {
    ToleranceSettings::<T>::standard().distance_tolerance
}

/// `standard` プロファイルの角度トレランスを返す。
pub fn default_angle_tolerance<T: Scalar>() -> T {
    ToleranceSettings::<T>::standard().angle_tolerance
}

/// 型に応じた平行判定（外積）誤差閾値を返す。
pub fn default_parallel_cross_error_tolerance<T: Scalar>() -> T {
    T::PARALLEL_CROSS_ERROR_TOLERANCE
}

/// 型に応じた直交判定（内積）誤差閘値を返す。
pub fn default_orthogonality_dot_error_tolerance<T: Scalar>() -> T {
    T::ORTHOGONALITY_DOT_ERROR_TOLERANCE
}

/// 幾何計算コンテキスト
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
