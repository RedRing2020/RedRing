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
        BBox, Circle2D, Circle3D, Direction as DirectionTrait, Direction2D as Direction2DTrait,
        Direction3D as Direction3DTrait, Normalizable, StepCompatible, Vector as VectorTrait,
        Vector2D as Vector2DTrait, Vector3D as Vector3DTrait,
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

// バウンディングボックス - commonフォルダのトレイトを使用
pub use geometry2d::BBox2D;
pub use geometry3d::BBox3D;
// BBoxOpsとBBoxContainmentはgeo_foundationから直接インポート
pub use geo_foundation::{BBoxContainment, BBoxOps, BBoxTransform};

// 基本幾何プリミティブ
pub use geometry2d::{
    // Arc as Arc2D, Circle as Circle2D, Direction2D, InfiniteLine2D, Point, Vector2D,  // 一時的にコメントアウト（Direction2D整理中）
    Point as Point2D,
    Vector2D,
};
pub use geometry3d::{
    // Arc as Arc3D, Circle as Circle3D, Direction3D, InfiniteLine3D, Point3D, Vector3D,  // 一時的にコメントアウト（Direction3D整理中）
    Point as Point3D,
    Vector3D,
};

// 曲面プリミティブ
// pub use surface::{Sphere, Sphere3D, SphereF32, SphereF64};  // 一時的にコメントアウト

// 名前空間の整理
pub mod primitives_2d {
    // pub use crate::{BBox2D, Direction2D, InfiniteLine2D, Point, Vector2D};  // 一時的にコメントアウト（Direction整理中）
    pub use crate::geometry2d::Point;
    pub use crate::{BBox2D, Vector2D};
}

pub mod primitives_3d {
    // pub use crate::{BBox3D, Direction3D, InfiniteLine3D, Point3D, Sphere, Vector3D};  // 一時的にコメントアウト（Direction整理中）
    pub use crate::{BBox3D, Point3D, Vector3D}; // Sphereは一時的にコメントアウト
}

/// 便利な再エクスポート
pub mod prelude {
    // 基本幾何プリミティブ
    pub use crate::geometry2d::Point;
    pub use crate::{
        // Direction2D, Direction3D, InfiniteLine2D, InfiniteLine3D, Point, Point3D, Vector2D,  // 一時的にコメントアウト（Direction整理中）
        Point3D,
        Vector2D,
        Vector3D,
    };

    // バウンディングボックス
    pub use crate::geometry2d::BBox as BBox2D;
    pub use crate::geometry3d::bbox::BBox3D;
    // pub use crate::traits::BBoxOps;  // 利用可能になった時にコメントアウト解除

    // プリミティブトレイトと分類
    pub use crate::{
        DimensionClass, GeometricPrimitive, MeasurablePrimitive, PrimitiveCollection,
        PrimitiveKind, TransformablePrimitive,
    };

    // Direction トレイト (geo_foundation から)
    pub use geo_foundation::abstract_types::geometry::{
        Direction as DirectionTrait, Direction2D as Direction2DTrait,
        Direction3D as Direction3DTrait,
    };

    // ベクトルトレイト
    pub use crate::traits::Normalizable;
}
