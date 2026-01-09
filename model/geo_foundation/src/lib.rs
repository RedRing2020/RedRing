//! geo_foundation - 幾何計算の基盤クレート
//!
//! geo_foundation は抽象化・インターフェース層
//! geo_primitives の具体実装を抽象化して呼び出すためのトレイト定義

// Core Foundation - 幾何形状の中核基盤トレイト
pub mod core_foundation;

// Extension Foundation - 幾何形状の拡張基盤トレイト
pub mod extension_foundation;

// Classification - 幾何プリミティブの分類システム
pub mod classification;

// Classification - 幾何プリミティブの分類
pub use classification::{DimensionClass, GeometryPrimitive, PrimitiveKind};

// Core Traits - 基本機能抽象化（主要インターフェース）
pub mod core;

// Commons - 共通計算トレイト
pub mod commons;

// Extension Traits - 拡張操作トレイト群
pub mod extensions;

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

// Core Traitsを再エクスポート（主要インターフェース）
pub use core::{
    arc_core_traits::{
        Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
        Arc3DMeasure, Arc3DProperties,
    },
    bbox_core_traits::{
        BBox2DConstructor, BBox2DCore, BBox2DMeasure, BBox2DProperties, BBox3DConstructor,
        BBox3DCore, BBox3DMeasure, BBox3DProperties,
    },
    circle_core_traits::{
        Circle2DConstructor, Circle2DCore, Circle2DMeasure, Circle2DProperties,
        Circle3DConstructor, Circle3DCore, Circle3DMeasure, Circle3DProperties,
    },
    // Surface/Solid Core Traits
    conical_solid_core_traits::{
        ConicalSolid3DConstructor, ConicalSolid3DCore, ConicalSolid3DMeasure,
        ConicalSolid3DProperties,
    },
    conical_surface_core_traits::{
        ConicalSurface3DConstructor, ConicalSurface3DCore, ConicalSurface3DMeasure,
        ConicalSurface3DProperties,
    },
    cylindrical_solid_core_traits::{
        CylindricalSolid3DConstructor, CylindricalSolid3DCore, CylindricalSolid3DMeasure,
        CylindricalSolid3DProperties,
    },
    cylindrical_surface_core_traits::{
        CylindricalSurface3DConstructor, CylindricalSurface3DCore, CylindricalSurface3DMeasure,
        CylindricalSurface3DProperties,
    },
    direction_core_traits::{
        Direction2DConstructor, Direction2DCore, Direction2DMeasure, Direction2DProperties,
        Direction3DConstructor, Direction3DCore, Direction3DMeasure, Direction3DProperties,
    },
    ellipse_arc_core_traits::{
        EllipseArc2DConstructor, EllipseArc2DCore, EllipseArc2DMeasure, EllipseArc2DProperties,
        EllipseArc3DConstructor, EllipseArc3DCore, EllipseArc3DMeasure, EllipseArc3DProperties,
    },
    ellipse_core_traits::{
        Ellipse2DConstructor, Ellipse2DCore, Ellipse2DMeasure, Ellipse2DProperties,
        Ellipse3DConstructor, Ellipse3DCore, Ellipse3DMeasure, Ellipse3DProperties,
    },
    ellipsoidal_solid_core_traits::{
        EllipsoidalSolid3DConstructor, EllipsoidalSolid3DCore, EllipsoidalSolid3DMeasure,
        EllipsoidalSolid3DProperties,
    },
    ellipsoidal_surface_core_traits::{
        EllipsoidalSurface3DConstructor, EllipsoidalSurface3DCore, EllipsoidalSurface3DMeasure,
        EllipsoidalSurface3DProperties,
    },
    infinite_line_core_traits::{
        InfiniteLine2DConstructor, InfiniteLine2DCore, InfiniteLine2DMeasure,
        InfiniteLine2DProperties, InfiniteLine3DConstructor, InfiniteLine3DCore,
        InfiniteLine3DMeasure, InfiniteLine3DProperties,
    },
    linesegment_core_traits::{
        LineSegment2DConstructor, LineSegment2DCore, LineSegment2DMeasure, LineSegment2DProperties,
        LineSegment3DConstructor, LineSegment3DCore, LineSegment3DMeasure, LineSegment3DProperties,
    },
    // NURBS Core Traits
    nurbs_curve_2d_core_traits::{
        NurbsCurve2DConstructor, NurbsCurve2DCore, NurbsCurve2DMeasure, NurbsCurve2DProperties,
    },
    nurbs_curve_3d_core_traits::{
        NurbsCurve3DConstructor, NurbsCurve3DMeasure, NurbsCurve3DProperties,
    },
    nurbs_surface_3d_core_traits::{
        NurbsSurface3DConstructor, NurbsSurface3DCore, NurbsSurface3DMeasure,
        NurbsSurface3DProperties,
    },
    nurbs_traits::{
        BasisFunction, BiParametricGeometry, KnotVector as KnotVectorTrait, NurbsCurve,
        NurbsCurveOperations, NurbsSurface, NurbsSurfaceOperations, ParametricGeometry,
        WeightedGeometry,
    },
    plane_core_traits::{Plane3DConstructor, Plane3DCore, Plane3DMeasure, Plane3DProperties},
    point_core_traits::{
        Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties, Point3DConstructor,
        Point3DCore, Point3DMeasure, Point3DProperties,
    },
    ray_core_traits::{
        Ray2DConstructor, Ray2DCore, Ray2DMeasure, Ray2DProperties, Ray3DConstructor, Ray3DCore,
        Ray3DMeasure, Ray3DProperties,
    },
    spherical_solid_core_traits::{
        SphericalSolid3DConstructor, SphericalSolid3DCore, SphericalSolid3DMeasure,
        SphericalSolid3DProperties,
    },
    spherical_surface_core_traits::{
        SphericalSurface3DConstructor, SphericalSurface3DCore, SphericalSurface3DMeasure,
        SphericalSurface3DProperties,
    },
    torus_solid_core_traits::{
        TorusSolid3DConstructor, TorusSolid3DCore, TorusSolid3DMeasure, TorusSolid3DProperties,
    },
    torus_surface_core_traits::{
        TorusSurface3DConstructor, TorusSurface3DCore, TorusSurface3DMeasure,
        TorusSurface3DProperties,
    },
    triangle_core_traits::{
        Triangle2DConstructor, Triangle2DCore, Triangle2DMeasure, Triangle2DProperties,
        Triangle3DConstructor, Triangle3DCore, Triangle3DMeasure, Triangle3DProperties,
    },
    triangle_traits::Triangle3D as Triangle3DTrait,
    vector_core_traits::{
        Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties,
        Vector3DConstructor, Vector3DCore, Vector3DMeasure, Vector3DProperties,
    },
};

// Extension Foundation Traitsを再エクスポート
pub use extension_foundation::{
    Bounded, CollectionExtension, ExtensionFoundation, MeasurableExtension, SpatialExtension,
    TransformableExtension,
};

// Commons Traitsを再エクスポート

// Core Transform Traitsを再エクスポート（extensions → core移動）
pub use core::transform::{
    AnalysisTransform2D, AnalysisTransform3D, AnalysisTransformSupport, AnalysisTransformVector2D,
    AnalysisTransformVector3D,
};

// Core Transform Errorを再エクスポート（段階的移行: 両方からアクセス可能）
pub use core::transform_error::{
    SafeTransform as CoreSafeTransform, TransformError as CoreTransformError,
};

// Extension Traitsを再エクスポート(既存互換性維持)
pub use extensions::{
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

    // Commons Bridge - geo_commons への Foundation Pattern準拠アクセス
    pub use geo_commons as commons;

    // Note: AABB型は geo_core から直接 import してください
    // Note: 具体的な幾何型は geo_primitives から直接 import してください
}
