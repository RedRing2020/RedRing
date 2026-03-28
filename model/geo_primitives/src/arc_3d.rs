//! Arc3D - Core Implementation
//!
//! 3次元円弧の基本実装とコンストラクタ、アクセサメソッド

use crate::{Angle, Direction3D, Point3D, Vector3D};
use geo_contracts::Scalar;
use geo_contracts::{default_angle_tolerance, default_distance_tolerance};
use geo_contracts::{Arc3DConstructor, Arc3DMeasure, Arc3DProperties as ContractsArc3DProperties};

/// 3次元円弧（基本実装）
///
/// 基本機能のみ：
/// - 作成・検証
/// - アクセサメソッド
/// - 基本的な幾何プロパティ
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc3D<T: Scalar> {
    pub(crate) center: Point3D<T>,
    pub(crate) radius: T,
    pub(crate) normal: Direction3D<T>, // 円弧平面の法線ベクトル（正規化済み）
    pub(crate) start_dir: Direction3D<T>, // 開始方向ベクトル（正規化済み）
    pub(crate) start_angle: Angle<T>,  // 開始角度
    pub(crate) end_angle: Angle<T>,    // 終了角度
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
        if dot_product > default_angle_tolerance::<T>() {
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

    // === 基本アクセサメソッド（内部使用） ===

    /// 円弧の中心点を取得（内部使用）
    pub(crate) fn center_internal(&self) -> Point3D<T> {
        self.center
    }

    /// 円弧の半径を取得（内部使用）
    pub(crate) fn radius_internal(&self) -> T {
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
        (span - two_pi).abs() < default_angle_tolerance::<T>()
    }

    /// 点が円弧の角度範囲内にあるかを判定
    ///
    /// 点が円弧上にあるかどうかではなく、角度範囲に収まっているかのみをチェック
    pub fn contains_point_angle(&self, point: Point3D<T>) -> bool {
        if self.is_full_circle() {
            return true; // 完全円の場合は全ての角度を含む
        }

        // 点から中心へのベクトルを計算
        let to_point = point - self.center;

        // 円弧平面への投影（法線に垂直な成分）
        let normal_vec = self.normal.as_vector();
        let projection = to_point - normal_vec * to_point.dot(&normal_vec);

        // 投影ベクトルがゼロの場合（点が円弧の中心軸上にある）
        if projection.magnitude() < default_distance_tolerance::<T>() {
            return false;
        }

        // 開始方向ベクトルとの角度を計算
        let start_vec = self.start_dir.as_vector();

        // 内積とcross積で角度を計算
        let cos_angle = projection.normalize().dot(&start_vec);
        let sin_angle = normal_vec.dot(&projection.normalize().cross(&start_vec));
        let point_angle = sin_angle.atan2(cos_angle);

        // 正規化（0 から 2π の範囲に）
        let normalize = |mut angle: T| {
            let two_pi = T::TAU;
            while angle < T::ZERO {
                angle += two_pi;
            }
            while angle >= two_pi {
                angle -= two_pi;
            }
            angle
        };

        let point_normalized = normalize(point_angle);
        let start_normalized = normalize(self.start_angle.to_radians());
        let end_normalized = normalize(self.end_angle.to_radians());

        // 角度範囲の判定
        if start_normalized <= end_normalized {
            point_normalized >= start_normalized && point_normalized <= end_normalized
        } else {
            point_normalized >= start_normalized || point_normalized <= end_normalized
        }
    }

    /// 指定角度での点を取得（内部用）
    fn point_at_angle_internal(&self, angle: Angle<T>) -> Point3D<T> {
        // 開始方向ベクトルを角度分回転
        let rotation_axis = self.normal.as_vector();
        let start_vec = self.start_dir.as_vector();

        // ロドリゲスの回転公式を使用
        let theta = angle.to_radians();
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let rotated = start_vec * cos_theta
            + rotation_axis.cross(&start_vec) * sin_theta
            + rotation_axis * (rotation_axis.dot(&start_vec) * (T::ONE - cos_theta));

        self.center + rotated * self.radius
    }
}

// ============================================================================
// Core Traits Implementation (Phase 1)
// ============================================================================

impl<T: Scalar> Arc3DConstructor<T> for Arc3D<T> {
    fn new(
        center: (T, T, T),
        radius: T,
        normal: (T, T, T),
        start_angle: T,
        end_angle: T,
    ) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let normal_dir = Direction3D::from_vector(normal_vec)?;

        // デフォルトの開始方向（法線に垂直なベクトル）を計算
        let start_dir = Self::compute_perpendicular(normal_dir)?;

        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);

        Self::new(center_point, radius, normal_dir, start_dir, start, end)
    }

    fn xy_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Self::xy_arc(center_point, radius, start, end)
    }

    fn from_three_points(start: (T, T, T), mid: (T, T, T), end: (T, T, T)) -> Option<Self> {
        let p1 = Point3D::new(start.0, start.1, start.2);
        let p2 = Point3D::new(mid.0, mid.1, mid.2);
        let p3 = Point3D::new(end.0, end.1, end.2);

        // 3点から円の中心と法線を計算
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal_vec = v1.cross(&v2);
        let normal_dir = Direction3D::from_vector(normal_vec)?;

        // 外接円の中心を計算
        let center = Self::circumcenter_3d(p1, p2, p3, normal_dir)?;
        let radius = center.distance_to(&p1);

        // 開始方向を計算
        let start_vec = p1 - center;
        let start_dir = Direction3D::from_vector(start_vec)?;

        // 角度を計算
        let start_angle = Angle::from_radians(T::ZERO);
        let end_vec = p3 - center;
        let end_angle = Self::angle_between_vectors(start_vec, end_vec, normal_vec);

        Self::new(
            center,
            radius,
            normal_dir,
            start_dir,
            start_angle,
            end_angle,
        )
    }

    // Phase 2: 追加コンストラクタ
    fn xz_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_dir = Direction3D::positive_y(); // XZ平面の法線はY軸
        let start_dir = Direction3D::positive_x(); // 開始方向はX軸
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Self::new(center_point, radius, normal_dir, start_dir, start, end)
    }

    fn yz_arc(center: (T, T, T), radius: T, start_angle: T, end_angle: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_dir = Direction3D::positive_x(); // YZ平面の法線はX軸
        let start_dir = Direction3D::positive_y(); // 開始方向はY軸
        let start = Angle::from_radians(start_angle);
        let end = Angle::from_radians(end_angle);
        Self::new(center_point, radius, normal_dir, start_dir, start, end)
    }

    fn full_circle(center: (T, T, T), normal: (T, T, T), radius: T) -> Option<Self> {
        let center_point = Point3D::new(center.0, center.1, center.2);
        let normal_vec = Vector3D::new(normal.0, normal.1, normal.2);
        let normal_dir = Direction3D::from_vector(normal_vec)?;
        let start_dir = Self::compute_perpendicular(normal_dir)?;
        Self::new(
            center_point,
            radius,
            normal_dir,
            start_dir,
            Angle::from_radians(T::ZERO),
            Angle::from_radians(T::from_f64(2.0) * T::PI),
        )
    }
}

