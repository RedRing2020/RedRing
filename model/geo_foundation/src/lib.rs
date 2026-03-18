//! geo_foundation - 幾何計算の基盤クレート
//!
//! geo_foundation は抽象化・インターフェース層
//! geo_primitives の具体実装を抽象化して呼び出すためのトレイト定義

// Classification - 幾何プリミティブの分類システム
pub mod classification;

// Classification - 幾何プリミティブの分類
pub use classification::{DimensionClass, GeometryPrimitive, PrimitiveKind};

// Geometry namespace - 幾何形状領域の構造化
pub mod geometry;

// Core namespace (backward compatibility: alias to geometry::core)
pub use geometry::core;

// Foundation namespace (backward compatibility: alias to geometry::foundation)
pub use geometry::foundation::core_foundation;
pub use geometry::foundation::extension_foundation;

// Entity namespace - エンティティ領域の構造化
pub mod entity;

// Commons - 共通計算トレイト
pub mod commons;

// Extensions namespace (backward compatibility: alias to geometry::extensions)
pub use geometry::extensions;

// BBox - 境界ボックス実装（共通ユーティリティ）

// 許容誤差管理モジュール
pub mod tolerance;

// 許容誤差移行支援モジュール（将来削除予定）
pub mod tolerance_migration;

// テストモジュール
#[cfg(test)]
mod tolerance_tests;

// analysisクレートからScalarトレイトを再エクスポート
pub use analysis::abstract_types::{Angle, Scalar, TolerantEq};

// analysisクレートから定数を再エクスポート
pub use analysis::{
    game, precision, GeometricTolerance, DEG_TO_RAD, E, GEOMETRIC_ANGLE_TOLERANCE,
    GEOMETRIC_DISTANCE_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6, RAD_TO_DEG, TAU,
};

// Geometry Core Traitsを再エクスポート（主要インターフェース）
pub use geometry::core::{
    arc_traits::{
        Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
        Arc3DMeasure, Arc3DProperties,
    },
    circle_traits::{
        Circle2DConstructor, Circle2DCore, Circle2DMeasure, Circle2DProperties,
        Circle3DConstructor, Circle3DCore, Circle3DMeasure, Circle3DProperties,
    },
    // Surface/Solid Core Traits
    conical_solid_traits::{
        ConicalSolid3DConstructor, ConicalSolid3DCore, ConicalSolid3DMeasure,
        ConicalSolid3DProperties,
    },
    conical_surface_traits::{
        ConicalSurface3DConstructor, ConicalSurface3DCore, ConicalSurface3DMeasure,
        ConicalSurface3DProperties,
    },
    cylindrical_solid_traits::{
        CylindricalSolid3DConstructor, CylindricalSolid3DCore, CylindricalSolid3DMeasure,
        CylindricalSolid3DProperties,
    },
    cylindrical_surface_traits::{
        CylindricalSurface3DConstructor, CylindricalSurface3DCore, CylindricalSurface3DMeasure,
        CylindricalSurface3DProperties,
    },
    direction_traits::{
        Direction2DConstructor, Direction2DCore, Direction2DMeasure, Direction2DProperties,
        Direction3DConstructor, Direction3DCore, Direction3DMeasure, Direction3DProperties,
    },
    ellipse_arc_traits::{
        EllipseArc2DConstructor, EllipseArc2DCore, EllipseArc2DMeasure, EllipseArc2DProperties,
        EllipseArc3DConstructor, EllipseArc3DCore, EllipseArc3DMeasure, EllipseArc3DProperties,
    },
    ellipse_traits::{
        Ellipse2DConstructor, Ellipse2DCore, Ellipse2DMeasure, Ellipse2DProperties,
        Ellipse3DConstructor, Ellipse3DCore, Ellipse3DMeasure, Ellipse3DProperties,
    },
    ellipsoidal_solid_traits::{
        EllipsoidalSolid3DConstructor, EllipsoidalSolid3DCore, EllipsoidalSolid3DMeasure,
        EllipsoidalSolid3DProperties,
    },
    ellipsoidal_surface_traits::{
        EllipsoidalSurface3DConstructor, EllipsoidalSurface3DCore, EllipsoidalSurface3DMeasure,
        EllipsoidalSurface3DProperties,
    },
    infinite_line_traits::{
        InfiniteLine2DConstructor, InfiniteLine2DCore, InfiniteLine2DMeasure,
        InfiniteLine2DProperties, InfiniteLine3DConstructor, InfiniteLine3DCore,
        InfiniteLine3DMeasure, InfiniteLine3DProperties,
    },
    linesegment_traits::{
        LineSegment2DConstructor, LineSegment2DCore, LineSegment2DMeasure, LineSegment2DProperties,
        LineSegment3DCollisionDetection, LineSegment3DConstructor, LineSegment3DCore,
        LineSegment3DMeasure, LineSegment3DProperties,
    },
    // NURBS Core Traits
    nurbs_curve_2d_traits::{
        NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
    },
    nurbs_curve_3d_traits::{NurbsCurve3DConstructor, NurbsCurve3DMeasure, NurbsCurve3DProperties},
    nurbs_surface_3d_traits::{
        NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure,
        NurbsSurface3DProperties,
    },
    plane_traits::{Plane3DConstructor, Plane3DCore, Plane3DMeasure, Plane3DProperties},
    point_traits::{
        Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties, Point3DConstructor,
        Point3DCore, Point3DMeasure, Point3DProperties,
    },
    ray_traits::{
        Ray2DConstructor, Ray2DCore, Ray2DMeasure, Ray2DProperties, Ray3DConstructor, Ray3DCore,
        Ray3DMeasure, Ray3DProperties,
    },
    rectangle_traits::{
        Rect2DConstructor, Rect2DCore, Rect2DMeasure, Rect2DProperties, Rect3DConstructor,
        Rect3DCore, Rect3DMeasure, Rect3DProperties,
    },
    spherical_solid_traits::{
        SphericalSolid3DConstructor, SphericalSolid3DCore, SphericalSolid3DMeasure,
        SphericalSolid3DProperties,
    },
    spherical_surface_traits::{
        SphericalSurface3DConstructor, SphericalSurface3DCore, SphericalSurface3DMeasure,
        SphericalSurface3DProperties,
    },
    torus_solid_traits::{
        TorusSolid3DConstructor, TorusSolid3DCore, TorusSolid3DMeasure, TorusSolid3DProperties,
    },
    torus_surface_traits::{
        TorusSurface3DConstructor, TorusSurface3DCore, TorusSurface3DMeasure,
        TorusSurface3DProperties,
    },
    triangle_traits::{
        Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
        Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
    },
    vector_traits::{
        Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties,
        Vector3DConstructor, Vector3DCore, Vector3DMeasure, Vector3DProperties,
    },
};

