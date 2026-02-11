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
/// Phase 1では基本的な3種類に対応します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    /// フラットエンドミル
    ///
    /// 底面が平らな標準的なエンドミル。2D輪郭加工や面削り用。
    /// コーナーR = 0
    FlatEndMill,

    /// ラジアスエンドミル
    ///
    /// 底面コーナーにRがついたエンドミル。仕上げ面品質向上用。
    /// コーナーR = 指定値（通常 R0.5〜R3）
    RadiusEndMill,

    /// ボールエンドミル
    ///
    /// 底面が球状のエンドミル。3D曲面加工用。
    /// コーナーR = 工具半径（R = 直径/2）
    BallEndMill,
}

/// CAM工具定義
///
/// NC加工で使用する工具の基本パラメータを定義します。
///
/// # 内部表現
///
/// 切削シミュレーションでは半径を直接使用するため、内部的には
/// `radius` と `corner_radius` を保持します。UIでは `diameter` で
/// 指定可能です。
///
/// # 注意
///
/// Phase 1では基本的な工具パラメータのみをサポートします。
/// オフセット補正、工具寿命管理、切削条件などは後のPhaseで実装予定です。
#[derive(Debug, Clone, PartialEq)]
pub struct Tool<T: Scalar = f64> {
    /// 工具識別子（例: "EM10", "BEM6", "REM3R1"）
    pub id: String,

    /// 工具種別
    pub tool_type: ToolType,

    /// 工具半径（mm）
    ///
    /// 内部データとして半径を保持。切削計算で直接使用。
    radius: T,

    /// 先端コーナーR（mm）
    ///
    /// - フラットエンドミル: 0
    /// - ラジアスエンドミル: 指定値（R0.5〜R3など）
    /// - ボールエンドミル: radius と同値
    corner_radius: T,

    /// 刃長（mm）
    ///
    /// 工具の有効切削長さ。
    pub cutting_length: T,
}

impl<T: Scalar> Tool<T> {
    /// 新しい工具を作成（直径指定）
    ///
    /// UIフレンドリーなコンストラクタ。工具径は直径で指定します。
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
    /// // フラットエンドミル（コーナーR = 0）
    /// let flat = Tool::new(
    ///     "EM10".to_string(),
    ///     ToolType::FlatEndMill,
    ///     10.0,
    ///     50.0,
    /// );
    ///
    /// // ボールエンドミル（コーナーR = 半径）
    /// let ball = Tool::new(
    ///     "BEM6".to_string(),
    ///     ToolType::BallEndMill,
    ///     6.0,
    ///     30.0,
    /// );
    /// ```
    pub fn new(id: String, tool_type: ToolType, diameter: T, cutting_length: T) -> Self {
        let radius = diameter / T::from_f64(2.0);
        let corner_radius = match tool_type {
            ToolType::FlatEndMill => T::from_f64(0.0),
            ToolType::BallEndMill => radius,
            ToolType::RadiusEndMill => T::from_f64(0.0), // デフォルト、with_corner_radius() で指定
        };
        Self {
            id,
            tool_type,
            radius,
            corner_radius,
            cutting_length,
        }
    }

    /// ラジアスエンドミルを作成（コーナーR指定）
    ///
    /// # 引数
    ///
    /// - `id`: 工具識別子
    /// - `diameter`: 工具径（mm）
    /// - `corner_radius`: コーナーR（mm）
    /// - `cutting_length`: 刃長（mm）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::Tool;
    ///
    /// // 直径10mm、コーナーR1のラジアスエンドミル
    /// let radius_mill = Tool::radius_end_mill(
    ///     "REM10R1".to_string(),
    ///     10.0,
    ///     1.0,
    ///     50.0,
    /// );
    /// assert_eq!(radius_mill.corner_radius(), 1.0);
    /// ```
    pub fn radius_end_mill(
        id: String,
        diameter: T,
        corner_radius: T,
        cutting_length: T,
    ) -> Self {
        let radius = diameter / T::from_f64(2.0);
        Self {
            id,
            tool_type: ToolType::RadiusEndMill,
            radius,
            corner_radius,
            cutting_length,
        }
    }

