//! 2D Geometry Module
//! 2次元幾何プリミティブと関連構造体

pub mod direction;
pub mod point;
pub mod vector;
pub mod bbox;

// Keep old module names for compatibility during transition
pub mod point2d;
pub mod vector2d;
pub mod bbox2d;

pub use direction::Direction2D;
pub use point::Point;
pub use vector::Vector;
pub use bbox::BBox;

// Re-export with old names for compatibility
pub use point::Point as Point2D;
pub use vector::Vector as Vector2D;
pub use bbox::BBox as BBox2D;
