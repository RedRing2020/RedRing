//! 3D Geometry Module
//! 3次元幾何プリミティブ（f64ベース）

pub mod bbox;
pub mod circle;
pub mod direction;
pub mod point;
pub mod vector;

// Re-export with consistent naming
pub use bbox::BBox;
pub use circle::Circle;
pub use direction::Direction3D;
pub use point::Point;
pub use vector::Vector;

// Type aliases for external compatibility
pub use bbox::BBox as BBox3D;
pub use point::Point as Point3D;
pub use vector::Vector as Vector3D;