    /// 工具半径を取得
    ///
    /// 内部データとして保持しているため、計算コストなし。
    ///
    /// # 戻り値
    ///
    /// 工具半径（mm）
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
        self.radius
    }

    /// 工具径を取得
    ///
    /// UIでの表示用。内部の半径から計算します。
    ///
    /// # 戻り値
    ///
    /// 工具径（mm）= `radius * 2`
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{Tool, ToolType};
    ///
    /// let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
    /// assert_eq!(tool.diameter(), 10.0);
    /// ```
    pub fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    /// 先端コーナーRを取得
    ///
    /// # 戻り値
    ///
    /// - フラットエンドミル: 0
    /// - ラジアスエンドミル: 指定値
    /// - ボールエンドミル: 工具半径
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{Tool, ToolType};
    ///
    /// let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
    /// assert_eq!(flat.corner_radius(), 0.0);
    ///
    /// let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 30.0);
    /// assert_eq!(ball.corner_radius(), 3.0); // 半径
    ///
    /// let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    /// assert_eq!(radius.corner_radius(), 1.0);
    /// ```
    pub fn corner_radius(&self) -> T {
        self.corner_radius
    }

    /// フラットエンドミルかどうか判定
    pub fn is_flat_end_mill(&self) -> bool {
        self.tool_type == ToolType::FlatEndMill
    }

    /// ラジアスエンドミルかどうか判定
    pub fn is_radius_end_mill(&self) -> bool {
        self.tool_type == ToolType::RadiusEndMill
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
        assert_eq!(tool.diameter(), 10.0);
        assert_eq!(tool.radius(), 5.0);
        assert_eq!(tool.corner_radius(), 0.0);
        assert_eq!(tool.cutting_length, 50.0);
    }

    #[test]
    fn test_radius_and_diameter() {
        let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
        assert_eq!(tool.radius(), 5.0);
        assert_eq!(tool.diameter(), 10.0);
        
        // 半径は内部データなので計算コストなし
        let r1 = tool.radius();
        let r2 = tool.radius();
        assert_eq!(r1, r2);
    }

    #[test]
    fn test_flat_end_mill() {
        let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
        assert!(flat.is_flat_end_mill());
        assert!(!flat.is_radius_end_mill());
        assert!(!flat.is_ball_end_mill());
        assert_eq!(flat.corner_radius(), 0.0);
    }

    #[test]
    fn test_ball_end_mill() {
        let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 30.0);
        assert!(!ball.is_flat_end_mill());
        assert!(!ball.is_radius_end_mill());
        assert!(ball.is_ball_end_mill());
        assert_eq!(ball.radius(), 3.0);
        assert_eq!(ball.corner_radius(), 3.0); // ボールエンドミルはコーナーR = 半径
    }

    #[test]
    fn test_radius_end_mill() {
        let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
        assert_eq!(radius.id, "REM10R1");
        assert_eq!(radius.tool_type, ToolType::RadiusEndMill);
        assert_eq!(radius.diameter(), 10.0);
        assert_eq!(radius.radius(), 5.0);
        assert_eq!(radius.corner_radius(), 1.0);
        assert_eq!(radius.cutting_length, 50.0);
        assert!(radius.is_radius_end_mill());
        assert!(!radius.is_flat_end_mill());
        assert!(!radius.is_ball_end_mill());
    }

    #[test]
    fn test_corner_radius_consistency() {
        // フラット: R=0
        let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
        assert_eq!(flat.corner_radius(), 0.0);

        // ボール: R=半径
        let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 30.0);
        assert_eq!(ball.corner_radius(), ball.radius());

        // ラジアス: R=指定値
        let radius = Tool::radius_end_mill("REM8R2".to_string(), 8.0, 2.0, 40.0);
        assert_eq!(radius.corner_radius(), 2.0);
    }

    #[test]
    fn test_tool_type_checks() {
        let flat = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 50.0);
        assert!(flat.is_flat_end_mill());
        assert!(!flat.is_ball_end_mill());
        assert!(!flat.is_radius_end_mill());

        let ball = Tool::new("BEM6".to_string(), ToolType::BallEndMill, 6.0, 30.0);
        assert!(!ball.is_flat_end_mill());
        assert!(ball.is_ball_end_mill());
        assert!(!ball.is_radius_end_mill());

        let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
        assert!(!radius.is_flat_end_mill());
        assert!(!radius.is_ball_end_mill());
        assert!(radius.is_radius_end_mill());
    }
}
