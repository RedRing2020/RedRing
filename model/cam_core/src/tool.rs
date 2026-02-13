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
//! - **ラジアスエンドミル**: 底面コーナーにRがついたエンドミル
//!
//! # 例
//!
//! ```
//! use cam_core::Tool;
//!
//! // フラットエンドミル（直径10mm、R=0）
//! let flat_mill = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
//!
//! // ボールエンドミル（直径6mm、R=半径）
//! let ball_mill = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
//!
//! // ラジアスエンドミル（直径10mm、R=1mm）
//! let radius_mill = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
//! ```

use analysis::Scalar;

/// 工具種別
///
/// corner_radiusの値から自動判定されます。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolType {
    /// フラットエンドミル
    ///
    /// 底面が平らな標準的なエンドミル。2D輪郭加工や面削り用。
    /// 判定条件: corner_radius == 0
    FlatEndMill,

    /// ラジアスエンドミル
    ///
    /// 底面コーナーにRがついたエンドミル。仕上げ面品質向上用。
    /// 判定条件: 0 < corner_radius < radius
    RadiusEndMill,

    /// ボールエンドミル
    ///
    /// 底面が球状のエンドミル。3D曲面加工用。
    /// 判定条件: corner_radius == radius
    BallEndMill,
}

/// 工具経路での参照点
///
/// G-codeや工具経路での座標が工具のどの位置を示すかを定義します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolReferencePoint {
    /// 工具中心（工具軸の中心点）
    ///
    /// - フラットエンドミル: 底面中心
    /// - ボールエンドミル: 球面の中心
    /// - ラジアスエンドミル: 底面平坦部の中心
    Center,

    /// 工具先端（最下点）
    ///
    /// - フラットエンドミル: 底面中心（Centerと同じ）
    /// - ボールエンドミル: 球面の最下点
    /// - ラジアスエンドミル: R部の最下点
    Tip,
}

/// CAM工具定義
///
/// NC加工で使用する工具の基本パラメータを定義します。
///
/// # 設計方針
///
/// 3つの実数パラメータで工具形状を表現します：
/// - **radius**: 工具半径（mm）
/// - **corner_radius**: コーナーR（mm）
/// - **cutting_length**: 工具長（mm）
///
/// 工具種別は `corner_radius` の値から自動判定されます：
/// - フラットエンドミル: `corner_radius == 0`
/// - ボールエンドミル: `corner_radius == radius`
/// - ラジアスエンドミル: `0 < corner_radius < radius`
///
/// # 注意
///
/// Phase 1では基本的な工具パラメータのみをサポートします。
/// オフセット補正、工具寿命管理、切削条件などは後のPhaseで実装予定です。
#[derive(Debug, Clone, PartialEq)]
pub struct Tool<T: Scalar = f64> {
    /// 工具識別子（例: "EM10", "BEM6", "REM3R1"）
    pub id: String,

    /// 工具半径（mm）
    ///
    /// 切削計算で直接使用される基本パラメータ。
    radius: T,

    /// コーナーR（mm）
    ///
    /// - フラットエンドミル: 0
    /// - ラジアスエンドミル: 指定値（0 < R < radius）
    /// - ボールエンドミル: radius（半径と同値）
    corner_radius: T,

    /// 刃長（mm）
    ///
    /// 工具の有効切削長さ。
    pub cutting_length: T,
}

impl<T: Scalar> Tool<T> {
    /// 工具を作成（3つの実数パラメータで定義）
    ///
    /// # 引数
    ///
    /// - `id`: 工具識別子
    /// - `diameter`: 工具径（mm）
    /// - `corner_radius`: コーナーR（mm）
    /// - `cutting_length`: 刃長（mm）
    ///
    /// # 工具種別の自動判定
    ///
    /// - `corner_radius == 0` → フラットエンドミル
    /// - `corner_radius == radius` → ボールエンドミル
    /// - `0 < corner_radius < radius` → ラジアスエンドミル
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::Tool;
    ///
    /// // フラットエンドミル（R=0）
    /// let flat = Tool::new("EM10".to_string(), 10.0, 0.0, 50.0);
    ///
    /// // ボールエンドミル（R=半径）
    /// let ball = Tool::new("BEM6".to_string(), 6.0, 3.0, 30.0);
    ///
    /// // ラジアスエンドミル（0 < R < 半径）
    /// let radius = Tool::new("REM10R1".to_string(), 10.0, 1.0, 50.0);
    /// ```
    pub fn new(id: String, diameter: T, corner_radius: T, cutting_length: T) -> Self {
        let radius = diameter / T::from_f64(2.0);
        Self {
            id,
            radius,
            corner_radius,
            cutting_length,
        }
    }

