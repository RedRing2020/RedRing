//! ベクトル演算モジュール
//!
//! 次元別に分離された高性能ベクトル演算ライブラリ
//! - 動的サイズベクトル：任意次元対応
//! - 固定サイズベクトル：2D, 3D, 4D（高速演算）
//! - 同次座標系対応：3D → 4D変換
#[allow(clippy::module_inception)]
pub mod vector;    // Vector<T> - 動的サイズベクトル
pub mod vector2;    // Vector2<T> - 2Dベクトル
pub mod vector3;    // Vector3<T> - 3Dベクトル
pub mod vector4;    // Vector4<T> - 4Dベクトル（同次座標）

pub use vector::Vector;
pub use vector2::Vector2;
pub use vector3::Vector3;
pub use vector4::Vector4;

// 便利な型エイリアス
pub type Vec2f = Vector2<f32>;
pub type Vec3f = Vector3<f32>;
pub type Vec4f = Vector4<f32>;

pub type Vec2d = Vector2<f64>;
pub type Vec3d = Vector3<f64>;
pub type Vec4d = Vector4<f64>;
