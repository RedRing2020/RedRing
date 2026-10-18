//! NURBSサーフェス実装
//!
//! Non-Uniform Rational B-Spline surfaces の基本実装です。
//! 制御点の2次元グリッド、重み、2方向のノットベクトルを使用して自由形状曲面を表現します。

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::basis::basis_derivatives;
use crate::curve_evaluation::clamp_to_domain;
use crate::{KnotVector, Scalar, WeightStorage};
use geo_contracts::{
    default_distance_tolerance, default_parallel_cross_error_tolerance, NurbsError,
    NurbsSurface3DConstructor, NurbsSurface3DEvaluation, NurbsSurface3DProperties,
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

    /// 定義域内の `(u, v)` での、`k + l <= order` の偏導関数 `∂^(k+l) S / ∂u^k ∂v^l`（`[k][l]`。`[0][0]` は点）
    ///
    /// 同次座標 `A = ΣΣ N_i M_j w_ij P_ij`、`w = ΣΣ N_i M_j w_ij` の偏導関数から、`A = w S` を
    /// ライプニッツの公式で微分した式を `S` の偏導関数について解く（The NURBS Book の A4.4）。
    fn partial_derivatives(&self, u: T, v: T, order: usize) -> Vec<Vec<[T; 3]>> {
        let u_span = crate::knot::find_knot_span(u, &self.u_knots, self.u_degree);
        let v_span = crate::knot::find_knot_span(v, &self.v_knots, self.v_degree);
        let u_basis = basis_derivatives(u_span, self.u_degree, u, &self.u_knots, order);
        let v_basis = basis_derivatives(v_span, self.v_degree, v, &self.v_knots, order);

        // スパンに効く制御点の [x, y, z, w] の同次座標（u 方向優先）
        let mut control_points = Vec::with_capacity((self.u_degree + 1) * (self.v_degree + 1));
        for i in 0..=self.u_degree {
            for j in 0..=self.v_degree {
                let index =
                    (u_span - self.u_degree + i) * self.v_count + (v_span - self.v_degree + j);
                let weight = self.weights.weight(index);
                control_points.push([
                    self.coordinates[index * 3] * weight,
                    self.coordinates[index * 3 + 1] * weight,
                    self.coordinates[index * 3 + 2] * weight,
                    weight,
                ]);
            }
        }

        // 同次座標の偏導関数
        let mut homogeneous = vec![vec![[T::ZERO; 4]; order + 1]; order + 1];
        for k in 0..=order {
            for l in 0..=order - k {
                let mut points = control_points.iter();
                for &u_value in &u_basis[k] {
                    for &v_value in &v_basis[l] {
                        let basis = u_value * v_value;
                        let point = points
                            .next()
                            .expect("スパンに効く制御点の数は基底関数の数の積");
                        for (sum, &coordinate) in homogeneous[k][l].iter_mut().zip(point) {
                            *sum += basis * coordinate;
                        }
                    }
                }
            }
        }

        let weight_sum = homogeneous[0][0][3];
        let mut derivatives = vec![vec![[T::ZERO; 3]; order + 1]; order + 1];
        for k in 0..=order {
            for l in 0..=order - k {
                let mut value = [
                    homogeneous[k][l][0],
                    homogeneous[k][l][1],
                    homogeneous[k][l][2],
                ];
                for i in 0..=k {
                    for j in 0..=l {
                        if i == 0 && j == 0 {
                            continue;
                        }
                        let factor =
                            binomial::<T>(k, i) * binomial::<T>(l, j) * homogeneous[i][j][3];
                        for (c, lower) in value.iter_mut().zip(derivatives[k - i][l - j]) {
                            *c -= factor * lower;
                        }
                    }
                }
                derivatives[k][l] = value.map(|c| c / weight_sum);
            }
        }
        derivatives
    }

    /// 定義域内の `(u, v)` での点と、u・v 方向の偏導関数
    pub(super) fn point_and_partials(&self, u: T, v: T) -> (Point3D<T>, Vector3D<T>, Vector3D<T>) {
        let derivatives = self.partial_derivatives(u, v, 1);
        let point = derivatives[0][0];
        (
            Point3D::new(point[0], point[1], point[2]),
            to_vector(derivatives[1][0]),
            to_vector(derivatives[0][1]),
        )
    }
}

