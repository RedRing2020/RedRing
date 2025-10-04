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

// 基本幾何プリミティブ（CAD統合層は削除済み）

// 2D/3D幾何プリミティブ（フォルダ別に分離）
pub mod geometry2d;
pub use geometry2d::{Triangle2D, Polygon2D};

pub mod geometry3d;
pub use geometry3d::{Point3D, Vector3D, Triangle3D, Polygon3D, Plane, TriangleMesh};

// サーフェスプリミティブ
pub mod surface;
pub use surface::{Sphere, Cylinder, Cone, Ellipsoid, Torus, TorusType};

// 共有ユーティリティ
pub mod util;

// 名前空間の整理
pub mod primitives_2d {
    pub use geo_core::{Point2D, LineSegment2D};
    pub use crate::{Triangle2D, Polygon2D};
}

pub mod primitives_3d {
    pub use geo_core::{Point3D, Sphere};
    pub use crate::{Triangle3D, Plane, Polygon3D, TriangleMesh};
}

/// 便利な再エクスポート
pub mod prelude {
    pub use crate::{
        GeometricPrimitive, PrimitiveKind, BoundingBox,
        // 2D基本プリミティブ
        geometry2d::{Point2D, Circle2D, Triangle2D, Polygon2D, InfiniteLine2D, Ray2D, Line2D, Arc2D},
        // 3D基本プリミティブ
        geometry3d::{Point3D, Vector3D, Triangle3D, Polygon3D, Plane, TriangleMesh, InfiniteLine3D, Ray3D, Line3D, Arc3D},
        // サーフェスプリミティブ
        surface::{Sphere, Cylinder, Cone, Ellipsoid, Torus, TorusType},
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
