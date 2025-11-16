//! Arc3D - Core Implementation
//!
//! 3次元円弧の基本実装とコンストラクタ、アクセサメソッド

use crate::{Angle, Direction3D, Point3D, Vector3D};
use geo_foundation::{tolerance_migration::DefaultTolerances, Scalar};

/// 3次元円弧（基本実装）
///
/// 基本機能のみ：
/// - 作成・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc3D<T: Scalar> {
    center: Point3D<T>,
    radius: T,
    normal: Direction3D<T>,    // 円弧平面の法線ベクトル（正規化済み）
    start_dir: Direction3D<T>, // 開始方向ベクトル（正規化済み）
    start_angle: Angle<T>,     // 開始角度
    end_angle: Angle<T>,       // 終了角度
}

impl<T: Scalar> Arc3D<T> {
    /// 新しい3D円弧を作成
    ///
    /// # 引数
    /// * `center` - 円弧の中心点
    /// * `radius` - 円弧の半径
    /// * `normal` - 円弧平面の法線方向（正規化済み）
    /// * `start_dir` - 開始方向（中心から開始点への方向、正規化済み）
    /// * `start_angle` - 開始角度
    /// * `end_angle` - 終了角度
    ///
    /// # 制約
    /// - `radius > 0`
    /// - `normal` と `start_dir` は直交していること
    pub fn new(
        center: Point3D<T>,
        radius: T,
        normal: Direction3D<T>,
        start_dir: Direction3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        // 半径の検証
        if radius <= T::ZERO {
            return None;
        }

        // 法線と開始方向の直交性チェック
        let dot_product = normal.as_vector().dot(&start_dir.as_vector()).abs();
        if dot_product > DefaultTolerances::angle::<T>() {
            return None;
        }

        Some(Self {
            center,
            radius,
            normal,
            start_dir,
            start_angle,
            end_angle,
        })
    }

    /// XY平面上の円弧を作成（便利メソッド）
    ///
    /// # 引数
    /// * `center` - 中心点
    /// * `radius` - 半径
    /// * `start_angle` - 開始角度
    /// * `end_angle` - 終了角度
    pub fn xy_arc(
        center: Point3D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let normal = Direction3D::from_vector(Vector3D::unit_z())?;
        let start_dir = Direction3D::from_vector(Vector3D::unit_x())?;
        Self::new(center, radius, normal, start_dir, start_angle, end_angle)
    }

    // === 基本アクセサメソッド ===

    /// 円弧の中心点を取得
    pub fn center(&self) -> Point3D<T> {
        self.center
    }

    /// 円弧の半径を取得
    pub fn radius(&self) -> T {
        self.radius
    }

    /// 円弧平面の法線ベクトルを取得
    pub fn normal(&self) -> Direction3D<T> {
        self.normal
    }

    /// 開始方向ベクトルを取得
    pub fn start_direction(&self) -> Direction3D<T> {
        self.start_dir
    }

    /// 開始角度を取得
    pub fn start_angle(&self) -> Angle<T> {
        self.start_angle
    }

    /// 終了角度を取得
    pub fn end_angle(&self) -> Angle<T> {
        self.end_angle
    }

    /// 円弧の角度範囲を取得
    pub fn angle_span(&self) -> Angle<T> {
        let mut span = self.end_angle - self.start_angle;
        if span.to_radians() < T::ZERO {
            span += Angle::from_radians(T::from_f64(2.0) * T::PI);
        }
        span
    }

    /// 円弧の長さを計算
    pub fn arc_length(&self) -> T {
        self.radius * self.angle_span().to_radians()
    }

    /// 完全円（360度）かどうか判定
    pub fn is_full_circle(&self) -> bool {
        let span = self.angle_span().to_radians();
        let two_pi = T::from_f64(2.0) * T::PI;
        (span - two_pi).abs() < DefaultTolerances::angle::<T>()
    }
}
