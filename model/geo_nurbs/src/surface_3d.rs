//! NURBSサーフェス実装
//!
//! Non-Uniform Rational B-Spline surfaces の基本実装です。
//! 制御点の2次元グリッド、重み、2方向のノットベクトルを使用して自由形状曲面を表現します。

use crate::{KnotVector, NurbsError, Result, Scalar};
use analysis::linalg::vector::Vector3;

/// 重み配列の効率的管理
#[derive(Debug, Clone)]
pub enum WeightStorage<T: Scalar> {
    /// 非有理サーフェス（全重み = 1.0）
    Uniform,
    /// 有理サーフェス（個別重み）- フラット配列
    Individual(Vec<T>),
}

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
    pub(crate) fn new_internal(
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
        let u_basis = self.compute_u_basis_functions(u, u_span);
        let v_basis = self.compute_v_basis_functions(v, v_span);

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
        let h = T::from_f64(1e-8);
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
        let h = T::from_f64(1e-8);
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

    /// サーフェスの面積を近似計算
    ///
    /// # 引数
    /// * `u_subdivisions` - u方向の分割数
    /// * `v_subdivisions` - v方向の分割数
    ///
    /// # 戻り値
    /// 近似面積
    #[must_use]
    pub fn approximate_area(&self, u_subdivisions: usize, v_subdivisions: usize) -> T {
        if u_subdivisions == 0 || v_subdivisions == 0 {
            return T::ZERO;
        }

        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(u_subdivisions);
        let dv = (v_max - v_min) / T::from_usize(v_subdivisions);

        let mut total_area = T::ZERO;

        for i in 0..u_subdivisions {
            for j in 0..v_subdivisions {
                let u1 = u_min + du * T::from_usize(i);
                let u2 = u_min + du * T::from_usize(i + 1);
                let v1 = v_min + dv * T::from_usize(j);
                let v2 = v_min + dv * T::from_usize(j + 1);

                // 4角形パッチの面積を近似計算
                let p00 = self.evaluate_at(u1, v1);
                let p10 = self.evaluate_at(u2, v1);
                let p01 = self.evaluate_at(u1, v2);
                let p11 = self.evaluate_at(u2, v2);

                // 三角形2つに分割して面積計算（直接実装）
                let area1 = {
                    let v1 = p10 - p00;
                    let v2 = p01 - p00;
                    v1.cross(&v2).norm() / (T::ONE + T::ONE) // 外積の大きさの半分
                };
                let area2 = {
                    let v1 = p11 - p10;
                    let v2 = p01 - p10;
                    v1.cross(&v2).norm() / (T::ONE + T::ONE)
                };

                total_area += area1 + area2;
            }
        }

        total_area
    }

    /// u方向のB-スプライン基底関数を計算
    fn compute_u_basis_functions(&self, u: T, span: usize) -> Vec<T> {
        let mut basis = vec![T::ZERO; self.u_degree + 1];
        let mut left = vec![T::ZERO; self.u_degree + 1];
        let mut right = vec![T::ZERO; self.u_degree + 1];

        basis[0] = T::ONE;

        for j in 1..=self.u_degree {
            left[j] = u - self.u_knots[span + 1 - j];
            right[j] = self.u_knots[span + j] - u;

            let mut saved = T::ZERO;
            for r in 0..j {
                let temp = basis[r] / (right[r + 1] + left[j - r]);
                basis[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            basis[j] = saved;
        }

        basis
    }

    /// v方向のB-スプライン基底関数を計算
    fn compute_v_basis_functions(&self, v: T, span: usize) -> Vec<T> {
        let mut basis = vec![T::ZERO; self.v_degree + 1];
        let mut left = vec![T::ZERO; self.v_degree + 1];
        let mut right = vec![T::ZERO; self.v_degree + 1];

        basis[0] = T::ONE;

        for j in 1..=self.v_degree {
            left[j] = v - self.v_knots[span + 1 - j];
            right[j] = self.v_knots[span + j] - v;

            let mut saved = T::ZERO;
            for r in 0..j {
                let temp = basis[r] / (right[r + 1] + left[j - r]);
                basis[r] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            basis[j] = saved;
        }

        basis
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nurbs_surface_creation() {
        use geo_foundation::NurbsSurface3DConstructor;
        // 2x2 制御点グリッド
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 1.0)],
        ];

        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            Some(weights), // 有理サーフェスとして
            u_knots,
            v_knots,
            1, // u_degree
            1, // v_degree
        );

        assert!(surface.is_ok());

        let surface = surface.unwrap();
        assert_eq!(surface.grid_size(), (2, 2));
        assert_eq!(surface.u_degree(), 1);
        assert_eq!(surface.v_degree(), 1);
    }

    #[test]
    fn test_surface_evaluation() {
        use geo_foundation::NurbsSurface3DConstructor;
        // 平面サーフェスのテスト
        let control_points = vec![
            vec![(0.0, 0.0, 0.0), (0.0, 1.0, 0.0)],
            vec![(1.0, 0.0, 0.0), (1.0, 1.0, 0.0)],
        ];

        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = <NurbsSurface3D<f64> as NurbsSurface3DConstructor<f64>>::new(
            control_points,
            Some(weights), // 有理サーフェスとして
            u_knots,
            v_knots,
            1,
            1,
        )
        .unwrap();

        // 中央点の評価
        let point = surface.evaluate_at(0.5, 0.5);
        assert!((point.x() - 0.5).abs() < 1e-10);
        assert!((point.y() - 0.5).abs() < 1e-10);
        assert!((point.z() - 0.0).abs() < 1e-10);
    }
}

