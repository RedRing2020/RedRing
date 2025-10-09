//! 2D Geometry Module
//! 2次元幾何プリミティブと関連構造体

pub mod arc;
pub mod bbox;
pub mod circle;
pub mod direction; // Direction2Dのジェネリック実装
pub mod ellipse;
pub mod ellipse_arc;
pub mod infinite_line;  // InfiniteLine2D実装
pub mod point;
pub mod ray; // Ray2D実装
pub mod vector;

// Re-export with consistent naming
pub use arc::Arc;
pub use bbox::{BBox, BBox2D}; // BBox2D エイリアスも公開
pub use circle::Circle;
pub use direction::{Direction2D, Direction2DF32, Direction2DF64}; // ジェネリックDirection2D
pub use ellipse::Ellipse;
pub use ellipse_arc::EllipseArc;
pub use infinite_line::InfiniteLine2D;  // InfiniteLine2D公開
pub use point::{Point2D, Point2DF32, Point2DF64};
pub use ray::{Ray2D, Ray2DF32, Ray2DF64}; // Ray2D公開
pub use vector::Vector;

// Type aliases for external compatibility and backward compatibility
pub type Arc2D = Arc;
pub type Circle2D = Circle<f64>; // f64専用の Circle エイリアス
pub use ellipse::Ellipse as Ellipse2D;
pub use ellipse_arc::EllipseArc as EllipseArc2D;
// 型パラメータ化された Vector の後方互換エイリアス
pub use vector::{Vector2D, Vector2Df};

// Backward compatibility: Add Point alias for Point2D<f64>
pub type Point = Point2D<f64>; // Default Point (f64-based) for simple usage
pub type PointF64 = Point2D<f64>;
pub type PointF32 = Point2D<f32>;
