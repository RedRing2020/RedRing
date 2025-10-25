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
#[derive(Debug, Clone, PartialEq)]
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

        // 基本的な直交性チェック
        let dot_product = normal.dot(&start_dir);
        if dot_product.abs() > DefaultTolerances::distance::<T>() {
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

    /// 3点を通る円弧を作成
    ///
    /// # 引数
    /// * `start` - 開始点
    /// * `middle` - 中間点
    /// * `end` - 終了点
    pub fn from_three_points(
        start: Point3D<T>,
        middle: Point3D<T>,
        end: Point3D<T>,
    ) -> Option<Self> {
        // 3点が一直線上にあるかチェック
        let v1 = middle - start;
        let v2 = end - start;
        let cross = v1.cross(&v2);

        if cross.length() <= DefaultTolerances::distance::<T>() {
            return None; // 一直線上の点
        }

        // 円の中心を計算（外心）
        let v1_len_sq = v1.length_squared();
        let v2_len_sq = v2.length_squared();
        let cross_len_sq = cross.length_squared();

        let alpha = v2_len_sq * v1.dot(&(v1 - v2)) / (T::from_f64(2.0) * cross_len_sq);
        let beta = v1_len_sq * v2.dot(&(v2 - v1)) / (T::from_f64(2.0) * cross_len_sq);

        let center = start + v1 * alpha + v2 * beta;
        let radius = (start - center).length();

        // 法線ベクトル（右手系）
        let normal_dir = Direction3D::from_vector(cross.normalize())?;

        // 開始方向
        let start_dir = Direction3D::from_vector((start - center).normalize())?;

        // 角度計算は簡略化（基本実装では0から2πの範囲とする）
        Self::new(
            center,
            radius,
            normal_dir,
            start_dir,
            Angle::from_radians(T::ZERO),
            Angle::from_radians(T::PI),
        )
    }

    /// Vector3Dから円弧を作成（後方互換性）
    pub fn from_vectors(
        center: Point3D<T>,
        radius: T,
        normal: Vector3D<T>,
        start_dir: Vector3D<T>,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        let normal_dir = Direction3D::from_vector(normal)?;
        let start_dir_dir = Direction3D::from_vector(start_dir)?;
        Self::new(
            center,
            radius,
            normal_dir,
            start_dir_dir,
            start_angle,
            end_angle,
        )
    }

    /// XY平面上の円弧を作成
    pub fn xy_arc(
        center: Point3D<T>,
        radius: T,
        start_angle: Angle<T>,
        end_angle: Angle<T>,
    ) -> Option<Self> {
        Self::new(
            center,
            radius,
            Direction3D::from_vector(Vector3D::unit_z()).unwrap(),
            Direction3D::from_vector(Vector3D::unit_x()).unwrap(),
            start_angle,
            end_angle,
        )
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

    // === 基本的な幾何プロパティ ===

    /// 円弧の長さを計算
    pub fn arc_length(&self) -> T {
        self.radius * self.angle_span().to_radians()
    }

    /// 円弧が完全な円かどうかを判定
    pub fn is_full_circle(&self) -> bool {
        let full_circle = Angle::from_radians(T::from_f64(2.0) * T::PI);
        let diff = (self.angle_span().to_radians() - full_circle.to_radians()).abs();
        diff <= DefaultTolerances::angle::<T>()
    }

    /// 円弧が退化しているかどうかを判定
    pub fn is_degenerate(&self) -> bool {
        let tolerance = DefaultTolerances::distance::<T>();
        self.radius <= tolerance
            || self.angle_span().to_radians() <= DefaultTolerances::angle::<T>()
    }

    // === 基本的なパラメトリック操作 ===

    /// パラメータ t での円弧上の点を計算
    /// t ∈ [0, 1] で正規化
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle = self.start_angle.to_radians() + self.angle_span().to_radians() * t;
        self.point_at_angle(angle)
    }

    /// 角度 θ での円弧上の点を計算
    pub fn point_at_angle(&self, angle: T) -> Point3D<T> {
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // 円弧平面内での2つの直交軸
        let u_axis = self.start_dir;
        let v_axis = self.normal.cross(&u_axis);

        let point_on_circle = u_axis.as_vector() * (self.radius * cos_angle)
            + v_axis.as_vector() * (self.radius * sin_angle);

        self.center + point_on_circle
    }

    /// 開始点を取得
    pub fn start_point(&self) -> Point3D<T> {
        self.point_at_angle(self.start_angle.to_radians())
    }

    /// 終了点を取得
    pub fn end_point(&self) -> Point3D<T> {
        self.point_at_angle(self.end_angle.to_radians())
    }

    /// 中点を取得
    pub fn mid_point(&self) -> Point3D<T> {
        let mid_angle =
            self.start_angle.to_radians() + self.angle_span().to_radians() / T::from_f64(2.0);
        self.point_at_angle(mid_angle)
    }

    /// パラメータ範囲を取得
    pub fn parameter_range(&self) -> (T, T) {
        (T::ZERO, T::ONE)
    }

    // === 基本的な検証メソッド ===

    /// 角度が円弧の範囲内にあるかチェック
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        let normalized_angle = self.normalize_angle(angle);
        let start = self.normalize_angle(self.start_angle);
        let end = self.normalize_angle(self.end_angle);

        if start.to_radians() <= end.to_radians() {
            normalized_angle.to_radians() >= start.to_radians()
                && normalized_angle.to_radians() <= end.to_radians()
        } else {
            // 0度をまたぐ場合
            normalized_angle.to_radians() >= start.to_radians()
                || normalized_angle.to_radians() <= end.to_radians()
        }
    }

    /// 角度を [0, 2π] の範囲に正規化
    fn normalize_angle(&self, angle: Angle<T>) -> Angle<T> {
        let two_pi = Angle::from_radians(T::from_f64(2.0) * T::PI);
        let mut normalized = angle;
        while normalized.to_radians() < T::ZERO {
            normalized += two_pi;
        }
        while normalized >= two_pi {
            normalized -= two_pi;
        }
        normalized
    }
}
