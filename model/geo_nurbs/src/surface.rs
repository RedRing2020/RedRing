//! NURBSサーフェス実装
//!
//! Non-Uniform Rational B-Spline surfaces の基本実装です。
//! 制御点の2次元グリッド、重み、2方向のノットベクトルを使用して自由形状曲面を表現します。

use crate::{KnotVector, NurbsError, Result};
use analysis::Scalar;
use geo_primitives::{Point3D, Triangle3D, Vector3D};

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
    /// 新しいNURBSサーフェスを作成
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
    pub fn new(
        control_points: Vec<Vec<Point3D<T>>>,
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
        let mut coordinates = Vec::with_capacity(u_count * v_count * 3);
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
    pub fn control_point(&self, u: usize, v: usize) -> Point3D<T> {
        let idx = self.control_point_index(u, v);
        Point3D::new(
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
    pub fn evaluate_at(&self, u: T, v: T) -> Point3D<T> {
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

        Point3D::new(
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
    pub fn u_derivative_at(&self, u: T, v: T) -> Vector3D<T> {
        let h = T::from_f64(1e-8);
        let p1 = self.evaluate_at(u - h, v);
        let p2 = self.evaluate_at(u + h, v);

        let dx = (p2.x() - p1.x()) / (h + h);
        let dy = (p2.y() - p1.y()) / (h + h);
        let dz = (p2.z() - p1.z()) / (h + h);

        Vector3D::new(dx, dy, dz)
    }

    /// v方向の偏導関数を計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// v方向接線ベクトル
    pub fn v_derivative_at(&self, u: T, v: T) -> Vector3D<T> {
        let h = T::from_f64(1e-8);
        let p1 = self.evaluate_at(u, v - h);
        let p2 = self.evaluate_at(u, v + h);

        let dx = (p2.x() - p1.x()) / (h + h);
        let dy = (p2.y() - p1.y()) / (h + h);
        let dz = (p2.z() - p1.z()) / (h + h);

        Vector3D::new(dx, dy, dz)
    }

    /// 指定点での法線ベクトルを計算
    ///
    /// # 引数
    /// * `u` - u方向パラメータ値
    /// * `v` - v方向パラメータ値
    ///
    /// # 戻り値
    /// 正規化された法線ベクトル
    pub fn normal_at(&self, u: T, v: T) -> Vector3D<T> {
        let u_tangent = self.u_derivative_at(u, v);
        let v_tangent = self.v_derivative_at(u, v);

        // 外積で法線ベクトルを計算
        u_tangent.cross(&v_tangent).normalize()
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

                // 三角形2つに分割して面積計算
                let area1 = if let Some(triangle) = Triangle3D::new(p00, p10, p01) {
                    triangle.area()
                } else {
                    T::ZERO
                };
                let area2 = if let Some(triangle) = Triangle3D::new(p10, p11, p01) {
                    triangle.area()
                } else {
                    T::ZERO
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
        // 2x2 制御点グリッド
        let control_points = vec![
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
            vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 1.0)],
        ];

        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = NurbsSurface3D::new(
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
        // 平面サーフェスのテスト
        let control_points = vec![
            vec![Point3D::new(0.0, 0.0, 0.0), Point3D::new(0.0, 1.0, 0.0)],
            vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 1.0, 0.0)],
        ];

        let weights = vec![vec![1.0, 1.0], vec![1.0, 1.0]];

        let u_knots = vec![0.0, 0.0, 1.0, 1.0];
        let v_knots = vec![0.0, 0.0, 1.0, 1.0];

        let surface = NurbsSurface3D::new(
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
