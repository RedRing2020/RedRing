//! NURBSサーフェス実装
//!
//! Non-Uniform Rational B-Spline surfaces の基本実装です。
//! 制御点の2次元グリッド、重み、2方向のノットベクトルを使用して自由形状曲面を表現します。

mod bounds;
mod extensions;
#[cfg(test)]
mod tests;
mod transform;

use crate::{constants, KnotVector, NurbsError, Result, Scalar, WeightStorage};
use analysis::linalg::vector::Vector3;
use geo_contracts::{
    NurbsSurface3DConstructor, NurbsSurface3DDerived, NurbsSurface3DEvaluation,
    NurbsSurface3DProperties,
};

/// NURBSサーフェス - 3次元（メモリ最適化版）
///
/// # 特徴
/// - フラット配列による高効率メモリ配置
/// - u, v 方向それぞれのノットベクトル
/// - 双方向パラメトリック制御
/// - 滑らかな曲面補間
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
    /// 内部用コンストラクタ（クレート内専用）
    ///
    /// # 引数
    /// * `control_points` - 制御点の2次元グリッド [`u_count`][v_count]
    /// * `weights` - 重みの2次元グリッド（Noneの場合は非有理）
    /// * `u_knots` - u方向ノットベクトル
    /// * `v_knots` - v方向ノットベクトル
    /// * `u_degree` - u方向次数
    /// * `v_degree` - v方向次数
    ///
    /// # Errors
    /// * 制御点と重みのサイズが一致しない場合
    /// * ノットベクトルが無効な場合
    #[allow(clippy::needless_pass_by_value)] // 制御点グリッドを所有する必要がある
    fn new_internal(
        control_points: Vec<Vec<Vector3<T>>>,
        weights: Option<Vec<Vec<T>>>,
        u_knots: KnotVector<T>,
        v_knots: KnotVector<T>,
        u_degree: usize,
        v_degree: usize,
    ) -> Result<Self> {
        // グリッドサイズの検証
        if control_points.is_empty() || control_points[0].is_empty() {
            return Err(NurbsError::DegenerateGeometry {
                reason: "制御点グリッドが空です".to_string(),
            });
        }

        let u_count = control_points.len();
        let v_count = control_points[0].len();

        // 制御点グリッドの一貫性確認
        for row in &control_points {
            if row.len() != v_count {
                return Err(NurbsError::DegenerateGeometry {
                    reason: "制御点グリッドの行サイズが一致しません".to_string(),
                });
            }
        }

        // フラット座標配列を構築
        let mut coordinates = Vec::with_capacity(u_count * v_count * 3_usize);
        #[allow(clippy::needless_range_loop)] // 2次元グリッドアクセスの標準パターン
        for u in 0..u_count {
            #[allow(clippy::needless_range_loop)] // 2次元グリッドアクセスの標準パターン
            for v in 0..v_count {
                let point = &control_points[u][v];
                coordinates.push(point.x());
                coordinates.push(point.y());
                coordinates.push(point.z());
            }
        }

        // 重み配列を処理
        let weight_storage = if let Some(weight_grid) = weights {
            // 重みグリッドの検証
            if weight_grid.len() != u_count {
                return Err(NurbsError::WeightCountMismatch {
                    actual: weight_grid.len(),
                    expected: u_count,
                });
            }

            let mut flat_weights = Vec::with_capacity(u_count * v_count);
            #[allow(clippy::needless_range_loop)] // 重みグリッドの検証と変換
            for u in 0..u_count {
                if weight_grid[u].len() != v_count {
                    return Err(NurbsError::WeightCountMismatch {
                        actual: weight_grid[u].len(),
                        expected: v_count,
                    });
                }

                #[allow(clippy::needless_range_loop)] // 重みグリッドの検証と変換
                for v in 0..v_count {
                    let weight = weight_grid[u][v];
                    if weight <= T::ZERO {
                        return Err(NurbsError::InvalidWeight {
                            weight: weight.to_f64(),
                        });
                    }
                    flat_weights.push(weight);
                }
            }
            WeightStorage::Individual(flat_weights)
        } else {
            WeightStorage::Uniform
        };

        // ノットベクトルの検証
        crate::knot::validate_knot_vector(&u_knots, u_degree, u_count)?;
        crate::knot::validate_knot_vector(&v_knots, v_degree, v_count)?;

        Ok(NurbsSurface3D {
            coordinates,
            weights: weight_storage,
            u_knots,
            v_knots,
            u_degree,
            v_degree,
            u_count,
            v_count,
        })
    }

    /// 制御点アクセス用インデックス計算
    #[inline]
    fn control_point_index(&self, u: usize, v: usize) -> usize {
        debug_assert!(u < self.u_count);
        debug_assert!(v < self.v_count);
        (u * self.v_count + v) * 3
    }

    /// 重みアクセス用インデックス計算
    #[inline]
    fn weight_index(&self, u: usize, v: usize) -> usize {
        debug_assert!(u < self.u_count);
        debug_assert!(v < self.v_count);
        u * self.v_count + v
    }

    /// 制御点の座標のみを写像した曲面を返す（重み・ノットベクトル・次数は保持する）
    fn map_control_points(&self, f: impl Fn((T, T, T)) -> (T, T, T)) -> Self {
        let mut coordinates = Vec::with_capacity(self.coordinates.len());
        let (points, _) = self.coordinates.as_chunks::<3>();
        for &[x, y, z] in points {
            let (x, y, z) = f((x, y, z));
            coordinates.extend([x, y, z]);
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

    /// 制御点取得
    #[must_use]
    pub fn control_point(&self, u: usize, v: usize) -> Vector3<T> {
        let idx = self.control_point_index(u, v);
        Vector3::new(
            self.coordinates[idx],
            self.coordinates[idx + 1],
            self.coordinates[idx + 2],
        )
    }

    /// 重み取得
    #[must_use]
    pub fn weight(&self, u: usize, v: usize) -> T {
        match &self.weights {
            WeightStorage::Uniform => T::ONE,
            WeightStorage::Individual(weights) => weights[self.weight_index(u, v)],
        }
    }

    /// フラット座標配列への参照を取得
    #[must_use]
    pub fn coordinates(&self) -> &[T] {
        &self.coordinates
    }

    /// 重みストレージへの参照を取得
    #[must_use]
    pub fn weights(&self) -> &WeightStorage<T> {
        &self.weights
    }

    /// u方向ノットベクトルを取得
    #[must_use]
    pub fn u_knots(&self) -> &KnotVector<T> {
        &self.u_knots
    }

    /// v方向ノットベクトルを取得
    #[must_use]
    pub fn v_knots(&self) -> &KnotVector<T> {
        &self.v_knots
    }

    /// u方向次数を取得
    #[must_use]
    pub fn u_degree(&self) -> usize {
        self.u_degree
    }

    /// v方向次数を取得
    #[must_use]
    pub fn v_degree(&self) -> usize {
        self.v_degree
    }

    /// 制御点グリッドサイズを取得 (`u_count`, `v_count`)
    #[must_use]
    pub fn grid_size(&self) -> (usize, usize) {
        (self.u_count, self.v_count)
    }

    /// パラメータ定義域を取得 ((`u_min`, `u_max`), (`v_min`, `v_max`))
    #[must_use]
    pub fn parameter_domain(&self) -> ((T, T), (T, T)) {
        let u_domain = crate::knot::get_parameter_domain(&self.u_knots, self.u_degree);
        let v_domain = crate::knot::get_parameter_domain(&self.v_knots, self.v_degree);
        (u_domain, v_domain)
    }

    /// 指定パラメータでのサーフェス上の点を計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// サーフェス上の点
    pub fn evaluate_at(&self, u: T, v: T) -> Vector3<T> {
        // ノットスパンを見つける
        let u_span = crate::knot::find_knot_span(u, &self.u_knots, self.u_degree);
        let v_span = crate::knot::find_knot_span(v, &self.v_knots, self.v_degree);

        // 基底関数を計算
        let u_basis = crate::basis::basis_functions(u_span, self.u_degree, u, &self.u_knots);
        let v_basis = crate::basis::basis_functions(v_span, self.v_degree, v, &self.v_knots);

        // 重み付き制御点を使用してサーフェス点を計算
        let mut numerator_x = T::ZERO;
        let mut numerator_y = T::ZERO;
        let mut numerator_z = T::ZERO;
        let mut denominator = T::ZERO;

        #[allow(clippy::needless_range_loop)] // NURBS曲面評価の標準アルゴリズム
        for i in 0..=self.u_degree {
            #[allow(clippy::needless_range_loop)] // NURBS曲面評価の標準アルゴリズム
            for j in 0..=self.v_degree {
                let u_index = u_span - self.u_degree + i;
                let v_index = v_span - self.v_degree + j;

                if u_index < self.u_count && v_index < self.v_count {
                    let control_point = self.control_point(u_index, v_index);
                    let weight = self.weight(u_index, v_index);
                    let basis_product = u_basis[i] * v_basis[j] * weight;

                    numerator_x += control_point.x() * basis_product;
                    numerator_y += control_point.y() * basis_product;
                    numerator_z += control_point.z() * basis_product;
                    denominator += basis_product;
                }
            }
        }

        Vector3::new(
            numerator_x / denominator,
            numerator_y / denominator,
            numerator_z / denominator,
        )
    }

    /// u方向の偏導関数を計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// u方向接線ベクトル
    pub fn u_derivative_at(&self, u: T, v: T) -> Vector3<T> {
        let h = constants::solver::derivative_step::<T>();
        let p1 = self.evaluate_at(u - h, v);
        let p2 = self.evaluate_at(u + h, v);

        (p2 - p1) / (h + h)
    }

    /// v方向の偏導関数を計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// v方向接線ベクトル
    pub fn v_derivative_at(&self, u: T, v: T) -> Vector3<T> {
        let h = constants::solver::derivative_step::<T>();
        let p1 = self.evaluate_at(u, v - h);
        let p2 = self.evaluate_at(u, v + h);

        (p2 - p1) / (h + h)
    }

    /// 指定点での法線ベクトルを計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// 正規化された法線ベクトル
    pub fn normal_at(&self, u: T, v: T) -> Vector3<T> {
        let u_tangent = self.u_derivative_at(u, v);
        let v_tangent = self.v_derivative_at(u, v);

        // 外積で法線ベクトルを計算
        u_tangent
            .cross(&v_tangent)
            .normalize()
            .unwrap_or_else(|_| Vector3::zero())
    }
}

impl<T: Scalar> NurbsSurface3DConstructor<T> for NurbsSurface3D<T> {
    fn new(
        control_points: Vec<Vec<(T, T, T)>>,
        weights: Option<Vec<Vec<T>>>,
        u_knots: Vec<T>,
        v_knots: Vec<T>,
        u_degree: usize,
        v_degree: usize,
    ) -> std::result::Result<Self, String> {
        // タプルをVector3に変換
        let points: Vec<Vec<Vector3<T>>> = control_points
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|(x, y, z)| Vector3::new(x, y, z))
                    .collect()
            })
            .collect();

        Self::new_internal(points, weights, u_knots, v_knots, u_degree, v_degree)
            .map_err(|e| e.to_string())
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

    fn coordinates(&self) -> &[T] {
        &self.coordinates
    }

    fn weights(&self) -> Option<&[T]> {
        match &self.weights {
            WeightStorage::Uniform => None,
            WeightStorage::Individual(w) => Some(w),
        }
    }
}

