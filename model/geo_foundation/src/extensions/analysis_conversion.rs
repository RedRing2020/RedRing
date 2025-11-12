//! analysisクレートとの型変換
//!
//! geo_foundationの抽象型とanalysisの具体型間の変換を提供

use crate::Scalar;

/// analysisクレートの Vector2 との相互変換
pub trait ToAnalysisVector2<T: Scalar> {
    /// analysis::Vector2 に変換
    fn to_analysis_vector2(&self) -> analysis::linalg::Vector2<T>;
}

/// analysisクレートの Vector2 から変換
pub trait FromAnalysisVector2<T: Scalar> {
    /// analysis::Vector2 から変換
    fn from_analysis_vector2(vector: &analysis::linalg::Vector2<T>) -> Self;
}

/// analysisクレートの Vector3 との相互変換
pub trait ToAnalysisVector3<T: Scalar> {
    /// analysis::Vector3 に変換
    fn to_analysis_vector3(&self) -> analysis::linalg::Vector3<T>;
}

/// analysisクレートの Vector3 から変換
pub trait FromAnalysisVector3<T: Scalar> {
    /// analysis::Vector3 から変換
    fn from_analysis_vector3(vector: &analysis::linalg::Vector3<T>) -> Self;
}