    /// フラットエンドミルを作成
    ///
    /// # 引数
    ///
    /// - `id`: 工具識別子
    /// - `diameter`: 工具径（mm）
    /// - `cutting_length`: 刃長（mm）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::Tool;
    ///
    /// let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    /// assert_eq!(flat.corner_radius(), 0.0);
    /// ```
    pub fn flat_end_mill(id: String, diameter: T, cutting_length: T) -> Self {
        Self::new(id, diameter, T::from_f64(0.0), cutting_length)
    }

    /// ボールエンドミルを作成
    ///
    /// # 引数
    ///
    /// - `id`: 工具識別子
    /// - `diameter`: 工具径（mm）
    /// - `cutting_length`: 刃長（mm）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::Tool;
    ///
    /// let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    /// assert_eq!(ball.corner_radius(), 3.0);  // 半径
    /// ```
    pub fn ball_end_mill(id: String, diameter: T, cutting_length: T) -> Self {
        let radius = diameter / T::from_f64(2.0);
        Self::new(id, diameter, radius, cutting_length)
    }

    /// ラジアスエンドミルを作成
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
    /// let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    /// assert_eq!(radius.corner_radius(), 1.0);
    /// ```
    pub fn radius_end_mill(id: String, diameter: T, corner_radius: T, cutting_length: T) -> Self {
        Self::new(id, diameter, corner_radius, cutting_length)
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
    /// use cam_core::Tool;
    ///
    /// let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
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
    /// use cam_core::Tool;
    ///
    /// let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    /// assert_eq!(tool.diameter(), 10.0);
    /// ```
    pub fn diameter(&self) -> T {
        self.radius * T::from_f64(2.0)
    }

    /// コーナーRを取得
    ///
    /// # 戻り値
    ///
    /// コーナーR（mm）
    ///
    /// - フラットエンドミル: 0
    /// - ボールエンドミル: 半径
    /// - ラジアスエンドミル: 指定値
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::Tool;
    ///
    /// let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    /// assert_eq!(flat.corner_radius(), 0.0);
    ///
    /// let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    /// assert_eq!(ball.corner_radius(), 3.0); // 半径
    ///
    /// let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    /// assert_eq!(radius.corner_radius(), 1.0);
    /// ```
    pub fn corner_radius(&self) -> T {
        self.corner_radius
    }

    /// 工具種別を判定
    ///
    /// `corner_radius` の値から自動判定します。
    ///
    /// # 戻り値
    ///
    /// - `corner_radius == 0` → FlatEndMill
    /// - `corner_radius == radius` → BallEndMill
    /// - `0 < corner_radius < radius` → RadiusEndMill
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{Tool, ToolType};
    ///
    /// let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    /// assert_eq!(flat.tool_type(), ToolType::FlatEndMill);
    ///
    /// let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    /// assert_eq!(ball.tool_type(), ToolType::BallEndMill);
    ///
    /// let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    /// assert_eq!(radius.tool_type(), ToolType::RadiusEndMill);
    /// ```
    pub fn tool_type(&self) -> ToolType {
        let epsilon = T::from_f64(1e-10);
        if self.corner_radius.abs() < epsilon {
            ToolType::FlatEndMill
        } else if (self.corner_radius - self.radius).abs() < epsilon {
            ToolType::BallEndMill
        } else {
            ToolType::RadiusEndMill
        }
    }

    /// フラットエンドミルかどうか判定
    pub fn is_flat_end_mill(&self) -> bool {
        matches!(self.tool_type(), ToolType::FlatEndMill)
    }

    /// ラジアスエンドミルかどうか判定
    pub fn is_radius_end_mill(&self) -> bool {
        matches!(self.tool_type(), ToolType::RadiusEndMill)
    }

    /// ボールエンドミルかどうか判定
    pub fn is_ball_end_mill(&self) -> bool {
        matches!(self.tool_type(), ToolType::BallEndMill)
    }

    /// 工具中心から先端までのオフセット（Z軸方向）を取得
    ///
    /// # 戻り値
    ///
    /// - フラットエンドミル: 0（底面 = 中心）
    /// - ボールエンドミル: radius（球面中心から球面底までの距離）
    /// - ラジアスエンドミル: corner_radius（平坦部からR部底までの距離）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::Tool;
    ///
    /// let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
    /// assert_eq!(flat.tip_offset(), 0.0);
    ///
    /// let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    /// assert_eq!(ball.tip_offset(), 3.0);  // 半径
    ///
    /// let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
    /// assert_eq!(radius.tip_offset(), 1.0);  // R値
    /// ```
    pub fn tip_offset(&self) -> T {
        self.corner_radius
    }

    /// 指定した参照点のZ座標を、別の参照点のZ座標に変換
    ///
    /// # 引数
    ///
    /// - `z`: Z座標値（mm）
    /// - `from`: 入力座標の参照点
    /// - `to`: 出力座標の参照点
    ///
    /// # 戻り値
    ///
    /// 変換後のZ座標（mm）
    ///
    /// # 例
    ///
    /// ```
    /// use cam_core::{Tool, tool::ToolReferencePoint};
    ///
    /// let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
    ///
    /// // 工具中心Z=10.0 → 工具先端Z=7.0（3mm下）
    /// let tip_z = ball.convert_z(10.0, ToolReferencePoint::Center, ToolReferencePoint::Tip);
    /// assert_eq!(tip_z, 7.0);
    ///
    /// // 工具先端Z=7.0 → 工具中心Z=10.0（3mm上）
    /// let center_z = ball.convert_z(7.0, ToolReferencePoint::Tip, ToolReferencePoint::Center);
    /// assert_eq!(center_z, 10.0);
    /// ```
    pub fn convert_z(&self, z: T, from: ToolReferencePoint, to: ToolReferencePoint) -> T {
        match (from, to) {
            (ToolReferencePoint::Center, ToolReferencePoint::Tip) => z - self.tip_offset(),
            (ToolReferencePoint::Tip, ToolReferencePoint::Center) => z + self.tip_offset(),
            _ => z, // 同じ参照点なら変換不要
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_end_mill_creation() {
        let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);

        assert_eq!(tool.id, "EM10");
        assert_eq!(tool.tool_type(), ToolType::FlatEndMill);
        assert_eq!(tool.diameter(), 10.0);
        assert_eq!(tool.radius(), 5.0);
        assert_eq!(tool.corner_radius(), 0.0);
        assert_eq!(tool.cutting_length, 50.0);
        assert_eq!(tool.tip_offset(), 0.0);
    }

    #[test]
    fn test_ball_end_mill_creation() {
        let tool = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);

        assert_eq!(tool.id, "BEM6");
        assert_eq!(tool.tool_type(), ToolType::BallEndMill);
        assert_eq!(tool.diameter(), 6.0);
        assert_eq!(tool.radius(), 3.0);
        assert_eq!(tool.corner_radius(), 3.0); // == radius
        assert_eq!(tool.cutting_length, 30.0);
        assert_eq!(tool.tip_offset(), 3.0); // == radius
    }

    #[test]
    fn test_radius_end_mill_creation() {
        let tool = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);

        assert_eq!(tool.id, "REM10R1");
        assert_eq!(tool.tool_type(), ToolType::RadiusEndMill);
        assert_eq!(tool.diameter(), 10.0);
        assert_eq!(tool.radius(), 5.0);
        assert_eq!(tool.corner_radius(), 1.0);
        assert_eq!(tool.cutting_length, 50.0);
        assert_eq!(tool.tip_offset(), 1.0);
    }

