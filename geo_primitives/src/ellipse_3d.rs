//! 3D楕円のCore実装
//!
//! Foundation統一システムに基づくEllipse3Dの必須機能のみ

use crate::{Angle, Circle3D, Direction3D, Point3D, Vector3D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

/// 3次元楕円（Core実装）
///
/// Core機能のみ：
/// - 基本構築・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ
/// - 基本パラメトリック操作
#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse3D<T: Scalar> {
    center: Point3D<T>,
    semi_major_axis: T,
    semi_minor_axis: T,
    normal: Direction3D<T>,         // 楕円平面の法線ベクトル（正規化済み）
    major_axis_dir: Direction3D<T>, // 長軸方向ベクトル（正規化済み）
}

// ============================================================================
// Core Implementation (必須機能のみ)
// ============================================================================

impl<T: Scalar> Ellipse3D<T> {
    // ========================================================================
    // Core Construction Methods
    // ========================================================================
    /// 新しい3D楕円を作成
    ///
    /// 基本的な検証のみ実行
    pub fn new(
        center: Point3D<T>,
        semi_major_axis: T,
        semi_minor_axis: T,
        normal: Vector3D<T>,
        major_axis_dir: Vector3D<T>,
    ) -> Option<Self> {
        // 半軸の長さの検証
        if semi_major_axis < semi_minor_axis || semi_minor_axis <= T::ZERO {
            return None;
        }

        // Direction3Dに変換（自動的に正規化される）
        let normal_dir = Direction3D::from_vector(normal)?;
        let major_axis_dir = Direction3D::from_vector(major_axis_dir)?;

        // 基本的な直交性チェック
        let dot_product = normal_dir.dot(&major_axis_dir);
        if dot_product.abs() > DefaultTolerances::distance::<T>() {
            return None;
        }

        Some(Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            normal: normal_dir,
            major_axis_dir,
        })
    }

    /// XY平面上の軸に平行な楕円を作成
    pub fn xy_aligned(center: Point3D<T>, semi_major_axis: T, semi_minor_axis: T) -> Option<Self> {
        Self::new(
            center,
            semi_major_axis,
            semi_minor_axis,
            Vector3D::unit_z(),
            Vector3D::unit_x(),
        )
    }

    /// 3D円から楕円を作成
    pub fn from_circle(circle: &Circle3D<T>) -> Option<Self> {
        let normal_dir = circle.normal();
        let u_axis_dir = circle.u_axis();

        Some(Self {
            center: circle.center(),
            semi_major_axis: circle.radius(),
            semi_minor_axis: circle.radius(),
            normal: normal_dir,
            major_axis_dir: u_axis_dir,
        })
    }

    // ========================================================================
    // Core Accessor Methods
    // ========================================================================

    /// 楕円の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 長半軸の長さを取得
    pub fn semi_major_axis(&self) -> T {
        self.semi_major_axis
    }

    /// 短半軸の長さを取得
    pub fn semi_minor_axis(&self) -> T {
        self.semi_minor_axis
    }

    /// 楕円平面の法線ベクトルを取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// 長軸方向ベクトルを取得
    pub fn major_axis_direction(&self) -> Direction3D<T> {
        self.major_axis_dir
    }

    /// 短軸方向ベクトルを取得
    pub fn minor_axis_direction(&self) -> Direction3D<T> {
        self.normal.cross(&self.major_axis_dir)
    }

    // ========================================================================
    // Core Geometric Properties
    // ========================================================================

    /// 離心率を計算
    pub fn eccentricity(&self) -> T {
        if self.semi_major_axis == T::ZERO {
            return T::ZERO;
        }
        let ratio = self.semi_minor_axis / self.semi_major_axis;
        (T::ONE - ratio * ratio).sqrt()
    }

    /// 楕円の面積を計算
    pub fn area(&self) -> T {
        T::PI * self.semi_major_axis * self.semi_minor_axis
    }

    /// 楕円が円かどうかを判定
    pub fn is_circle(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        (self.semi_major_axis - self.semi_minor_axis).abs() <= tolerance
    }

    /// 楕円が退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        self.semi_minor_axis <= tolerance
    }

    /// 円への変換（円の場合のみ）
    pub fn to_circle(&self) -> Option<Circle3D<T>> {
        if self.is_circle() {
            Circle3D::new(self.center, self.normal, self.semi_major_axis)
        } else {
            None
        }
    }

    // ========================================================================
    // Core Parametric Methods
    // ========================================================================

    /// パラメータ t での楕円上の点を計算
    /// t ∈ [0, 2π]
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir.as_vector() * (self.semi_major_axis * cos_t);
        let minor_component =
            self.minor_axis_direction().as_vector() * (self.semi_minor_axis * sin_t);

        Point3D::new(
            self.center.x() + major_component.x() + minor_component.x(),
            self.center.y() + major_component.y() + minor_component.y(),
            self.center.z() + major_component.z() + minor_component.z(),
        )
    }

    /// パラメータ t での楕円の接線ベクトルを計算
    /// t ∈ [0, 2π]
    pub fn tangent_at_parameter(&self, t: T) -> Vector3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir.as_vector() * (-self.semi_major_axis * sin_t);
        let minor_component =
            self.minor_axis_direction().as_vector() * (self.semi_minor_axis * cos_t);

        major_component + minor_component
    }

    /// 角度 θ での楕円上の点を計算
    pub fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        self.point_at_parameter(angle.to_radians())
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::TAU)
    }
}
