//! 2D 相似変換（回転・平行移動・正の一様スケール）
//!
//! 操作を組み合わせて 1 つの 3×3 行列に合成し、形状には 1 回で適用する。
//! 非一様スケール・ミラーリング・射影は表現できない。

use crate::{Aabb2D, Direction2D, Point2D, Vector2D};
use analysis::linalg::matrix::Matrix3x3;
use analysis::linalg::vector::Vector2;
use geo_contracts::{
    default_kernel_numerical_zero_tolerance, default_orthogonality_dot_error_tolerance, Angle,
    Scalar, SimilarityTransform2DCore, SimilarityTransformable2D, TransformError,
};

/// 2D 相似変換
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarityTransform2D<T: Scalar> {
    /// 合成済みの変換行列（線形部分 = スケール係数 × 回転）
    matrix: Matrix3x3<T>,
    /// 一様スケール係数（正）
    scale: T,
}

impl<T: Scalar> SimilarityTransform2D<T> {
    /// 恒等変換
    pub fn identity() -> Self {
        Self {
            matrix: Matrix3x3::identity(),
            scale: T::ONE,
        }
    }

    /// 平行移動
    pub fn translation(offset: Vector2D<T>) -> Self {
        Self {
            matrix: Matrix3x3::translation(offset.x(), offset.y()),
            scale: T::ONE,
        }
    }

    /// 中心点まわりの回転
    pub fn rotation_about(center: Point2D<T>, angle: Angle<T>) -> Self {
        Self {
            matrix: about_center(center, Matrix3x3::rotation_2d(angle.to_radians())),
            scale: T::ONE,
        }
    }

    /// 中心点まわりの一様スケール
    ///
    /// `factor` が 0 以下の場合は `InvalidParameter`（負のスケールはミラーリングのため扱わない）。
    pub fn uniform_scale_about(center: Point2D<T>, factor: T) -> Result<Self, TransformError> {
        if factor <= default_kernel_numerical_zero_tolerance::<T>() {
            return Err(TransformError::InvalidParameter(
                "scale factor must be positive".to_string(),
            ));
        }
        Ok(Self {
            matrix: about_center(center, Matrix3x3::uniform_scale_2d(factor)),
            scale: factor,
        })
    }

    /// `self` の後に `next` を適用する変換
    pub fn then(&self, next: &Self) -> Self {
        Self {
            matrix: next.matrix * self.matrix,
            scale: self.scale * next.scale,
        }
    }

    /// 行列から構築する
    ///
    /// 射影・非一様スケール・ミラーリングを含む場合は `Unsupported`、特異な場合は `InvalidParameter`。
    pub fn from_matrix(matrix: Matrix3x3<T>) -> Result<Self, TransformError> {
        let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
        let ortho_tol = default_orthogonality_dot_error_tolerance::<T>();

        let bottom_is_affine = matrix.get(2, 0).abs() <= zero_tol
            && matrix.get(2, 1).abs() <= zero_tol
            && (matrix.get(2, 2) - T::ONE).abs() <= zero_tol;
        if !bottom_is_affine {
            return Err(TransformError::Unsupported(
                "projective transform".to_string(),
            ));
        }

        let [c0, c1] = linear_columns(&matrix);
        let (len0, len1) = (c0.length(), c1.length());
        if len0 <= zero_tol || len1 <= zero_tol {
            return Err(TransformError::InvalidParameter(
                "singular matrix".to_string(),
            ));
        }

        let uniform = (len1 - len0).abs() <= ortho_tol * len0;
        let (u0, u1) = (c0 * (T::ONE / len0), c1 * (T::ONE / len1));
        if !uniform || u0.dot(&u1).abs() > ortho_tol {
            return Err(TransformError::Unsupported("non-uniform scale".to_string()));
        }
        if u0.cross(&u1) < T::ZERO {
            return Err(TransformError::Unsupported("mirroring".to_string()));
        }

        Ok(Self {
            matrix,
            scale: len0,
        })
    }

    /// 変換行列
    pub fn to_matrix(&self) -> Matrix3x3<T> {
        self.matrix
    }