impl<T: Scalar> ContractsArc3DProperties<T> for Arc3D<T> {
    fn center(&self) -> (T, T, T) {
        (self.center.x(), self.center.y(), self.center.z())
    }

    fn radius(&self) -> T {
        self.radius
    }

    fn start_angle(&self) -> T {
        self.start_angle.to_radians()
    }

    fn end_angle(&self) -> T {
        self.end_angle.to_radians()
    }

    fn dimension(&self) -> u32 {
        3
    }

    fn angle_span(&self) -> T {
        (self.end_angle.to_radians() - self.start_angle.to_radians()).abs()
    }

    fn is_full_circle(&self) -> bool {
        let span = (self.end_angle.to_radians() - self.start_angle.to_radians()).abs();
        (span - T::from_f64(2.0) * T::PI).abs() <= default_angle_tolerance::<T>()
    }

    fn is_on_xy_plane(&self) -> bool {
        let z_axis = Direction3D::positive_z();
        (self.normal.x() - z_axis.x()).abs() <= default_distance_tolerance::<T>()
            && (self.normal.y() - z_axis.y()).abs() <= default_distance_tolerance::<T>()
            && (self.normal.z() - z_axis.z()).abs() <= default_distance_tolerance::<T>()
    }
}

impl<T: Scalar> Arc3DMeasure<T> for Arc3D<T> {
    fn measure(&self) -> T {
        // arc_length の計算を直接展開: radius * angle_span
        let mut span = self.end_angle - self.start_angle;
        if span.to_radians() < T::ZERO {
            span += Angle::from_radians(T::from_f64(2.0) * T::PI);
        }
        self.radius * span.to_radians()
    }

