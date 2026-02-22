//! CAMツールセット定義（工具 + ホルダー）
//!
//! Issue #246で定義した要件に基づき、以下を提供します。
//! - ツールセット（工具 + ホルダー + 参照点）
//! - 多段ホルダー（円柱/テーパー + R）
//! - 干渉距離（側面/底面）の定義
//! - 干渉判定用オフセット形状の生成

use analysis::Scalar;

use crate::Tool;

/// ツールセットで使用する参照点
///
/// 工具原点（形状配置の基準）とは独立した、
/// ツールパス座標の意味付け用参照点です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolSetReferencePoint {
    /// 工具先端最下点
    Tip,

    /// 工具中心（工具軸上の基準点）
    Center,

    /// ゲージライン参照点
    Gauge,
}

/// ホルダー段形状種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderSegmentKind {
    /// 円柱段
    Cylinder,

    /// テーパー段
    Taper,
}

/// ホルダー1段分の定義
#[derive(Debug, Clone, PartialEq)]
pub struct HolderSegment<T: Scalar = f64> {
    /// 段形状（円柱/テーパー）
    pub kind: HolderSegmentKind,

    /// 段長さ（mm）
    pub length: T,

    /// 上端径（mm）
    pub top_diameter: T,

    /// 下端径（mm）
    pub bottom_diameter: T,

    /// 上端コーナーR（mm）
    pub top_corner_radius: T,

    /// 下端コーナーR（mm）
    pub bottom_corner_radius: T,
}

impl<T: Scalar> HolderSegment<T> {
    /// 円柱段を作成
    pub fn cylinder(length: T, diameter: T, top_corner_radius: T, bottom_corner_radius: T) -> Self {
        Self {
            kind: HolderSegmentKind::Cylinder,
            length,
            top_diameter: diameter,
            bottom_diameter: diameter,
            top_corner_radius,
            bottom_corner_radius,
        }
    }

    /// テーパー段を作成
    pub fn taper(
        length: T,
        top_diameter: T,
        bottom_diameter: T,
        top_corner_radius: T,
        bottom_corner_radius: T,
    ) -> Self {
        Self {
            kind: HolderSegmentKind::Taper,
            length,
            top_diameter,
            bottom_diameter,
            top_corner_radius,
            bottom_corner_radius,
        }
    }

    /// 形状パラメータ妥当性を検証
    pub fn validate_parameters(&self) -> bool {
        if self.length <= T::ZERO {
            return false;
        }
        if self.top_diameter <= T::ZERO || self.bottom_diameter <= T::ZERO {
            return false;
        }
        if self.top_corner_radius < T::ZERO || self.bottom_corner_radius < T::ZERO {
            return false;
        }

        let is_cylinder = self.top_diameter == self.bottom_diameter;
        match self.kind {
            HolderSegmentKind::Cylinder => {
                if !is_cylinder {
                    return false;
                }
            }
            HolderSegmentKind::Taper => {
                if is_cylinder {
                    return false;
                }
            }
        }

        let top_radius = self.top_diameter / T::from_f64(2.0);
        let bottom_radius = self.bottom_diameter / T::from_f64(2.0);
        let max_corner = top_radius.min(bottom_radius);

        if self.top_corner_radius > max_corner || self.bottom_corner_radius > max_corner {
            return false;
        }

        self.top_corner_radius + self.bottom_corner_radius <= self.length
    }

    /// 側面干渉距離を反映した段を返す
    fn apply_side_offset(&self, side_offset: T) -> Self {
        let double_offset = side_offset * T::from_f64(2.0);

        Self {
            kind: self.kind,
            length: self.length,
            top_diameter: self.top_diameter + double_offset,
            bottom_diameter: self.bottom_diameter + double_offset,
            top_corner_radius: self.top_corner_radius + side_offset,
            bottom_corner_radius: self.bottom_corner_radius + side_offset,
        }
    }
}

/// ホルダー干渉距離定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolderInterferenceOffset<T: Scalar = f64> {
    /// 側面干渉距離（半径方向）
    pub side: T,

    /// 底面干渉距離（軸方向）
    ///
    /// 多段時は最下面（最終段の下端面）のみに適用します。
    pub bottom: T,
}

