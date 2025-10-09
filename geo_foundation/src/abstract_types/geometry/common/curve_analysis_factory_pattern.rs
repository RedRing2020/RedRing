//! 統一ファクトリパターンの実装例
//!
//! 形状判定→計算ロジックディスパッチによる統一API

use crate::abstract_types::geometry::curve_analysis::{CurveAnalysis3D, CurveType, DifferentialGeometry};
use crate::abstract_types::Scalar;

/// 曲線解析の統一ファクトリ
pub struct CurveAnalysisFactory;

impl CurveAnalysisFactory {
    /// 統一された曲率計算
    pub fn curvature_at_parameter<T: Scalar>(
        curve_type: CurveType,
        geometry_data: &CurveGeometryData<T>,
        t: T,
    ) -> T {
        match curve_type {
            CurveType::Circle => {
                // 円: 一定曲率 1/半径
                T::ONE / geometry_data.radius.expect("Circle requires radius")
            }
            CurveType::CircleArc => {
                // 円弧: 円と同じ
                T::ONE / geometry_data.radius.expect("Arc requires radius")
            }
            CurveType::Ellipse => {
                // 楕円: 位置依存の曲率計算
                let a = geometry_data.semi_major_axis.expect("Ellipse requires semi_major_axis");
                let b = geometry_data.semi_minor_axis.expect("Ellipse requires semi_minor_axis");
                Self::ellipse_curvature_at_parameter(a, b, t)
            }
            CurveType::Line => {
                // 直線: 曲率ゼロ
                T::ZERO
            }
            CurveType::Nurbs => {
                // NURBS: 数値微分
                Self::numerical_curvature(geometry_data, t)
            }
            _ => {
                // その他: 数値微分にフォールバック
                Self::numerical_curvature(geometry_data, t)
            }
        }
    }

    /// 統一された接線計算
    pub fn tangent_at_parameter<T: Scalar>(
        curve_type: CurveType,
        geometry_data: &CurveGeometryData<T>,
        t: T,
    ) -> Vector3D<T> {
        match curve_type {
            CurveType::Circle | CurveType::CircleArc => {
                Self::circle_tangent_at_parameter(geometry_data, t)
            }
            CurveType::Ellipse => {
                Self::ellipse_tangent_at_parameter(geometry_data, t)
            }
            CurveType::Line => {
                geometry_data.direction.expect("Line requires direction").to_vector()
            }
            _ => {
                Self::numerical_tangent(geometry_data, t)
            }
        }
    }

    /// 統一された微分幾何学的情報取得（最も効率的）
    pub fn differential_geometry_at_parameter<T: Scalar>(
        curve_type: CurveType,
        geometry_data: &CurveGeometryData<T>,
        t: T,
    ) -> DifferentialGeometry<T, Vector3D<T>> {
        // 一括計算でパフォーマンス最適化
        match curve_type {
            CurveType::Circle | CurveType::CircleArc => {
                // 円/円弧: 解析的一括計算
                let (tangent, normal, curvature) = Self::circle_differential_geometry(geometry_data, t);
                DifferentialGeometry::new(tangent, normal, curvature)
            }
            CurveType::Ellipse => {
                // 楕円: 解析的一括計算
                let (tangent, normal, curvature) = Self::ellipse_differential_geometry(geometry_data, t);
                DifferentialGeometry::new(tangent, normal, curvature)
            }
            _ => {
                // その他: 個別計算（数値微分等）
                let tangent = Self::tangent_at_parameter(curve_type, geometry_data, t);
                let normal = Self::normal_at_parameter(curve_type, geometry_data, t);
                let curvature = Self::curvature_at_parameter(curve_type, geometry_data, t);
                DifferentialGeometry::new(tangent, normal, curvature)
            }
        }
    }

    // ========== 形状固有の計算ロジック ==========
    
    fn circle_tangent_at_parameter<T: Scalar>(
        data: &CurveGeometryData<T>,
        t: T,
    ) -> Vector3D<T> {
        let angle = data.start_angle.unwrap_or(T::ZERO) + t * data.angle_range.unwrap_or(T::TAU);
        let radius = data.radius.expect("Circle requires radius");
        Vector3D::new(-angle.sin() * radius, angle.cos() * radius, T::ZERO)
    }

