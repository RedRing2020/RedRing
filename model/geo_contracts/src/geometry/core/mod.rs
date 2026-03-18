//! Geometry core contracts.
// 実装ロジックは置かず、shape contract の定義だけを保持する。

pub mod arc_traits;
pub mod circle_traits;
pub mod point_traits;
pub mod vector_traits;

pub use arc_traits::{
    Arc2DConstructor, Arc2DCore, Arc2DMeasure, Arc2DProperties, Arc3DConstructor, Arc3DCore,
    Arc3DMeasure, Arc3DProperties,
};
pub use circle_traits::{
    Circle2DConstructor, Circle2DCore, Circle2DMeasure, Circle2DProperties, Circle3DConstructor,
    Circle3DCore, Circle3DMeasure, Circle3DProperties,
};
pub use point_traits::{
    Point2DConstructor, Point2DCore, Point2DMeasure, Point2DProperties, Point3DConstructor,
    Point3DCore, Point3DMeasure, Point3DProperties,
};
pub use vector_traits::{
    Vector2DConstructor, Vector2DCore, Vector2DMeasure, Vector2DProperties, Vector3DConstructor,
    Vector3DCore, Vector3DMeasure, Vector3DProperties,
};
