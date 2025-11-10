//! NURBS計算のエラー型定義

use thiserror::Error;

/// NURBS計算で発生するエラー
#[derive(Error, Debug, Clone, PartialEq)]
pub enum NurbsError {
    /// 無効な次数
    #[error("無効な次数: {degree}. 次数は0以上 {max_degree} 以下である必要があります")]
    InvalidDegree {
        /// 指定された次数
        degree: usize,
        /// 最大許可次数
        max_degree: usize,
    },

    /// 制御点数が不足
    #[error("制御点数が不足: {actual}個. 次数{degree}には最低{required}個必要です")]
    InsufficientControlPoints {
        /// 実際の制御点数
        actual: usize,
        /// 必要な制御点数
        required: usize,
        /// NURBS次数
        degree: usize,
    },

    /// 無効なノットベクトル
    #[error("無効なノットベクトル: {reason}")]
    InvalidKnotVector {
        /// エラーの理由
        reason: String,
    },

    /// 重みベクトルの長さが不正
    #[error("重みベクトルの長さが不正: {actual}個. 制御点数{expected}個と一致する必要があります")]
    WeightCountMismatch {
        /// 実際の重み数
        actual: usize,
        /// 期待される重み数（制御点数と同じ）
        expected: usize,
    },

    /// 無効な重み値
    #[error("無効な重み値: {weight}. 重みは正の値である必要があります")]
    InvalidWeight {
        /// 無効な重み値
        weight: f64,
    },

    /// パラメータが範囲外
    #[error("パラメータが範囲外: {parameter}. 有効範囲は[{min}, {max}]です")]
    ParameterOutOfRange {
        /// 指定されたパラメータ
        parameter: f64,
        /// 最小値
        min: f64,
        /// 最大値
        max: f64,
    },

    /// 数値計算エラー
    #[error("数値計算エラー: {message}")]
    NumericalError {
        /// エラーメッセージ
        message: String,
    },

    /// 退化した幾何要素
    #[error("退化した幾何要素: {reason}")]
    DegenerateGeometry {
        /// 退化の理由
        reason: String,
    },

    /// 互換性のない操作
    #[error("互換性のない操作: {operation}. 理由: {reason}")]
    IncompatibleOperation {
        /// 試行された操作
        operation: String,
        /// 非互換の理由
        reason: String,
    },
}

/// NURBS計算の結果型
pub type Result<T> = std::result::Result<T, NurbsError>;

impl NurbsError {
    /// 無効なノットベクトルエラーを作成
    pub fn invalid_knot_vector<S: Into<String>>(reason: S) -> Self {
        Self::InvalidKnotVector {
            reason: reason.into(),
        }
    }

    /// 数値計算エラーを作成
    pub fn numerical_error<S: Into<String>>(message: S) -> Self {
        Self::NumericalError {
            message: message.into(),
        }
    }

    /// 退化した幾何要素エラーを作成
    pub fn degenerate_geometry<S: Into<String>>(reason: S) -> Self {
        Self::DegenerateGeometry {
            reason: reason.into(),
        }
    }

    /// 互換性のない操作エラーを作成
    pub fn incompatible_operation<S: Into<String>, R: Into<String>>(
        operation: S,
        reason: R,
    ) -> Self {
        Self::IncompatibleOperation {
            operation: operation.into(),
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = NurbsError::InvalidDegree {
            degree: 15,
            max_degree: 10,
        };
        assert!(error.to_string().contains("15"));
        assert!(error.to_string().contains("10"));
    }

    #[test]
    fn test_helper_functions() {
        let error = NurbsError::invalid_knot_vector("非単調");
        if let NurbsError::InvalidKnotVector { reason } = error {
            assert_eq!(reason, "非単調");
        } else {
            panic!("Expected InvalidKnotVector error");
        }
    }

    #[test]
    fn test_error_equality() {
        let error1 = NurbsError::InvalidDegree {
            degree: 5,
            max_degree: 3,
        };
        let error2 = NurbsError::InvalidDegree {
            degree: 5,
            max_degree: 3,
        };
        let error3 = NurbsError::InvalidDegree {
            degree: 6,
            max_degree: 3,
        };

        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
    }
}
