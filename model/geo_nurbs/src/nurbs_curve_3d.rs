//! NURBS 3D曲線実装（メモリ最適化版）
//!
//! Non-Uniform Rational B-Spline 3D curves の基本実装です。
//! フラット配列による高効率メモリ配置で制御点、重み、ノットベクトルを管理します。

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::curve_evaluation::{validate_curve, CurveDefinition};
use crate::{KnotVector, Scalar, WeightStorage};
use geo_contracts::{
    default_distance_tolerance, NurbsCurve3DConstructor, NurbsCurve3DDerived, NurbsCurve3DDistance,
    NurbsCurve3DEvaluation, NurbsCurve3DProperties, NurbsError,
};
use geo_core::{Point3D, Vector3D};

/// NURBS曲線 - 3次元（メモリ最適化版）
///
/// # 特徴
/// - フラット配列による高効率メモリ配置
/// - 有理ベーシス関数による滑らかな補間
/// - 局所的制御性（制御点の変更が局所的影響のみ）
/// - アフィン変換の不変性
///
/// # メモリ構造
/// - 座標: `[x0,y0,z0, x1,y1,z1, x2,y2,z2, ...]` (フラット配列)
/// - インデックス: `i * 3 + coord_offset`
#[derive(Debug, Clone)]
pub struct NurbsCurve3D<T: Scalar> {
    /// フラット座標配列 [x0,y0,z0, x1,y1,z1, ...]
    coordinates: Vec<T>,
    /// 効率的重み管理
    weights: WeightStorage<T>,
    /// ノットベクトル
    knot_vector: KnotVector<T>,
    /// NURBS次数
    degree: usize,
    /// 制御点数（高速化のため保持）
    num_points: usize,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for NurbsCurve3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::NurbsCurve3D
    }
}

impl<T: Scalar> NurbsCurve3D<T> {
    /// 評価に使う曲線の定義
    fn definition(&self) -> CurveDefinition<'_, T, 3> {
        CurveDefinition {
            coordinates: &self.coordinates,
            weights: &self.weights,
            knots: &self.knot_vector,
            degree: self.degree,
        }
    }

    /// 制御点の座標のみを写像した曲線を返す（重み・ノットベクトル・次数は保持する）
    fn map_control_points(&self, f: impl Fn(Point3D<T>) -> Point3D<T>) -> Self {
        let mut coordinates = Vec::with_capacity(self.coordinates.len());
        let (points, _) = self.coordinates.as_chunks::<3>();
        for &[x, y, z] in points {
            let p = f(Point3D::new(x, y, z));
            coordinates.extend([p.x(), p.y(), p.z()]);
        }
        Self {
            coordinates,
            weights: self.weights.clone(),
            knot_vector: self.knot_vector.clone(),
            degree: self.degree,
            num_points: self.num_points,
        }
    }
}

impl<T: Scalar> NurbsCurve3DConstructor<T> for NurbsCurve3D<T> {
    fn new(
        degree: usize,
        knots: Vec<T>,
        control_points: &[Point3D<T>],
        weights: Option<Vec<T>>,
    ) -> Result<Self, NurbsError> {
        let weights = validate_curve(degree, &knots, control_points.len(), weights)?;
        let coordinates = control_points
            .iter()
            .flat_map(|p| [p.x(), p.y(), p.z()])
            .collect();

        Ok(Self {
            coordinates,
            weights,
            knot_vector: knots,
            degree,
            num_points: control_points.len(),
        })
    }
}

impl<T: Scalar> NurbsCurve3DProperties<T> for NurbsCurve3D<T> {
    fn degree(&self) -> usize {
        self.degree
    }

    fn knot_vector(&self) -> &[T] {
        &self.knot_vector
    }

    fn control_point_count(&self) -> usize {
        self.num_points
    }

    fn control_point(&self, index: usize) -> Option<Point3D<T>> {
        let [x, y, z] = self.coordinates.as_chunks::<3>().0.get(index)?;
        Some(Point3D::new(*x, *y, *z))
    }

    fn weight(&self, index: usize) -> Option<T> {
        (index < self.num_points).then(|| self.weights.weight(index))
    }

    fn coordinates(&self) -> &[T] {
        &self.coordinates
    }

    fn weights(&self) -> Option<&[T]> {
        self.weights.as_slice()
    }

    fn parameter_domain(&self) -> (T, T) {
        self.definition().parameter_domain()
    }
}

impl<T: Scalar> NurbsCurve3DEvaluation<T> for NurbsCurve3D<T> {
    fn point_at_parameter(&self, t: T) -> Option<Point3D<T>> {
        let definition = self.definition();
        let ([x, y, z], _) = definition.point_and_derivative(definition.clamp_to_domain(t)?);
        Some(Point3D::new(x, y, z))
    }

    fn derivative_at(&self, t: T) -> Option<Vector3D<T>> {
        let definition = self.definition();
        let (_, [x, y, z]) = definition.point_and_derivative(definition.clamp_to_domain(t)?);
        Some(Vector3D::new(x, y, z))
    }
}

impl<T: Scalar> NurbsCurve3DDistance<T> for NurbsCurve3D<T> {
    fn distance_to_point(&self, point: Point3D<T>) -> T {
        geo_commons::nurbs_curve_minimum_distance(
            self.degree(),
            self.knot_vector(),
            self.coordinates(),
            self.weights(),
            &geo_commons::ConvexSetDistance::new(|_| [point.x(), point.y(), point.z()]),
            default_distance_tolerance(),
        )
    }
}

impl<T: Scalar> NurbsCurve3DDerived<T> for NurbsCurve3D<T> {
    fn length(&self) -> T {
        self.definition().length()
    }
}
