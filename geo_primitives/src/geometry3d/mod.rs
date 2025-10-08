//! 3D Geometry Module
//! 3次元幾何プリミティブ（f64ベース）

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
pub use direction::Direction3D;
pub use ellipse::Ellipse;
pub use ellipse_arc::EllipseArc;
pub use point::Point;
pub use vector::Vector;

// Type aliases for external compatibility
pub use arc::Arc as Arc3D;
pub use bbox::BBox as BBox3D;
pub use ellipse::Ellipse as Ellipse3D;
pub use ellipse_arc::EllipseArc as EllipseArc3D;
pub use point::Point as Point3D;
pub use vector::Vector as Vector3D;