    /// 線形部分をスケール係数で割った行列の、直交行列からのずれ
    ///
    /// `RᵀR − I` の要素の絶対値の最大値を返す。回転を多数回合成すると増加する。
    pub fn orthogonality_error(&self) -> T {
        let [u0, u1] = linear_columns(&self.matrix).map(|c| c * (T::ONE / self.scale));
        let errors = [
            (u0.dot(&u0) - T::ONE).abs(),
            (u1.dot(&u1) - T::ONE).abs(),
            u0.dot(&u1).abs(),
        ];
        errors.into_iter().fold(T::ZERO, |acc, e| acc.max(e))
    }

    /// 線形部分を直交化した変換を返す
    ///
    /// 線形部分をスケール係数で割り、1 列目を正規化して 2 列目をその直交方向とし、
    /// スケールを掛け直す。平行移動成分は変更しない。合成（`then`）は自動では
    /// 再正規化しないため、必要に応じて呼び出し側で適用する。
    pub fn renormalized(&self) -> Self {
        let [c0, _] = linear_columns(&self.matrix);
        let u0 = c0.normalize();
        let u1 = Vector2D::new(-u0.y(), u0.x());

        let mut matrix = self.matrix;
        for (col, unit) in [u0, u1].iter().enumerate() {
            matrix.set(0, col, unit.x() * self.scale);
            matrix.set(1, col, unit.y() * self.scale);
        }
        Self {
            matrix,
            scale: self.scale,
        }
    }
}

/// 中心点まわりに線形変換を適用する行列
fn about_center<T: Scalar>(center: Point2D<T>, linear: Matrix3x3<T>) -> Matrix3x3<T> {
    let to_origin = Matrix3x3::translation(-center.x(), -center.y());
    let back = Matrix3x3::translation(center.x(), center.y());
    back * linear * to_origin
}

/// 行列の線形部分（左上 2×2）の列ベクトル
fn linear_columns<T: Scalar>(matrix: &Matrix3x3<T>) -> [Vector2D<T>; 2] {
    [0, 1].map(|c| Vector2D::new(matrix.get(0, c), matrix.get(1, c)))
}

impl<T: Scalar> SimilarityTransform2DCore<T> for SimilarityTransform2D<T> {
    fn apply_point(&self, point: (T, T)) -> (T, T) {
        let p = self
            .matrix
            .transform_point_2d(&Vector2::new(point.0, point.1));
        (p.x(), p.y())
    }

    fn apply_vector(&self, vector: (T, T)) -> (T, T) {
        let v = self
            .matrix
            .transform_vector_2d(&Vector2::new(vector.0, vector.1));
        (v.x(), v.y())
    }

    fn scale_factor(&self) -> T {
        self.scale
    }

    fn has_rotation(&self) -> bool {
        let tol = default_orthogonality_dot_error_tolerance::<T>();
        let [u0, u1] = linear_columns(&self.matrix).map(|c| c * (T::ONE / self.scale));
        (u0 - Vector2D::new(T::ONE, T::ZERO)).length() > tol
            || (u1 - Vector2D::new(T::ZERO, T::ONE)).length() > tol
    }
}

impl<T: Scalar> SimilarityTransformable2D<T> for Point2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let (x, y) = transform.apply_point((self.x(), self.y()));
        Ok(Point2D::new(x, y))
    }
}

impl<T: Scalar> SimilarityTransformable2D<T> for Vector2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(transform.apply_vector((self.x(), self.y())).into())
    }
}

impl<T: Scalar> SimilarityTransformable2D<T> for Direction2D<T> {
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let (x, y) = transform.apply_vector((self.x(), self.y()));
        Direction2D::new(x, y).ok_or_else(|| {
            TransformError::InvalidGeometry("direction degenerated to zero".to_string())
        })
    }
}

impl<T: Scalar> SimilarityTransformable2D<T> for Aabb2D<T> {
    /// 平行移動・一様スケールのみ受け付ける。回転を含む場合は `Unsupported`。
    ///
    /// 回転後の境界ボックスが必要な場合は、形状側で再構築する。
    fn transform_similarity<X: SimilarityTransform2DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        if transform.has_rotation() {
            return Err(TransformError::Unsupported(
                "rotation of an axis-aligned bounding box".to_string(),
            ));
        }
        let min = self.min_point().transform_similarity(transform)?;
        let max = self.max_point().transform_similarity(transform)?;
        Ok(Aabb2D::new(min, max))
    }
}
