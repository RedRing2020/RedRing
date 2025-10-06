/// 幾何プリミティブクレート
///
/// geo_core数学基礎処理を利用して、点・線分・円・平面・
/// 多角形・三角形メッシュ等のプリミティブ形状を定義して分類分けを行う

/// 幾何計算ユーティリティ
pub mod geometry_utils;

/// 3Dバウンディングボックス
#[derive(Debug, Clone)]
pub struct BoundingBox {
    pub min: geo_core::Point3D,
    pub max: geo_core::Point3D,
}

impl BoundingBox {
    pub fn new(min: geo_core::Point3D, max: geo_core::Point3D) -> Self {
        Self { min, max }
    }

    /// 2D点から3Dバウンディングボックスを作成（Z=0）
    pub fn from_2d(min: geo_core::Point2D, max: geo_core::Point2D) -> Self {
        Self {
            min: geo_core::Point3D::new(*min.x(), *min.y(), geo_core::Scalar::new(0.0)),
            max: geo_core::Point3D::new(*max.x(), *max.y(), geo_core::Scalar::new(0.0)),
        }
    }

    pub fn width(&self) -> f64 {
        self.max.x().value() - self.min.x().value()
    }

    pub fn height(&self) -> f64 {
        self.max.y().value() - self.min.y().value()
    }

    pub fn depth(&self) -> f64 {
        self.max.z().value() - self.min.z().value()
    }
}

/// 全ての幾何プリミティブが実装する共通トレイト
pub trait GeometricPrimitive {
    /// プリミティブの種類を返す
    fn primitive_kind(&self) -> PrimitiveKind;

    /// バウンディングボックスを返す
    fn bounding_box(&self) -> BoundingBox;

    /// プリミティブの測定値（長さ、面積、体積など）を返す
    fn measure(&self) -> Option<f64>;
}

// 分類システム
pub mod classification;
pub use classification::{PrimitiveKind, GeometryClassification, ComplexityLevel};

// CAD統合層: analysisからの依存を解決して常時利用可能に
pub mod cad_point;
pub use cad_point::CadPoint;

pub mod cad_vector;
pub use cad_vector::CadVector;

pub mod cad_direction;
pub use cad_direction::CadDirection;

pub mod cad_circle;
pub use cad_circle::CadCircle;

pub mod cad_ellipse;
pub use cad_ellipse::CadEllipse;

pub mod cad_ellipse_arc;
pub use cad_ellipse_arc::CadEllipseArc;

// 2Dプリミティブ（削除済み - geo_coreの基本構造体を使用）

pub mod triangle;
pub use triangle::{Triangle2D, Triangle3D};

// 3Dプリミティブ / geometry3d 統合
// pub mod geometry3d; // legacy (Scalar-based) - removed, use f64geom instead

// Alias layer: expose canonical f64 implementations under old names (no deprecation yet)
pub mod aliases {
    pub use crate::f64geom::{
        FLineSegment3 as LineSegment3D,
        FPlane as Plane,
        FCircle3 as Circle3D,
        FDirection3 as Direction3D,
    };
}
pub use aliases::{LineSegment3D, Plane, Circle3D, Direction3D};

// 新しい f64 ベース幾何カーネル (移行中)
pub mod f64geom {
    pub mod vector3; // FVector3
    pub mod point3;  // FPoint3
    pub mod direction3; // FDirection3
    pub mod line_segment3; // FLineSegment3
    pub mod plane; // FPlane
    pub mod circle3; // FCircle3

    pub use vector3::FVector3;
    pub use point3::FPoint3;
    pub use direction3::FDirection3;
    pub use line_segment3::FLineSegment3;
    pub use plane::FPlane;
    pub use circle3::FCircle3;
}

pub mod polygon;
pub use polygon::{Polygon2D, Polygon3D};

pub mod mesh;
pub use mesh::TriangleMesh;

// 名前空間の整理
pub mod primitives_2d {
    pub use geo_core::{Point2D, LineSegment2D};
    pub use crate::{Triangle2D, Polygon2D};
}

pub mod primitives_3d {
    pub use geo_core::Point3D;
    pub use crate::{Triangle3D, Plane, Polygon3D, TriangleMesh};
}

/// 便利な再エクスポート
pub mod prelude {
    pub use crate::{
        GeometricPrimitive, PrimitiveKind, BoundingBox,
        // 2D/3Dプリミティブ
        Triangle2D, Triangle3D,
    Plane, LineSegment3D, Circle3D, Direction3D,
    // f64canonical (移行中)
    f64geom::FVector3, f64geom::FPoint3, f64geom::FDirection3,
    f64geom::FLineSegment3, f64geom::FPlane, f64geom::FCircle3,
        Polygon2D, Polygon3D,
        TriangleMesh,
    };

    // CAD統合層 (analysis依存を削除済み)
    pub use crate::{
        CadPoint, CadVector, CadDirection,
        CadCircle, CadEllipse, CadEllipseArc,
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use geo_core::Scalar;

    #[test]
    fn test_bounding_box_dimensions() {
        let min = geo_core::Point3D::new(
            Scalar::new(0.0),
            Scalar::new(0.0),
            Scalar::new(0.0),
        );
        let max = geo_core::Point3D::new(
            Scalar::new(2.0),
            Scalar::new(3.0),
            Scalar::new(4.0),
        );

        let bbox = BoundingBox::new(min, max);

        assert!((bbox.width() - 2.0).abs() < 1e-10);
        assert!((bbox.height() - 3.0).abs() < 1e-10);
        assert!((bbox.depth() - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_from_2d_bounding_box() {
        let min_2d = geo_core::Point2D::from_f64(1.0, 2.0);
        let max_2d = geo_core::Point2D::from_f64(3.0, 4.0);

        let bbox = BoundingBox::from_2d(min_2d, max_2d);

        assert!((bbox.width() - 2.0).abs() < 1e-10);
        assert!((bbox.height() - 2.0).abs() < 1e-10);
        assert!((bbox.depth() - 0.0).abs() < 1e-10);
    }
}
