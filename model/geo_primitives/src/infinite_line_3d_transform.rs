//! InfiniteLine3D Transform実装
//!
//! geo_foundation::extensions::BasicTransformトレイトの実装
//! Core/Extension分離パターンに従った変換機能の実装

use crate::{InfiniteLine3D, Point3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for InfiniteLine3D<T> {
    type Transformed = InfiniteLine3D<T>;
    type Vector2D = Vector3D<T>; // 3D空間での2D操作として扱う
    type Point2D = Point3D<T>; // 3D点を2D操作として扱う
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_point = self.point() + translation;
        Self::new(new_point, self.direction().as_vector()).unwrap()
    }

    /// Z軸周りの回転（簡易実装）
    fn rotate(&self, _center: Self::Point2D, _angle: Self::Angle) -> Self::Transformed {
        // 簡易実装：3D回転は複雑なので、現在は元の直線をコピーして返す
        self.clone()
    }

    /// 均一スケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let relative_point = Vector3D::from_points(&center, &self.point());
        let scaled_point = relative_point * factor;
        let new_point = center + scaled_point;

        // 方向ベクトルはスケールされない（正規化済み）
        Self::new(new_point, self.direction().as_vector()).unwrap()
    }
}
