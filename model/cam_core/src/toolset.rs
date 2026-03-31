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

/// シャンク段形状種別
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShankSegmentKind {
    /// 円柱段
    Cylinder,

    /// テーパー段
    Taper,
}

/// シャンク1段分の定義
#[derive(Debug, Clone, PartialEq)]
pub struct ShankSegment<T: Scalar = f64> {
    /// 段形状（円柱/テーパー）
    pub kind: ShankSegmentKind,

    /// 段長さ（mm）
    pub length: T,

    /// 上端径（mm）
    pub top_diameter: T,

    /// 下端径（mm）
    pub bottom_diameter: T,
}

impl<T: Scalar> ShankSegment<T> {
    /// 円柱段を作成
    pub fn cylinder(length: T, diameter: T) -> Self {
        Self {
            kind: ShankSegmentKind::Cylinder,
            length,
            top_diameter: diameter,
            bottom_diameter: diameter,
        }
    }

    /// テーパー段を作成
    pub fn taper(length: T, top_diameter: T, bottom_diameter: T) -> Self {
        Self {
            kind: ShankSegmentKind::Taper,
            length,
            top_diameter,
            bottom_diameter,
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

        let is_cylinder = self.top_diameter == self.bottom_diameter;
        match self.kind {
            ShankSegmentKind::Cylinder => {
                if !is_cylinder {
                    return false;
                }
            }
            ShankSegmentKind::Taper => {
                if is_cylinder {
                    return false;
                }
            }
        }

        true
    }

    /// 側面干渉距離を反映した段を返す
    fn apply_side_offset(&self, side_offset: T) -> Self {
        let double_offset = side_offset * T::from_f64(2.0);
        Self {
            kind: self.kind,
            length: self.length,
            top_diameter: self.top_diameter + double_offset,
            bottom_diameter: self.bottom_diameter + double_offset,
        }
    }
}

/// シャンク干渉距離定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShankInterferenceOffset<T: Scalar = f64> {
    /// 側面干渉距離（半径方向）
    pub side: T,

    /// 底面干渉距離（軸方向）
    ///
    /// 多段時は最下面（最終段の下端面）のみに適用します。
    pub bottom: T,
}

impl<T: Scalar> ShankInterferenceOffset<T> {
    pub fn new(side: T, bottom: T) -> Self {
        Self { side, bottom }
    }

    pub fn validate_parameters(&self) -> bool {
        self.side >= T::ZERO && self.bottom >= T::ZERO
    }
}

impl<T: Scalar> Default for ShankInterferenceOffset<T> {
    fn default() -> Self {
        Self {
            side: T::ZERO,
            bottom: T::ZERO,
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

    /// 上端→下端順のシャンク段配列
    pub shank_segments: Vec<ShankSegment<T>>,

    /// シャンク専用干渉距離
    pub shank_interference_offset: ShankInterferenceOffset<T>,

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
    ) -> Self {
        Self {
            id,
            name,
            tool,
            holder,
            reference_point: ToolSetReferencePoint::Tip,
            overall_length,
            stickout_length,
            shank_segments: Vec::new(),
            shank_interference_offset: ShankInterferenceOffset::default(),
            enabled: true,
        }
    }

    pub fn with_shank_segments(mut self, shank_segments: Vec<ShankSegment<T>>) -> Self {
        self.shank_segments = shank_segments;
        self
    }

    pub fn with_shank_segment(mut self, shank_segment: ShankSegment<T>) -> Self {
        self.shank_segments.push(shank_segment);
        self
    }

    pub fn with_shank_interference_offset(mut self, offset: ShankInterferenceOffset<T>) -> Self {
        self.shank_interference_offset = offset;
        self
    }

    pub fn with_reference_point(mut self, reference_point: ToolSetReferencePoint) -> Self {
        self.reference_point = reference_point;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// シャンク底面クリアランスの実効値を返す
    ///
    /// 無効化条件:
    /// - 最下段がテーパー
    /// - 工具径 > 最下段シャンク径
    fn effective_shank_bottom_clearance(&self) -> T {
        if self.shank_interference_offset.bottom <= T::ZERO {
            return T::ZERO;
        }

        let Some(last_segment) = self.shank_segments.last() else {
            return T::ZERO;
        };

        if last_segment.kind != ShankSegmentKind::Cylinder {
            return T::ZERO;
        }

        let epsilon = T::from_f64(1.0e-9);
        if self.tool.diameter() > last_segment.bottom_diameter + epsilon {
            return T::ZERO;
        }

        self.shank_interference_offset.bottom
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
        if self.shank_segments.is_empty() {
            return false;
        }
        if !self.shank_interference_offset.validate_parameters() {
            return false;
        }

        if !self
            .shank_segments
            .iter()
            .all(ShankSegment::validate_parameters)
        {
            return false;
        }

        let mut shank_total_length = T::ZERO;
        for segment in &self.shank_segments {
            shank_total_length += segment.length;
        }
        // シャンク全長が突き出し長と同値以上の場合、突出部が全て非切削部になる。
        if shank_total_length >= self.stickout_length {
            return false;
        }

        self.holder.validate_parameters()
    }

    /// 干渉判定用ホルダー形状を取得
    pub fn holder_interference_shape(&self) -> Holder<T> {
        self.holder.to_interference_shape()
    }

    /// 干渉判定用シャンク形状を取得
    pub fn shank_interference_shape(&self) -> Vec<ShankSegment<T>> {
        if self.shank_segments.is_empty() {
            return Vec::new();
        }

        let mut segments: Vec<ShankSegment<T>> = self
            .shank_segments
            .iter()
            .map(|segment| segment.apply_side_offset(self.shank_interference_offset.side))
            .collect();

        let effective_bottom = self.effective_shank_bottom_clearance();

        if let Some(last_segment) = segments.last_mut() {
            last_segment.length += effective_bottom;
        }

        segments
    }
}

#[cfg(test)]
#[path = "toolset_tests.rs"]
mod tests;
