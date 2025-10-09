//! 幾何プリミティブクレート
//!
//! geo_core数学基礎処理を利用して、点・線分・円・平面・
//! 多角形・三角形メッシュ等のプリミティブ形状を定義して分類分けを行う

// コアモジュール
pub mod geometry2d;
pub mod geometry3d;
// pub mod surface;  // 一時的にコメントアウト（Point3Dジェネリック化によるコンパイルエラーのため）

// Traitsモジュールの内容を統合
pub mod traits {
    //! Traits module organization
    //! トレイトとユーティリティの整理されたモジュール構成

    // Common utilities - 共通ユーティリティと分類
    pub mod common;

    // Re-export traits directly from geo_foundation
    pub use geo_foundation::abstract_types::geometry::{
        BBox, BBoxOps, Circle2D, Circle3D, CollisionBBox, Direction, Direction2D, Direction3D,
        Normalizable, StepCompatible, Vector, Vector2D, Vector2DExt, Vector3D, Vector3DExt,
    };

    // Re-export abstract primitive traits from geo_foundation
    pub use geo_foundation::abstract_types::geometry::{
        DimensionClass, GeometricPrimitive, GeometryPrimitive, MeasurablePrimitive,
        PrimitiveCollection, PrimitiveKind, SpatialRelation, TransformablePrimitive,
    };

    // Re-export utility functions from geo_foundation
    pub use geo_foundation::abstract_types::geometry::utils::{
        clamp, f64_max, f64_min, in_range, lerp, scalar_distance, scalar_max, scalar_min,
    };

    // Re-export local Arc2D trait from geometry2d module
    pub use crate::geometry2d::Arc2D;

    // Re-export geo_primitives specific implementations
    pub use common::GeometryUnion;
}

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
pub use traits::{BBox, BBoxOps, CollisionBBox};

// 基本幾何プリミティブ
pub use geometry2d::{
    // Arc as Arc2D, Circle as Circle2D, Direction2D, InfiniteLine2D, Point2D, Vector2D,  // 一時的にコメントアウト（Direction2D整理中）
    Point2D,
    Vector2D,
};
pub use geometry3d::{
    // Arc as Arc3D, Circle as Circle3D, Direction3D, InfiniteLine3D, Point3D, Vector3D,  // 一時的にコメントアウト（Direction3D整理中）
    Point3D,
    Vector3D,
};

// 曲面プリミティブ
// pub use surface::{Sphere, Sphere3D, SphereF32, SphereF64};  // 一時的にコメントアウト

// 名前空間の整理
pub mod primitives_2d {
    // pub use crate::{BBox2D, Direction2D, InfiniteLine2D, Point2D, Vector2D};  // 一時的にコメントアウト（Direction整理中）
    pub use crate::{BBox2D, Point2D, Vector2D};
}

pub mod primitives_3d {
    // pub use crate::{BBox3D, Direction3D, InfiniteLine3D, Point3D, Sphere, Vector3D};  // 一時的にコメントアウト（Direction整理中）
    pub use crate::{BBox3D, Point3D, Vector3D}; // Sphereは一時的にコメントアウト
}

/// 便利な再エクスポート
pub mod prelude {
    // 基本幾何プリミティブ
    pub use crate::{
        // Direction2D, Direction3D, InfiniteLine2D, InfiniteLine3D, Point2D, Point3D, Vector2D,  // 一時的にコメントアウト（Direction整理中）
        Point2D,
        Point3D,
        Vector2D,
        Vector3D,
    };

    // バウンディングボックス
    pub use crate::{BBox, BBox2D, BBox3D, BBoxOps, CollisionBBox};

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
