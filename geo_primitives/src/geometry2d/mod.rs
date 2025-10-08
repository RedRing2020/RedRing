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
pub use point::Point;
pub use vector::Vector;

// Type aliases for external compatibility
pub type Arc2D = Arc;
pub use bbox::BBox as BBox2D;
pub type Circle2D = Circle<f64>; // f64専用の Circle エイリアス
pub use ellipse::Ellipse as Ellipse2D;
pub use ellipse_arc::EllipseArc as EllipseArc2D;
// 型パラメータ化された Point/Vector の後方互換エイリアス
pub use point::{Point2D, Point2Df};
pub use vector::{Vector2D, Vector2Df};
