//! 3D Geometry Module
//! 3次元幾何プリミティブ（f64ベース）

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
pub use arc::{Arc, Arc3D, Arc3DF64};
pub use bbox::{BBox3D, BBox3DF64}; // BBox3D と f64特化版エイリアスを公開
pub use circle::Circle;
pub use direction::Direction3D;
pub use ellipse::Ellipse;
pub use ellipse_arc::EllipseArc;
pub use infinite_line::InfiniteLine3D;
pub use point::Point;
pub use vector::{Vector, Vector3D, Vector3Df};

// Type aliases for external compatibility
pub use ellipse::Ellipse as Ellipse3D;
pub use ellipse_arc::EllipseArc as EllipseArc3D;
pub use point::Point as Point3D;
// Vector3D, Vector3Df are now directly imported from vector module
