//! 2D Geometry Module
//! 2次元幾何プリミティブと関連構造体

pub mod arc;
pub mod bbox;
pub mod circle;
pub mod direction;
pub mod ellipse;
pub mod ellipse_arc;
pub mod point;
pub mod vector;

// Re-export with consistent naming
pub use arc::Arc;
pub use bbox::BBox;
pub use circle::Circle;
pub use direction::Direction2D;
pub use ellipse::Ellipse;
pub use ellipse_arc::EllipseArc;
pub use point::Point;
pub use vector::Vector;

// Type aliases for external compatibility
pub type Arc2D = Arc;
pub use bbox::BBox as BBox2D;
pub use ellipse::Ellipse as Ellipse2D;
pub use ellipse_arc::EllipseArc as EllipseArc2D;
pub use point::Point as Point2D;
pub use vector::Vector as Vector2D;