    #[test]
    fn test_new_with_parameters() {
        // フラット: R=0
        let flat = Tool::new("EM10".to_string(), 10.0, 0.0, 50.0);
        assert_eq!(flat.tool_type(), ToolType::FlatEndMill);
        assert_eq!(flat.corner_radius(), 0.0);

        // ボール: R=半径
        let ball = Tool::new("BEM6".to_string(), 6.0, 3.0, 30.0);
        assert_eq!(ball.tool_type(), ToolType::BallEndMill);
        assert_eq!(ball.corner_radius(), 3.0);
        assert_eq!(ball.radius(), 3.0);

        // ラジアス: 0 < R < 半径
        let radius = Tool::new("REM10R1".to_string(), 10.0, 1.0, 50.0);
        assert_eq!(radius.tool_type(), ToolType::RadiusEndMill);
        assert_eq!(radius.corner_radius(), 1.0);
        assert!(radius.corner_radius() > 0.0);
        assert!(radius.corner_radius() < radius.radius());
    }

    #[test]
    fn test_radius_and_diameter() {
        let tool = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
        assert_eq!(tool.radius(), 5.0);
        assert_eq!(tool.diameter(), 10.0);
    }

    #[test]
    fn test_tool_type_detection() {
        // フラット: R=0
        let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
        assert!(flat.is_flat_end_mill());
        assert!(!flat.is_radius_end_mill());
        assert!(!flat.is_ball_end_mill());

        // ボール: R=半径
        let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
        assert!(!ball.is_flat_end_mill());
        assert!(!ball.is_radius_end_mill());
        assert!(ball.is_ball_end_mill());

        // ラジアス: 0 < R < 半径
        let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
        assert!(!radius.is_flat_end_mill());
        assert!(radius.is_radius_end_mill());
        assert!(!radius.is_ball_end_mill());
    }

