//! 幾何プリミティブクレート
//!
//! geo_core数学基礎処理を利用して、点・線分・円・平面・
//! 多角形・三角形メッシュ等のプリミティブ形状を定義して分類分けを行う

// コアモジュール
pub mod traits;
pub mod geometry2d;
pub mod geometry3d;



// テストモジュール
#[cfg(test)]
mod unit_tests;

// 分類システムとプリミティブトレイト
pub use traits::common::{
    PrimitiveKind, DimensionClass, GeometryPrimitive,
    GeometricPrimitive, TransformablePrimitive, MeasurablePrimitive, PrimitiveCollection
};

// バウンディングボックス
pub use geometry2d::BBox2D;
pub use geometry3d::BBox3D;
pub use traits::{BoundingBox, BoundingBoxOps, CollisionBounds};

// 基本幾何プリミティブ
// 基本幾何プリミティブ
pub use geometry2d::{Point2D, Vector2D, Direction2D};
pub use geometry3d::{Point3D, Vector3D, Direction3D};

// 名前空間の整理
pub mod primitives_2d {
    pub use crate::{Point2D, Vector2D, Direction2D, BBox2D};
}

pub mod primitives_3d {
    pub use crate::{Point3D, Vector3D, Direction3D, BBox3D};
}

/// 便利な再エクスポート
pub mod prelude {
    // 基本幾何プリミティブ
    pub use crate::{Point2D, Vector2D, Direction2D, Point3D, Vector3D, Direction3D};

    // バウンディングボックス
    pub use crate::{BBox2D, BBox3D, BoundingBox, BoundingBoxOps, CollisionBounds};

    // プリミティブトレイトと分類
    pub use crate::{
        GeometricPrimitive, TransformablePrimitive, MeasurablePrimitive, PrimitiveCollection,
        PrimitiveKind, DimensionClass
    };

    // Direction トレイト (geo_foundation から)
    pub use geo_foundation::abstract_types::geometry::{Direction, Direction2D as Direction2DTrait, Direction3D as Direction3DTrait};

    // ベクトルトレイト
    pub use crate::traits::{Vector, Vector2DExt, Vector3DExt, Normalizable};
}
