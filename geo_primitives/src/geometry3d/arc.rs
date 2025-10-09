//! 3D Arc implementation
//!
//! 3次元円弧の基本実装

use crate::geometry3d::{Circle, Point3D, Vector, Direction3D};
use geo_foundation::abstract_types::{Angle, Scalar};
use geo_foundation::abstract_types::geometry::common::{CurveAnalysis3D, AnalyticalCurve, CurveType, DifferentialGeometry};

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

    /// 指定角度での点取得
    pub fn point_at_angle(&self, angle: Angle<T>) -> Point3D<T> {
        self.circle.point_at_angle(angle)
    }

    /// パラメータt（0.0〜1.0）での点取得
    pub fn point_at_parameter(&self, t: T) -> Point3D<T> {
        let angle = self.interpolate_angle(t);
        self.circle.point_at_angle(angle)
    }

    /// 角度包含判定
    pub fn contains_angle(&self, angle: Angle<T>) -> bool {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();
        let check_rad = angle.to_radians();
        
        if start_rad <= end_rad {
            check_rad >= start_rad && check_rad <= end_rad
        } else {
            // 角度が2π境界をまたぐ場合
            check_rad >= start_rad || check_rad <= end_rad
        }
    }

    /// 中点取得
    pub fn midpoint(&self) -> Point3D<T> {
        let mid_angle = self.interpolate_angle(T::ONE / (T::ONE + T::ONE)); // 0.5
        self.circle.point_at_angle(mid_angle)
    }

    /// 角度範囲の取得（ラジアン）
    pub fn angle_span(&self) -> T {
        let start_rad = self.start_angle.to_radians();
        let end_rad = self.end_angle.to_radians();
        
        if end_rad >= start_rad {
            end_rad - start_rad
        } else {
            // 2π境界をまたぐ場合
            T::TAU - start_rad + end_rad
        }
    }

    /// パラメータtから実際の角度への変換
    fn interpolate_angle(&self, t: T) -> Angle<T> {
        let start_rad = self.start_angle.to_radians();
        let span = self.angle_span();
        let angle_rad = start_rad + span * t;
        Angle::from_radians(angle_rad)
    }
}

// =============================================================================
// 統一曲線解析インターフェイスの実装
// =============================================================================

/// Arc<T>に統一曲線解析インターフェイスを実装
impl<T: Scalar> CurveAnalysis3D<T> for Arc<T> {
    type Point = Point3D<T>;
    type Vector = Vector<T>;
    type Direction = Direction3D<T>;

    /// 指定されたパラメータ位置での点を取得
    /// t: 0.0=開始点、1.0=終了点
    fn point_at_parameter(&self, t: T) -> Self::Point {
        self.point_at_parameter(t)
    }

    /// 指定されたパラメータ位置での接線ベクトルを取得（正規化済み）
    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        let angle = self.interpolate_angle(t);
        // 基底円の接線を取得し、パラメータt位置での実際の角度で計算
        let circle_param = angle.to_radians() / T::TAU; // 円の0-1パラメータに変換
        self.circle.tangent_at_parameter(circle_param)
    }

    /// 指定されたパラメータ位置での主法線ベクトルを取得（正規化済み）
    fn normal_at_parameter(&self, t: T) -> Self::Vector {
        let angle = self.interpolate_angle(t);
        let circle_param = angle.to_radians() / T::TAU;
        self.circle.normal_at_parameter(circle_param)
    }

    /// 指定されたパラメータ位置での双法線ベクトルを取得（正規化済み）
    fn binormal_at_parameter(&self, t: T) -> Self::Vector {
        // 円弧の双法線は基底円と同じ（平面の法線）
        self.circle.binormal_at_parameter(T::ZERO) // 位置に依存しない
    }

    /// 指定されたパラメータ位置での曲率を取得
    fn curvature_at_parameter(&self, _t: T) -> T {
        // 円弧の曲率は基底円と同じ一定値
        self.circle.curvature_at_parameter(T::ZERO)
    }

    /// 指定されたパラメータ位置での捩率（ねじれ）を取得
    fn torsion_at_parameter(&self, _t: T) -> T {
        // 平面曲線（円弧）の捩率は常にゼロ
        T::ZERO
    }

    /// 指定されたパラメータ位置での微分幾何学的情報を一括取得（最も効率的）
    fn differential_geometry_at_parameter(&self, t: T) -> DifferentialGeometry<T, Self::Vector> {
        let angle = self.interpolate_angle(t);
        let circle_param = angle.to_radians() / T::TAU;
        
        // 基底円の微分幾何学情報を取得
        self.circle.differential_geometry_at_parameter(circle_param)
    }

    /// 最大曲率の位置と値を取得（円弧は一定曲率）
    fn max_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ONE / self.circle.radius())) // 任意の位置で一定曲率
    }

    /// 最小曲率の位置と値を取得（円弧は一定曲率）
    fn min_curvature(&self) -> Option<(T, T)> {
        Some((T::ZERO, T::ONE / self.circle.radius())) // 任意の位置で一定曲率
    }

    /// 曲率がゼロになる位置を取得（円弧では存在しない）
    fn inflection_points(&self) -> Vec<T> {
        Vec::new() // 円弧に変曲点は存在しない
    }

    /// 曲線が平面曲線かどうかを判定（円弧は常に平面曲線）
    fn is_planar(&self) -> bool {
        true
    }
}

/// Arc<T>に解析的曲線インターフェイスを実装
impl<T: Scalar> AnalyticalCurve<T> for Arc<T> {
    /// 曲線の種類（円弧）
    fn curve_type(&self) -> CurveType {
        CurveType::CircleArc
    }

    /// 一定曲率かどうか（円弧は常に一定曲率）
    fn has_constant_curvature(&self) -> bool {
        true
    }

    /// 解析的に計算可能な曲率の定数値（円弧の場合: 1/半径）
    fn constant_curvature(&self) -> Option<T> {
        Some(T::ONE / self.circle.radius())
    }

    /// 解析的に計算可能な曲率半径の定数値（円弧の場合: 半径）
    fn constant_curvature_radius(&self) -> Option<T> {
        Some(self.circle.radius())
    }
}

// 型エイリアス：他の形状実装との統一を目指したパターン
pub type Arc3DF64 = Arc<f64>; // f64版（ジェネリック実装）
pub type Arc3D = Arc3DF64; // デフォルトはf64版
pub type Arc3DF32 = Arc<f32>; // f32版（ジェネリック実装）
