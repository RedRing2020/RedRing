//! CAM工具定義
//!
//! このモジュールは、NC加工で使用する工具の定義を提供します。
//!
//! # 概要
//!
//! Phase 1では基本的な工具パラメータのみをサポートします：
//!
//! - **フラットエンドミル**: 底面が平らな標準的なエンドミル
//! - **ボールエンドミル**: 底面が球状のエンドミル（3D加工用）
//!
//! # 例
//!
//! ```
//! use cam_core::{Tool, ToolType};
//!
//! // フラットエンドミル（直径10mm）
//! let flat_mill = Tool::new(
//!     "EM10".to_string(),
//!     ToolType::FlatEndMill,
//!     10.0,
//!     50.0,  // 刃長
//! );
//!
//! // ボールエンドミル（直径6mm）
//! let ball_mill = Tool::new(
//!     "BEM6".to_string(),
//!     ToolType::BallEndMill,
//!     6.0,
//!     30.0,
//! );
//! ```

use analysis::Scalar;

/// 工具種別
///
/// Phase 1では基本的な2種類のみ対応します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    /// フラットエンドミル
    ///
    /// 底面が平らな標準的なエンドミル。2D輪郭加工や面削り用。
    FlatEndMill,

    /// ボールエンドミル
    ///
    /// 底面が球状のエンドミル。3D曲面加工用。
    BallEndMill,
}

/// CAM工具定義
///
/// NC加工で使用する工具の基本パラメータを定義します。
///
/// # 注意
///
/// Phase 1では基本的な工具パラメータのみをサポートします。
/// オフセット補正、工具寿命管理、切削条件などは後のPhaseで実装予定です。
#[derive(Debug, Clone, PartialEq)]
pub struct Tool<T: Scalar = f64> {
    /// 工具識別子（例: "EM10", "BEM6"）
    pub id: String,

    /// 工具種別
    pub tool_type: ToolType,

    /// 工具径（mm）
    ///
    /// フラットエンドミルの場合は底面の直径、
    /// ボールエンドミルの場合は球の直径。
    pub diameter: T,

    /// 刃長（mm）
    ///
    /// 工具の有効切削長さ。
    pub cutting_length: T,
}

impl<T: Scalar> Tool<T> {
    /// 新しい工具を作成
    ///
    /// # 引数
    ///
    /// - `id`: 工具識別子
    /// - `tool_type`: 工具種別
    /// - `diameter`: 工具径（mm）
    /// - `cutting_length`: 刃長（mm）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{Tool, ToolType};
    ///
    /// let tool = Tool::new(
    ///     "EM10".to_string(),
    ///     ToolType::FlatEndMill,
    ///     10.0,
    ///     50.0,
    /// );
    /// ```
    pub fn new(id: String, tool_type: ToolType, diameter: T, cutting_length: T) -> Self {
        Self {
            id,
            tool_type,
            diameter,
            cutting_length,
        }
    }

    /// 工具半径を取得
    ///
    /// # 戻り値
    ///
    /// `diameter / 2`
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{Tool, ToolType};
    ///
    /// let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
    /// assert_eq!(tool.radius(), 5.0);
    /// ```
    pub fn radius(&self) -> T {
        self.diameter / T::from_f64(2.0)
    }

    /// フラットエンドミルかどうか判定
    pub fn is_flat_end_mill(&self) -> bool {
        self.tool_type == ToolType::FlatEndMill
    }

    /// ボールエンドミルかどうか判定
    pub fn is_ball_end_mill(&self) -> bool {
        self.tool_type == ToolType::BallEndMill
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);

        assert_eq!(tool.id, "EM10");
        assert_eq!(tool.tool_type, ToolType::FlatEndMill);
        assert_eq!(tool.diameter, 10.0);
        assert_eq!(tool.cutting_length, 50.0);
    }

    #[test]
    fn test_radius() {
        let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
        assert_eq!(tool.radius(), 5.0);
    }

    #[test]
    fn test_tool_type_checks() {
        let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
        assert!(flat.is_flat_end_mill());
        assert!(!flat.is_ball_end_mill());

        let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 30.0);
        assert!(!ball.is_flat_end_mill());
        assert!(ball.is_ball_end_mill());
    }
}