    fn ellipse_curvature_at_parameter<T: Scalar>(a: T, b: T, t: T) -> T {
        // 楕円の曲率: κ = ab / (a²sin²θ + b²cos²θ)^(3/2)
        let angle = t * T::TAU;
        let sin_t = angle.sin();
        let cos_t = angle.cos();
        let denominator = (a * a * sin_t * sin_t + b * b * cos_t * cos_t).powi(3).sqrt();
        a * b / denominator
    }

    fn circle_differential_geometry<T: Scalar>(
        data: &CurveGeometryData<T>,
        t: T,
    ) -> (Vector3D<T>, Vector3D<T>, T) {
        let angle = data.start_angle.unwrap_or(T::ZERO) + t * data.angle_range.unwrap_or(T::TAU);
        let radius = data.radius.expect("Circle requires radius");
        
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        let tangent = Vector3D::new(-sin_a, cos_a, T::ZERO).normalize();
        let normal = Vector3D::new(cos_a, sin_a, T::ZERO);
        let curvature = T::ONE / radius;
        
        (tangent, normal, curvature)
    }

    fn ellipse_differential_geometry<T: Scalar>(
        data: &CurveGeometryData<T>,
        t: T,
    ) -> (Vector3D<T>, Vector3D<T>, T) {
        // 楕円の解析的微分幾何学計算
        // TODO: 実装
        todo!("Ellipse differential geometry implementation")
    }

    fn numerical_curvature<T: Scalar>(data: &CurveGeometryData<T>, t: T) -> T {
        // 数値微分による曲率計算
        let delta = T::TOLERANCE.sqrt();
        // TODO: 実装
        todo!("Numerical curvature calculation")
    }

    fn numerical_tangent<T: Scalar>(data: &CurveGeometryData<T>, t: T) -> Vector3D<T> {
        // 数値微分による接線計算
        todo!("Numerical tangent calculation")
    }

    fn normal_at_parameter<T: Scalar>(
        curve_type: CurveType,
        data: &CurveGeometryData<T>,
        t: T,
    ) -> Vector3D<T> {
        // TODO: 実装
        todo!("Normal calculation")
    }
}

/// 曲線の幾何データを統一的に保持する構造体
#[derive(Debug, Clone)]
pub struct CurveGeometryData<T: Scalar> {
    // 共通フィールド
    pub center: Option<Point3D<T>>,
    pub start_point: Option<Point3D<T>>,
    pub end_point: Option<Point3D<T>>,
    
    // 円/円弧用
    pub radius: Option<T>,
    pub start_angle: Option<T>,
    pub angle_range: Option<T>,
    
    // 楕円用
    pub semi_major_axis: Option<T>,
    pub semi_minor_axis: Option<T>,
    
    // 直線用
    pub direction: Option<Direction3D<T>>,
    
    // NURBS用
    pub control_points: Option<Vec<Point3D<T>>>,
    pub weights: Option<Vec<T>>,
    pub knots: Option<Vec<T>>,
    pub degree: Option<usize>,
}

// 具象型での使用例
impl<T: Scalar> Circle<T> {
    pub fn to_geometry_data(&self) -> CurveGeometryData<T> {
        CurveGeometryData {
            center: Some(self.center),
            radius: Some(self.radius),
            start_angle: Some(self.start_angle),
            angle_range: Some(self.angle_range),
            ..Default::default()
        }
    }
}

// 統一インターフェース実装
impl<T: Scalar> CurveAnalysis3D<T> for Circle<T> {
    type Point = Point3D<T>;
    type Vector = Vector3D<T>;
    type Direction = Direction3D<T>;

    fn curvature_at_parameter(&self, t: T) -> T {
        CurveAnalysisFactory::curvature_at_parameter(
            CurveType::Circle,
            &self.to_geometry_data(),
            t,
        )
    }

    fn tangent_at_parameter(&self, t: T) -> Self::Vector {
        CurveAnalysisFactory::tangent_at_parameter(
            CurveType::Circle,
            &self.to_geometry_data(),
            t,
        )
    }

    fn differential_geometry_at_parameter(&self, t: T) -> DifferentialGeometry<T, Self::Vector> {
        CurveAnalysisFactory::differential_geometry_at_parameter(
            CurveType::Circle,
            &self.to_geometry_data(),
            t,
        )
    }

    // その他のメソッドも同様にファクトリ経由
}