impl<T: Scalar> HolderInterferenceOffset<T> {
    pub fn new(side: T, bottom: T) -> Self {
        Self { side, bottom }
    }

    pub fn validate_parameters(&self) -> bool {
        self.side >= T::ZERO && self.bottom >= T::ZERO
    }
}

impl<T: Scalar> Default for HolderInterferenceOffset<T> {
    fn default() -> Self {
        Self {
            side: T::ZERO,
            bottom: T::ZERO,
        }
    }
}

/// ホルダー定義（多段円柱/テーパー）
#[derive(Debug, Clone, PartialEq)]
pub struct Holder<T: Scalar = f64> {
    /// 識別子
    pub id: String,

    /// 上端→下端順の段配列
    pub segments: Vec<HolderSegment<T>>,

    /// 干渉距離
    pub interference_offset: HolderInterferenceOffset<T>,
}

impl<T: Scalar> Holder<T> {
    pub fn new(id: String, segments: Vec<HolderSegment<T>>) -> Self {
        Self {
            id,
            segments,
            interference_offset: HolderInterferenceOffset::default(),
        }
    }

    pub fn with_interference_offset(mut self, offset: HolderInterferenceOffset<T>) -> Self {
        self.interference_offset = offset;
        self
    }

    /// 全長（mm）
    pub fn total_length(&self) -> T {
        let mut sum = T::ZERO;
        for seg in &self.segments {
            sum += seg.length;
        }
        sum
    }

    /// パラメータ妥当性を検証
    pub fn validate_parameters(&self) -> bool {
        if self.id.is_empty() {
            return false;
        }
        if self.segments.is_empty() {
            return false;
        }
        if !self.interference_offset.validate_parameters() {
            return false;
        }

        self.segments.iter().all(HolderSegment::validate_parameters)
    }

    /// 干渉距離反映後のホルダー形状を生成
    ///
    /// - 側面: 全段に反映
    /// - 底面: 最下面（最終段）のみ長さ方向へ反映
    pub fn to_interference_shape(&self) -> Self {
        if self.segments.is_empty() {
            return self.clone();
        }

        let mut segments: Vec<HolderSegment<T>> = self
            .segments
            .iter()
            .map(|seg| seg.apply_side_offset(self.interference_offset.side))
            .collect();

        if let Some(last_segment) = segments.last_mut() {
            last_segment.length += self.interference_offset.bottom;
        }

        Self {
            id: format!("{}_interference", self.id),
            segments,
            interference_offset: self.interference_offset,
        }
    }
}

/// ツールセット定義（工具 + ホルダー）
#[derive(Debug, Clone, PartialEq)]
pub struct ToolSet<T: Scalar = f64> {
    /// ツールセット識別子
    pub id: String,

    /// ツールセット名称
    pub name: String,

    /// 工具
    pub tool: Tool<T>,

    /// ホルダー
    pub holder: Holder<T>,

    /// 工具経路座標の参照点
    pub reference_point: ToolSetReferencePoint,

    /// 全長（mm）
    pub overall_length: T,

    /// 突き出し長（mm）
    pub stickout_length: T,

    /// シャンク径（mm）
    pub shank_diameter: T,

    /// 有効フラグ
    pub enabled: bool,
}

impl<T: Scalar> ToolSet<T> {
    pub fn new(
        id: String,
        name: String,
        tool: Tool<T>,
        holder: Holder<T>,
        overall_length: T,
        stickout_length: T,
        shank_diameter: T,
    ) -> Self {
        Self {
            id,
            name,
            tool,
            holder,
            reference_point: ToolSetReferencePoint::Tip,
            overall_length,
            stickout_length,
            shank_diameter,
            enabled: true,
        }
    }

