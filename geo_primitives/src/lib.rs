//! 幾何プリミティブクレート
//!
//! geo_core数学基礎処理を利用して、点・線分・円・平面・
//! 多角形・三角形メッシュ等のプリミティブ形状を定義して分類分けを行う

// コアモジュール
pub mod geometry2d;
pub mod geometry3d;
pub mod traits;

// テストモジュール
#[cfg(test)]
mod unit_tests;

// 分類システムとプリミティブトレイト
pub use traits::common::{
    DimensionClass, GeometricPrimitive, GeometryPrimitive, MeasurablePrimitive,
    PrimitiveCollection, PrimitiveKind, TransformablePrimitive,
};

// バウンディングボックス
pub use geometry2d::BBox2D;
pub use geometry3d::BBox3D;
pub use traits::{BoundingBox, BoundingBoxOps, CollisionBounds};

// 基本幾何プリミティブ
pub use geometry2d::{Arc as Arc2D, Circle as Circle2D, Direction2D, Point2D, Vector2D};
pub use geometry3d::{Arc as Arc3D, Circle as Circle3D, Direction3D, Point3D, Vector3D};

// 名前空間の整理
pub mod primitives_2d {
    pub use crate::{BBox2D, Direction2D, Point2D, Vector2D};
}

pub mod primitives_3d {
    pub use crate::{BBox3D, Direction3D, Point3D, Vector3D};
}

/// 便利な再エクスポート
pub mod prelude {
    // 基本幾何プリミティブ
    pub use crate::{Direction2D, Direction3D, Point2D, Point3D, Vector2D, Vector3D};

    // バウンディングボックス
    pub use crate::{BBox2D, BBox3D, BoundingBox, BoundingBoxOps, CollisionBounds};

    // プリミティブトレイトと分類
    pub use crate::{
        DimensionClass, GeometricPrimitive, MeasurablePrimitive, PrimitiveCollection,
        PrimitiveKind, TransformablePrimitive,
    };

    // Direction トレイト (geo_foundation から)
    pub use geo_foundation::abstract_types::geometry::{
        Direction, Direction2D as Direction2DTrait, Direction3D as Direction3DTrait,
    };

    // ベクトルトレイト
    pub use crate::traits::{Normalizable, Vector, Vector2DExt, Vector3DExt};
}
