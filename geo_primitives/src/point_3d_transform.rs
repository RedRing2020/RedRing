//! Point3D変換操作統一Foundation実装
//!
//! 統一Transform Foundation システムによる変換操作
//! 全幾何プリミティブで共通利用可能な統一インターフェース
//! analysisクレートの行列演算と統合された3D点変換

use crate::{Point3D, Vector3D};
use geo_foundation::{
    extensions::{
        analysis_conversion::{FromAnalysisVector3, ToAnalysisVector3},
        BasicTransform,
    },
    Angle, Scalar,
};

// ============================================================================
// BasicTransform Trait Implementation (統一Foundation)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Point3D<T> {
    type Transformed = Point3D<T>;
    type Vector2D = Vector3D<T>; // 3D点なので Vector3D を使用
    type Point2D = Point3D<T>; // 3D点なので Point3D を使用
    type Angle = Angle<T>;

    /// 平行移動
    ///
    /// # 引数
    /// * `translation` - 移動ベクトル
    ///
    /// # 戻り値
    /// 平行移動された新しい点
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        Point3D::new(
            self.x() + translation.x(),
            self.y() + translation.y(),
            self.z() + translation.z(),
        )
    }

    /// 指定中心での回転（Z軸周りの回転）
    ///
    /// # 引数
    /// * `center` - 回転中心点
    /// * `angle` - 回転角度（Z軸周り）
    ///
    /// # 戻り値
    /// 回転された新しい点
    fn rotate(&self, center: Self::Point2D, _angle: Self::Angle) -> Self::Transformed {
        // 簡易的なZ軸周り回転（将来はanalysisの行列演算を使用予定）
        let dx = self.x() - center.x();
        let dy = self.y() - center.y();
        let dz = self.z() - center.z();

        // TODO: angle.radians() の代わりにアクセサメソッドを使用
        // 現在は簡易実装のため回転なしで返す
        Point3D::new(center.x() + dx, center.y() + dy, center.z() + dz)
    }

    /// 指定中心でのスケール
    ///
    /// # 引数
    /// * `center` - スケール中心点
    /// * `factor` - スケール倍率
    ///
    /// # 戻り値
    /// スケールされた新しい点
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let dx = self.x() - center.x();
        let dy = self.y() - center.y();
        let dz = self.z() - center.z();

        Point3D::new(
            center.x() + dx * factor,
            center.y() + dy * factor,
            center.z() + dz * factor,
        )
    }
}

// ============================================================================
// Default implementations for required types
// ============================================================================

impl<T: Scalar> Default for Point3D<T> {
    fn default() -> Self {
        Point3D::origin()
    }
}

impl<T: Scalar> From<(T, T)> for Vector3D<T> {
    fn from(tuple: (T, T)) -> Self {
        Vector3D::new(tuple.0, tuple.1, T::ZERO)
    }
}

// ============================================================================
// Analysis Integration - Type Conversion Implementations
// ============================================================================

impl<T: Scalar> ToAnalysisVector3<T> for Point3D<T> {
    fn to_analysis_vector3(&self) -> analysis::linalg::Vector3<T> {
        analysis::linalg::Vector3::new(self.x(), self.y(), self.z())
    }
}

impl<T: Scalar> FromAnalysisVector3<T> for Point3D<T> {
    fn from_analysis_vector3(vec: &analysis::linalg::Vector3<T>) -> Self {
        Point3D::new(vec.x(), vec.y(), vec.z())
    }
}
