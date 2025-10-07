//! 3D Geometry Module
//! 3次元幾何プリミティブ（f64ベース）

pub mod direction;
pub mod vector;
pub mod point;
pub mod bbox;

pub use direction::Direction3D;
pub use vector::Vector;
pub use point::Point;
pub use bbox::BBox;

// Re-export with old names for compatibility
pub use vector::Vector as Vector3D;
pub use point::Point as Point3D;
pub use bbox::BBox as BBox3D;
