//! 2次元円弧（Arc2D）の Core 実装
//!
//! Core Foundation パターンに基づく Arc2D の必須機能のみ
//! 拡張機能は arc_2d_extensions.rs を参照

use crate::{Circle2D, Point2D, Vector2D};
use geo_foundation::{abstract_types::abstracts::Arc2D as Arc2DTrait, Angle, Scalar};

/// 2次元円弧
///
/// geo_foundation::Arc2D<T> トレイトの基本実装
/// 基底円と角度範囲による円弧の定義
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc2D<T: Scalar> {
    /// 基底となる円
    circle: Circle2D<T>,
    /// 開始角度
    start_angle: Angle<T>,
    /// 終了角度
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
    /// * `circle` - 基底となる円
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn new(circle: Circle2D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Option<Self> {
        // 基底円の有効性チェック
        if circle.radius() <= T::ZERO {
            return None;
        }

        Some(Self {
            circle,
            start_angle,
            end_angle,
        })
    }

    /// 中心点・半径・角度から円弧を作成
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径（正の値）
    /// * `start_angle` - 開始角度（Angle）
    /// * `end_angle` - 終了角度（Angle）
    pub fn from_center_radius(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let circle = Circle2D::new(center, radius)?;
        Self::new(circle, start_angle, end_angle)
    }

    /// XY平面円弧の便利な作成メソッド（テスト用）
    ///
    /// `from_center_radius` のエイリアス
    pub fn xy_arc(
        center: Point2D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Self::from_center_radius(center, radius, start_angle, end_angle)
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 基底円を取得
    pub fn circle(&self) -> &Circle2D<T> {
        &self.circle
    }

    /// 中心点を取得
    pub fn center(&self) -> Point2D<T> {
        self.circle.center()
    }

    /// 半径を取得
    pub fn radius(&self) -> T {
        self.circle.radius()
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
    // Core Geometric Methods
    // ========================================================================

    /// 指定角度における点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        let x = self.radius() * angle.cos();
        let y = self.radius() * angle.sin();
        self.center() + Vector2D::new(x, y)
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 開始方向ベクトルを取得
    pub fn start_direction(&self) -> Vector2D<T> {
        let angle = self.start_angle.to_radians();
        // 円の接線方向（時計回り）
        Vector2D::new(angle.cos(), angle.sin())
    }
}

// ============================================================================
// geo_foundation Arc2D Trait Implementation
// ============================================================================

impl<T: Scalar> Arc2DTrait<T> for Arc2D<T> {
    type Circle = Circle2D<T>;
    type Point = Point2D<T>;
    type Angle = Angle<T>;

    fn circle(&self) -> &Circle2D<T> {
        self.circle()
    }

    fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    fn is_full_circle(&self) -> bool {
        // extensionsファイルの実装を使用
        self.is_full_circle()
    }

    fn start_point(&self) -> Point2D<T> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    fn end_point(&self) -> Point2D<T> {
        self.point_at_angle(self.end_angle.to_radians())
    }
}

// ============================================================================
// Foundation System Trait Implementation
// ============================================================================

use geo_foundation::abstract_types::foundation::ArcCore;

impl<T: Scalar> ArcCore<T> for Arc2D<T> {
    type Circle = Circle2D<T>;
    type Point = Point2D<T>;
    type Angle = Angle<T>;

    fn circle(&self) -> &Self::Circle {
        &self.circle
    }

    fn start_angle(&self) -> Self::Angle {
        self.start_angle
    }

    fn end_angle(&self) -> Self::Angle {
        self.end_angle
    }

    fn is_full_circle(&self) -> bool {
        self.is_full_circle()
    }

    fn start_point(&self) -> Self::Point {
        self.start_point()
    }

    fn end_point(&self) -> Self::Point {
        self.end_point()
    }
}