// ============================================================================
// Core Traits 実装
// ============================================================================

use geo_foundation::{
    Bounded, ExtensionFoundation, NurbsSurface3DConstructor, NurbsSurface3DCore,
    NurbsSurface3DMeasure, NurbsSurface3DProperties, PrimitiveKind,
};

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

    fn from_control_points(
        control_points: Vec<Vec<(T, T, T)>>,
        u_degree: usize,
        v_degree: usize,
    ) -> std::result::Result<Self, String> {
        if control_points.is_empty() || control_points[0].is_empty() {
            return Err("Control points grid is empty".to_string());
        }

        let u_count = control_points.len();
        let v_count = control_points[0].len();

        if u_count < u_degree + 1 || v_count < v_degree + 1 {
            return Err(format!(
                "Insufficient control points: ({}, {}) < ({}, {})",
                u_count,
                v_count,
                u_degree + 1,
                v_degree + 1
            ));
        }

        // クランプド・ノットベクトルを生成
        let u_knots = crate::knot::clamped_knot_vector(u_degree, u_count);
        let v_knots = crate::knot::clamped_knot_vector(v_degree, v_count);

        <Self as NurbsSurface3DConstructor<T>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            u_degree,
            v_degree,
        )
    }

    fn unit_plane() -> Self {
        // XY平面上の1×1正方形
        let control_points = vec![
            vec![(T::ZERO, T::ZERO, T::ZERO), (T::ZERO, T::ONE, T::ZERO)],
            vec![(T::ONE, T::ZERO, T::ZERO), (T::ONE, T::ONE, T::ZERO)],
        ];
        let u_degree = 1;
        let v_degree = 1;
        let u_knots = vec![T::ZERO, T::ZERO, T::ONE, T::ONE];
        let v_knots = vec![T::ZERO, T::ZERO, T::ONE, T::ONE];

        <Self as NurbsSurface3DConstructor<T>>::new(
            control_points,
            None,
            u_knots,
            v_knots,
            u_degree,
            v_degree,
        )
        .expect("Unit plane creation should never fail")
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

    fn is_rational(&self) -> bool {
        matches!(self.weights, WeightStorage::Individual(_))
    }
}

impl<T: Scalar> NurbsSurface3DMeasure<T> for NurbsSurface3D<T> {
    fn point_at_uv(&self, u: T, v: T) -> (T, T, T) {
        let point = self.evaluate_at(u, v);
        (point.x(), point.y(), point.z())
    }

    fn normal_at(&self, u: T, v: T) -> (T, T, T) {
        let _h = T::from_f64(1e-8);

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

    fn surface_area(&self) -> T {
        // 簡易実装: 中央差分で近似
        let subdivisions = 20;
        let ((u_min, u_max), (v_min, v_max)) = self.parameter_domain();
        let du = (u_max - u_min) / T::from_usize(subdivisions);
        let dv = (v_max - v_min) / T::from_usize(subdivisions);

        let mut total_area = T::ZERO;

        for i in 0..subdivisions {
            for j in 0..subdivisions {
                let u = u_min + du * T::from_usize(i) + du / T::from_f64(2.0);
                let v = v_min + dv * T::from_usize(j) + dv / T::from_f64(2.0);

                #[allow(clippy::similar_names)]
                let d_du = self.u_derivative_at(u, v);
                #[allow(clippy::similar_names)]
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

    fn tangent_vectors_at(&self, u: T, v: T) -> ((T, T, T), (T, T, T)) {
        let du = self.u_derivative_at(u, v);
        let dv = self.v_derivative_at(u, v);

        ((du.x(), du.y(), du.z()), (dv.x(), dv.y(), dv.z()))
    }
}

impl<T: Scalar> NurbsSurface3DCore<T> for NurbsSurface3D<T> {}

// ============================================================================
// Extension Foundation 実装
// ============================================================================

impl<T: Scalar> ExtensionFoundation<T> for NurbsSurface3D<T> {
    fn primitive_kind(&self) -> PrimitiveKind {
        PrimitiveKind::NurbsSurface3D
    }

    fn measure(&self) -> Option<T> {
        Some(self.surface_area())
    }
}

impl<T: Scalar> Bounded<T> for NurbsSurface3D<T> {
    type Aabb = geo_core::Aabb3D<T>;

    fn aabb(&self) -> Option<Self::Aabb> {
        // 制御点ベースの境界ボックスを計算
        let mut min_x = T::from_f64(f64::INFINITY);
        let mut min_y = T::from_f64(f64::INFINITY);
        let mut min_z = T::from_f64(f64::INFINITY);
        let mut max_x = T::from_f64(f64::NEG_INFINITY);
        let mut max_y = T::from_f64(f64::NEG_INFINITY);
        let mut max_z = T::from_f64(f64::NEG_INFINITY);

        for u in 0..self.u_count {
            for v in 0..self.v_count {
                let point = self.control_point(u, v);
                min_x = min_x.min(point.x());
                min_y = min_y.min(point.y());
                min_z = min_z.min(point.z());
                max_x = max_x.max(point.x());
                max_y = max_y.max(point.y());
                max_z = max_z.max(point.z());
            }
        }

        Some(geo_core::Aabb3D::new(
            geo_core::Point3D::new(min_x, min_y, min_z),
            geo_core::Point3D::new(max_x, max_y, max_z),
        ))
    }
}
