/// 幾何プリミティブクレート
/// 
/// geo_core数学基礎処理を利用して、点・線分・円・平面・
/// 多角形・三角形メッシュ等のプリミティブ形状を定義して分類分けを行う

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

// 2Dプリミティブ
pub mod point;
pub use point::{Point2D, Point3D};

pub mod line;
pub use line::{LineSegment2D, LineSegment3D};

pub mod circle;
pub use circle::{Circle2D, Circle3D};

pub mod triangle;
pub use triangle::{Triangle2D, Triangle3D};

// 3Dプリミティブ
pub mod plane;
pub use plane::Plane3D;

pub mod polygon;
pub use polygon::{Polygon2D, Polygon3D};

pub mod mesh;
pub use mesh::{TriangleMesh3D, VertexIndex, Face};

// 名前空間の整理
pub mod primitives_2d {
    pub use crate::{Point2D, LineSegment2D, Circle2D, Triangle2D, Polygon2D};
}

pub mod primitives_3d {
    pub use crate::{Point3D, LineSegment3D, Circle3D, Triangle3D, Plane3D, Polygon3D, TriangleMesh3D};
}

/// 便利な再エクスポート
pub mod prelude {
    pub use crate::{
        GeometricPrimitive, PrimitiveKind, BoundingBox,
        Point2D, Point3D,
        LineSegment2D, LineSegment3D,
        Circle2D, Circle3D,
        Triangle2D, Triangle3D,
        Plane3D,
        Polygon2D, Polygon3D,
        TriangleMesh3D, VertexIndex, Face,
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