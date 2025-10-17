//! 3D楕円の基本実装
//!
//! 最小限の機能に集中：作成、アクセサ、基本プロパティのみ

use crate::{Circle3D, Point3D, Vector3D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

/// 3次元楕円（基本実装）
///
/// 基本機能のみ：
/// - 作成・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ
/// - シンプルなパラメトリック操作
#[derive(Debug, Clone, PartialEq)]
pub struct Ellipse3D<T: Scalar> {
    center: Point3D<T>,
    semi_major_axis: T,
    semi_minor_axis: T,
    normal: Vector3D<T>,         // 楕円平面の法線ベクトル（正規化済み）
    major_axis_dir: Vector3D<T>, // 長軸方向ベクトル（正規化済み）
}

impl<T: Scalar> Ellipse3D<T> {
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

        // 基本的な長さチェック
        let normal_length = normal.length();
        let major_axis_length = major_axis_dir.length();
        if normal_length <= T::TOLERANCE || major_axis_length <= T::TOLERANCE {
            return None;
        }

        // 正規化（除算による簡単な方法）
        let normal_normalized = normal * (T::ONE / normal_length);
        let major_axis_normalized = major_axis_dir * (T::ONE / major_axis_length);

        // 基本的な直交性チェック（厳密ではない）
        let dot_product = normal_normalized.dot(&major_axis_normalized);
        if dot_product.abs() > DefaultTolerances::distance::<T>() {
            return None;
        }

        Some(Self {
            center,
            semi_major_axis,
            semi_minor_axis,
            normal: normal_normalized,
            major_axis_dir: major_axis_normalized,
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
    pub fn from_circle(circle: &Circle3D<T>) -> Self {
        Self {
            center: circle.center(),
            semi_major_axis: circle.radius(),
            semi_minor_axis: circle.radius(),
            normal: circle.normal(),
            major_axis_dir: circle.u_axis(),
        }
    }

    // === 基本アクセサメソッド ===

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
    pub fn normal(&self) -> Vector3D<T> {
        self.normal
    }

    /// 長軸方向ベクトルを取得
    pub fn major_axis_direction(&self) -> Vector3D<T> {
        self.major_axis_dir
    }

    /// 短軸方向ベクトルを取得
    pub fn minor_axis_direction(&self) -> Vector3D<T> {
        self.normal.cross(&self.major_axis_dir)
    }

    // === 基本的な幾何プロパティ ===

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

    // === 基本的なパラメトリック操作 ===

    /// パラメータ t での楕円上の点を計算
    /// t ∈ [0, 2π]
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let cos_t = t.cos();
        let sin_t = t.sin();

        let major_component = self.major_axis_dir * (self.semi_major_axis * cos_t);
        let minor_component = self.minor_axis_direction() * (self.semi_minor_axis * sin_t);

        Point3D::new(
            self.center.x() + major_component.x() + minor_component.x(),
            self.center.y() + major_component.y() + minor_component.y(),
            self.center.z() + major_component.z() + minor_component.z(),
        )
    }

    /// 角度 θ での楕円上の点を計算
    pub fn point_at_angle(&self, angle: T) -> Point3D<T> {
        self.point_at_parameter(angle)
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::from_f64(2.0) * T::PI)
    }
}
