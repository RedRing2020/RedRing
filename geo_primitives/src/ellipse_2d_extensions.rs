//! Ellipse2D拡張メソッド
//!
//! Core Foundation パターンに基づく Ellipse2D の拡張機能
//! 基本機能は ellipse_2d.rs を参照

use crate::{Circle2D, Ellipse2D, Point2D, Vector2D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

impl<T: Scalar> Ellipse2D<T> {
    // ========================================================================
    // Extension Construction Methods
    // ========================================================================

    /// 単位楕円を作成（中心が原点、a=1、b指定）
    pub fn unit_ellipse(semi_minor: T) -> Option<Self> {
        Self::axis_aligned(Point2D::origin(), T::ONE, semi_minor)
    }

    /// 5点から楕円を作成（楕円フィッティング）
    /// 実装は簡略化: とりあえず境界ボックスベースの近似
    pub fn from_five_points(points: [Point2D<T>; 5]) -> Option<Self> {
        // 点群の境界ボックスを計算
        let min_x = points
            .iter()
            .map(|p| p.x())
            .fold(points[0].x(), |min, x| min.min(x));
        let max_x = points
            .iter()
            .map(|p| p.x())
            .fold(points[0].x(), |max, x| max.max(x));
        let min_y = points
            .iter()
            .map(|p| p.y())
            .fold(points[0].y(), |min, y| min.min(y));
        let max_y = points
            .iter()
            .map(|p| p.y())
            .fold(points[0].y(), |max, y| max.max(y));

        let center = Point2D::new(
            (min_x + max_x) / (T::ONE + T::ONE),
            (min_y + max_y) / (T::ONE + T::ONE),
        );

        let width = max_x - min_x;
        let height = max_y - min_y;

        if width > height {
            Self::axis_aligned(
                center,
                width / (T::ONE + T::ONE),
                height / (T::ONE + T::ONE),
            )
        } else {
            Self::axis_aligned(
                center,
                height / (T::ONE + T::ONE),
                width / (T::ONE + T::ONE),
            )
        }
    }

    // ========================================================================
    // Extension Predicate Methods
    // ========================================================================

    /// 離心率を計算
    pub fn eccentricity(&self) -> T {
        if self.semi_major_axis() <= T::ZERO {
            return T::ZERO;
        }

        let a = self.semi_major_axis();
        let b = self.semi_minor_axis();
        let c_squared = a * a - b * b;
        if c_squared <= T::ZERO {
            T::ZERO // 円の場合
        } else {
            (c_squared / (a * a)).sqrt()
        }
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        (self.semi_major_axis() - self.semi_minor_axis()).abs() <= tolerance
    }

    /// 楕円が退化しているか（軸の長さが0に近い）を判定
    pub fn is_degenerate(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        self.semi_major_axis() <= tolerance || self.semi_minor_axis() <= tolerance
    }

    /// 点が楕円境界上にあるかを判定
    pub fn on_boundary(&self, point: &Point2D<T>) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        let distance = self.distance_to_point(point);
        distance <= tolerance
    }

    // ========================================================================
    // Extension Geometric Methods
    // ========================================================================

    /// 指定角度での点を取得
    pub fn point_at_angle(&self, angle: T) -> Point2D<T> {
        self.point_at_parameter(angle)
    }

    // ========================================================================
    // Extension Transformation Methods
    // ========================================================================

    /// 楕円を平行移動
    pub fn translate(&self, offset: &Vector2D<T>) -> Self {
        Self::new(
            self.center() + *offset,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.rotation(),
        )
        .unwrap() // Core で作成された楕円なので常に有効
    }

    /// 楕円を拡大縮小
    pub fn scale(&self, factor: T) -> Option<Self> {
        if factor > T::ZERO {
            Self::new(
                self.center(),
                self.semi_major_axis() * factor,
                self.semi_minor_axis() * factor,
                self.rotation(),
            )
        } else {
            None
        }
    }

    /// 楕円を回転
    pub fn rotate(&self, angle: T) -> Self {
        Self::new(
            self.center(),
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.rotation() + angle,
        )
        .unwrap() // Core で作成された楕円なので常に有効
    }

    /// 原点中心での回転
    pub fn rotate_around_origin(&self, angle: T) -> Self {
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        let new_center = Point2D::new(
            self.center().x() * cos_a - self.center().y() * sin_a,
            self.center().x() * sin_a + self.center().y() * cos_a,
        );

        Self::new(
            new_center,
            self.semi_major_axis(),
            self.semi_minor_axis(),
            self.rotation() + angle,
        )
        .unwrap() // Core で作成された楕円なので常に有効
    }

    // ========================================================================
    // Extension Type Conversion Methods
    // ========================================================================

    /// 楕円を円に変換（可能な場合）
    pub fn to_circle(&self) -> Option<Circle2D<T>> {
        if self.is_circle() {
            Circle2D::new(self.center(), self.semi_major_axis())
        } else {
            None
        }
    }

    // TODO: 3D楕円への変換は将来の実装予定
    // pub fn to_3d(&self) -> crate::geometry3d::ellipse::Ellipse3D<T> {
    //     // Z=0平面上の楕円として3D楕円を作成
    //     // 実装は3D楕円の実装後に追加予定
    // }
}