impl<T: Scalar> NurbsSurface3DEvaluation<T> for NurbsSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T) {
        let point = self.evaluate_at(u, v);
        (point.x(), point.y(), point.z())
    }

    fn normal_at(&self, u: T, v: T) -> (T, T, T) {
        // 偏導関数を数値微分で近似
        let du = self.u_derivative_at(u, v);
        let dv = self.v_derivative_at(u, v);

        // 法線 = du × dv
        let nx = du.y() * dv.z() - du.z() * dv.y();
        let ny = du.z() * dv.x() - du.x() * dv.z();
        let nz = du.x() * dv.y() - du.y() * dv.x();

        // 正規化
        let len_sq = nx * nx + ny * ny + nz * nz;
        let len = len_sq.sqrt();

        if len.is_zero() {
            (T::ZERO, T::ZERO, T::ONE) // デフォルトのz方向
        } else {
            (nx / len, ny / len, nz / len)
        }
    }
}

impl<T: Scalar> NurbsSurface3DDerived<T> for NurbsSurface3D<T> {
    #[allow(clippy::similar_names)]
    fn surface_area(&self) -> T {
        // パラメータ領域を格子に分割し、各格子の面積（偏導関数の外積の大きさ）を数値積分する
        let subdivisions = constants::SURFACE_AREA_SUBDIVISIONS;
        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(subdivisions);
        let dv = (v_max - v_min) / T::from_usize(subdivisions);

        let mut total_area = T::ZERO;

        for i in 0..subdivisions {
            for j in 0..subdivisions {
                let u = u_min + du * T::from_usize(i) + du / T::from_f64(2.0);
                let v = v_min + dv * T::from_usize(j) + dv / T::from_f64(2.0);

                let d_du = self.u_derivative_at(u, v);
                let d_dv = self.v_derivative_at(u, v);

                // 外積の大きさ
                let cross_x = d_du.y() * d_dv.z() - d_du.z() * d_dv.y();
                let cross_y = d_du.z() * d_dv.x() - d_du.x() * d_dv.z();
                let cross_z = d_du.x() * d_dv.y() - d_du.y() * d_dv.x();
                let cross_norm = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();

                total_area += cross_norm * du * dv;
            }
        }

        total_area
    }
}
