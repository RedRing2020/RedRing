//! 2D Geometry Module
//! 2次元幾何プリミティブと関連構造体

pub mod arc;
pub mod bbox;
pub mod circle;
pub mod direction;
pub mod ellipse;
pub mod ellipse_arc;
pub mod infinite_line;
pub mod point;
pub mod vector;

// Re-export with consistent naming
pub use arc::Arc;
pub use bbox::BBox;
pub use circle::Circle;
pub use direction::Direction2D;
pub use ellipse::Ellipse;
pub use ellipse_arc::EllipseArc;
pub use infinite_line::InfiniteLine2D;
pub use point::{Point2D, Point2DF32, Point2DF64};
pub use vector::Vector;

// Type aliases for external compatibility and backward compatibility
pub type Arc2D = Arc;
pub use bbox::BBox as BBox2D;
pub type Circle2D = Circle<f64>; // f64専用の Circle エイリアス
pub use ellipse::Ellipse as Ellipse2D;
pub use ellipse_arc::EllipseArc as EllipseArc2D;
// 型パラメータ化された Vector の後方互換エイリアス
pub use vector::{Vector2D, Vector2Df};

// Backward compatibility: Add Point alias for Point2D<f64>
pub type Point = Point2D<f64>; // Default Point (f64-based) for simple usage
pub type PointF64 = Point2D<f64>;
pub type PointF32 = Point2D<f32>;