    #[test]
    fn test_tip_offset() {
        // フラット: オフセット = 0
        let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);
        assert_eq!(flat.tip_offset(), 0.0);

        // ボール: オフセット = 半径
        let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);
        assert_eq!(ball.tip_offset(), 3.0);

        // ラジアス: オフセット = R
        let radius = Tool::radius_end_mill("REM10R1".to_string(), 10.0, 1.0, 50.0);
        assert_eq!(radius.tip_offset(), 1.0);
    }

    #[test]
    fn test_convert_z() {
        let ball = Tool::ball_end_mill("BEM6".to_string(), 6.0, 30.0);

        // 工具中心 → 工具先端（3mm下）
        let tip_z = ball.convert_z(10.0, ToolReferencePoint::Center, ToolReferencePoint::Tip);
        assert_eq!(tip_z, 7.0);

        // 工具先端 → 工具中心（3mm上）
        let center_z = ball.convert_z(7.0, ToolReferencePoint::Tip, ToolReferencePoint::Center);
        assert_eq!(center_z, 10.0);

        // 同じ参照点なら変化なし
        let same_z = ball.convert_z(10.0, ToolReferencePoint::Center, ToolReferencePoint::Center);
        assert_eq!(same_z, 10.0);
    }

    #[test]
    fn test_flat_end_mill_reference_points() {
        let flat = Tool::flat_end_mill("EM10".to_string(), 10.0, 50.0);

        // フラットエンドミルは中心=先端（オフセット0）
        let z = 10.0;
        let tip_z = flat.convert_z(z, ToolReferencePoint::Center, ToolReferencePoint::Tip);
        assert_eq!(tip_z, z);
    }

    #[test]
    fn test_corner_radius_consistency() {
        // フラット: R=0
        let flat = Tool::new("EM10".to_string(), 10.0, 0.0, 50.0);
        assert_eq!(flat.corner_radius(), 0.0);
        assert_eq!(flat.tool_type(), ToolType::FlatEndMill);

        // ボール: R=半径
        let ball = Tool::new("BEM6".to_string(), 6.0, 3.0, 30.0);
        assert_eq!(ball.corner_radius(), ball.radius());
        assert_eq!(ball.tool_type(), ToolType::BallEndMill);

        // ラジアス: 0 < R < 半径
        let radius = Tool::new("REM8R2".to_string(), 8.0, 2.0, 40.0);
        assert_eq!(radius.corner_radius(), 2.0);
        assert!(radius.corner_radius() > 0.0);
        assert!(radius.corner_radius() < radius.radius());
        assert_eq!(radius.tool_type(), ToolType::RadiusEndMill);
    }
}
