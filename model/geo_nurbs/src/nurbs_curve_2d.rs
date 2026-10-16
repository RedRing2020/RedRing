//! NURBS 2D曲線実装（メモリ最適化版）
//!
//! Non-Uniform Rational B-Spline 2D curves の基本実装です。
//! フラット配列による高効率メモリ配置で制御点、重み、ノットベクトルを管理します。

mod bounds;
#[cfg(test)]
mod tests;
mod transform;

use crate::curve_evaluation::{validate_curve, CurveDefinition};
use crate::{KnotVector, Scalar, WeightStorage};
use geo_contracts::{
    NurbsCurve2DConstructor, NurbsCurve2DDerived, NurbsCurve2DEvaluation, NurbsCurve2DProperties,
    NurbsError,
};
use geo_core::{Point2D, Vector2D};

/// NURBS曲線 - 2次元（Foundation パターン準拠・ジェネリック）
///
/// # 特徴
/// - フラット配列による高効率メモリ配置
/// - 有理ベーシス関数による滑らかな補間
/// - 局所的制御性（制御点の変更が局所的影響のみ）
/// - アフィン変換の不変性
///
/// # メモリ構造
/// - 座標: `[x0,y0, x1,y1, x2,y2, ...]` (フラット配列)
/// - インデックス: `i * 2 + coord_offset`
#[derive(Debug, Clone)]
pub struct NurbsCurve2D<T: Scalar> {
    /// フラット座標配列 [x0,y0, x1,y1, ...]
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

impl<T: Scalar> geo_contracts::PrimitiveMetadata for NurbsCurve2D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::NurbsCurve2D
    }
}

impl<T: Scalar> NurbsCurve2D<T> {
    /// 評価に使う曲線の定義
    fn definition(&self) -> CurveDefinition<'_, T, 2> {
        CurveDefinition {
            coordinates: &self.coordinates,
            weights: &self.weights,
            knots: &self.knot_vector,
            degree: self.degree,
        }
    }

    /// 制御点の座標のみを写像した曲線を返す（重み・ノットベクトル・次数は保持する）
    fn map_control_points(&self, f: impl Fn(Point2D<T>) -> Point2D<T>) -> Self {
        let mut coordinates = Vec::with_capacity(self.coordinates.len());
        let (points, _) = self.coordinates.as_chunks::<2>();
        for &[x, y] in points {
            let p = f(Point2D::new(x, y));
            coordinates.extend([p.x(), p.y()]);
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

impl<T: Scalar> NurbsCurve2DConstructor<T> for NurbsCurve2D<T> {
    fn new(
        degree: usize,
        knots: Vec<T>,
        control_points: &[Point2D<T>],
        weights: Option<Vec<T>>,
    ) -> Result<Self, NurbsError> {
        let weights = validate_curve(degree, &knots, control_points.len(), weights)?;
        let coordinates = control_points.iter().flat_map(|p| [p.x(), p.y()]).collect();

        Ok(Self {
            coordinates,
            weights,
            knot_vector: knots,
            degree,
            num_points: control_points.len(),
        })
    }
}

impl<T: Scalar> NurbsCurve2DProperties<T> for NurbsCurve2D<T> {
    fn degree(&self) -> usize {
        self.degree
    }

    fn knot_vector(&self) -> &[T] {
        &self.knot_vector
    }

    fn control_point_count(&self) -> usize {
        self.num_points
    }

    fn control_point(&self, index: usize) -> Option<Point2D<T>> {
        let [x, y] = self.coordinates.as_chunks::<2>().0.get(index)?;
        Some(Point2D::new(*x, *y))
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

impl<T: Scalar> NurbsCurve2DEvaluation<T> for NurbsCurve2D<T> {
    fn point_at_parameter(&self, t: T) -> Option<Point2D<T>> {
        let definition = self.definition();
        let ([x, y], _) = definition.point_and_derivative(definition.clamp_to_domain(t)?);
        Some(Point2D::new(x, y))
    }

    fn derivative_at(&self, t: T) -> Option<Vector2D<T>> {
        let definition = self.definition();
        let (_, [x, y]) = definition.point_and_derivative(definition.clamp_to_domain(t)?);
        Some(Vector2D::new(x, y))
    }
}

impl<T: Scalar> NurbsCurve2DDerived<T> for NurbsCurve2D<T> {
    fn length(&self) -> T {
        self.definition().length()
    }
}
