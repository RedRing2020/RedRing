//! Analysis Transform Traits - 効率的なMatrix/Vector変換抽象化
//!
//! Analysis Matrix/Vectorクレートを基盤とした高効率変換トレイト群
//! geo_nurbsのmatrix_transformパターンを基盤とする統一実装

use crate::Scalar;

/// 3D Analysis Matrix変換トレイト（座標点用）
///
/// Matrix4x4を使用した効率的な座標変換を提供
/// 平行移動、回転、スケールの統合変換をサポート
pub trait AnalysisTransform3D<T: Scalar> {
    /// Matrix4x4型（analysis::linalg::matrix::Matrix4x4）
    type Matrix4x4;
    /// Vector3D型（実装依存）
    type Vector3D;
    /// Angle型（geo_foundation::Angle）
    type Angle;
    /// 変換結果型（通常はSelf）
    type Output;

    /// Matrix4x4による直接座標変換（平行移動、回転、スケール全て適用）
    fn transform_point_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;

    /// 平行移動変換（Analysis Vector使用）
    fn translate_analysis(
        &self,
        translation: &Self::Vector3D,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 軸回転変換（Analysis Matrix4x4使用）
    fn rotate_analysis(
        &self,
        center: &Self,
        axis: &Self::Vector3D,
        angle: Self::Angle,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// スケール変換（Analysis Matrix4x4使用）
    fn scale_analysis(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 均等スケール変換（Analysis Matrix4x4使用）
    fn uniform_scale_analysis(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 複合変換（平行移動+回転+スケール）
    fn apply_composite_transform(
        &self,
        translation: Option<&Self::Vector3D>,
        rotation: Option<(&Self, &Self::Vector3D, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 複合変換（均等スケール版）
    fn apply_composite_transform_uniform(
        &self,
        translation: Option<&Self::Vector3D>,
        rotation: Option<(&Self, &Self::Vector3D, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;
}

/// 3D Analysis Vector変換トレイト（方向ベクトル用）
///
/// Matrix4x4を使用した方向ベクトル専用変換
/// 平行移動成分を自動的に無視する特性
pub trait AnalysisTransformVector3D<T: Scalar> {
    /// Matrix4x4型（analysis::linalg::matrix::Matrix4x4）
    type Matrix4x4;
    /// Vector3D型（実装依存）  
    type Vector3D;
    /// Angle型（geo_foundation::Angle）
    type Angle;
    /// 変換結果型（通常はSelf）
    type Output;

    /// Matrix4x4による方向ベクトル変換（平行移動成分は無視される）
    fn transform_vector_matrix(&self, matrix: &Self::Matrix4x4) -> Self::Output;

    /// 軸回転変換（方向ベクトル用、中心点不要）
    fn rotate_vector_analysis(
        &self,
        axis: &Self::Vector3D,
        angle: Self::Angle,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// スケール変換（方向ベクトル用）
    fn scale_vector_analysis(
        &self,
        scale_x: T,
        scale_y: T,
        scale_z: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 均等スケール変換（方向ベクトル用）
    fn uniform_scale_vector_analysis(
        &self,
        scale_factor: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 複合変換（回転+スケール、方向ベクトル用）
    fn apply_vector_composite_transform(
        &self,
        rotation: Option<(&Self::Vector3D, Self::Angle)>,
        scale: Option<(T, T, T)>,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 複合変換（回転+均等スケール、方向ベクトル用）
    fn apply_vector_composite_transform_uniform(
        &self,
        rotation: Option<(&Self::Vector3D, Self::Angle)>,
        scale: Option<T>,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// Analysis Vector正規化
    fn normalize_analysis(&self) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;
}

/// 2D Analysis Matrix変換トレイト（座標点用）
///
/// Matrix3x3を使用した効率的な2D座標変換を提供
pub trait AnalysisTransform2D<T: Scalar> {
    /// Matrix3x3型（analysis::linalg::matrix::Matrix3x3）
    type Matrix3x3;
    /// Vector2D型（実装依存）
    type Vector2D;
    /// Angle型（geo_foundation::Angle）
    type Angle;
    /// 変換結果型（通常はSelf）
    type Output;

    /// Matrix3x3による直接座標変換（2D用）
    fn transform_point_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output;

    /// 平行移動変換（2D用）
    fn translate_analysis_2d(
        &self,
        translation: &Self::Vector2D,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 回転変換（2D用、中心点指定）
    fn rotate_analysis_2d(
        &self,
        center: &Self,
        angle: Self::Angle,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// スケール変換（2D用）
    fn scale_analysis_2d(
        &self,
        center: &Self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 均等スケール変換（2D用）
    fn uniform_scale_analysis_2d(
        &self,
        center: &Self,
        scale_factor: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;
}

/// 2D Analysis Vector変換トレイト（方向ベクトル用）
///
/// Matrix3x3を使用した2D方向ベクトル専用変換
pub trait AnalysisTransformVector2D<T: Scalar> {
    /// Matrix3x3型（analysis::linalg::matrix::Matrix3x3）
    type Matrix3x3;
    /// Vector2D型（実装依存）
    type Vector2D;
    /// Angle型（geo_foundation::Angle）
    type Angle;
    /// 変換結果型（通常はSelf）
    type Output;

    /// Matrix3x3による方向ベクトル変換（2D用、平行移動成分は無視される）
    fn transform_vector_matrix_2d(&self, matrix: &Self::Matrix3x3) -> Self::Output;

    /// 回転変換（2D方向ベクトル用、中心点不要）
    fn rotate_vector_analysis_2d(
        &self,
        angle: Self::Angle,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// スケール変換（2D方向ベクトル用）
    fn scale_vector_analysis_2d(
        &self,
        scale_x: T,
        scale_y: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 均等スケール変換（2D方向ベクトル用）
    fn uniform_scale_vector_analysis_2d(
        &self,
        scale_factor: T,
    ) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;

    /// 2D Analysis Vector正規化
    fn normalize_analysis_2d(&self) -> Result<Self::Output, crate::TransformError>
    where
        Self: Sized;
}

/// Analysis変換実装のマーカートレイト
///
/// geo_nurbsレベルの効率性を示すマーカー
/// パフォーマンスクリティカルな用途での適用可能性を示す
pub trait AnalysisTransformSupport {
    /// Analysis Matrix/Vector統合済みかどうか
    const HAS_ANALYSIS_INTEGRATION: bool = true;

    /// geo_nurbsレベルのパフォーマンスを実現するかどうか
    const PERFORMANCE_OPTIMIZED: bool = true;
}