// #318 移行用ブリッジ: 既存公開名は維持し、contracts 版は別名前空間で併置する。
pub mod contracts {
    pub use geo_contracts::classification::{DimensionClass, GeometryPrimitive, PrimitiveKind};
    pub use geo_contracts::entity::{
        EntityDisplayProperties, EntityIdentity, LineEntity3DProperties,
    };
    pub use geo_contracts::geometry::core::{
        Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
        Arc3DMeasure, Arc3DProperties, Circle2DConstructor, Circle2DCore, Circle2DMeasure,
        Circle2DProperties, Circle3DConstructor, Circle3DCore, Circle3DMeasure, Circle3DProperties,
        NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
        NurbsCurve3DConstructor, NurbsCurve3DCore, NurbsCurve3DMeasure, NurbsCurve3DProperties,
        NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure,
        NurbsSurface3DProperties, Point2DConstructor, Point2DCore, Point2DMeasure,
        Point2DProperties, Point3DConstructor, Point3DCore, Point3DMeasure, Point3DProperties,
        Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
        Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
        Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties,
        Vector3DConstructor, Vector3DCore, Vector3DMeasure, Vector3DProperties,
    };
}

// Entity Core Traitsを再エクスポート（Entity namespace）
pub use entity::core::{EntityDisplayProperties, EntityIdentity, LineEntity3DProperties};

// Extension Foundation Traitsを再エクスポート
pub use geometry::foundation::extension_foundation::{
    Bounded, CollectionExtension, ExtensionFoundation, MeasurableExtension, SpatialExtension,
    TransformableExtension,
};

// Commons Traitsを再エクスポート

// Geometry Core Transform Traitsを再エクスポート（extensions → core移動）
pub use geometry::core::transform::{
    AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport, AnalysisTransformVector2D,
    AnalysisTransformVector3D,
};

// Geometry Core Transform Errorを再エクスポート（段階的移行: 両方からアクセス可能）
pub use geometry::core::transform_error::{
    SafeTransform as CoreSafeTransform, TransformError as CoreTransformError,
};

// Extension Traitsを再エクスポート(既存互換性維持)
pub use geometry::extensions::{
    AdvancedCollision,
    BasicCollision,
    BasicIntersection,
    BooleanError,
    BooleanOperations,
    MultipleBooleanOperations,
    MultipleIntersection,
    PointDistance,
    SafeTransform,
    SelfIntersection,
    TolerantBooleanOperations,
    TransformError, // 既存ルート維持
};

// Geometry Core Foundationを再エクスポート
// 注意: 実際の実装はabstract_types/foundation/にある
// pub use geometry::{
//     BasicContainment, BasicDirectional, BasicMetrics, BasicParametric, CoreFoundation,
// };

// 許容誤差管理を再エクスポート
pub use tolerance::{GeometryContext, ToleranceSettings};

// Note: 具体的な型は geo_primitives から直接 import してください
// 循環依存を避けるため、geo_foundation では型の再エクスポートは行いません

/// 便利な再エクスポート
pub mod prelude {
    // Core Transform Traits（core::transformから）
    pub use crate::{
        AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport,
        AnalysisTransformVector2D, AnalysisTransformVector3D,
    };
    // Extension機能(既存互換性維持)
    pub use crate::{
        GeometryContext, SafeTransform, ToleranceSettings, TransformError, DEG_TO_RAD, E,
        GEOMETRIC_ANGLE_TOLERANCE, GEOMETRIC_DISTANCE_TOLERANCE, PI, PI_2, PI_3, PI_4, PI_6,
        RAD_TO_DEG, TAU,
    };
    // Core Transform Errorルート(段階的移行用)
    pub use crate::{CoreSafeTransform, CoreTransformError};
    pub use analysis::abstract_types::{Angle, Scalar, TolerantEq};

    // Ellipse Calculation Traits (commons経由)
    pub use crate::commons::{
        EllipseAccuracyAnalysis, EllipseAdaptiveCalculation, EllipseCalculation,
    };

    // Commons Bridge - Foundation commons へのアクセス
    pub use crate::commons;

    // Note: AABB型は geo_core から直接 import してください
    // Note: 具体的な幾何型は geo_primitives から直接 import してください
}
