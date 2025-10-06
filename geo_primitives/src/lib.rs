/// 幾何プリミティブクレート
///
/// geo_core数学基礎処理を利用して、点・線分・円・平面・
/// 多角形・三角形メッシュ等のプリミティブ形状を定義して分類分けを行う

/// トレイト定義群（再構成済み）
pub mod traits;

/// ユニットテスト群
#[cfg(test)]
mod unit_tests;

/// 2D幾何プリミティブ
pub mod geometry2d;

/// 3D幾何プリミティブ (f64ベース)
pub mod geometry3d;





// 分類システム、プリミティブトレイト（traitsモジュールから）
pub use traits::common::{
    PrimitiveKind, DimensionClass, GeometryPrimitive,
    GeometricPrimitive, TransformablePrimitive, MeasurablePrimitive, PrimitiveCollection
};

// バウンディングボックス（geometryモジュールから）
pub use geometry2d::BBox2D;
pub use geometry3d::BBox3D;

// バウンディングボックストレイト
pub use traits::{BoundingBox, BoundingBoxOps, CollisionBounds};

// CAD統合層（traitsモジュールから）
pub use traits::geometry::{CadPoint, CadVector};

// 残りのCAD構造体（独立）
pub mod cad_direction;
pub mod cad_circle;
pub mod cad_ellipse;
pub mod cad_ellipse_arc;

pub use cad_direction::CadDirection;
pub use cad_circle::CadCircle;
pub use cad_ellipse::CadEllipse;
pub use cad_ellipse_arc::CadEllipseArc;

// 2Dプリミティブ（削除済み - geo_coreの基本構造体を使用）
// Triangle modules removed - use f64geom or create minimal implementations if needed

// 3Dプリミティブ / geometry3d 統合
// pub mod geometry3d; // legacy (Scalar-based) - removed, use f64geom instead

// 正準f64幾何プリミティブの公開（最小構成）
pub use geometry2d::{Point2D, Vector2D};
pub use geometry3d::{Vector3D, Point3D};

// Polygon module removed - use f64geom or create minimal implementations if needed

// pub mod mesh;  // 一時的にコメントアウト (Point3D依存のため)
// pub use mesh::TriangleMesh;

// 名前空間の整理
pub mod primitives_2d {
    pub use crate::Point2D;
    // Triangle2D, Polygon2D removed - use f64geom implementations if needed
    // LineSegment2D - TODO: implement f64 version
}

pub mod primitives_3d {
    // All primitives temporarily commented out - minimal implementation
}

/// 便利な再エクスポート
pub mod prelude {
    pub use crate::{
        GeometricPrimitive, TransformablePrimitive, MeasurablePrimitive, PrimitiveCollection,
        PrimitiveKind, DimensionClass,
        // バウンディングボックス
        BBox2D, BBox3D, BoundingBox, BoundingBoxOps, CollisionBounds,
        // 2D/3Dプリミティブ
        Point2D, Vector2D, Point3D, Vector3D,
        // CAD統合層
        CadPoint, CadVector, CadDirection,
        CadCircle, CadEllipse, CadEllipseArc,
    };

    // ベクトルトレイト
    pub use crate::traits::{Vector, Vector2DExt, Vector3DExt, Normalizable};
}


