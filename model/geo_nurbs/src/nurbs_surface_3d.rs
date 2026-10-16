//! NURBSサーフェス実装
//!
//! Non-Uniform Rational B-Spline surfaces の基本実装です。
//! 制御点の2次元グリッド、重み、2方向のノットベクトルを使用して自由形状曲面を表現します。

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::basis::{basis_first_derivatives, basis_functions};
use crate::curve_evaluation::clamp_to_domain;
use crate::{KnotVector, Scalar, WeightStorage};
use geo_contracts::{
    default_parallel_cross_error_tolerance, NurbsError, NurbsSurface3DConstructor,
    NurbsSurface3DEvaluation, NurbsSurface3DProperties,
};
use geo_core::{Direction3D, Point3D, Vector3D};

/// NURBSサーフェス - 3次元（メモリ最適化版）
///
/// # 特徴
/// - フラット配列による高効率メモリ配置
/// - u, v 方向それぞれのノットベクトル
///
/// # メモリ構造
/// - 座標: `[x0,y0,z0, x1,y1,z1, ...]` (u方向優先配列)
/// - インデックス: `(u * v_count + v) * 3 + coord_offset`
#[derive(Debug, Clone)]
pub struct NurbsSurface3D<T: Scalar> {
    /// フラット座標配列 [x0,y0,z0, x1,y1,z1, ...] (u方向優先)
    coordinates: Vec<T>,
    /// 効率的重み管理
    weights: WeightStorage<T>,
    /// u方向のノットベクトル
    u_knots: KnotVector<T>,
    /// v方向のノットベクトル
    v_knots: KnotVector<T>,
    /// u方向の次数
    u_degree: usize,
    /// v方向の次数
    v_degree: usize,
    /// グリッドサイズ（高速化のため保持）
    u_count: usize,
    v_count: usize,
}

impl<T: Scalar> geo_contracts::PrimitiveMetadata for NurbsSurface3D<T> {
    fn primitive_kind(&self) -> geo_contracts::PrimitiveKind {
        geo_contracts::PrimitiveKind::NurbsSurface3D
    }
}

impl<T: Scalar> NurbsSurface3D<T> {
    /// 制御点の座標のみを写像した曲面を返す（重み・ノットベクトル・次数は保持する）
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
            u_knots: self.u_knots.clone(),
            v_knots: self.v_knots.clone(),
            u_degree: self.u_degree,
            v_degree: self.v_degree,
            u_count: self.u_count,
            v_count: self.v_count,
        }
    }

    /// 定義域から外れない `(u, v)` を定義域に収めて返す
    fn clamp_to_domain(&self, u: T, v: T) -> Option<(T, T)> {
        let (u_domain, v_domain) = self.parameter_domain();
        Some((clamp_to_domain(u, u_domain)?, clamp_to_domain(v, v_domain)?))
    }

    /// 定義域内の `(u, v)` での点と、u・v 方向の偏導関数
    ///
    /// 同次座標 `A = ΣΣ N_i M_j w_ij P_ij`、`w = ΣΣ N_i M_j w_ij` から、
    /// `S = A / w`、`S_u = (A_u - S w_u) / w`（v 方向も同様）とする。
    pub(super) fn point_and_partials(&self, u: T, v: T) -> (Point3D<T>, Vector3D<T>, Vector3D<T>) {
        let u_span = crate::knot::find_knot_span(u, &self.u_knots, self.u_degree);
        let v_span = crate::knot::find_knot_span(v, &self.v_knots, self.v_degree);
        let u_basis = basis_functions(u_span, self.u_degree, u, &self.u_knots);
        let v_basis = basis_functions(v_span, self.v_degree, v, &self.v_knots);
        let u_basis_derivatives = basis_first_derivatives(u_span, self.u_degree, u, &self.u_knots);
        let v_basis_derivatives = basis_first_derivatives(v_span, self.v_degree, v, &self.v_knots);

        // [x, y, z, w] の同次座標とその偏導関数
        let mut numerator = [T::ZERO; 4];
        let mut numerator_u = [T::ZERO; 4];
        let mut numerator_v = [T::ZERO; 4];
        for i in 0..=self.u_degree {
            for j in 0..=self.v_degree {
                let index =
                    (u_span - self.u_degree + i) * self.v_count + (v_span - self.v_degree + j);
                let weight = self.weights.weight(index);
                let point = [
                    self.coordinates[index * 3] * weight,
                    self.coordinates[index * 3 + 1] * weight,
                    self.coordinates[index * 3 + 2] * weight,
                    weight,
                ];
                let value = u_basis[i] * v_basis[j];
                let value_u = u_basis_derivatives[i] * v_basis[j];
                let value_v = u_basis[i] * v_basis_derivatives[j];
                for k in 0..4 {
                    numerator[k] += value * point[k];
                    numerator_u[k] += value_u * point[k];
                    numerator_v[k] += value_v * point[k];
                }
            }
        }

        let w = numerator[3];
        let point = Point3D::new(numerator[0] / w, numerator[1] / w, numerator[2] / w);
        let partial = |n: [T; 4]| {
            Vector3D::new(
                (n[0] - point.x() * n[3]) / w,
                (n[1] - point.y() * n[3]) / w,
                (n[2] - point.z() * n[3]) / w,
            )
        };
        (point, partial(numerator_u), partial(numerator_v))
    }
}