fn to_vector<T: Scalar>([x, y, z]: [T; 3]) -> Vector3D<T> {
    Vector3D::new(x, y, z)
}

/// 二項係数 `n` C `k`
fn binomial<T: Scalar>(n: usize, k: usize) -> T {
    (0..k).fold(T::ONE, |value, i| {
        value * T::from_usize(n - i) / T::from_usize(i + 1)
    })
}

/// 外積の和 `Σ a × b` の向き
///
/// 和の大きさが、各項の大きさ `|a||b|` の和に対して外積が平行の判定の許容誤差以下の場合は `None`。
fn cross_sum_direction<T: Scalar>(terms: &[(Vector3D<T>, Vector3D<T>)]) -> Option<Direction3D<T>> {
    let mut sum = Vector3D::zero();
    let mut scale = T::ZERO;
    for &(a, b) in terms {
        sum = sum + a.cross(b);
        scale += a.length() * b.length();
    }
    if sum.length() <= default_parallel_cross_error_tolerance::<T>() * scale {
        return None;
    }
    Direction3D::from_vector(sum)
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

    /// `∂u × ∂v` の向きの法線
    ///
    /// 偏導関数が平行または 0 の点（極、1 点に縮退した辺など）では、定義域の中心へ向かう向き `(du, dv)`
    /// （各成分 ±1）から近づいたときの極限を返す。`ε` だけ近づいた点の `∂u × ∂v` を `ε` で展開し、
    /// 0 でない最初の項の向きとする（2 次の項まで 0 の場合は `None`）。展開は定義域の幅を掛けて長さの次元に
    /// した偏導関数で行い、長さが距離トレランス以下の偏導関数は 0 とみなす。
    fn normal_at(&self, u: T, v: T) -> Option<Direction3D<T>> {
        let (u, v) = self.clamp_to_domain(u, v)?;
        let ((u_start, u_end), (v_start, v_end)) = self.parameter_domain();
        let (u_width, v_width) = (u_end - u_start, v_end - v_start);
        let tolerance = default_distance_tolerance::<T>();
        let scaled = |derivative: Vector3D<T>, factor: T| {
            let vector = derivative * factor;
            if vector.length() <= tolerance {
                Vector3D::zero()
            } else {
                vector
            }
        };

        let first = self.partial_derivatives(u, v, 1);
        let partial_u = scaled(to_vector(first[1][0]), u_width);
        let partial_v = scaled(to_vector(first[0][1]), v_width);
        if let Some(normal) = cross_sum_direction(&[(partial_u, partial_v)]) {
            return Some(normal);
        }

        let two = T::ONE + T::ONE;
        let toward_center = |value: T, start: T, end: T| {
            if value <= (start + end) / two {
                T::ONE
            } else {
                -T::ONE
            }
        };
        let du = toward_center(u, u_start, u_end) * u_width;
        let dv = toward_center(v, v_start, v_end) * v_width;
        let second = self.partial_derivatives(u, v, 2);
        let [second_along_u, second_mixed, second_along_v] =
            [second[2][0], second[1][1], second[0][2]].map(to_vector);
        // 近づく向きへの ∂u・∂v の変化率
        let rate_of_partial_u = scaled(second_along_u * du + second_mixed * dv, u_width);
        let rate_of_partial_v = scaled(second_mixed * du + second_along_v * dv, v_width);
        cross_sum_direction(&[
            (partial_u, rate_of_partial_v),
            (rate_of_partial_u, partial_v),
        ])
        .or_else(|| cross_sum_direction(&[(rate_of_partial_u, rate_of_partial_v)]))
    }
}
