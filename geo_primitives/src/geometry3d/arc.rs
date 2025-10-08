//! 3D Arc implementation
//!
//! 3次元円弧の基本実装

use crate::geometry3d::{Circle, Point3D};
use geo_foundation::abstract_types::{Scalar, Angle};

/// 円弧の種類を表現する列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcKind {
    /// 短弧（π未満）
    MinorArc,
    /// 長弧（πより大きい）
    MajorArc,
    /// 半円（π）
    Semicircle,
    /// 完全な円（2π）
    FullCircle,
}

/// 3D空間上の円弧を表現する構造体
#[derive(Debug, Clone)]
pub struct Arc<T: Scalar> {
    circle: Circle<T>,
    start_angle: Angle<T>,
    end_angle: Angle<T>,
}

impl<T: Scalar> Arc<T> {
    /// 新しい円弧を作成
    pub fn new(circle: Circle<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self {
            circle,
            start_angle,
            end_angle,
        }
    }

    /// ラジアン角度から円弧を作成（利便性メソッド）
    pub fn from_radians(circle: Circle<T>, start_angle: T, end_angle: T) -> Self {
        Self::new(
            circle,
            Angle::from_radians(start_angle),
            Angle::from_radians(end_angle),
        )
    }

    /// 度数角度から円弧を作成（利便性メソッド）
    pub fn from_degrees(circle: Circle<T>, start_angle: T, end_angle: T) -> Self {
        Self::new(
            circle,
            Angle::from_degrees(start_angle),
            Angle::from_degrees(end_angle),
        )
    }

    /// 円弧の基底円を取得
    pub fn circle(&self) -> &Circle<T> {
        &self.circle
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    /// 円弧の中心を取得
    pub fn center(&self) -> Point3D<T> {
        self.circle.center()
    }

    /// 円弧の半径を取得
    pub fn radius(&self) -> T {
        self.circle.radius()
    }

    /// 円弧の長さを計算
    pub fn arc_length(&self) -> T {
        let angle_diff = (self.end_angle.to_radians() - self.start_angle.to_radians()).abs();
        self.circle.radius() * angle_diff
    }

    /// 円弧の種類を判定
    pub fn arc_kind(&self) -> ArcKind {
        let angle_diff = (self.end_angle.to_radians() - self.start_angle.to_radians()).abs();
        let pi = T::PI;
        let two_pi = T::TAU;
        
        if angle_diff <= T::TOLERANCE {
            ArcKind::FullCircle
        } else if (angle_diff - pi).abs() <= T::TOLERANCE {
            ArcKind::Semicircle
        } else if angle_diff < pi {
            ArcKind::MinorArc
        } else if (angle_diff - two_pi).abs() <= T::TOLERANCE {
            ArcKind::FullCircle
        } else {
            ArcKind::MajorArc
        }
    }

    // TODO: 以下のメソッドは将来実装予定
    // - from_three_points: 3点からの円弧作成
    // - point_at_angle: 指定角度での点取得  
    // - contains_angle: 角度包含判定
    // - midpoint: 中点取得
    // - reverse: 方向反転
    // - approximate_with_points: 点列による近似
}

// 型エイリアス：他の形状実装との統一を目指したパターン
pub type Arc3DF64 = Arc<f64>; // f64版（ジェネリック実装）
pub type Arc3D = Arc3DF64; // デフォルトはf64版
pub type Arc3DF32 = Arc<f32>; // f32版（ジェネリック実装）