    fn start_point(&self) -> (T, T, T) {
        let p = self.point_at_angle_internal(self.start_angle);
        (p.x(), p.y(), p.z())
    }

    fn end_point(&self) -> (T, T, T) {
        let p = self.point_at_angle_internal(self.end_angle);
        (p.x(), p.y(), p.z())
    }

    fn point_at_parameter(&self, t: T) -> (T, T, T) {
        let angle = self.start_angle + (self.end_angle - self.start_angle) * t;
        let p = self.point_at_angle_internal(angle);
        (p.x(), p.y(), p.z())
    }

    // Phase 2: 追加測度メソッド
    fn midpoint(&self) -> (T, T, T) {
        let mid_angle = (self.start_angle + self.end_angle) / (T::ONE + T::ONE);
        let p = self.point_at_angle_internal(mid_angle);
        (p.x(), p.y(), p.z())
    }

    fn point_at_angle(&self, angle: T) -> (T, T, T) {
        let p = self.point_at_angle_internal(Angle::from_radians(angle));
        (p.x(), p.y(), p.z())
    }

    fn distance_to_point(&self, point: (T, T, T)) -> T {
        // 簡易実装: 円弧上の最近点までの距離
        let center_pt = self.center_internal();
        let dx = point.0 - center_pt.x();
        let dy = point.1 - center_pt.y();
        let dz = point.2 - center_pt.z();
        ((dx * dx + dy * dy + dz * dz).sqrt() - self.radius_internal()).abs()
    }

    fn contains_point(&self, point: (T, T, T)) -> bool {
        // 簡易実装: 半径と角度範囲をチェック
        let center_pt = self.center_internal();
        let dx = point.0 - center_pt.x();
        let dy = point.1 - center_pt.y();
        let dz = point.2 - center_pt.z();
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
        (dist - self.radius_internal()).abs() <= default_distance_tolerance::<T>()
    }
}

// ============================================================================
// Helper methods for Arc3D
// ============================================================================

impl<T: Scalar> Arc3D<T> {
    /// 法線に垂直なベクトルを計算
    fn compute_perpendicular(normal: Direction3D<T>) -> Option<Direction3D<T>> {
        let n = normal.as_vector();

        // X軸との外積を試す
        let x_axis = Vector3D::unit_x();
        let perp1 = n.cross(&x_axis);

        if perp1.length() > default_distance_tolerance::<T>() {
            Direction3D::from_vector(perp1)
        } else {
            // X軸と平行な場合、Y軸を使用
            let y_axis = Vector3D::unit_y();
            let perp2 = n.cross(&y_axis);
            Direction3D::from_vector(perp2)
        }
    }

    /// 3D空間での外接円の中心を計算
    fn circumcenter_3d(
        p1: Point3D<T>,
        p2: Point3D<T>,
        p3: Point3D<T>,
        _normal: Direction3D<T>,
    ) -> Option<Point3D<T>> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;

        let v1_sq = v1.dot(&v1);
        let v2_sq = v2.dot(&v2);
        let v1_v2 = v1.dot(&v2);

        let denom = (T::ONE + T::ONE) * (v1_sq * v2_sq - v1_v2 * v1_v2);
        if denom.abs() < default_distance_tolerance::<T>() {
            return None;
        }

        let alpha = v2_sq * (v1_sq - v1_v2) / denom;
        let beta = v1_sq * (v2_sq - v1_v2) / denom;

        Some(p1 + v1 * alpha + v2 * beta)
    }

    /// 2つのベクトル間の角度を計算
    fn angle_between_vectors(v1: Vector3D<T>, v2: Vector3D<T>, normal: Vector3D<T>) -> Angle<T> {
        let cos_angle = v1.dot(&v2) / (v1.length() * v2.length());
        let angle = cos_angle.acos();

        // 符号を確認（法線との外積で判定）
        let cross = v1.cross(&v2);
        if cross.dot(&normal) < T::ZERO {
            Angle::from_radians(T::TAU - angle)
        } else {
            Angle::from_radians(angle)
        }
    }
}
