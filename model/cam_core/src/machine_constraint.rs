//! 機械制約の最小定義
//!
//! `cam_core` では加工機のDB実体は持たず、
//! ToolPath消費側が参照できる最小の物理制約のみを保持します。

use analysis::Scalar;

/// 商用機で一般的な機械軸ラベル
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MachineAxisLabel {
    X,
    Y,
    Z,
    A,
    B,
    C,
    U,
    V,
    W,
}

/// 直動軸ラベル（移動可能な軸）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LinearAxisLabel {
    X,
    Y,
    Z,
    U,
    V,
    W,
}

impl From<LinearAxisLabel> for MachineAxisLabel {
    fn from(value: LinearAxisLabel) -> Self {
        match value {
            LinearAxisLabel::X => MachineAxisLabel::X,
            LinearAxisLabel::Y => MachineAxisLabel::Y,
            LinearAxisLabel::Z => MachineAxisLabel::Z,
            LinearAxisLabel::U => MachineAxisLabel::U,
            LinearAxisLabel::V => MachineAxisLabel::V,
            LinearAxisLabel::W => MachineAxisLabel::W,
        }
    }
}

/// 回転軸ラベル（角度制限の対象）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotaryAxisLabel {
    A,
    B,
    C,
}

impl From<RotaryAxisLabel> for MachineAxisLabel {
    fn from(value: RotaryAxisLabel) -> Self {
        match value {
            RotaryAxisLabel::A => MachineAxisLabel::A,
            RotaryAxisLabel::B => MachineAxisLabel::B,
            RotaryAxisLabel::C => MachineAxisLabel::C,
        }
    }
}

/// 直動軸の移動範囲（mm 単位）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearAxisLimit<T: Scalar = f64> {
    /// 対象軸ラベル（X/Y/Z/U/V/W）
    pub axis: LinearAxisLabel,
    /// 最小移動位置 [mm]
    pub min_mm: T,
    /// 最大移動位置 [mm]
    pub max_mm: T,
}

impl<T: Scalar> LinearAxisLimit<T> {
    /// 直動軸制限を作成
    pub fn new(axis: LinearAxisLabel, min_mm: T, max_mm: T) -> Self {
        Self {
            axis,
            min_mm,
            max_mm,
        }
    }

    /// 指定座標がトラベル範囲に含まれるか
    pub fn contains_mm(&self, mm: T) -> bool {
        self.min_mm <= mm && mm <= self.max_mm
    }
}

/// 回転軸の旋回範囲（度）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RotaryAxisLimit<T: Scalar = f64> {
    /// 対象軸ラベル（A/B/C）
    pub axis: RotaryAxisLabel,
    /// 最小角度（deg）
    pub min_deg: T,
    /// 最大角度（deg）
    pub max_deg: T,
}

impl<T: Scalar> RotaryAxisLimit<T> {
    /// 回転軸制限を作成
    pub fn new(axis: RotaryAxisLabel, min_deg: T, max_deg: T) -> Self {
        Self {
            axis,
            min_deg,
            max_deg,
        }
    }

    /// 指定角度が旋回範囲に含まれるか
    pub fn contains_deg(&self, deg: T) -> bool {
        self.min_deg <= deg && deg <= self.max_deg
    }
}

/// ToolPathとは独立に管理する機械の物理制約
#[derive(Debug, Clone, PartialEq)]
pub struct MachineConstraint<T: Scalar = f64> {
    /// 直動軸のトラベル範囲定義
    pub linear_axis_limits: Vec<LinearAxisLimit<T>>,
    /// 回転軸の旋回範囲定義
    pub rotary_axis_limits: Vec<RotaryAxisLimit<T>>,
}

impl<T: Scalar> MachineConstraint<T> {
    /// 制約情報を作成
    pub fn new(rotary_axis_limits: Vec<RotaryAxisLimit<T>>) -> Self {
        Self {
            linear_axis_limits: Vec::new(),
            rotary_axis_limits,
        }
    }

    /// 制約未定義の空状態を作成
    pub fn empty() -> Self {
        Self {
            linear_axis_limits: Vec::new(),
            rotary_axis_limits: Vec::new(),
        }
    }

    /// 対象軸の直動リミット定義を取得
    pub fn linear_limit_for_axis(&self, axis: LinearAxisLabel) -> Option<&LinearAxisLimit<T>> {
        self.linear_axis_limits
            .iter()
            .find(|limit| limit.axis == axis)
    }

    /// 対象軸の制限定義を取得
    pub fn rotary_limit_for_axis(&self, axis: RotaryAxisLabel) -> Option<&RotaryAxisLimit<T>> {
        self.rotary_axis_limits
            .iter()
            .find(|limit| limit.axis == axis)
    }
}

#[cfg(test)]
#[path = "machine_constraint_tests.rs"]
mod tests;
