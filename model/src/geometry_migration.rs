/// 段階的移行のための型エイリアス
///
/// 既存のmodel::geometry APIを段階的にgeo_coreに移行するための
/// 型エイリアスとフィーチャーフラグ

// フィーチャーフラグによる段階的移行
#[cfg(feature = "use_geo_core")]
pub use crate::geometry_adapter::{
    Vector2D,
    Vector3D,
    Point2D,
    Point3D,
    Direction3D,
    Normalize,
    Normed,
};

#[cfg(not(feature = "use_geo_core"))]
pub use crate::geometry::{
    geometry2d::{
        vector::Vector as Vector2D,
        point::Point as Point2D,
    },
    geometry3d::{
        vector::Vector as Vector3D,
        point::Point as Point3D,
        direction::Direction as Direction3D,
    },
};

#[cfg(not(feature = "use_geo_core"))]
pub use crate::geometry_trait::{
    normalize::Normalize,
    normed::Normed,
};

/// モジュール互換性のための re-export
pub mod geometry2d {
    #[cfg(feature = "use_geo_core")]
    pub use crate::geometry_adapter::{Vector2D as Vector, Point2D as Point};

    #[cfg(not(feature = "use_geo_core"))]
    pub use crate::geometry::geometry2d::{vector::Vector, point::Point};

    // Direction は既存実装を一時的に維持
    pub use crate::geometry::geometry2d::direction::Direction;
}

pub mod geometry3d {
    #[cfg(feature = "use_geo_core")]
    pub use crate::geometry_adapter::{
        Vector3D as Vector,
        Point3D as Point,
        Direction3D as Direction
    };

    #[cfg(not(feature = "use_geo_core"))]
    pub use crate::geometry::geometry3d::{
        vector::Vector,
        point::Point,
        direction::Direction
    };
}

/// 型変換ヘルパー
#[cfg(feature = "use_geo_core")]
pub mod conversion {
    use super::*;

    /// 既存のVector3Dからgeo_core Vector3Dへの変換
    pub fn legacy_to_geo_vector3d(v: &crate::geometry::geometry3d::vector::Vector) -> Vector3D {
        Vector3D::new(v.x(), v.y(), v.z())
    }

    /// geo_core Vector3Dから既存のVector3Dへの変換
    pub fn geo_to_legacy_vector3d(v: &Vector3D) -> crate::geometry::geometry3d::vector::Vector {
        crate::geometry::geometry3d::vector::Vector::new(v.x(), v.y(), v.z())
    }

    /// 既存のPoint3Dからgeo_core Point3Dへの変換
    pub fn legacy_to_geo_point3d(p: &crate::geometry::geometry3d::point::Point) -> Point3D {
        Point3D::new(p.x(), p.y(), p.z())
    }

    /// geo_core Point3Dから既存のPoint3Dへの変換
    pub fn geo_to_legacy_point3d(p: &Point3D) -> crate::geometry::geometry3d::point::Point {
        crate::geometry::geometry3d::point::Point::new(p.x(), p.y(), p.z())
    }
}

/// 移行用テストヘルパー
#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// 両実装での結果を比較するテストヘルパー
    pub fn compare_vector_operations() {
        let legacy_v1 = crate::geometry::geometry3d::vector::Vector::new(1.0, 2.0, 3.0);
        let legacy_v2 = crate::geometry::geometry3d::vector::Vector::new(4.0, 5.0, 6.0);

        #[cfg(feature = "use_geo_core")]
        {
            let geo_v1 = Vector3D::new(1.0, 2.0, 3.0);
            let geo_v2 = Vector3D::new(4.0, 5.0, 6.0);

            // 内積の比較
            let legacy_dot = legacy_v1.dot(&legacy_v2);
            let geo_dot = geo_v1.dot(&geo_v2);
            assert!((legacy_dot - geo_dot).abs() < 1e-10, "Dot product mismatch");

            // 外積の比較
            let legacy_cross = legacy_v1.cross(&legacy_v2);
            let geo_cross = geo_v1.cross(&geo_v2);
            assert!((legacy_cross.x() - geo_cross.x()).abs() < 1e-10, "Cross product X mismatch");
            assert!((legacy_cross.y() - geo_cross.y()).abs() < 1e-10, "Cross product Y mismatch");
            assert!((legacy_cross.z() - geo_cross.z()).abs() < 1e-10, "Cross product Z mismatch");
        }
    }
}
