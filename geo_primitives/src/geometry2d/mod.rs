//! 2D Geometry Module
//! 2次元幾何プリミティブと関連構造体

pub mod arc;
pub mod bbox;
pub mod circle;
pub mod direction; // Directionのジェネリック実装
pub mod ellipse;
// pub mod ellipse_arc; // 一時的に無効化（ジェネリクス型変換のため）
pub mod infinite_line; // InfiniteLine2D実装
pub mod point;
pub mod ray; // Ray2D実装
pub mod vector;

// Re-export with consistent naming
pub use arc::Arc;
pub use bbox::{BBox, BBox2D}; // BBox2D エイリアスも公開
pub use circle::Circle;
pub use direction::{Direction, Direction2D, Direction2DF32, Direction2DF64};
pub use ellipse::{Ellipse, Ellipse2D, EllipseF32, EllipseF64};
// pub use ellipse_arc::{EllipseArc, EllipseArcF32, EllipseArcF64}; // 一時的に無効化
pub use infinite_line::{InfiniteLine, InfiniteLine2D, InfiniteLineF32, InfiniteLineF64}; // InfiniteLine公開
pub use point::{Point, Point2D, Point2DF32, Point2DF64};
pub use ray::{Ray, Ray2D, Ray2DF32, Ray2DF64}; // Ray公開
pub use vector::{Vector, Vector2D, Vector2DF32, Vector2DF64};

// Type aliases for external compatibility and backward compatibility
pub type Arc2D<T> = Arc<T>; // ジェネリック版
pub type Circle2D = Circle<f64>; // f64専用の Circle エイリアス

// Backward compatibility: Add Point alias for Point<f64>
pub type PointF64 = Point<f64>;
pub type PointF32 = Point<f32>;
