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
    /// 回転軸の旋回範囲定義
    pub rotary_axis_limits: Vec<RotaryAxisLimit<T>>,
}

impl<T: Scalar> MachineConstraint<T> {
    /// 制約情報を作成
    pub fn new(rotary_axis_limits: Vec<RotaryAxisLimit<T>>) -> Self {
        Self { rotary_axis_limits }
    }

    /// 制約未定義の空状態を作成
    pub fn empty() -> Self {
        Self {
            rotary_axis_limits: Vec::new(),
        }
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
