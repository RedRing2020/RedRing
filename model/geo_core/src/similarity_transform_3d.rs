//! 3D 相似変換（回転・平行移動・正の一様スケール）
//!
//! 操作を組み合わせて 1 つの 4×4 行列に合成し、形状には 1 回で適用する。
//! 非一様スケール・ミラーリング・射影は表現できない。
//! 設計: TRANSFORM_TRAIT_DESIGN.md

use crate::{Aabb3D, Direction3D, Point3D, Vector3D};
use analysis::linalg::matrix::Matrix4x4;
use analysis::linalg::vector::Vector3;
use geo_contracts::{
    default_kernel_numerical_zero_tolerance, default_orthogonality_dot_error_tolerance, Angle,
    Scalar, SimilarityTransform3DCore, SimilarityTransformable3D, TransformError,
};

/// 3D 相似変換
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimilarityTransform3D<T: Scalar> {
    /// 合成済みの変換行列（線形部分 = スケール係数 × 回転）
    matrix: Matrix4x4<T>,
    /// 一様スケール係数（正）
    scale: T,
}

impl<T: Scalar> SimilarityTransform3D<T> {
    /// 恒等変換
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4x4::identity(),
            scale: T::ONE,
        }
    }

    /// 平行移動
    pub fn translation(offset: Vector3D<T>) -> Self {
        Self {
            matrix: Matrix4x4::translation(offset.x(), offset.y(), offset.z()),
            scale: T::ONE,
        }
    }

    /// 中心点・軸まわりの回転
    pub fn rotation_about_axis(center: Point3D<T>, axis: Direction3D<T>, angle: Angle<T>) -> Self {
        let axis = Vector3::new(axis.x(), axis.y(), axis.z());
        let rotation = Matrix4x4::rotation_axis(&axis, angle.to_radians());
        Self {
            matrix: about_center(center, rotation),
            scale: T::ONE,
        }
    }

    /// 中心点まわりの一様スケール
    ///
    /// `factor` が 0 以下の場合は `InvalidParameter`（負のスケールはミラーリングのため扱わない）。
    pub fn uniform_scale_about(center: Point3D<T>, factor: T) -> Result<Self, TransformError> {
        if factor <= default_kernel_numerical_zero_tolerance::<T>() {
            return Err(TransformError::InvalidParameter(
                "scale factor must be positive".to_string(),
            ));
        }
        Ok(Self {
            matrix: about_center(center, Matrix4x4::scale(factor, factor, factor)),
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
    pub fn from_matrix(matrix: Matrix4x4<T>) -> Result<Self, TransformError> {
        let zero_tol = default_kernel_numerical_zero_tolerance::<T>();
        let ortho_tol = default_orthogonality_dot_error_tolerance::<T>();

        let bottom_is_affine = matrix.get(3, 0).abs() <= zero_tol
            && matrix.get(3, 1).abs() <= zero_tol
            && matrix.get(3, 2).abs() <= zero_tol
            && (matrix.get(3, 3) - T::ONE).abs() <= zero_tol;
        if !bottom_is_affine {
            return Err(TransformError::Unsupported(
                "projective transform".to_string(),
            ));
        }

        let columns = linear_columns(&matrix);
        let lengths = columns.map(|c| c.length());
        if lengths.iter().any(|&len| len <= zero_tol) {
            return Err(TransformError::InvalidParameter(
                "singular matrix".to_string(),
            ));
        }

        let scale = lengths[0];
        let uniform = lengths
            .iter()
            .all(|&len| (len - scale).abs() <= ortho_tol * scale);
        let units = columns.map(|c| c * (T::ONE / c.length()));
        let orthogonal = units[0].dot(&units[1]).abs() <= ortho_tol
            && units[1].dot(&units[2]).abs() <= ortho_tol
            && units[0].dot(&units[2]).abs() <= ortho_tol;
        if !uniform || !orthogonal {
            return Err(TransformError::Unsupported("non-uniform scale".to_string()));
        }
        if units[0].cross(&units[1]).dot(&units[2]) < T::ZERO {
            return Err(TransformError::Unsupported("mirroring".to_string()));
        }

        Ok(Self { matrix, scale })
    }

    /// 変換行列
    pub fn to_matrix(&self) -> Matrix4x4<T> {
        self.matrix
    }

    /// 線形部分をスケール係数で割った行列の、直交行列からのずれ
    ///
    /// `RᵀR − I` の要素の絶対値の最大値を返す。回転を多数回合成すると増加する。
    pub fn orthogonality_error(&self) -> T {
        let units = linear_columns(&self.matrix).map(|c| c * (T::ONE / self.scale));
        let mut max_error = T::ZERO;
        for (i, ci) in units.iter().enumerate() {
            for (j, cj) in units.iter().enumerate() {
                let expected = if i == j { T::ONE } else { T::ZERO };
                max_error = max_error.max((ci.dot(cj) - expected).abs());
            }
        }
        max_error
    }

    /// 線形部分を直交化した変換を返す
    ///
    /// 線形部分をスケール係数で割り、Gram–Schmidt 法で直交化してスケールを掛け直す。
    /// 平行移動成分は変更しない。合成（`then`）は自動では再正規化しないため、
    /// 必要に応じて呼び出し側で適用する。
    pub fn renormalized(&self) -> Self {
        let [c0, c1, _] = linear_columns(&self.matrix);
        let u0 = c0.normalize();
        let u1 = (c1 - u0 * c1.dot(&u0)).normalize();
        let u2 = u0.cross(&u1);

        let mut matrix = self.matrix;
        for (col, unit) in [u0, u1, u2].iter().enumerate() {
            matrix.set(0, col, unit.x() * self.scale);
            matrix.set(1, col, unit.y() * self.scale);
            matrix.set(2, col, unit.z() * self.scale);
        }
        Self {
            matrix,
            scale: self.scale,
        }
    }
}

/// 中心点まわりに線形変換を適用する行列
fn about_center<T: Scalar>(center: Point3D<T>, linear: Matrix4x4<T>) -> Matrix4x4<T> {
    let to_origin = Matrix4x4::translation(-center.x(), -center.y(), -center.z());
    let back = Matrix4x4::translation(center.x(), center.y(), center.z());
    back * linear * to_origin
}

/// 行列の線形部分（左上 3×3）の列ベクトル
fn linear_columns<T: Scalar>(matrix: &Matrix4x4<T>) -> [Vector3D<T>; 3] {
    [0, 1, 2].map(|c| Vector3D::new(matrix.get(0, c), matrix.get(1, c), matrix.get(2, c)))
}

impl<T: Scalar> SimilarityTransform3DCore<T> for SimilarityTransform3D<T> {
    fn apply_point(&self, point: (T, T, T)) -> (T, T, T) {
        let p = self
            .matrix
            .transform_point_3d(&Vector3::new(point.0, point.1, point.2));
        (p.x(), p.y(), p.z())
    }

    fn apply_vector(&self, vector: (T, T, T)) -> (T, T, T) {
        let v = self
            .matrix
            .transform_vector_3d(&Vector3::new(vector.0, vector.1, vector.2));
        (v.x(), v.y(), v.z())
    }

    fn scale_factor(&self) -> T {
        self.scale
    }

    fn has_rotation(&self) -> bool {
        let tol = default_orthogonality_dot_error_tolerance::<T>();
        let units = linear_columns(&self.matrix).map(|c| c * (T::ONE / self.scale));
        let identity = [Vector3D::unit_x(), Vector3D::unit_y(), Vector3D::unit_z()];
        units
            .iter()
            .zip(identity.iter())
            .any(|(u, e)| (*u - *e).length() > tol)
    }
}

impl<T: Scalar> SimilarityTransformable3D<T> for Point3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(transform.apply_point((self.x(), self.y(), self.z())).into())
    }
}

impl<T: Scalar> SimilarityTransformable3D<T> for Vector3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        Ok(transform
            .apply_vector((self.x(), self.y(), self.z()))
            .into())
    }
}

impl<T: Scalar> SimilarityTransformable3D<T> for Direction3D<T> {
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        let (x, y, z) = transform.apply_vector((self.x(), self.y(), self.z()));
        Direction3D::new(x, y, z).ok_or_else(|| {
            TransformError::InvalidGeometry("direction degenerated to zero".to_string())
        })
    }
}

impl<T: Scalar> SimilarityTransformable3D<T> for Aabb3D<T> {
    /// 平行移動・一様スケールのみ受け付ける。回転を含む場合は `Unsupported`。
    ///
    /// 回転後の境界ボックスが必要な場合は、形状側で再構築する。
    fn transform_similarity<X: SimilarityTransform3DCore<T>>(
        &self,
        transform: &X,
    ) -> Result<Self, TransformError> {
        if transform.has_rotation() {
            return Err(TransformError::Unsupported(
                "rotation of an axis-aligned bounding box".to_string(),
            ));
        }
        let min = self.min().transform_similarity(transform)?;
        let max = self.max().transform_similarity(transform)?;
        Ok(Aabb3D::new(min, max))
    }
}
