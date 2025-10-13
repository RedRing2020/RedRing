//! 2次元円弧（Arc2D）の Core 実装
//!
//! Core Foundation パターンに基づく Arc2D の必須機能のみ
//! 拡張機能は arc_2d_extensions.rs を参照

use crate::{Point2D, Vector2D};
use geo_foundation::{Angle, Scalar};

/// 2次元円弧
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc2D<T: Scalar> {
    center: Point2D<T>,
    radius: T,
    start_direction: Vector2D<T>,
    start_angle: Angle<T>,
    end_angle: Angle<T>,
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Arc2D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい円弧を作成
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径（正の値）
    /// * `start_direction` - 開始方向ベクトル（正規化される）
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn new(
        center: Point2D<T>,
        radius: T,
        start_direction: Vector2D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        if radius <= T::ZERO {
            return None;
        }

        let normalized_dir = start_direction.try_normalize()?;

        Some(Self {
            center,
            radius,
            start_direction: normalized_dir,
            start_angle,
            end_angle,
        })
    }

    /// XY平面上の円弧を作成（開始角度はX軸正方向から）
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn xy_arc(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Self::new(center, radius, Vector2D::unit_x(), start_angle, end_angle)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.center
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 開始方向ベクトルを取得
    pub fn start_direction(&self) -> Vector2D<T> {
        self.start_direction
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    // ========================================================================
    // Core Metrics Methods
    // ========================================================================

    /// 角度範囲を取得
    pub fn angle_span(&self) -> Angle<T> {
        let mut span = self.end_angle - self.start_angle;
        if span.to_radians() < T::ZERO {
            span += Angle::from_radians(T::TAU);
        }
        span
    }

    /// 円弧長を計算
    pub fn arc_length(&self) -> T {
        self.radius * self.angle_span().to_radians()
    }

    // ========================================================================
    // Core Geometric Methods
    // ========================================================================

    /// パラメータ t (0.0 ～ 1.0) における点を取得
    pub fn point_at_parameter(&self, t: T) -> Point2D<T> {
        let angle = self.start_angle.to_radians() + t * self.angle_span().to_radians();
        self.point_at_angle(angle)
    }

    /// 指定角度における点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let x = self.radius * angle.cos();
        let y = self.radius * angle.sin();
        let point_on_circle = Vector2D::new(x, y);
        self.center + point_on_circle
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle.to_radians())
    }
}