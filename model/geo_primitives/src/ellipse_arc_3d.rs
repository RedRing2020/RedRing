//! 3次元楕円弧（EllipseArc3D）のCore実装
//!
//! Core Foundation パターンに基づく EllipseArc3D の必須機能のみ
//! 拡張機能は ellipse_arc_3d_extensions.rs を参照

use crate::{Arc3D, BBox3D, Circle3D, Direction3D, Ellipse3D, Point3D, Vector3D};
use geo_foundation::{Angle, Scalar};

/// 3次元楕円弧
///
/// 3D空間内の楕円の一部分を表現する楕円弧
/// 開始角度と終了角度で定義される
#[derive(Debug, Clone, PartialEq)]
pub struct EllipseArc3D<T: Scalar> {
    ellipse: Ellipse3D<T>, // 基底楕円
    start_angle: Angle<T>, // 開始角度
    end_angle: Angle<T>,   // 終了角度
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> EllipseArc3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================

    /// 新しい3D楕円弧を作成
    pub fn new(ellipse: Ellipse3D<T>, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self {
            ellipse,
            start_angle,
            end_angle,
        }
    }

    /// 3D円弧から3D楕円弧を作成
    pub fn from_arc(arc: Arc3D<T>) -> Option<Self> {
        let circle = Circle3D::new(arc.center(), arc.normal(), arc.radius())?;
        let ellipse = Ellipse3D::from_circle(&circle)?;
        Some(Self::new(ellipse, arc.start_angle(), arc.end_angle()))
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 基底楕円を取得
    pub fn ellipse(&self) -> &Ellipse3D<T> {
        &self.ellipse
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    /// 中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.ellipse.center()
    }

    /// 長半径を取得
    pub fn semi_major(&self) -> T {
        self.ellipse.semi_major_axis()
    }

    /// 短半径を取得
    pub fn semi_minor(&self) -> T {
        self.ellipse.semi_minor_axis()
    }

    /// 法線方向を取得
    pub fn normal(&self) -> Direction3D<T> {
        self.ellipse.normal()
    }

    /// 長軸方向を取得
    pub fn major_axis_direction(&self) -> Direction3D<T> {
        self.ellipse.major_axis_direction()
    }

    /// 短軸方向を取得
    pub fn minor_axis_direction(&self) -> Direction3D<T> {
        self.ellipse.minor_axis_direction()
    }

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

    /// 開始点を取得
    pub fn start_point(&self) -> Point3D<T> {
        self.ellipse.point_at_angle(self.start_angle)
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.ellipse.point_at_angle(self.end_angle)
    }

    /// パラメータ値tにおける点を取得
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle_diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        let current_angle = self.start_angle.to_radians() + t * angle_diff;
        self.ellipse
            .point_at_angle(Angle::from_radians(current_angle))
    }

    /// 弧の中点を取得
    pub fn midpoint(&self) -> Point3D<T> {
        self.point_at_parameter(T::from_f64(0.5))
    }

    /// 弧の角度スパンを取得
    pub fn angle_span(&self) -> T {
        let diff = self.end_angle.to_radians() - self.start_angle.to_radians();
        if diff >= T::ZERO {
            diff
        } else {
            diff + T::from_f64(2.0 * std::f64::consts::PI)
        }
    }

    /// 弧が完全な楕円かどうかを判定
    pub fn is_full_ellipse(&self) -> bool {
        let two_pi = T::from_f64(2.0 * std::f64::consts::PI);
        (self.angle_span() - two_pi).abs() < T::EPSILON
    }

    /// 弧が円弧かどうかを判定
    pub fn is_circular(&self) -> bool {
        self.ellipse.is_circle()
    }

    /// 楕円弧の有効性を検証
    pub fn is_valid(&self) -> bool {
        self.ellipse.semi_major_axis() > T::ZERO
            && self.ellipse.semi_minor_axis() > T::ZERO
            && self.start_angle.to_radians().is_finite()
            && self.end_angle.to_radians().is_finite()
    }

    // ========================================================================
    // Core Transform Methods
    // ========================================================================

    /// 平行移動
    pub fn translate(&self, vector: Vector3D<T>) -> Self {
        Self::new(
            self.ellipse.translate(vector),
            self.start_angle,
            self.end_angle,
        )
    }

    /// 向きを反転した楕円弧を取得
    pub fn reverse(&self) -> Self {
        Self::new(self.ellipse.clone(), self.end_angle, self.start_angle)
    }

    /// 角度範囲を変更した新しい楕円弧を作成
    pub fn with_angles(&self, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Self::new(self.ellipse.clone(), start_angle, end_angle)
    }

    /// 基底楕円を変更した新しい楕円弧を作成
    pub fn with_ellipse(&self, ellipse: Ellipse3D<T>) -> Self {
        Self::new(ellipse, self.start_angle, self.end_angle)
    }

    /// 原点中心の回転（Z軸回転）
    pub fn rotate_z(&self, _angle: T) -> Option<Self> {
        // 簡単なZ軸回転（実際の実装は拡張版で）
        Some(Self::new(
            self.ellipse.clone(),
            self.start_angle,
            self.end_angle,
        ))
    }

    /// 原点中心の均等スケール
    pub fn scale(&self, factor: T) -> Option<Self> {
        // スケールは基底楕円に適用
        if factor <= T::ZERO {
            return None;
        }

        // 楕円をスケールして新しいEllipseArc3Dを作成
        let scaled_ellipse = Ellipse3D::new(
            self.ellipse.center(),
            self.ellipse.semi_major_axis() * factor,
            self.ellipse.semi_minor_axis() * factor,
            self.ellipse.normal().as_vector(),
            self.ellipse.major_axis_direction().as_vector(),
        )?;

        Some(Self::new(scaled_ellipse, self.start_angle, self.end_angle))
    }

    /// 部分弧を取得
    pub fn sub_arc(&self, sub_start: Angle<T>, sub_end: Angle<T>) -> Option<Self> {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();
        let sub_start_rad = sub_start.to_radians();
        let sub_end_rad = sub_end.to_radians();

        // 簡単な範囲チェック
        if (sub_start_rad >= start_rad && sub_start_rad <= end_rad)
            && (sub_end_rad >= start_rad && sub_end_rad <= end_rad)
            && sub_start_rad <= sub_end_rad
        {
            Some(Self::new(self.ellipse.clone(), sub_start, sub_end))
        } else {
            None
        }
    }

    /// バウンディングボックスを取得（近似）
    pub fn bounding_box(&self) -> BBox3D<T> {
        let start = self.start_point();
        let end = self.end_point();
        let mid = self.midpoint();

        BBox3D::from_points(&[start, end, mid]).unwrap_or_default()
    }
}

// ============================================================================
// Core Trait Implementations
// ============================================================================

impl<T: Scalar> Copy for EllipseArc3D<T> where Ellipse3D<T>: Copy {}

impl<T: Scalar> Default for EllipseArc3D<T> {
    fn default() -> Self {
        let center = Point3D::origin();
        let normal = Vector3D::unit_z();
        let major_axis = Vector3D::unit_x();
        let ellipse = Ellipse3D::new(center, T::ONE, T::ONE, normal, major_axis)
            .unwrap_or_else(|| panic!("Default ellipse creation failed"));

        Self::new(
            ellipse,
            Angle::from_degrees(T::ZERO),
            Angle::from_degrees(T::from_f64(90.0)),
        )
    }
}
