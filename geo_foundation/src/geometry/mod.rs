//! 幾何型定義モジュール
//!
//! 基本的な幾何型（角度、円、点、ベクトル、境界ボックス、円弧など）の実装を提供

pub mod angle;
pub mod arc;
pub mod bbox;
pub mod circle;
pub mod ellipse;
pub mod point;
pub mod vector;

pub use angle::{Angle, Angle32, Angle64};
pub use arc::{Arc2D, Arc2DImpl, Arc3D, Arc3DImpl, ArcError, ArcKind};
pub use bbox::{BoundingBox2D, BoundingBox3D};
pub use circle::{Circle, Circle2D, Circle2D32, Circle2D64, Circle2DImpl, Circle3D, Circle3DImpl};
pub use ellipse::{Ellipse, Ellipse2D, Ellipse2DImpl, Ellipse3D, Ellipse3DImpl, EllipseError};
pub use point::{Point2D, Point2D32, Point2D64, Point3D, Point3D32, Point3D64};
pub use vector::{Vector2D, Vector3D};
