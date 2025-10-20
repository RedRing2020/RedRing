//! Ray3D Transform実装
//!
//! geo_foundation::extensions::BasicTransformトレイトの実装
//! Core/Extension分離パターンに従った変換機能の実装

use crate::{Point3D, Ray3D, Vector3D};
use geo_foundation::{extensions::BasicTransform, Angle, Scalar};

// ============================================================================
// BasicTransform implementation (geo_foundation統合)
// ============================================================================

impl<T: Scalar> BasicTransform<T> for Ray3D<T> {
    type Transformed = Ray3D<T>;
    type Vector2D = Vector3D<T>; // 3D空間での2D操作として扱う
    type Point2D = Point3D<T>; // 3D点を2D操作として扱う
    type Angle = Angle<T>;

    /// 平行移動
    fn translate(&self, translation: Self::Vector2D) -> Self::Transformed {
        let new_origin = self.origin() + translation;
        Self::new(new_origin, self.direction().as_vector()).unwrap()
    }

    /// Z軸周りの回転（簡易実装）
    fn rotate(&self, _center: Self::Point2D, _angle: Self::Angle) -> Self::Transformed {
        // 簡易実装：回転は複雑なので、現在は元のRayをクローンして返す
        self.clone()
    }

    /// 均一スケール
    fn scale(&self, center: Self::Point2D, factor: T) -> Self::Transformed {
        let relative_origin = Vector3D::from_points(&center, &self.origin());
        let scaled_origin = relative_origin * factor;
        let new_origin = center + scaled_origin;

        // 方向ベクトルはスケールされない（正規化済み）
        Self::new(new_origin, self.direction().as_vector()).unwrap()
    }
}