    pub fn with_reference_point(mut self, reference_point: ToolSetReferencePoint) -> Self {
        self.reference_point = reference_point;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// ツールセット定義全体の妥当性を検証
    pub fn validate_parameters(&self) -> bool {
        if self.id.is_empty() || self.name.is_empty() {
            return false;
        }
        if !self.tool.validate_parameters() {
            return false;
        }

        if self.overall_length <= T::ZERO || self.stickout_length <= T::ZERO {
            return false;
        }
        if self.stickout_length > self.overall_length {
            return false;
        }
        if self.shank_diameter <= T::ZERO {
            return false;
        }

        self.holder.validate_parameters()
    }

    /// 干渉判定用ホルダー形状を取得
    pub fn holder_interference_shape(&self) -> Holder<T> {
        self.holder.to_interference_shape()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Tool, ToolType};

    #[test]
    fn test_holder_segment_creation() {
        let cyl = HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0);
        assert_eq!(cyl.kind, HolderSegmentKind::Cylinder);
        assert_eq!(cyl.top_diameter, 30.0);
        assert_eq!(cyl.bottom_diameter, 30.0);
        assert_eq!(cyl.top_corner_radius, 1.0);
        assert_eq!(cyl.bottom_corner_radius, 1.0);

        let taper = HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25);
        assert_eq!(taper.kind, HolderSegmentKind::Taper);
        assert_eq!(taper.top_diameter, 30.0);
        assert_eq!(taper.bottom_diameter, 20.0);
    }

    #[test]
    fn test_holder_validation() {
        let holder = Holder::new(
            "HOLDER-A".to_string(),
            vec![
                HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0),
                HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25),
            ],
        )
        .with_interference_offset(HolderInterferenceOffset::new(10.0, 0.0));

        assert!(holder.validate_parameters());
        assert_eq!(holder.total_length(), 35.0);
    }

    #[test]
    fn test_holder_segment_kind_validation() {
        let invalid_cylinder = HolderSegment::taper(10.0, 20.0, 20.0, 0.0, 0.0);
        assert!(!invalid_cylinder.validate_parameters());

        let invalid_taper = HolderSegment::cylinder(10.0, 20.0, 0.0, 0.0);
        let mut invalid_taper = invalid_taper;
        invalid_taper.kind = HolderSegmentKind::Taper;
        assert!(!invalid_taper.validate_parameters());

        let invalid_corners = HolderSegment::cylinder(10.0, 20.0, 6.0, 5.0);
        assert!(!invalid_corners.validate_parameters());
    }

    #[test]
    fn test_holder_interference_shape_applies_bottom_to_last_segment_only() {
        let holder = Holder::new(
            "HOLDER-B".to_string(),
            vec![
                HolderSegment::cylinder(20.0, 30.0, 1.0, 1.0),
                HolderSegment::taper(15.0, 30.0, 20.0, 0.5, 0.25),
            ],
        )
        .with_interference_offset(HolderInterferenceOffset::new(10.0, 2.0));

        let offset_shape = holder.to_interference_shape();

        assert_eq!(offset_shape.segments.len(), 2);

        // 側面オフセット: 径 + 2 * side
        assert_eq!(offset_shape.segments[0].top_diameter, 50.0);
        assert_eq!(offset_shape.segments[0].bottom_diameter, 50.0);
        assert_eq!(offset_shape.segments[1].top_diameter, 50.0);
        assert_eq!(offset_shape.segments[1].bottom_diameter, 40.0);

        // 底面オフセット: 最終段のみ長さ延長
        assert_eq!(offset_shape.segments[0].length, 20.0);
        assert_eq!(offset_shape.segments[1].length, 17.0);
    }

    #[test]
    fn test_toolset_validation() {
        let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
        let holder = Holder::new(
            "HOLDER-C".to_string(),
            vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
        );

        let toolset = ToolSet::new(
            "TS-001".to_string(),
            "Flat 10 Set".to_string(),
            tool,
            holder,
            75.0,
            40.0,
            10.0,
        )
        .with_reference_point(ToolSetReferencePoint::Gauge);

        assert!(toolset.validate_parameters());
        assert_eq!(toolset.reference_point, ToolSetReferencePoint::Gauge);
    }

    #[test]
    fn test_toolset_validation_rejects_invalid_lengths() {
        let tool = Tool::new("EM10".to_string(), ToolType::FlatEndMill, 10.0, 0.0, 30.0);
        let holder = Holder::new(
            "HOLDER-D".to_string(),
            vec![HolderSegment::cylinder(20.0, 30.0, 0.0, 0.0)],
        );

        let toolset = ToolSet::new(
            "TS-002".to_string(),
            "Invalid Set".to_string(),
            tool,
            holder,
            40.0,
            50.0,
            10.0,
        );

        assert!(!toolset.validate_parameters());
    }
}