impl<T: Scalar> NurbsSurface3DConstructor<T> for NurbsSurface3D<T> {
    fn new(
        u_degree: usize,
        v_degree: usize,
        u_knots: Vec<T>,
        v_knots: Vec<T>,
        control_points: &[Vec<Point3D<T>>],
        weights: Option<Vec<Vec<T>>>,
    ) -> Result<Self, NurbsError> {
        // 制御点の格子の検証
        if control_points.is_empty() || control_points[0].is_empty() {
            return Err(NurbsError::EmptyControlGrid);
        }
        let u_count = control_points.len();
        let v_count = control_points[0].len();
        for (row, points) in control_points.iter().enumerate() {
            if points.len() != v_count {
                return Err(NurbsError::ControlGridRowMismatch {
                    row,
                    actual: points.len(),
                    expected: v_count,
                });
            }
        }
        for (degree, count) in [(u_degree, u_count), (v_degree, v_count)] {
            if count < degree + 1 {
                return Err(NurbsError::InsufficientControlPoints {
                    actual: count,
                    required: degree + 1,
                    degree,
                });
            }
        }

        // 重みの格子の検証（制御点と同じ u 方向優先の並びにする）
        let weights = match weights {
            Some(grid) => {
                if grid.len() != u_count {
                    return Err(NurbsError::WeightCountMismatch {
                        actual: grid.len(),
                        expected: u_count,
                    });
                }
                let mut flat = Vec::with_capacity(u_count * v_count);
                for row in &grid {
                    if row.len() != v_count {
                        return Err(NurbsError::WeightCountMismatch {
                            actual: row.len(),
                            expected: v_count,
                        });
                    }
                    flat.extend_from_slice(row);
                }
                if let Some(index) = flat.iter().position(|&weight| weight <= T::ZERO) {
                    return Err(NurbsError::NonPositiveWeight {
                        index,
                        weight: flat[index].to_f64(),
                    });
                }
                WeightStorage::Individual(flat)
            }
            None => WeightStorage::Uniform,
        };

        crate::knot::validate_knot_vector(&u_knots, u_degree, u_count)?;
        crate::knot::validate_knot_vector(&v_knots, v_degree, v_count)?;

        let coordinates = control_points
            .iter()
            .flatten()
            .flat_map(|p| [p.x(), p.y(), p.z()])
            .collect();

        Ok(Self {
            coordinates,
            weights,
            u_knots,
            v_knots,
            u_degree,
            v_degree,
            u_count,
            v_count,
        })
    }
}

impl<T: Scalar> NurbsSurface3DProperties<T> for NurbsSurface3D<T> {
    fn u_degree(&self) -> usize {
        self.u_degree
    }

    fn v_degree(&self) -> usize {
        self.v_degree
    }

    fn u_count(&self) -> usize {
        self.u_count
    }

    fn v_count(&self) -> usize {
        self.v_count
    }

    fn u_knots(&self) -> &[T] {
        &self.u_knots
    }

    fn v_knots(&self) -> &[T] {
        &self.v_knots
    }

    fn control_point(&self, u: usize, v: usize) -> Option<Point3D<T>> {
        if u >= self.u_count || v >= self.v_count {
            return None;
        }
        let coordinates = self.coordinates.as_chunks::<3>().0[u * self.v_count + v];
        Some(Point3D::new(coordinates[0], coordinates[1], coordinates[2]))
    }

    fn weight(&self, u: usize, v: usize) -> Option<T> {
        (u < self.u_count && v < self.v_count).then(|| self.weights.weight(u * self.v_count + v))
    }

    fn coordinates(&self) -> &[T] {
        &self.coordinates
    }

    fn weights(&self) -> Option<&[T]> {
        self.weights.as_slice()
    }

    fn parameter_domain(&self) -> ((T, T), (T, T)) {
        (
            crate::knot::get_parameter_domain(&self.u_knots, self.u_degree),
            crate::knot::get_parameter_domain(&self.v_knots, self.v_degree),
        )
    }
}

impl<T: Scalar> NurbsSurface3DEvaluation<T> for NurbsSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> Option<Point3D<T>> {
        let (u, v) = self.clamp_to_domain(u, v)?;
        Some(self.point_and_partials(u, v).0)
    }

    fn normal_at(&self, u: T, v: T) -> Option<Direction3D<T>> {
        let (u, v) = self.clamp_to_domain(u, v)?;
        let (_, partial_u, partial_v) = self.point_and_partials(u, v);
        let normal = partial_u.cross(partial_v);
        // 偏導関数が平行または 0 の場合（極など）は定まらない
        if normal.length()
            <= default_parallel_cross_error_tolerance::<T>()
                * partial_u.length()
                * partial_v.length()
        {
            return None;
        }
        Direction3D::from_vector(normal)
    }
}
