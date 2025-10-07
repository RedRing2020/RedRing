//! 2D Geometry Module
//! 2次元幾何プリミティブと関連構造体

pub mod bbox;
pub mod circle;
pub mod direction;
pub mod point;
pub mod vector;

// Re-export with consistent naming
pub use bbox::BBox;
pub use circle::Circle;
pub use direction::Direction2D;
pub use point::Point;
pub use vector::Vector;

// Type aliases for external compatibility
pub type BBox2D = BBox;
pub type Point2D = Point;
pub type Vector2D = Vector;
