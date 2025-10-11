//! 3D Geometry Module
//! 3次元幾何プリミティブ（f64ベース）

pub mod arc; // Arc3D<T>ジェネリック実装（🔄 型変換中）
pub mod bbox;
pub mod circle; // Circle3D<T>ジェネリック実装（✅ 型変換完了）
pub mod direction; // Direction3D<T>ジェネリック実装
                   // pub mod ellipse;  // 一時的にコメントアウト（複雑すぎるため後回し）
pub mod ellipse_arc; // EllipseArc基本実装（2D Ellipseベース）
pub mod infinite_line; // InfiniteLine3D実装を有効化
pub mod point;
pub mod ray; // Ray3D実装
pub mod vector;

// Re-export with consistent naming
pub use arc::{Arc, Arc3D, Arc3DF32, Arc3DF64, ArcKind}; // Arc3D<T>ジェネリック実装（🔄 型変換中）
pub use bbox::{BBox3D, BBox3DF32, BBox3DF64}; // BBox3D<T>ジェネリック実装と特化版エイリアス（✅ 型変換完了）
pub use circle::{Circle, Circle3DF32, Circle3DF64}; // Circle3D<T>ジェネリック実装（✅ 型変換完了）
pub use direction::{Direction3D, Direction3DF32, Direction3DF64}; // ジェネリックDirection3D
                                                                  // pub use ellipse::Ellipse;  // 一時的にコメントアウト（複雑すぎるため後回し）
pub use ellipse_arc::{EllipseArc, EllipseArcF64}; // EllipseArc基本実装（2D Ellipseベース）
pub use infinite_line::InfiniteLine3D; // InfiniteLine3D公開
pub use point::{Point, Point3D, Point3DF32, Point3DF64};
pub use ray::{Ray3D, Ray3DF32, Ray3DF64}; // Ray3D公開
pub use vector::{Vector, Vector3D};

// Type aliases for external compatibility
// pub use ellipse::Ellipse as Ellipse3D;  // 一時的にコメントアウト（Direction3D依存）
pub use ellipse_arc::EllipseArc as EllipseArc3D; // EllipseArc基本実装（2D Ellipseベース）
                                                 // pub use point::Point as Point;  // Point は point.rs から直接エクスポート
                                                 // Vector3D, Vector3Df are now directly imported from vector module
